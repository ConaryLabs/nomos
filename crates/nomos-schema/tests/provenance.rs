//! Mutation proof for typed forensic provenance.

use nomos_core::{EntityId, SourcePath, SourceSpan};
use nomos_schema::{
    DerivationInput, DerivationPass, DerivationProducer, DerivationStep, FactIdentity, FactOwner,
    FactOwnershipReceipt, ProjectionConsumer, ResolvedFactValue, WorldIr, source_schema,
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
