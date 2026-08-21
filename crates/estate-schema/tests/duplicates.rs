//! Fail-closed construction identities from checkpoint issue #21.

use estate_core::{
    ClaimRef, EntityId, Ident, NamespaceId, PrimitiveKindId, SourcePath, SourceSpan,
};
use estate_schema::{
    Binding, CapabilityKind, Cell, ClaimActivation, ClaimTemplate, ClaimValue,
    InteractionDefinition, InteractionPhase, InteractionTrigger, IrEntity, MachineTemplate,
    PrimitiveExpansion, TransitionDefinition, TransitionInput, TransitionTrigger, WorldIr,
    source_schema,
};

fn ident(value: &str) -> Ident {
    Ident::new(value).unwrap()
}

fn namespace(local: &str) -> NamespaceId {
    NamespaceId::new(EntityId::parse("north_gate").unwrap(), ident(local))
}

#[test]
fn duplicate_machine_namespaces_fail_before_encoding() {
    let machine = MachineTemplate::new(
        namespace("access"),
        vec![ident("locked"), ident("open")],
        ident("locked"),
    );
    let rejected = PrimitiveExpansion::new(
        [CapabilityKind::Machine],
        vec![machine.clone(), machine],
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0304");
    assert!(rejected.message().contains("machine namespace"));
}

#[test]
fn duplicate_claim_references_fail_before_encoding() {
    let claim = ClaimTemplate::new(
        ClaimRef::new(namespace("ward"), ident("blocks_ground")),
        CapabilityKind::BlocksGround,
        ClaimActivation::Always,
        ClaimValue::Bool(true),
    );
    let rejected = PrimitiveExpansion::new(
        [CapabilityKind::BlocksGround],
        Vec::new(),
        vec![claim.clone(), claim],
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0304");
    assert!(rejected.message().contains("claim reference"));
}

#[test]
fn duplicate_world_ir_entities_fail_before_encoding() {
    let expansion = PrimitiveExpansion::new(Vec::new(), Vec::new(), Vec::new()).unwrap();
    let entity = IrEntity::new(
        EntityId::parse("north_gate").unwrap(),
        PrimitiveKindId::parse("primitive/iron_barred_door").unwrap(),
        Binding::Cell(Cell::new(0, 0, 0)),
        None,
        expansion,
        SourceSpan::new(SourcePath::new("tests/fixture.estate").unwrap(), 0, 1, 1, 1).unwrap(),
    );
    let rejected = WorldIr::new(
        source_schema(),
        Vec::new(),
        vec![entity.clone(), entity],
        Vec::new(),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0304");
    assert!(rejected.message().contains("entity"));
}

#[test]
fn duplicate_transition_signatures_fail_before_encoding() {
    let transition = TransitionDefinition::new(
        TransitionTrigger::Command {
            action: ident("open"),
            input: TransitionInput::None,
        },
        ident("closed"),
        ident("open"),
    );
    let rejected = MachineTemplate::new(
        namespace("access"),
        vec![ident("closed"), ident("open")],
        ident("closed"),
    )
    .with_transitions(vec![transition.clone(), transition])
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0704");
}

#[test]
fn duplicate_interaction_identities_fail_before_encoding() {
    let interaction = InteractionDefinition::new(
        InteractionTrigger::OnEnter {
            namespace: namespace("combustion"),
            state: ident("burning"),
        },
        InteractionPhase::Causal,
        namespace("integrity"),
        ident("apply_damage"),
        TransitionInput::Damage {
            channel: ident("fire"),
            amount: 2,
        },
    );
    let rejected = PrimitiveExpansion::new(Vec::new(), Vec::new(), Vec::new())
        .unwrap()
        .with_interactions(vec![interaction.clone(), interaction])
        .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0705");
}
