//! The typed entity catalog of one strictly verified compiled world.
//!
//! The four persisted projections deliberately carry no entity kind: a
//! `simulation.json` entity is `{id, binding, machines}`, so a downstream tool
//! that needs to know a door from a water region has to guess — which is why
//! `experiments/executable-gaol/src/build-plan.mjs:25` classifies by
//! `machine.endsWith(".access")`. The kind and the typed capability set already
//! exist, in the stable World IR entity record; nothing in the tree exposes
//! them next to the projected binding, machines, and resolver claims.
//!
//! This module joins what the kernel already knows into one read-only
//! entity-sorted document. It classifies nothing, resolves nothing, and reads
//! no source: every field is copied from typed evidence the package opener has
//! already verified.

use std::collections::BTreeMap;

use nomos_core::canonical::keyed_array;
use nomos_core::id::{SchemaId, StableId};
use nomos_core::{
    CanonicalValue, ClaimRef, Diagnostic, EntityId, NamespaceId, RepairClass, SourceSpan,
};
use nomos_projection::{
    LightSubject, MachineDefinition, MovementClaim, MovementSubject, ProjectedEntity,
};
use nomos_schema::{CapabilityKind, IrEntity};

use crate::OpenedCompiledWorld;

/// The resolver a projected claim belongs to.
const MOVEMENT: &str = "movement";
/// The resolver a projected light claim belongs to.
const LIGHT: &str = "light";

