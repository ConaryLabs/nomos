//! The sealed three-kind Gate K primitive catalog.

use estate_core::{ClaimRef, Diagnostic, EntityId, Ident, NamespaceId, PrimitiveKindId};
use estate_schema::{
    CapabilityKind, ClaimActivation, ClaimTemplate, ClaimValue, MachineTemplate, PrimitiveExpansion,
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
    let access = machine(entity, "access", &["locked", "closed", "open"], "locked");
    let integrity = machine(
        entity,
        "integrity",
        &["intact", "damaged", "destroyed"],
        "intact",
    );
    let ward = machine(entity, "ward", &["sealed", "unsealed"], "sealed");
    let combustion = machine(entity, "combustion", &["cold", "burning", "spent"], "cold");
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
    )
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
    PrimitiveExpansion::new(
        [
            CapabilityKind::Machine,
            CapabilityKind::Interactable,
            CapabilityKind::EmitsLight,
            CapabilityKind::Authority,
            CapabilityKind::Persisted,
        ],
        vec![machine(entity, "emission", &["lit", "extinguished"], "lit")],
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
        NamespaceId::new(entity.clone(), ident(namespace)),
        states.iter().map(|state| ident(state)).collect(),
        ident(initial),
    )
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
