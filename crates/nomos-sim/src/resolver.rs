//! Command-time evaluation of compiler-projected effective-fact claims.

use std::collections::BTreeSet;

use nomos_core::{Diagnostic, Ident, NamespaceId};
use nomos_projection::{
    LightProjectionConsumer, MachineDefinition, MovementClaim, MovementConnectivity,
    MovementDisposition, ResolvedLight, ResolvedLightFacts, ResolvedMovement,
    ResolvedMovementFacts, SimulationPlan, activation_is_true,
};

use crate::SimulationState;

/// Resolves effective ground movement facts from immutable projected state.
///
/// # Errors
///
/// Returns a stable `EK09xx` diagnostic when a malicious projection omits a
/// referenced machine/state, changes the accepted composition/coherence rules,
/// or would produce an invalid disposition.
pub fn resolve_movement(
    plan: &SimulationPlan,
    state: &SimulationState,
) -> Result<ResolvedMovementFacts, Diagnostic> {
    let resolver = plan.movement_resolver();
    if resolver.channel().as_str() != "ground"
        || !resolver.blockers_any_active()
        || !resolver.costs_maximum_active()
        || !resolver.blockers_before_cost()
        || !resolver.requires_connectivity()
        || resolver.base_cost() == 0
    {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
            "runtime received a movement plan outside the Gate K resolver contract",
        ));
    }

    let state_equals = machine_state_lookup(plan, state);
    let mut facts = Vec::new();
    for subject in resolver.subjects() {
        validate_connectivity(subject.connectivity())?;
        let mut blockers = Vec::new();
        let mut active_costs = Vec::new();
        for claim in subject.claims() {
            if !activation_is_true(claim.activation(), &state_equals)? {
                continue;
            }
            match claim {
                MovementClaim::Blocker { id, value, .. } if *value => blockers.push(id.clone()),
                MovementClaim::TraversalCost { id, cost, .. } => {
                    active_costs.push((id.clone(), *cost));
                }
                MovementClaim::Blocker { .. } => {}
            }
        }
        let disposition = if blockers.is_empty() {
            let maximum = active_costs
                .iter()
                .map(|(_, cost)| *cost)
                .max()
                .unwrap_or(resolver.base_cost());
            let reasons = active_costs
                .into_iter()
                .filter(|(_, cost)| *cost == maximum)
                .map(|(id, _)| id)
                .collect();
            MovementDisposition::traversable(maximum, reasons)?
        } else {
            MovementDisposition::blocked(blockers)?
        };
        facts.push(ResolvedMovement::new(subject.entity().clone(), disposition));
    }
    ResolvedMovementFacts::new(facts)
}

/// Resolves effective light facts from immutable projected state.
///
/// # Errors
///
/// Returns a stable `EK09xx` diagnostic when the plan contradicts the Gate K
/// union law, carries a negative claim, or has a dangling activation.
pub fn resolve_light(
    plan: &SimulationPlan,
    state: &SimulationState,
) -> Result<ResolvedLightFacts, Diagnostic> {
    let resolver = plan.light_resolver();
    let expected_consumers: BTreeSet<_> = [
        LightProjectionConsumer::Diagnostics,
        LightProjectionConsumer::Persistence,
        LightProjectionConsumer::Simulation,
    ]
    .into_iter()
    .collect();
    if !resolver.union_active()
        || resolver
            .consumers()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            != expected_consumers
    {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::LIGHT_RESOLVER_PLAN_INVALID,
            "runtime received a light plan outside the Gate K union contract",
        ));
    }

    let state_equals = machine_state_lookup(plan, state);
    let mut facts = Vec::new();
    for subject in resolver.subjects() {
        let mut reasons = Vec::new();
        for claim in subject.claims() {
            if !claim.value() {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::LIGHT_CLAIM_INVALID,
                    format!(
                        "projected light claim `{}` contradicts positive-only union semantics",
                        claim.id()
                    ),
                ));
            }
            if activation_is_true(claim.activation(), &state_equals)? {
                reasons.push(claim.id().clone());
            }
        }
        facts.push(ResolvedLight::new(
            subject.entity().clone(),
            !reasons.is_empty(),
            reasons,
        )?);
    }
    ResolvedLightFacts::new(facts)
}

fn validate_connectivity(connectivity: &MovementConnectivity) -> Result<(), Diagnostic> {
    let valid = match connectivity {
        MovementConnectivity::FaceAdjacent { first, second } => {
            let dx = i64::from(first.x()) - i64::from(second.x());
            let dy = i64::from(first.y()) - i64::from(second.y());
            first.z() == second.z() && dx.abs() + dy.abs() == 1
        }
        MovementConnectivity::Region { min, max } => {
            min.x() <= max.x() && min.y() <= max.y() && min.z() <= max.z()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RESOLVER_CONNECTIVITY_INVALID,
            "runtime received invalid projected ground connectivity",
        ))
    }
}

/// The runtime state lookup `nomos_projection::activation_is_true` consumes.
///
/// It owns the three diagnostics the runtime resolver has always emitted for a
/// projection that names something it does not carry: a namespace absent from
/// the plan, a state absent from that machine, and a namespace absent from the
/// current state. Issue #136 moved the evaluator, not these messages.
fn machine_state_lookup<'a>(
    plan: &'a SimulationPlan,
    state: &'a SimulationState,
) -> impl Fn(&NamespaceId, &Ident) -> Result<bool, Diagnostic> + 'a {
    move |namespace, required| {
        let machine = find_machine(plan, namespace).ok_or_else(|| {
            missing_reference(format!(
                "claim activation namespace `{namespace}` is absent from the simulation plan"
            ))
        })?;
        if !machine.states().contains(required) {
            return Err(missing_reference(format!(
                "claim activation state `{required}` is absent from `{namespace}`"
            )));
        }
        let current = state.machine(namespace).ok_or_else(|| {
            missing_reference(format!(
                "claim activation namespace `{namespace}` is absent from current state"
            ))
        })?;
        Ok(current == required)
    }
}

fn find_machine<'a>(
    plan: &'a SimulationPlan,
    namespace: &NamespaceId,
) -> Option<&'a MachineDefinition> {
    plan.machines()
        .binary_search_by(|machine| machine.namespace().cmp(namespace))
        .ok()
        .map(|index| &plan.machines()[index])
}

fn missing_reference(message: String) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RESOLVER_RUNTIME_REFERENCE_MISSING,
        message,
    )
}
