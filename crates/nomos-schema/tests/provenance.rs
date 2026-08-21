//! Mutation proof for typed forensic provenance.

use nomos_core::{EntityId, PrimitiveKindId, SourcePath, SourceSpan};
use nomos_schema::{
    Binding, Cell, DerivationInput, DerivationPass, DerivationProducer, DerivationStep,
    FactIdentity, FactOwner, FactOwnershipReceipt, IrEntity, PrimitiveExpansion,
    ProjectionConsumer, ResolvedFactValue, WorldIr, source_schema,
};

fn entity(value: &str) -> EntityId {
    EntityId::parse(value).unwrap()
}

fn span() -> SourceSpan {
    SourceSpan::new(
        SourcePath::new("tests/provenance.nomos").unwrap(),
        0,
        1,
        1,
        1,
    )
    .unwrap()
}

fn identity_receipt(
    fact: FactIdentity,
    resolved: ResolvedFactValue,
    derivation: Vec<DerivationStep>,
) -> FactOwnershipReceipt {
    FactOwnershipReceipt::new(
        fact,
        FactOwner::Graph,
        span(),
        resolved,
        [ProjectionConsumer::Diagnostics],
        derivation,
    )
    .unwrap()
}

fn entity_record(
    id: EntityId,
    binding: Binding,
    credential: Option<nomos_core::CatalogValueId>,
) -> IrEntity {
    IrEntity::new(
        id,
        PrimitiveKindId::parse("primitive/iron_barred_door").unwrap(),
        binding,
        credential,
        PrimitiveExpansion::new(Vec::new(), Vec::new(), Vec::new()).unwrap(),
        span(),
    )
}

