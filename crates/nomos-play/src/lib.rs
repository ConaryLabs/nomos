//! The authoritative play runtime: actors, command batches, pursuit, receipts,
//! and replay, layered over the kernel's own transactions.
//!
//! `RUNTIME.md` section 5 R1-5. Before this crate, `apps/nomos-viewer/src/play.mjs`
//! owned the player's cell, the traversal cost of a step, mass collision, the
//! exit through a door, the gaoler's pursuit, capture, the area transition, and
//! both counters, while the kernel supplied a captured ladder of scenarios the
//! browser walked. Section 1 criterion 2 forbids a surviving shadow resolver.
//!
//! # The epoch decision
//!
//! Actors do not enter `nomos.runtime_state@2`. Adding a field to the kernel's
//! state envelope would change the canonical bytes of every run bundle and
//! every state hash in the tree, including worlds that declare no actor, and
//! `RUNTIME.md` section 3 option (a) admits kernel surface only when no Gate K
//! command, artifact, hash, or diagnostic changes. So this crate holds the
//! actors and treats the kernel's persisted state as an opaque embedded
//! authority: it hands the kernel bytes and commands, and takes back bytes,
//! hashes, and resolved facts.
//!
//! The consequence is stated once because every shape depends on it: **this
//! crate never invents a movement disposition, a traversal cost, a reason, or a
//! light fact.** Every one comes from `nomos_sim::resolve_movement` and
//! `nomos_sim::resolve_light` evaluated at the embedded kernel state. What this
//! crate owns is where the actors are, whose turn it is, and what the run has
//! cost — facts the kernel has no opinion about.
//!
//! # Five identities
//!
//! Each declared by its own module and registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`:
//!
//! | Identity | Owner file |
//! | --- | --- |
//! | `nomos.play_state@1` | [`state`] |
//! | `nomos.play_command@1` | [`command`] |
//! | `nomos.play_receipt@1` | [`receipt`] |
//! | `nomos.play_session@1` | [`session`] |
//! | `nomos.presentation_state@1` | [`presentation`] |
//!
//! # No clock, no float, no randomness
//!
//! Nothing here reads a clock, counts a frame, holds a floating-point value, or
//! draws a random number, and `tests/documents.rs` greps this crate's own
//! source to keep it that way. Determinism is therefore a property of the code
//! rather than of the environment: the only iteration is over `BTreeMap`,
//! `BTreeSet`, and `nomos_core::canonical::keyed_array`.

#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod batch;
pub mod command;
pub mod error;
pub mod occupancy;
pub mod plan;
pub mod presentation;
pub mod read;
pub mod receipt;
pub mod replay;
pub mod session;
pub mod state;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use batch::{Area, Committed};
pub use command::{Direction, PlayCommand, play_command_schema};
pub use error::{PlayError, PlayResult, codes};
pub use plan::{AreaPlan, Role, rendering_plan_schema};
pub use presentation::{presentation_state, presentation_state_schema};
pub use receipt::{ActorDelta, PlayReceipt, chain_origin, play_receipt_schema};
pub use replay::{Divergence, ReplayReport, replay};
pub use session::{
    PlaySession, RecordedSession, RouteRow, SessionOutcome, open, play_session_schema,
};
pub use state::{Actor, Counters, Outcome, PlayState, play_state_schema};
