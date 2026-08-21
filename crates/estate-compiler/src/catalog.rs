//! The sealed three-kind Gate K primitive catalog.

use estate_core::{ClaimRef, Diagnostic, EntityId, Ident, NamespaceId, PrimitiveKindId};
use estate_schema::{
    CapabilityKind, ClaimActivation, ClaimTemplate, ClaimValue, InteractionDefinition,
    InteractionPhase, InteractionTrigger, MachineTemplate, PrimitiveExpansion,
    TransitionDefinition, TransitionInput, TransitionTrigger,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ApprovedKind {
    Door,
    Water,
    Light,
}

pub(crate) fn lookup(id: &PrimitiveKindId) -> Option<ApprovedKind> {
    match id.name().as_str() {
        "iron_barred_door" => Some(ApprovedKind::Door),
        "shallow_water_region" => Some(ApprovedKind::Water),
        "extinguishable_light" => Some(ApprovedKind::Light),
        _ => None,
    }
}

pub(crate) fn expand(
    kind: ApprovedKind,
    entity: &EntityId,
) -> Result<PrimitiveExpansion, Diagnostic> {
    match kind {
        ApprovedKind::Door => door(entity),
        ApprovedKind::Water => water(entity),
        ApprovedKind::Light => light(entity),
    }
}

fn door(entity: &EntityId) -> Result<PrimitiveExpansion, Diagnostic> {
    let access = machine(entity, "access", &["locked", "closed", "open"], "locked")
        .with_transitions(vec![
            command(
                "unlock",
                TransitionInput::ResolvedEntityCredential,
                "locked",
                "closed",
            ),
            command("open", TransitionInput::None, "closed", "open"),
            command("close", TransitionInput::None, "open", "closed"),
        ])?;
    let integrity = machine(
        entity,
        "integrity",
        &["intact", "damaged", "destroyed"],
        "intact",
    )
    .with_transitions(vec![event(
        "apply_damage",
        TransitionInput::Damage {
            channel: ident("fire"),
            amount: 2,
        },
        "intact",
        "destroyed",
    )])?;
    let ward =
        machine(entity, "ward", &["sealed", "unsealed"], "sealed").with_transitions(vec![
            command("unseal", TransitionInput::None, "sealed", "unsealed"),
        ])?;
    let combustion =
        machine(entity, "combustion", &["cold", "burning", "spent"], "cold").with_transitions(
            vec![command("ignite", TransitionInput::None, "cold", "burning")],
        )?;
    let portal_open = ClaimActivation::Any(vec![
        state(entity, "access", "open"),
        state(entity, "integrity", "destroyed"),
    ]);
    let claims = vec![
        ClaimTemplate::new(
            claim(entity, "portal", "blocks_ground"),
            CapabilityKind::BlocksGround,
            ClaimActivation::Not(Box::new(portal_open)),
            ClaimValue::Bool(true),
        ),
        ClaimTemplate::new(
            claim(entity, "ward", "blocks_ground"),
            CapabilityKind::BlocksGround,
            state(entity, "ward", "sealed"),
            ClaimValue::Bool(true),
        ),
    ];
    PrimitiveExpansion::new(
        [
            CapabilityKind::Boundary,
            CapabilityKind::Portal,
            CapabilityKind::BlocksGround,
            CapabilityKind::Machine,
            CapabilityKind::Interactable,
            CapabilityKind::Authority,
            CapabilityKind::Persisted,
        ],
        vec![access, integrity, ward, combustion],
        claims,
    )?
    .with_interactions(vec![InteractionDefinition::new(
        InteractionTrigger::OnEnter {
            namespace: namespace(entity, "combustion"),
            state: ident("burning"),
        },
        InteractionPhase::Causal,
        namespace(entity, "integrity"),
        ident("apply_damage"),
        TransitionInput::Damage {
            channel: ident("fire"),
            amount: 2,
        },
    )])
}

fn water(entity: &EntityId) -> Result<PrimitiveExpansion, Diagnostic> {
    PrimitiveExpansion::new(
        [
            CapabilityKind::Region,
            CapabilityKind::TraversalCostGround,
            CapabilityKind::Authority,
            CapabilityKind::Persisted,
        ],
        Vec::new(),
        vec![ClaimTemplate::new(
            claim(entity, "region", "traversal_cost_ground"),
            CapabilityKind::TraversalCostGround,
            ClaimActivation::Always,
            ClaimValue::Uint(3),
        )],
    )
}

fn light(entity: &EntityId) -> Result<PrimitiveExpansion, Diagnostic> {
    let emission =
        machine(entity, "emission", &["lit", "extinguished"], "lit").with_transitions(vec![
            command("extinguish", TransitionInput::None, "lit", "extinguished"),
        ])?;
    PrimitiveExpansion::new(
        [
            CapabilityKind::Machine,
            CapabilityKind::Interactable,
            CapabilityKind::EmitsLight,
            CapabilityKind::Authority,
            CapabilityKind::Persisted,
        ],
        vec![emission],
        vec![ClaimTemplate::new(
            claim(entity, "emission", "emits_light"),
            CapabilityKind::EmitsLight,
            state(entity, "emission", "lit"),
            ClaimValue::Bool(true),
        )],
    )
}

fn machine(entity: &EntityId, namespace: &str, states: &[&str], initial: &str) -> MachineTemplate {
    MachineTemplate::new(
        self::namespace(entity, namespace),
        states.iter().map(|state| ident(state)).collect(),
        ident(initial),
    )
}

fn command(
    action: &str,
    input: TransitionInput,
    source: &str,
    target: &str,
) -> TransitionDefinition {
    TransitionDefinition::new(
        TransitionTrigger::Command {
            action: ident(action),
            input,
        },
        ident(source),
        ident(target),
    )
}

fn event(
    handler: &str,
    input: TransitionInput,
    source: &str,
    target: &str,
) -> TransitionDefinition {
    TransitionDefinition::new(
        TransitionTrigger::Event {
            handler: ident(handler),
            input,
        },
        ident(source),
        ident(target),
    )
}

fn namespace(entity: &EntityId, local: &str) -> NamespaceId {
    NamespaceId::new(entity.clone(), ident(local))
}

fn state(entity: &EntityId, namespace: &str, state: &str) -> ClaimActivation {
    ClaimActivation::StateEquals {
        namespace: NamespaceId::new(entity.clone(), ident(namespace)),
        state: ident(state),
    }
}

fn claim(entity: &EntityId, namespace: &str, capability: &str) -> ClaimRef {
    ClaimRef::new(
        NamespaceId::new(entity.clone(), ident(namespace)),
        ident(capability),
    )
}

fn ident(literal: &str) -> Ident {
    Ident::new(literal).expect("the built-in primitive catalog uses legal identifiers")
}
