//! Compiler-owned invariants recoverable from persisted stable IR.

use std::collections::{BTreeMap, BTreeSet};

use nomos_core::{Diagnostic, EntityId, RepairClass};
use nomos_schema::{
    DerivationInput, DerivationPass, DerivationProducer, DerivationStep, FactIdentity, FactOwner,
    FactOwnershipReceipt, IrEntity, ProjectionConsumer, ResolvedFactValue, StableWorldIr,
};

use crate::catalog::{self, ApprovedKind};

pub(crate) fn validate_rehydrated_ir(ir: &StableWorldIr) -> Result<(), Diagnostic> {
    let construction = ir.construction();
    let catalog_values = construction
        .catalog_values()
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    for entity in construction.entities() {
        let kind = catalog::lookup(entity.primitive()).ok_or_else(|| {
            invalid(format!(
                "entity `{}` names primitive `{}` outside the sealed catalog",
                entity.id(),
                entity.primitive()
            ))
        })?;
        validate_entity_shape(entity, kind, &catalog_values)?;
        let expansion =
            catalog::expand(kind, entity.id()).map_err(|error| invalid(error.message()))?;
        if entity.expansion() != &expansion {
            return Err(invalid(format!(
                "entity `{}` does not carry the exact compiler expansion of `{}`",
                entity.id(),
                entity.primitive()
            )));
        }
    }

    let expected_movement = crate::resolver::construction_plan(construction.entities())
        .map_err(|error| invalid(error.message()))?;
    if construction.movement_resolver() != &expected_movement {
        return Err(invalid(
            "movement resolver is not the exact compiler consequence of entity bindings and claims",
        ));
    }
    let expected_light = crate::resolver::light_construction_plan(construction.entities())
        .map_err(|error| invalid(error.message()))?;
    if construction.light_resolver() != &expected_light {
        return Err(invalid(
            "light resolver is not the exact compiler consequence of entity claims",
        ));
    }
    let expected_movement_v1 = crate::projection::initial_movement_v1(construction)
        .map_err(|error| invalid(error.message()))?;
    if ir.movement_v1() != expected_movement_v1 {
        return Err(invalid(
            "stable-v1 movement rows are not the exact compiler consequence of initial resolver state",
        ));
    }

    let entities = construction
        .entities()
        .iter()
        .map(|entity| (entity.id().clone(), entity))
        .collect::<BTreeMap<_, _>>();
    for relation in construction.relations() {
        if relation.kind().as_str() != "owns"
            || !entities.contains_key(relation.subject())
            || !entities.contains_key(relation.object())
        {
            return Err(invalid(format!(
                "relation `{} {} {}` is outside the linked Gate K relation vocabulary",
                relation.subject(),
                relation.kind(),
                relation.object()
            )));
        }
    }
    validate_receipts(ir, &entities)
}

fn validate_entity_shape(
    entity: &IrEntity,
    kind: ApprovedKind,
    catalog_values: &BTreeSet<nomos_core::CatalogValueId>,
) -> Result<(), Diagnostic> {
    let expected_binding = match kind {
        ApprovedKind::Door => "face",
        ApprovedKind::Water => "region",
        ApprovedKind::Light => "cell",
    };
    if entity.binding().kind() != expected_binding {
        return Err(invalid(format!(
            "primitive `{}` requires a `{expected_binding}` binding",
            entity.primitive()
        )));
    }
    match (kind, entity.credential()) {
        (ApprovedKind::Door, Some(value))
            if value.catalog().as_str() == "credential" && catalog_values.contains(value) =>
        {
            Ok(())
        }
        (ApprovedKind::Door, _) => Err(invalid(format!(
            "door `{}` requires one declared credential catalog value",
            entity.id()
        ))),
        (ApprovedKind::Water | ApprovedKind::Light, None) => Ok(()),
        (ApprovedKind::Water | ApprovedKind::Light, Some(_)) => Err(invalid(format!(
            "primitive `{}` does not accept a credential",
            entity.primitive()
        ))),
    }
}

fn validate_receipts(
    ir: &StableWorldIr,
    entities: &BTreeMap<EntityId, &IrEntity>,
) -> Result<(), Diagnostic> {
    let receipts = ir
        .construction()
        .ownership_receipts()
        .iter()
        .map(|receipt| (receipt.fact().clone(), receipt))
        .collect::<BTreeMap<_, _>>();
    let mut expected_facts = BTreeSet::new();
    for entity in entities.values() {
        expected_facts.extend([
            FactIdentity::EntityIdentity(entity.id().clone()),
            FactIdentity::EntitySpatialAnchor(entity.id().clone()),
            FactIdentity::EntitySpatialBinding(entity.id().clone()),
        ]);
        if entity.credential().is_some() {
            expected_facts.insert(FactIdentity::EntityCredential(entity.id().clone()));
        }
    }
    expected_facts.extend(ir.construction().relations().iter().map(|relation| {
        FactIdentity::Relation {
            subject: relation.subject().clone(),
            kind: relation.kind().clone(),
            object: relation.object().clone(),
        }
    }));
    if receipts.keys().cloned().collect::<BTreeSet<_>>() != expected_facts {
        return Err(invalid(
            "ownership receipts do not exactly cover compiler-owned world facts",
        ));
    }

    for (fact, actual) in receipts {
        let expected = expected_receipt(&fact, actual, entities)?;
        if actual != &expected {
            return Err(invalid(format!(
                "ownership receipt `{fact}` is not the exact compiler-owned provenance edge",
            )));
        }
    }
    Ok(())
}