#[test]
fn dangling_derivation_fact_reference_fails_closed() {
    let gate = entity("north_gate");
    let missing = FactIdentity::EntityIdentity(entity("missing_gate"));
    let receipt = identity_receipt(
        FactIdentity::EntityIdentity(gate.clone()),
        ResolvedFactValue::Entity(gate),
        vec![
            DerivationStep::new(
                DerivationProducer::Source,
                DerivationPass::DeclareEntity,
                [DerivationInput::Fact(missing)],
            )
            .unwrap(),
        ],
    );

    let rejected = WorldIr::new(
        source_schema(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![receipt],
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK1001");
}

#[test]
fn incompatible_resolved_value_fails_closed() {
    let gate = entity("north_gate");
    let receipt = identity_receipt(
        FactIdentity::EntityIdentity(gate),
        ResolvedFactValue::Entity(entity("another_gate")),
        vec![
            DerivationStep::new(
                DerivationProducer::Source,
                DerivationPass::DeclareEntity,
                Vec::new(),
            )
            .unwrap(),
        ],
    );

    let rejected = WorldIr::new(
        source_schema(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![receipt],
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK1002");
}

#[test]
fn receipt_root_for_absent_world_entity_fails_closed() {
    let gate = entity("north_gate");
    let receipt = identity_receipt(
        FactIdentity::EntityIdentity(gate.clone()),
        ResolvedFactValue::Entity(gate),
        vec![
            DerivationStep::new(
                DerivationProducer::Source,
                DerivationPass::DeclareEntity,
                Vec::new(),
            )
            .unwrap(),
        ],
    );

    let rejected = WorldIr::new(
        source_schema(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![receipt],
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK1001");
    assert!(
        rejected
            .message()
            .contains("absent from the compiled world")
    );
}

#[test]
fn undeclared_resolved_catalog_value_fails_closed() {
    let gate = entity("north_gate");
    let value = nomos_core::CatalogValueId::parse("credential/ghost_key").unwrap();
    let receipt = FactOwnershipReceipt::new(
        FactIdentity::EntityCredential(gate.clone()),
        FactOwner::WorldLinker,
        span(),
        ResolvedFactValue::CatalogValue(value),
        [ProjectionConsumer::Simulation],
        vec![
            DerivationStep::new(
                DerivationProducer::WorldLinker,
                DerivationPass::ResolveCatalogValue,
                Vec::new(),
            )
            .unwrap(),
        ],
    )
    .unwrap();

    let rejected = WorldIr::new(
        source_schema(),
        Vec::new(),
        vec![entity_record(
            gate,
            Binding::Cell(Cell::new(0, 0, 0)),
            Some(nomos_core::CatalogValueId::parse("credential/ghost_key").unwrap()),
        )],
        Vec::new(),
        vec![receipt],
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK1006");
}

#[test]
fn resolved_binding_and_credential_must_match_the_entity_record() {
    let gate = entity("north_gate");
    let binding_receipt = FactOwnershipReceipt::new(
        FactIdentity::EntitySpatialBinding(gate.clone()),
        FactOwner::WorldLinker,
        span(),
        ResolvedFactValue::Binding(Binding::Cell(Cell::new(9, 0, 0))),
        [ProjectionConsumer::Simulation],
        vec![
            DerivationStep::new(
                DerivationProducer::WorldLinker,
                DerivationPass::ResolveSpatialBinding,
                Vec::new(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let rejected_binding = WorldIr::new(
        source_schema(),
        Vec::new(),
        vec![entity_record(
            gate.clone(),
            Binding::Cell(Cell::new(0, 0, 0)),
            None,
        )],
        Vec::new(),
        vec![binding_receipt],
    )
    .unwrap_err();
    assert_eq!(rejected_binding.code().as_str(), "EK1002");

    let actual = nomos_core::CatalogValueId::parse("credential/gaoler_key").unwrap();
    let wrong = nomos_core::CatalogValueId::parse("credential/spare_key").unwrap();
    let credential_receipt = FactOwnershipReceipt::new(
        FactIdentity::EntityCredential(gate.clone()),
        FactOwner::WorldLinker,
        span(),
        ResolvedFactValue::CatalogValue(wrong.clone()),
        [ProjectionConsumer::Simulation],
        vec![
            DerivationStep::new(
                DerivationProducer::WorldLinker,
                DerivationPass::ResolveCatalogValue,
                Vec::new(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let rejected_credential = WorldIr::new(
        source_schema(),
        vec![actual.clone(), wrong],
        vec![entity_record(
            gate,
            Binding::Cell(Cell::new(0, 0, 0)),
            Some(actual),
        )],
        Vec::new(),
        vec![credential_receipt],
    )
    .unwrap_err();
    assert_eq!(rejected_credential.code().as_str(), "EK1002");
}

#[test]
fn unknown_producer_and_pass_ids_fail_closed() {
    assert_eq!(
        DerivationProducer::parse("mystery_compiler")
            .unwrap_err()
            .code()
            .as_str(),
        "EK1003"
    );
    assert_eq!(
        DerivationPass::parse("guess_what_happened")
            .unwrap_err()
            .code()
            .as_str(),
        "EK1004"
    );
}

#[test]
fn unsupported_typed_producer_pass_pair_fails_closed() {
    let gate = entity("north_gate");
    let receipt = identity_receipt(
        FactIdentity::EntityIdentity(gate.clone()),
        ResolvedFactValue::Entity(gate),
        vec![
            DerivationStep::new(
                DerivationProducer::WorldLinker,
                DerivationPass::DeclareEntity,
                Vec::new(),
            )
            .unwrap(),
        ],
    );

    let rejected = WorldIr::new(
        source_schema(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![receipt],
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK1005");
}

#[test]
fn fact_owner_and_fact_specific_pass_mismatches_fail_closed() {
    let gate = entity("north_gate");
    let wrong_owner = FactOwnershipReceipt::new(
        FactIdentity::EntityIdentity(gate.clone()),
        FactOwner::Lattice,
        span(),
        ResolvedFactValue::Entity(gate.clone()),
        [ProjectionConsumer::Diagnostics],
        vec![
            DerivationStep::new(
                DerivationProducer::Source,
                DerivationPass::DeclareEntity,
                Vec::new(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let rejected_owner = WorldIr::new(
        source_schema(),
        Vec::new(),
        vec![entity_record(
            gate.clone(),
            Binding::Cell(Cell::new(0, 0, 0)),
            None,
        )],
        Vec::new(),
        vec![wrong_owner],
    )
    .unwrap_err();
    assert_eq!(rejected_owner.code().as_str(), "EK1007");

    let wrong_pass = identity_receipt(
        FactIdentity::EntityIdentity(gate.clone()),
        ResolvedFactValue::Entity(gate.clone()),
        vec![
            DerivationStep::new(
                DerivationProducer::Source,
                DerivationPass::DeclareSpatialAnchor,
                Vec::new(),
            )
            .unwrap(),
        ],
    );
    let rejected_pass = WorldIr::new(
        source_schema(),
        Vec::new(),
        vec![entity_record(gate, Binding::Cell(Cell::new(0, 0, 0)), None)],
        Vec::new(),
        vec![wrong_pass],
    )
    .unwrap_err();
    assert_eq!(rejected_pass.code().as_str(), "EK1005");
}

#[test]
fn structured_semantics_and_readable_rendering_are_separate_outputs() {
    let gate = entity("north_gate");
    let receipt = identity_receipt(
        FactIdentity::EntityIdentity(gate.clone()),
        ResolvedFactValue::Entity(gate),
        vec![
            DerivationStep::new(
                DerivationProducer::Source,
                DerivationPass::DeclareEntity,
                Vec::new(),
            )
            .unwrap(),
        ],
    );

    let bytes = receipt.to_canonical().to_canonical_bytes();
    let structured = String::from_utf8(bytes).unwrap();
    assert!(structured.contains(r#""fact":{"entity":"north_gate","kind":"entity_identity"}"#));
    assert!(structured.contains(r#""producer":"source""#));
    assert!(structured.contains(r#""pass":"declare_entity""#));

    let readable = receipt.render_text();
    assert!(readable.contains("entity.north_gate.identity is owned by graph"));
    assert!(readable.contains("source/declare_entity"));
    assert_ne!(readable, structured);
}

#[test]
fn duplicate_typed_inputs_and_consumers_fail_before_encoding() {
    let input = DerivationInput::Fact(FactIdentity::EntityIdentity(entity("north_gate")));
    let duplicate_input = DerivationStep::new(
        DerivationProducer::Source,
        DerivationPass::DeclareEntity,
        [input.clone(), input],
    )
    .unwrap_err();
    assert_eq!(duplicate_input.code().as_str(), "EK0304");

    let gate = entity("north_gate");
    let duplicate_consumer = FactOwnershipReceipt::new(
        FactIdentity::EntityIdentity(gate.clone()),
        FactOwner::Graph,
        span(),
        ResolvedFactValue::Entity(gate),
        [
            ProjectionConsumer::Diagnostics,
            ProjectionConsumer::Diagnostics,
        ],
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(duplicate_consumer.code().as_str(), "EK0304");
}

#[test]
fn empty_derivations_fail_and_step_order_is_canonical() {
    let gate = entity("north_gate");
    let rejected = FactOwnershipReceipt::new(
        FactIdentity::EntityIdentity(gate.clone()),
        FactOwner::Graph,
        span(),
        ResolvedFactValue::Entity(gate.clone()),
        [ProjectionConsumer::Diagnostics],
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK1005");

    let first = DerivationStep::new(
        DerivationProducer::Source,
        DerivationPass::DeclareEntity,
        [DerivationInput::Fact(FactIdentity::EntityIdentity(entity(
            "brazier_02",
        )))],
    )
    .unwrap();
    let second = DerivationStep::new(
        DerivationProducer::Source,
        DerivationPass::DeclareEntity,
        [DerivationInput::Fact(FactIdentity::EntityIdentity(entity(
            "flooded_section",
        )))],
    )
    .unwrap();
    let build = |steps| {
        FactOwnershipReceipt::new(
            FactIdentity::EntityIdentity(gate.clone()),
            FactOwner::Graph,
            span(),
            ResolvedFactValue::Entity(gate.clone()),
            [ProjectionConsumer::Diagnostics],
            steps,
        )
        .unwrap()
        .to_canonical()
        .to_canonical_bytes()
    };
    assert_eq!(
        build(vec![first.clone(), second.clone()]),
        build(vec![second, first])
    );
}
