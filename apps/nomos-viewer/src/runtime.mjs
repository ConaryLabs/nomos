// The loader for the authoritative runtime.
//
// `crates/nomos-play` compiled to `wasm32-unknown-unknown` is the only
// authority for where the actors are, what a step costs, whether a gate opens,
// and whether the gaoler has caught anyone. This module is the whole bridge to
// it: about a hundred lines, no dependency, and no glue generator.
//
// The module declares **zero imports** — `crates/nomos-play/src/wasm.rs` is
// hand-written `extern "C"` with no `wasm-bindgen` — so
// `WebAssembly.instantiate(bytes, {})` is the entire contract, and
// `test/runtime.test.mjs` asserts the import list is empty rather than trusting
// it. `instantiate` rather than `instantiateStreaming`, so the loader does not
// depend on the server sending `application/wasm`.
//
// The memory contract, in three rules:
//
//   * arguments are `(ptr, len)` pairs the caller allocates and frees;
//   * results are one packed `u64`, `(ptr << 32) | len`, pointing at bytes the
//     **caller** now owns and must free;
//   * `ptr === 0` means the call failed, and `nomos_play_last_error()` returns
//     the diagnostic line.
//
// Every read of linear memory re-reads `exports.memory.buffer`, because an
// allocation can detach the previous `ArrayBuffer` and a cached view would be
// silently empty.

import { CODES, ViewerError } from "./plan.mjs";

/// The ABI revision this loader speaks. A `.wasm` that answers anything else is
/// refused by name rather than left to trap somewhere inside a later call.
export const ABI_VERSION = 1;

const decoder = new TextDecoder();
const encoder = new TextEncoder();

const unpack = (packed) => [Number(packed >> 32n), Number(packed & 0xffffffffn)];

/// Instantiates the runtime from `url` and returns the typed surface the viewer
/// uses. `fetchImpl` is passed in so the smoke lane and the node tests can hand
/// it a reader that is not the page's.
export async function loadRuntime(url, fetchImpl) {
  const response = await fetchImpl(url);
  if (!response.ok) {
    throw new ViewerError(CODES.UNREADABLE, `${url} responded ${response.status}`, url);
  }
  const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), {});
  const exports = instance.exports;

  const declared = exports.nomos_play_abi_version();
  if (declared !== ABI_VERSION) {
    throw new ViewerError(
      CODES.SCHEMA_MISMATCH,
      `the play runtime declares ABI ${declared}, and this viewer speaks ${ABI_VERSION}`,
      url,
    );
  }

  const bytesAt = (pointer, length) =>
    new Uint8Array(exports.memory.buffer).slice(pointer, pointer + length);

  const refusal = () => {
    const [pointer, length] = unpack(exports.nomos_play_last_error());
    if (pointer === 0) return new ViewerError(CODES.RUNTIME_REFUSED, "the play runtime failed");
    const text = decoder.decode(bytesAt(pointer, length));
    exports.nomos_play_free(pointer, length);
    return new ViewerError(CODES.RUNTIME_REFUSED, text || "the play runtime failed");
  };

  const takeText = (packed) => {
    const [pointer, length] = unpack(packed);
    if (pointer === 0) throw refusal();
    const text = decoder.decode(bytesAt(pointer, length));
    exports.nomos_play_free(pointer, length);
    return text;
  };

  const take = (packed) => JSON.parse(takeText(packed));

  const put = (bytes) => {
    const pointer = exports.nomos_play_alloc(bytes.length);
    if (pointer === 0) {
      throw new ViewerError(CODES.RUNTIME_REFUSED, "the play runtime is out of memory");
    }
    new Uint8Array(exports.memory.buffer).set(bytes, pointer);
    return pointer;
  };

  // Two owned arguments, freed whether the call succeeded or threw.
  const withArea = (call, planBytes, semanticsBytes) => {
    const plan = put(planBytes);
    const semantics = put(semanticsBytes);
    try {
      return take(call(plan, planBytes.length, semantics, semanticsBytes.length));
    } finally {
      exports.nomos_play_free(plan, planBytes.length);
      exports.nomos_play_free(semantics, semanticsBytes.length);
    }
  };

  return {
    /// Begins a new session at one area. This is also reset.
    start: (planBytes, semanticsBytes) =>
      withArea(exports.nomos_play_start, planBytes, semanticsBytes),
    /// Continues the session into the area the last crossing named.
    enter: (planBytes, semanticsBytes) =>
      withArea(exports.nomos_play_enter, planBytes, semanticsBytes),
    /// Applies one `nomos.play_command@1` and returns the new presentation state.
    step(command) {
      const bytes = encoder.encode(JSON.stringify(command));
      const pointer = put(bytes);
      try {
        return take(exports.nomos_play_step(pointer, bytes.length));
      } finally {
        exports.nomos_play_free(pointer, bytes.length);
      }
    },
    /// The live presentation state, without stepping.
    presentationState: () => take(exports.nomos_play_presentation_state()),
    /// The whole `nomos.play_session@1` document — what the smoke lane records
    /// and what `nomos-play replay` re-executes.
    session: () => take(exports.nomos_play_session()),
    /// The same document, as the runtime's own canonical bytes.
    ///
    /// What gets recorded and replayed is these bytes, not a re-serialization
    /// of the parsed value: `nomos-play replay` reads with the kernel's strict
    /// canonical reader, and a JSON round trip through a host is not something
    /// to rely on preserving byte order.
    sessionText: () => takeText(exports.nomos_play_session()),
    /// The committed inputs, as a bare array. A window onto the session.
    commandLog: () => take(exports.nomos_play_command_log()),
    /// The receipts, as a bare array. A window onto the session.
    receipts: () => take(exports.nomos_play_receipts()),
    /// The instantiated module, for a test that wants to inspect it.
    instance,
  };
}