fn expected_receipt(
    fact: &FactIdentity,
    actual: &FactOwnershipReceipt,
    entities: &BTreeMap<EntityId, &IrEntity>,
) -> Result<FactOwnershipReceipt, Diagnostic> {
    let (owner, resolved, consumers, derivation) = match fact {
        FactIdentity::EntityIdentity(entity) => (
            FactOwner::Graph,
            ResolvedFactValue::Entity(entity.clone()),
            consumers(&[
                ProjectionConsumer::Diagnostics,
                ProjectionConsumer::Persistence,
                ProjectionConsumer::Simulation,
            ]),
            vec![step(DerivationPass::DeclareEntity, Vec::new())?],
        ),
        FactIdentity::EntitySpatialAnchor(entity) => {
            let record = entity_record(entities, entity)?;
            (
                FactOwner::Lattice,
                ResolvedFactValue::Binding(record.binding().clone()),
                consumers(&[
                    ProjectionConsumer::Diagnostics,
                    ProjectionConsumer::Navigation,
                    ProjectionConsumer::Simulation,
                ]),
                vec![step(
                    DerivationPass::DeclareSpatialAnchor,
                    vec![DerivationInput::Fact(FactIdentity::EntityIdentity(
                        entity.clone(),
                    ))],
                )?],
            )
        }
        FactIdentity::EntitySpatialBinding(entity) => {
            let record = entity_record(entities, entity)?;
            (
                FactOwner::WorldLinker,
                ResolvedFactValue::Binding(record.binding().clone()),
                consumers(&[
                    ProjectionConsumer::Diagnostics,
                    ProjectionConsumer::Navigation,
                    ProjectionConsumer::Persistence,
                    ProjectionConsumer::Simulation,
                ]),
                vec![
                    DerivationStep::new(
                        DerivationProducer::WorldLinker,
                        DerivationPass::ResolveSpatialBinding,
                        [
                            DerivationInput::Fact(FactIdentity::EntitySpatialAnchor(
                                entity.clone(),
                            )),
                            DerivationInput::Primitive(record.primitive().clone()),
                        ],
                    )
                    .map_err(|error| invalid(error.message()))?,
                ],
            )
        }
        FactIdentity::EntityCredential(entity) => {
            let record = entity_record(entities, entity)?;
            let credential = record
                .credential()
                .ok_or_else(|| invalid(format!("entity `{entity}` has no credential")))?;
            (
                FactOwner::WorldLinker,
                ResolvedFactValue::CatalogValue(credential.clone()),
                consumers(&[
                    ProjectionConsumer::Diagnostics,
                    ProjectionConsumer::Persistence,
                    ProjectionConsumer::Simulation,
                ]),
                vec![
                    DerivationStep::new(
                        DerivationProducer::WorldLinker,
                        DerivationPass::ResolveCatalogValue,
                        [
                            DerivationInput::Fact(FactIdentity::EntityIdentity(entity.clone())),
                            DerivationInput::Primitive(record.primitive().clone()),
                            DerivationInput::CatalogValue(credential.clone()),
                        ],
                    )
                    .map_err(|error| invalid(error.message()))?,
                ],
            )
        }
        FactIdentity::Relation {
            subject,
            kind,
            object,
        } => (
            FactOwner::Graph,
            ResolvedFactValue::Relation {
                subject: subject.clone(),
                kind: kind.clone(),
                object: object.clone(),
            },
            consumers(&[
                ProjectionConsumer::Diagnostics,
                ProjectionConsumer::Persistence,
                ProjectionConsumer::Simulation,
            ]),
            vec![step(
                DerivationPass::LinkRelation,
                vec![
                    DerivationInput::Fact(FactIdentity::EntityIdentity(subject.clone())),
                    DerivationInput::Fact(FactIdentity::EntityIdentity(object.clone())),
                ],
            )?],
        ),
    };
    FactOwnershipReceipt::new(
        fact.clone(),
        owner,
        actual.declared_at().clone(),
        resolved,
        consumers,
        derivation,
    )
    .map_err(|error| invalid(error.message()))
}

fn entity_record<'a>(
    entities: &'a BTreeMap<EntityId, &IrEntity>,
    entity: &EntityId,
) -> Result<&'a IrEntity, Diagnostic> {
    entities
        .get(entity)
        .copied()
        .ok_or_else(|| invalid(format!("receipt refers to absent entity `{entity}`")))
}

fn consumers(values: &[ProjectionConsumer]) -> Vec<ProjectionConsumer> {
    values.to_vec()
}

fn step(pass: DerivationPass, inputs: Vec<DerivationInput>) -> Result<DerivationStep, Diagnostic> {
    DerivationStep::new(DerivationProducer::Source, pass, inputs)
        .map_err(|error| invalid(error.message()))
}

fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::PACKAGE_MEMBER_SCHEMA_INVALID,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}
