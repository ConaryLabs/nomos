//! The composed effective-fact projection for one runtime state.
//!
//! `explain-entity` composes effective facts for a single entity at the initial
//! state and `explain-transition` reports one entity at one tick. Neither
//! answers "what are the effective facts for every resolver subject at *this*
//! state", which is the question every downstream consumer of a compiled world
//! actually asks. This module answers it by calling the two existing resolvers;
//! it contains no activation, blocking, cost, or union logic of its own.

use nomos_core::{CanonicalValue, Diagnostic, Sha256Digest};
use nomos_projection::SimulationPlan;

use crate::{PersistedRuntimeState, resolve_light, resolve_movement};

/// Composes every resolver subject's effective facts at one runtime state.
///
/// The `package_digest` binds the document to the exact verified world package
/// the state was resolved against, so a consumer cannot silently pair facts
/// with a different world.
///
/// # Errors
///
/// Propagates the `EK09xx` diagnostics raised by [`resolve_movement`] and
/// [`resolve_light`] when a projected plan leaves the Gate K resolver contract
/// or an activation names a machine or state that is absent.
pub fn effective_facts(
    plan: &SimulationPlan,
    state: &PersistedRuntimeState,
    package_digest: Sha256Digest,
) -> Result<CanonicalValue, Diagnostic> {
    let movement = resolve_movement(plan, state.state())?;
    let light = resolve_light(plan, state.state())?;
    let movement_count = movement.facts().len();
    let light_count = light.facts().len();

    Ok(CanonicalValue::object_declared([
        ("command", CanonicalValue::text("effective-facts")),
        (
            "effective_facts",
            CanonicalValue::object_declared([
                ("ground_movement", movement.to_canonical()),
                ("light_emission", light.to_canonical()),
            ]),
        ),
        (
            "package_digest",
            CanonicalValue::text(package_digest.to_hex()),
        ),
        (
            "runtime_semantics_digest",
            CanonicalValue::text(state.runtime_semantics_digest().to_hex()),
        ),
        ("schema", effective_facts_schema().to_canonical()),
        (
            "state_hash",
            CanonicalValue::text(state.state_hash().to_hex()),
        ),
        ("status", CanonicalValue::text("completed")),
        (
            "subject_count",
            CanonicalValue::object_declared([
                (
                    "ground_movement",
                    CanonicalValue::Uint(movement_count as u64),
                ),
                ("light_emission", CanonicalValue::Uint(light_count as u64)),
            ]),
        ),
        ("tick", CanonicalValue::Uint(state.state().tick())),
    ]))
}

/// Canonical schema for the composed effective-fact projection.
///
/// Unlike the `explain-*` reports, this document exists to be consumed and
/// persisted by a downstream tool, so it carries a versioned identity to bind
/// against.
///
/// # Panics
///
/// Panics if the built-in literal is not a valid schema id, which this crate's
/// tests rule out.
#[must_use]
pub fn effective_facts_schema() -> nomos_core::id::SchemaId {
    nomos_core::id::SchemaId::new("nomos.effective_facts", 1)
        .expect("the effective-facts schema id is a valid literal")
}
