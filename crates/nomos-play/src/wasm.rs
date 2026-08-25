//! The hand-written `extern "C"` ABI, and the only `unsafe` in the R1 tree.
//!
//! No `wasm-bindgen`, no third-party crate, no import: the module this compiles
//! to declares **zero** imports, so `WebAssembly.instantiate(bytes, {})` is the
//! whole loader contract. `apps/nomos-viewer/src/runtime.mjs` is the loader and
//! `apps/nomos-viewer/test/runtime.test.mjs` proves the import list is empty.
//!
//! # The memory contract
//!
//! * **Arguments are `(ptr, len)` pairs.** The caller allocates with
//!   [`nomos_play_alloc`], writes UTF-8 canonical JSON into linear memory, and
//!   calls. The callee copies what it needs and does not take ownership; the
//!   caller frees with [`nomos_play_free`].
//! * **Results are one packed `u64`**, `(ptr << 32) | len`, pointing at a
//!   freshly allocated UTF-8 canonical JSON document. The **caller** owns it and
//!   frees it with [`nomos_play_free`]. One scalar rather than an out-parameter
//!   keeps the ABI to a single calling convention and the loader to two lines of
//!   bit arithmetic.
//! * **Failure is `ptr == 0`.** The caller then calls [`nomos_play_last_error`],
//!   which returns a packed pair pointing at a plain UTF-8 diagnostic string —
//!   `PL0302 (3, 0) is inside masonry mass \`channel_buttress\`` — and not a
//!   document. Deliberately: an error envelope would be a sixth canonical
//!   identity to declare, own, and register, and an ABI failure channel is not a
//!   contract with content. A `0` from [`nomos_play_alloc`] is out of memory,
//!   the one failure a loader reports as fatal rather than as a refusal.
//! * **[`nomos_play_abi_version`] returns `1`.** The loader refuses any other
//!   value before it calls anything else, so a stale `.wasm` in a browser cache
//!   fails with a named error instead of trapping.
//!
//! # Panics
//!
//! The `wasm` profile sets `panic = "abort"`, so a Rust panic compiles to a wasm
//! `unreachable` and surfaces in JS as a `RuntimeError` from the call. The
//! loader catches it and rethrows, so the smoke lane sees a console error rather
//! than hanging. Everything reachable from an export returns a `Result`; the
//! only panics left are the `expect`s on schema-id literals, which this crate's
//! tests rule out.
//!
//! # What the `unsafe` does, exhaustively
//!
//! `std::alloc::alloc` and `dealloc` with a 1-byte-aligned `Layout`;
//! `std::slice::from_raw_parts` to read an argument;
//! `std::ptr::copy_nonoverlapping` to write a result. No pointer arithmetic, no
//! transmute, no `static mut`: the runtime singleton is a
//! `RefCell<Option<PlaySession>>` in a `thread_local!`, sound on wasm32's single
//! thread and needing no `unsafe` at all. The whole module is behind
//! `#[cfg(target_arch = "wasm32")]`, so the native build, the CLI, and every
//! test compile with `unsafe` forbidden.

#![allow(unsafe_code)]

use std::alloc::{Layout, alloc, dealloc};
use std::cell::RefCell;

use nomos_core::CanonicalValue;

use crate::command::PlayCommand;
use crate::error::{PlayError, PlayResult, codes};
use crate::presentation::presentation_state;
use crate::session::PlaySession;

/// The ABI revision this module implements.
pub const ABI_VERSION: u32 = 1;

thread_local! {
    static SESSION: RefCell<Option<PlaySession>> = const { RefCell::new(None) };
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

/// The ABI revision, so a loader can refuse a stale binary by name.
#[unsafe(no_mangle)]
pub extern "C" fn nomos_play_abi_version() -> u32 {
    ABI_VERSION
}

/// Allocates `len` bytes of linear memory for the caller to write into.
///
/// Returns null for `len == 0` and on allocation failure.
///
/// # Safety
///
/// The caller owns the returned pointer until it passes it back to
/// [`nomos_play_free`] with the same `len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nomos_play_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    match Layout::from_size_align(len, 1) {
        Ok(layout) => unsafe { alloc(layout) },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Releases memory obtained from [`nomos_play_alloc`] or returned by any export.
///
/// # Safety
///
/// `ptr` must come from this module with exactly this `len`, and must not have
/// been freed already.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nomos_play_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len, 1) {
        unsafe { dealloc(ptr, layout) };
    }
}

