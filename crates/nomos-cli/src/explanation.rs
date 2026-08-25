//! Read-only semantic explanations over already verified typed evidence.

use std::collections::BTreeSet;

use nomos_compiler::OpenedCompiledWorld;
use nomos_core::id::StableId;
use nomos_core::{CanonicalValue, ClaimRef, Diagnostic, EntityId};
use nomos_projection::MovementDisposition;
use nomos_sim::{CausalReceipt, resolve_light, resolve_movement};

use crate::{OpenedRunBundle, initial_state_from_package};

pub(crate) fn entity_report(
    world: &OpenedCompiledWorld,
    entity: &EntityId,
) -> Result<CanonicalValue, Diagnostic> {
    let construction = world.stable_ir().construction();
    let entity_record = construction
        .entities()
        .iter()
        .find(|candidate| candidate.id() == entity)
        .ok_or_else(|| missing_entity(entity))?;
    let initial = initial_state_from_package(world)?;
    let movement = resolve_movement(world.simulation(), &initial)?;
    let light = resolve_light(world.simulation(), &initial)?;
    let movement = movement.get(entity);
    let light = light.get(entity);

    let mut active_claims = BTreeSet::new();
    if let Some(disposition) = movement {
        active_claims.extend(disposition.reasons().iter().cloned());
    }
    if let Some(fact) = light {
        active_claims.extend(fact.reasons().iter().cloned());
    }

    let ownership = construction
        .ownership_receipts()
        .iter()
        .filter(|receipt| receipt.fact().mentions_entity(entity))
        .map(|receipt| receipt.to_canonical())
        .collect();
    let registry = world
        .registry()
        .entries()
        .iter()
        .map(|entry| {
            CanonicalValue::object_declared([
                ("artifact", CanonicalValue::text(entry.artifact())),
                ("owner", CanonicalValue::text(entry.owner().as_str())),
                ("schema", entry.schema().to_canonical()),
            ])
        })
        .collect();

    Ok(CanonicalValue::object_declared([
        (
            "active_initial_claims",
            CanonicalValue::Array(active_claims.iter().map(StableId::to_canonical).collect()),
        ),
        ("command", CanonicalValue::text("explain-entity")),
        ("entity", entity_record.to_canonical()),
        (
            "effective_initial_facts",
            CanonicalValue::object_declared([
                (
                    "ground_movement",
                    movement.map_or(CanonicalValue::Null, MovementDisposition::to_canonical),
                ),
                (
                    "light_emission",
                    light.map_or(CanonicalValue::Null, |fact| {
                        CanonicalValue::object_declared([
                            ("emitting", CanonicalValue::Bool(fact.emitting())),
                            (
                                "reasons",
                                CanonicalValue::Array(
                                    fact.reasons().iter().map(StableId::to_canonical).collect(),
                                ),
                            ),
                        ])
                    }),
                ),
            ]),
        ),
        ("ownership_receipts", CanonicalValue::Array(ownership)),
        (
            "package_digest",
            CanonicalValue::text(world.package_digest().to_hex()),
        ),
        (
            "schemas",
            CanonicalValue::object_declared([
                (
                    "construction_world_ir",
                    construction.schema().to_canonical(),
                ),
                ("package_members", CanonicalValue::Array(registry)),
                (
                    "runtime_state",
                    nomos_sim::runtime_state_schema().to_canonical(),
                ),
                ("source", construction.source_schema().to_canonical()),
                ("world_ir", world.stable_ir().schema().to_canonical()),
            ]),
        ),
        ("status", CanonicalValue::text("completed")),
    ]))
}

pub(crate) fn transition_report(
    world: &OpenedCompiledWorld,
    run: &OpenedRunBundle,
    entity: &EntityId,
    tick: u64,
) -> Result<CanonicalValue, Diagnostic> {
    let construction = world.stable_ir().construction();
    let entity_record = construction
        .entities()
        .iter()
        .find(|candidate| candidate.id() == entity)
        .ok_or_else(|| missing_entity(entity))?;
    let (index, receipt) = run
        .causal_receipts()
        .receipts()
        .iter()
        .enumerate()
        .find(|(_, receipt)| receipt.tick() == tick)
        .ok_or_else(|| {
            Diagnostic::new(
                nomos_core::diagnostic::codes::EXPLANATION_TICK_MISSING,
                format!("verified run contains no committed transition at tick {tick}"),
            )
        })?;
    let row = &run.command_log().rows()[index];
    if receipt.command().namespace().entity() != entity
        && !receipt
            .steps()
            .iter()
            .any(|step| step.namespace().entity() == entity)
    {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::EXPLANATION_ENTITY_UNRELATED,
            format!("entity `{entity}` is unrelated to the committed transition at tick {tick}"),
        ));
    }

    let before = active_claims(receipt, entity, true);
    let after = active_claims(receipt, entity, false);
    let claims_added = after
        .difference(&before)
        .map(StableId::to_canonical)
        .collect();
    let claims_removed = before
        .difference(&after)
        .map(StableId::to_canonical)
        .collect();

    Ok(CanonicalValue::object_declared([
        ("claims_added", CanonicalValue::Array(claims_added)),
        ("claims_removed", CanonicalValue::Array(claims_removed)),
        ("command", CanonicalValue::text("explain-transition")),
        ("entity", entity.to_canonical()),
        (
            "package_digest",
            CanonicalValue::text(world.package_digest().to_hex()),
        ),
        ("receipt", receipt.to_canonical()),
        ("request", row.request().to_canonical()),
        ("resolved_command", receipt.resolved_command_to_canonical()),
        (
            "run_result_digest",
            CanonicalValue::text(run.result_digest().to_hex()),
        ),
        ("source_mapping", entity_record.source_span().to_canonical()),
        ("status", CanonicalValue::text("completed")),
        ("tick", CanonicalValue::Uint(tick)),
    ]))
}

fn active_claims(receipt: &CausalReceipt, entity: &EntityId, before: bool) -> BTreeSet<ClaimRef> {
    let movement = if before {
        receipt.movement_before()
    } else {
        receipt.movement_after()
    };
    let light = if before {
        receipt.light_before()
    } else {
        receipt.light_after()
    };
    let mut claims = BTreeSet::new();
    if let Some(disposition) = movement.get(entity) {
        claims.extend(disposition.reasons().iter().cloned());
    }
    if let Some(fact) = light.get(entity) {
        claims.extend(fact.reasons().iter().cloned());
    }
    claims
}

fn missing_entity(entity: &EntityId) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::EXPLANATION_ENTITY_MISSING,
        format!("verified world contains no entity `{entity}`"),
    )
}