/// One entity-sorted catalog of every entity in a verified compiled world.
///
/// Each record carries the World IR primitive kind and capability set beside
/// the simulation projection's binding and machines and the movement and light
/// resolver claims whose subject the entity is. The document is derived: it is
/// written to stdout, never into a package or a run bundle, and is outside the
/// state-hash domain.
///
/// # Errors
///
/// Returns `EK0413` when the verified package's World IR and simulation
/// projection disagree about an entity or a machine namespace — evidence that
/// cannot be assembled truthfully is refused rather than filled in.
pub fn entity_catalog(world: &OpenedCompiledWorld) -> Result<CanonicalValue, Diagnostic> {
    let construction = world.stable_ir().construction();
    let simulation = world.simulation();

    let projected: BTreeMap<&EntityId, &ProjectedEntity> = simulation
        .entities()
        .iter()
        .map(|entity| (entity.id(), entity))
        .collect();
    let machines: BTreeMap<&NamespaceId, &MachineDefinition> = simulation
        .machines()
        .iter()
        .map(|machine| (machine.namespace(), machine))
        .collect();
    // The movement resolver plan is shared: `validate_member_integrity` proves
    // the simulation and navigation projections carry byte-identical
    // `movement_resolver` values before this function can be reached.
    let movement: BTreeMap<&EntityId, &MovementSubject> = world
        .navigation()
        .movement_resolver()
        .subjects()
        .iter()
        .map(|subject| (subject.entity(), subject))
        .collect();
    let light: BTreeMap<&EntityId, &LightSubject> = simulation
        .light_resolver()
        .subjects()
        .iter()
        .map(|subject| (subject.entity(), subject))
        .collect();

    let records = construction
        .entities()
        .iter()
        .map(|entity| {
            let record = entity_record(entity, &projected, &machines, &movement, &light)?;
            Ok((entity.id().clone(), record))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;

    Ok(CanonicalValue::object_declared([
        ("command", CanonicalValue::text("entity-catalog")),
        ("entities", keyed_array(records)?),
        (
            "schema",
            CanonicalValue::text(entity_catalog_schema().to_string()),
        ),
        ("status", CanonicalValue::text("completed")),
        (
            "world",
            CanonicalValue::object_declared([
                (
                    "manifest_digest",
                    CanonicalValue::text(world.package_digest().to_hex()),
                ),
                (
                    "world_ir_schema",
                    CanonicalValue::text(world.stable_ir().schema().to_string()),
                ),
            ]),
        ),
    ]))
}

fn entity_record(
    entity: &IrEntity,
    projected: &BTreeMap<&EntityId, &ProjectedEntity>,
    machines: &BTreeMap<&NamespaceId, &MachineDefinition>,
    movement: &BTreeMap<&EntityId, &MovementSubject>,
    light: &BTreeMap<&EntityId, &LightSubject>,
) -> Result<CanonicalValue, Diagnostic> {
    let id = entity.id();
    let projected = projected.get(id).ok_or_else(|| {
        inconsistent(format!(
            "World IR entity `{id}` has no simulation projection record"
        ))
    })?;

    Ok(CanonicalValue::object_declared([
        ("binding", projected.binding().to_canonical()),
        ("capabilities", capabilities(entity)),
        ("claims", claims(id, movement, light)?),
        ("id", id.to_canonical()),
        (
            "light_subject",
            CanonicalValue::Bool(light.contains_key(id)),
        ),
        ("machines", entity_machines(projected, machines)?),
        (
            "movement_subject",
            CanonicalValue::Bool(movement.contains_key(id)),
        ),
        ("primitive", entity.primitive().to_canonical()),
    ]))
}

/// The World IR capability set, sorted by its wire spelling.
///
/// The World IR itself emits these in `CapabilityKind` declaration order; the
/// catalog sorts every array by the identity a consumer sees, so the members
/// are the same set in lexicographic order.
fn capabilities(entity: &IrEntity) -> CanonicalValue {
    let mut spellings: Vec<&'static str> = entity
        .expansion()
        .capabilities()
        .iter()
        .map(|capability| capability.as_str())
        .collect();
    spellings.sort_unstable();
    CanonicalValue::Array(spellings.into_iter().map(CanonicalValue::text).collect())
}

fn entity_machines(
    projected: &ProjectedEntity,
    machines: &BTreeMap<&NamespaceId, &MachineDefinition>,
) -> Result<CanonicalValue, Diagnostic> {
    let rows = projected
        .machines()
        .iter()
        .map(|namespace| {
            let definition = machines.get(namespace).ok_or_else(|| {
                inconsistent(format!(
                    "projected entity `{}` names machine `{namespace}`, which the simulation projection does not define",
                    projected.id()
                ))
            })?;
            let states = definition
                .states()
                .iter()
                .map(|state| CanonicalValue::text(state.as_str()))
                .collect();
            Ok((
                namespace.clone(),
                CanonicalValue::object_declared([
                    ("initial", CanonicalValue::text(definition.initial().as_str())),
                    ("namespace", namespace.to_canonical()),
                    ("states", CanonicalValue::Array(states)),
                ]),
            ))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;
    keyed_array(rows)
}

fn claims(
    id: &EntityId,
    movement: &BTreeMap<&EntityId, &MovementSubject>,
    light: &BTreeMap<&EntityId, &LightSubject>,
) -> Result<CanonicalValue, Diagnostic> {
    let mut rows = Vec::new();
    if let Some(subject) = movement.get(id) {
        for claim in subject.claims() {
            let capability = match claim {
                MovementClaim::Blocker { .. } => CapabilityKind::BlocksGround,
                MovementClaim::TraversalCost { .. } => CapabilityKind::TraversalCostGround,
            };
            rows.push(claim_row(claim.id(), capability, MOVEMENT, claim.source()));
        }
    }
    if let Some(subject) = light.get(id) {
        for claim in subject.claims() {
            rows.push(claim_row(
                claim.id(),
                CapabilityKind::EmitsLight,
                LIGHT,
                claim.source(),
            ));
        }
    }
    keyed_array(rows)
}

fn claim_row(
    id: &ClaimRef,
    capability: CapabilityKind,
    resolver: &'static str,
    source: &SourceSpan,
) -> ((String, &'static str), CanonicalValue) {
    (
        (id.canonical_string(), resolver),
        CanonicalValue::object_declared([
            ("capability", CanonicalValue::text(capability.as_str())),
            ("id", id.to_canonical()),
            ("resolver", CanonicalValue::text(resolver)),
            ("source", source.to_canonical()),
        ]),
    )
}

fn inconsistent(message: String) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::PACKAGE_MEMBER_INCONSISTENT,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}

/// Canonical schema for the read-only entity catalog.
///
/// Like `nomos.effective_facts@1` this document exists to be consumed by a
/// downstream compiler rather than read by a human, so it carries a versioned
/// identity to bind against. It is declared here, in the crate that owns World
/// IR decoding and projection generation and is therefore the only kernel crate
/// that can see both halves of the join.
///
/// # Panics
///
/// Panics if the built-in literal is not a valid schema id, which this crate's
/// tests rule out.
#[must_use]
pub fn entity_catalog_schema() -> SchemaId {
    SchemaId::new("nomos.entity_catalog", 1)
        .expect("the entity-catalog schema id is a valid literal")
}

#[cfg(test)]
mod tests {
    use super::entity_catalog_schema;
    use crate::{consumed_schemas, produced_schemas};

    #[test]
    fn the_catalog_identity_is_versioned_and_is_not_a_package_artifact() {
        assert_eq!(
            entity_catalog_schema().to_string(),
            "nomos.entity_catalog@1"
        );
        // The catalog is derived stdout, never a package member, so its name
        // must not collide with an artifact this compiler reads or writes and
        // it must not enter the registry that binds the compiled members.
        assert!(!produced_schemas().contains(&entity_catalog_schema()));
        for schema in produced_schemas().iter().chain(consumed_schemas().iter()) {
            assert_ne!(
                schema.name(),
                entity_catalog_schema().name(),
                "the entity catalog versions independently of every compiled artifact"
            );
        }
    }
}