/// Begins a new session at one area. This is also reset.
///
/// Returns the packed `nomos.presentation_state@1` of the opening tick.
///
/// # Safety
///
/// Both pointers must address `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nomos_play_start(
    plan: *const u8,
    plan_len: usize,
    semantics: *const u8,
    semantics_len: usize,
) -> u64 {
    let (Some(plan), Some(semantics)) = (unsafe { borrow(plan, plan_len) }, unsafe {
        borrow(semantics, semantics_len)
    }) else {
        return fail(&PlayError::new(
            codes::DOCUMENT_SHAPE,
            "start was given a null pointer",
        ));
    };
    match PlaySession::start(plan, semantics) {
        Ok(session) => {
            let state = presentation_state(session.live());
            SESSION.with_borrow_mut(|slot| *slot = Some(session));
            emit(state)
        }
        Err(error) => fail(&error),
    }
}

/// Continues the session into the area the last crossing named.
///
/// # Safety
///
/// Both pointers must address `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nomos_play_enter(
    plan: *const u8,
    plan_len: usize,
    semantics: *const u8,
    semantics_len: usize,
) -> u64 {
    let (Some(plan), Some(semantics)) = (unsafe { borrow(plan, plan_len) }, unsafe {
        borrow(semantics, semantics_len)
    }) else {
        return fail(&PlayError::new(
            codes::DOCUMENT_SHAPE,
            "enter was given a null pointer",
        ));
    };
    with_session(|session| {
        session.enter(plan, semantics)?;
        presentation_state(session.live())
    })
}

/// Applies one `nomos.play_command@1` as exactly one committed batch.
///
/// # Safety
///
/// `command` must address `command_len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nomos_play_step(command: *const u8, command_len: usize) -> u64 {
    let Some(bytes) = (unsafe { borrow(command, command_len) }) else {
        return fail(&PlayError::new(
            codes::COMMAND_SHAPE,
            "step was given a null pointer",
        ));
    };
    let input = match PlayCommand::decode(bytes) {
        Ok(input) => input,
        Err(error) => return fail(&error),
    };
    with_session(|session| {
        session.step(&input)?;
        presentation_state(session.live())
    })
}

/// The live area's presentation state, without stepping.
#[unsafe(no_mangle)]
pub extern "C" fn nomos_play_presentation_state() -> u64 {
    with_session(|session| presentation_state(session.live()))
}

/// The whole `nomos.play_session@1` document.
#[unsafe(no_mangle)]
pub extern "C" fn nomos_play_session() -> u64 {
    with_session(|session| Ok(session.to_canonical()))
}

/// The committed inputs, as a bare array of `nomos.play_command@1` objects.
///
/// A window onto the session, not a second authority: it exists so a caller
/// does not have to parse a document carrying every state to read the log.
#[unsafe(no_mangle)]
pub extern "C" fn nomos_play_command_log() -> u64 {
    with_session(|session| Ok(session.log_value()))
}

/// The receipts, as a bare array of `nomos.play_receipt@1` objects.
#[unsafe(no_mangle)]
pub extern "C" fn nomos_play_receipts() -> u64 {
    with_session(|session| Ok(session.receipts_value()))
}

/// The last refusal, as a plain UTF-8 diagnostic line.
///
/// Returns a zero-length result when nothing has failed.
#[unsafe(no_mangle)]
pub extern "C" fn nomos_play_last_error() -> u64 {
    let text = LAST_ERROR.with_borrow(Clone::clone);
    handoff(text.into_bytes())
}

fn with_session<F>(action: F) -> u64
where
    F: FnOnce(&mut PlaySession) -> PlayResult<CanonicalValue>,
{
    let outcome = SESSION.with_borrow_mut(|slot| match slot.as_mut() {
        Some(session) => action(session),
        None => Err(PlayError::new(
            codes::ENTER_REFUSED,
            "no session: call nomos_play_start first",
        )),
    });
    emit(outcome)
}

fn emit(state: PlayResult<CanonicalValue>) -> u64 {
    match state {
        Ok(value) => handoff(value.to_canonical_bytes()),
        Err(error) => fail(&error),
    }
}

fn fail(error: &PlayError) -> u64 {
    LAST_ERROR.with_borrow_mut(|slot| *slot = error.to_string());
    0
}

/// Moves owned bytes into linear memory and packs `(ptr, len)`.
fn handoff(bytes: Vec<u8>) -> u64 {
    let len = bytes.len();
    if len == 0 {
        // A zero-length result still needs a non-null pointer so that `ptr == 0`
        // keeps meaning "failed" and nothing else. One byte is enough.
        let pointer = unsafe { nomos_play_alloc(1) };
        return pack(pointer, 0);
    }
    let pointer = unsafe { nomos_play_alloc(len) };
    if pointer.is_null() {
        return 0;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, len) };
    pack(pointer, len)
}

fn pack(pointer: *mut u8, len: usize) -> u64 {
    ((pointer as usize as u64) << 32) | (len as u64)
}

/// Borrows an argument without taking ownership of it.
///
/// # Safety
///
/// `pointer` must address `len` readable bytes for the duration of the call.
unsafe fn borrow<'a>(pointer: *const u8, len: usize) -> Option<&'a [u8]> {
    if pointer.is_null() {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(pointer, len) })
}
