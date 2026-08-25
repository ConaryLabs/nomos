//! `nomos.presentation_state@1`: what the renderer draws, per tick.
//!
//! This is the owner file for that identity, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.
//!
//! Derived, emitted once per tick, persisted nowhere, and outside every hash
//! domain — the standing `nomos.effective_facts@2` has under `RUNTIME.md`
//! section 5 R1-1.
//!
//! `machine_states`, `movement`, and `effective_light` are spelled exactly as
//! `nomos.rendering_plan@3`'s `scenarios[]` spells them, including the `null`
//! cost on a blocked subject that `RUNTIME.md` section 5 R1-1 names as the one
//! normalization. That is load-bearing rather than decorative: the viewer's
//! `machineState`, `doorState`, `wardSealed`, and `lightOf` accessors take a
//! scenario-shaped object, so they read a presentation state unchanged and the
//! renderer needs no second accessor set.
//!
//! There is no prose here. Guidance strings are assembled in the viewer from
//! identifiers the plan already publishes; putting authored prose into an
//! authoritative document would reopen the ownership audit's row 26 after R1-4
//! closed it.

use nomos_core::CanonicalValue;
use nomos_core::canonical::keyed_array;
use nomos_core::id::SchemaId;
use nomos_projection::MovementDisposition;

use crate::batch::{self, Area};
use crate::error::PlayResult;

/// The presentation-state identity.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn presentation_state_schema() -> SchemaId {
    SchemaId::new("nomos.presentation_state", 1)
        .expect("the presentation-state schema id is a literal")
}

/// Builds the presentation state of an area at its current tick.
///
/// # Errors
///
/// Returns `PL0308` when a kernel resolver refuses.
///
/// # Panics
///
/// Panics if the kernel resolves the same subject twice, which
/// `ResolvedMovementFacts` and `ResolvedLightFacts` both refuse to build.
pub fn presentation_state(area: &Area) -> PlayResult<CanonicalValue> {
    let movement = batch::movement_facts(area)?;
    let light = batch::light_facts(area)?;
    let interactions = batch::available_interactions(area)?;
    let state = &area.state;

    let machine_states = keyed_array(area.semantics.machines().iter().filter_map(|machine| {
        state
            .kernel
            .state()
            .machine(machine.namespace())
            .map(|now| {
                (
                    machine.namespace().clone(),
                    CanonicalValue::object_declared([
                        (
                            "namespace",
                            CanonicalValue::text(machine.namespace().to_string()),
                        ),
                        ("state", CanonicalValue::text(now.as_str())),
                    ]),
                )
            })
    }))
    .expect("a simulation plan validates unique machine namespaces");

    let movement_rows = keyed_array(movement.facts().iter().map(|fact| {
        let (cost, kind) = match fact.disposition() {
            MovementDisposition::Traversable { cost, .. } => (
                CanonicalValue::Uint(u64::from(*cost)),
                CanonicalValue::text("traversable"),
            ),
            MovementDisposition::Blocked { .. } => {
                (CanonicalValue::Null, CanonicalValue::text("blocked"))
            }
        };
        (
            fact.entity().clone(),
            CanonicalValue::object_declared([
                ("cost", cost),
                ("disposition", kind),
                ("entity", CanonicalValue::text(fact.entity().to_string())),
                (
                    "reasons",
                    CanonicalValue::Array(
                        fact.disposition()
                            .reasons()
                            .iter()
                            .map(|reason| CanonicalValue::text(reason.to_string()))
                            .collect(),
                    ),
                ),
            ]),
        )
    }))
    .expect("resolved movement facts validate unique subjects");

    let light_rows = keyed_array(light.facts().iter().map(|fact| {
        (
            fact.entity().clone(),
            CanonicalValue::object_declared([
                ("emitting", CanonicalValue::Bool(fact.emitting())),
                ("entity", CanonicalValue::text(fact.entity().to_string())),
            ]),
        )
    }))
    .expect("resolved light facts validate unique subjects");

    let actors = keyed_array(state.actors.iter().map(|actor| {
        (
            actor.id.clone(),
            CanonicalValue::object_declared([
                ("cell", crate::state::cell_value(actor.cell)),
                ("id", CanonicalValue::text(actor.id.to_string())),
                ("role", CanonicalValue::text(actor.role.as_str())),
            ]),
        )
    }))
    .expect("a play state validates unique actor identities");

    let hunting = light
        .get(&state.pursuit_light)
        .is_some_and(|fact| !fact.emitting());

    Ok(CanonicalValue::object_declared([
        ("actors", actors),
        ("area", CanonicalValue::text(state.area.clone())),
        (
            "counters",
            CanonicalValue::object_declared([
                ("moves", CanonicalValue::Uint(state.counters.moves)),
                (
                    "traversal_cost",
                    CanonicalValue::Uint(state.counters.traversal_cost),
                ),
            ]),
        ),
        ("effective_light", light_rows),
        (
            "interactions",
            // A plain array, not a keyed one: an entity can offer more than one
            // legal action, so entity is not a key here. The order is
            // `(entity, action)` ascending, which is the rule
            // `batch::available_interactions` states.
            CanonicalValue::Array(
                interactions
                    .iter()
                    .map(|(entity, action)| {
                        CanonicalValue::object_declared([
                            ("action", CanonicalValue::text(action.as_str())),
                            ("entity", CanonicalValue::text(entity.to_string())),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "kernel_state_hash",
            CanonicalValue::text(state.kernel_state_hash().to_hex()),
        ),
        ("machine_states", machine_states),
        ("movement", movement_rows),
        ("outcome", CanonicalValue::text(state.outcome.as_str())),
        (
            "pursuit",
            CanonicalValue::object_declared([
                ("hunting", CanonicalValue::Bool(hunting)),
                (
                    "light",
                    CanonicalValue::text(state.pursuit_light.to_string()),
                ),
                (
                    "moves_since_step",
                    CanonicalValue::Uint(state.moves_since_step),
                ),
            ]),
        ),
        (
            "schema",
            CanonicalValue::text(presentation_state_schema().to_string()),
        ),
        ("tick", CanonicalValue::Uint(state.tick)),
    ]))
}
