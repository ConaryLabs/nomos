//! The command-time half of the kernel.
//!
//! `KERNEL.md` section 10 assigns this crate *runtime state, command
//! transactions, replay, effective-fact resolution*. Section 2 fixes the
//! transaction order, and section 7 fixes what enters the state hash.
//!
//! # The boundary this crate exists to hold
//!
//! Section 10 names one edge as forbidden by name: `nomos-sim` may not depend
//! on `nomos-schema` or the source parser, and section 4 forbids any runtime
//! subsystem reparsing `.nomos` source. That is not a style preference. If the
//! runtime could read authoring source, "the compiler owns every consequence"
//! would be a hope rather than a structural fact.
//!
//! The rule is enforced by the dependency graph, so it is a compile error and
//! not a review comment:
//!
//! ```compile_fail
//! let _ = nomos_schema::construction_world_ir_schema();
//! ```
//!
//! ```compile_fail
//! let _ = nomos_compiler::consumed_schemas();
//! ```
//!
//! The permitted edges do resolve, so the failures above are about the
//! boundary and not about a broken doctest harness:
//!
//! ```
//! use nomos_core::id::SchemaId;
//! let _: SchemaId = nomos_sim::runtime_state_schema();
//! let _: SchemaId = nomos_projection::simulation_schema();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use nomos_core::id::SchemaId;

mod command_language;
mod commit;
mod receipt;
mod resolver;
mod state_persistence;
mod transaction;

pub use command_language::{CommandRequest, CommandScript, resolve_command};
pub use commit::{CommittedTransaction, commit_transaction, commit_transaction_with_budget};
pub use receipt::{CausalReceipt, EffectiveFactRef, EffectiveFactValue, ProjectionDelta};
pub use resolver::{resolve_light, resolve_movement};
pub use state_persistence::PersistedRuntimeState;
pub use transaction::{
    DEFAULT_TRANSITION_BUDGET, PreparedTransaction, RuntimeEntityState, SimulationState,
    TransitionCause, TransitionStep, prepare_transaction, prepare_transaction_with_budget,
};

/// The authoritative runtime-state schema.
///
/// Section 5 keeps this separate from the package: the package contains
/// initial-state material sufficient to create a snapshot, and is not itself
/// the mutable snapshot.
///
/// # Panics
///
/// Panics if the built-in literal is not a valid schema id, which this crate's
/// tests rule out.
#[must_use]
pub fn runtime_state_schema() -> SchemaId {
    SchemaId::new("nomos.runtime_state", 1).expect("the runtime state schema id is a valid literal")
}

/// Schema for a standalone state file bound to exact simulation semantics.
#[must_use]
pub fn persisted_runtime_state_schema() -> SchemaId {
    SchemaId::new("nomos.persisted_runtime_state", 1)
        .expect("the persisted runtime-state schema id is a valid literal")
}

/// Schema identity named by strict persisted command scripts.
#[must_use]
pub fn command_script_schema() -> SchemaId {
    SchemaId::new("nomos.command_script", 1)
        .expect("the command-script schema id is a valid literal")
}

/// The typed causal-receipt schema written beside runtime state.
#[must_use]
pub fn causal_receipt_schema() -> SchemaId {
    SchemaId::new("nomos.causal_receipt", 1)
        .expect("the causal receipt schema id is a valid literal")
}

/// The replay and command-log schema.
///
/// # Panics
///
/// Panics if the built-in literal is not a valid schema id, which this crate's
/// tests rule out.
#[must_use]
pub fn replay_log_schema() -> SchemaId {
    SchemaId::new("nomos.replay_log", 1).expect("the replay log schema id is a valid literal")
}

#[cfg(test)]
mod tests {
    use super::{causal_receipt_schema, replay_log_schema, runtime_state_schema};

    #[test]
    fn runtime_schemas_are_independent_of_the_projections_this_crate_consumes() {
        let runtime = runtime_state_schema();
        assert_ne!(runtime.name(), replay_log_schema().name());
        assert_ne!(runtime.name(), causal_receipt_schema().name());
        for projection in nomos_projection::all_schemas() {
            assert_ne!(
                runtime.name(),
                projection.name(),
                "runtime state versions independently of any projection"
            );
        }
    }
}
