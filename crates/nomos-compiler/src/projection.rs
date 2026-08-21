//! Validation and projection from construction IR to the simulation plan.

use std::collections::{BTreeMap, BTreeSet};

use nomos_core::{ClaimRef, Diagnostic, EntityId, Ident, NamespaceId};
use nomos_projection::{
    CausalEdge, CommandRequirement, CommandTransition, DiagnosticsPlan, EventHandler, EventPayload,
    LatticeCell, LightClaim, LightProjectionConsumer,
    LightResolverPlan as ProjectedLightResolverPlan, LightSubject, MachineDefinition,
    MovementClaim, MovementConnectivity, MovementResolverPlan as ProjectedMovementResolverPlan,
    MovementSubject, NavigationPlan, PersistencePlan, Phase, ProjectedActivation,
    ProjectedDirection, ProjectedEntity, RuntimeBinding, SimulationPlan,
    validate_light_projection_agreement,
};
use nomos_schema::{
    Binding, CapabilityKind, ClaimActivation, ClaimTemplate, ClaimValue, Direction,
    GroundConnectivity, InteractionPhase, LightCompositionLaw, MovementCompositionLaw,
    ProjectionConsumer, TransitionDefinition, TransitionInput, TransitionTrigger, WorldIr,
};

pub(crate) fn simulation_plan(ir: &WorldIr) -> Result<SimulationPlan, Diagnostic> {
    let movement_resolver = movement_plan(ir)?;
    let light_resolver = light_plan(ir)?;
    let entities = project_entities(ir, false)?;
    let mut machines = Vec::new();
    let mut edges = Vec::new();

    for entity in ir.entities() {
        for machine in entity.expansion().machines() {
            validate_machine_states(
                machine.namespace(),
                machine.states(),
                machine.initial(),
                machine.transitions(),
            )?;
            let mut commands = Vec::new();
            let mut handlers = Vec::new();
            for transition in machine.transitions() {
                match transition.trigger() {
                    TransitionTrigger::Command { action, input } => {
                        commands.push(CommandTransition::new(
                            action.clone(),
                            command_requirement(input, entity.credential())?,
                            transition.source().clone(),
                            transition.target().clone(),
                        ));
                    }
                    TransitionTrigger::Event { handler, input } => {
                        handlers.push(EventHandler::new(
                            handler.clone(),
                            event_payload(input)?,
                            transition.source().clone(),
                            transition.target().clone(),
                        ));
                    }
                }
            }
            machines.push(MachineDefinition::new(
                machine.namespace().clone(),
                machine.states().to_vec(),
                machine.initial().clone(),
                commands,
                handlers,
            )?);
        }

        for interaction in entity.expansion().interactions() {
            let phase = match interaction.phase() {
                InteractionPhase::Causal => Phase::Causal,
            };
            edges.push(CausalEdge::new(
                interaction.trigger().namespace().clone(),
                interaction.trigger().state().clone(),
                phase,
                interaction.target_namespace().clone(),
                interaction.target_handler().clone(),
                event_payload(interaction.payload())?,
            ));
        }
    }

    let plan = SimulationPlan::new(machines, edges)?
        .with_entities(entities)?
        .with_movement_resolver(movement_resolver)
        .with_light_resolver(light_resolver);
    validate_references(&plan)?;
    reject_cycles(&plan)?;
    Ok(plan)
}

pub(crate) fn navigation_plan(ir: &WorldIr) -> Result<NavigationPlan, Diagnostic> {
    Ok(NavigationPlan::new(movement_plan(ir)?))
}

pub(crate) fn persistence_plan(ir: &WorldIr) -> Result<PersistencePlan, Diagnostic> {
    PersistencePlan::new(project_entities(ir, true)?, light_plan(ir)?)
}

pub(crate) fn diagnostics_plan(ir: &WorldIr) -> Result<DiagnosticsPlan, Diagnostic> {
    DiagnosticsPlan::new(project_entities(ir, false)?, light_plan(ir)?)
}

pub(crate) fn validate_all_light_projections(ir: &WorldIr) -> Result<(), Diagnostic> {
    let simulation = simulation_plan(ir)?;
    let persistence = persistence_plan(ir)?;
    let diagnostics = diagnostics_plan(ir)?;
    validate_light_projection_agreement(simulation.light_resolver(), &persistence, &diagnostics)
}

fn light_plan(ir: &WorldIr) -> Result<ProjectedLightResolverPlan, Diagnostic> {
    validate_light_resolver_shape(ir)?;
    let machines: BTreeMap<&NamespaceId, &nomos_schema::MachineTemplate> = ir
        .entities()
        .iter()
        .flat_map(|entity| entity.expansion().machines())
        .map(|machine| (machine.namespace(), machine))
        .collect();
    let entities: BTreeMap<&EntityId, &nomos_schema::IrEntity> = ir
        .entities()
        .iter()
        .map(|entity| (entity.id(), entity))
        .collect();
    let consumers = ir
        .light_resolver()
        .consumers()
        .iter()
        .map(|consumer| match consumer {
            ProjectionConsumer::Diagnostics => LightProjectionConsumer::Diagnostics,
            ProjectionConsumer::Persistence => LightProjectionConsumer::Persistence,
            ProjectionConsumer::Simulation => LightProjectionConsumer::Simulation,
            ProjectionConsumer::Navigation => unreachable!("shape validation rejects navigation"),
        })
        .collect();
    let mut subjects = Vec::new();
    for subject in ir.light_resolver().subjects() {
        let entity = entities.get(subject.entity()).ok_or_else(|| {
            Diagnostic::new(
                nomos_core::diagnostic::codes::LIGHT_RESOLVER_PLAN_INVALID,
                format!(
                    "light resolver subject `{}` has no world entity",
                    subject.entity()
                ),
            )
        })?;
        let claims: BTreeMap<&ClaimRef, &ClaimTemplate> = entity
            .expansion()
            .claims()
            .iter()
            .map(|claim| (claim.id(), claim))
            .collect();
        let expected: BTreeSet<ClaimRef> = claims
            .values()
            .filter(|claim| claim.capability() == CapabilityKind::EmitsLight)
            .map(|claim| claim.id().clone())
            .collect();
        let declared: BTreeSet<ClaimRef> = subject.claims().iter().cloned().collect();
        if declared != expected {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::LIGHT_RESOLVER_PLAN_INVALID,
                format!(
                    "light resolver subject `{}` does not name exactly its light claims",
                    subject.entity()
                ),
            ));
        }
        let mut projected_claims = Vec::new();
        for claim_ref in subject.claims() {
            let claim = claims.get(claim_ref).ok_or_else(|| {
                Diagnostic::new(
                    nomos_core::diagnostic::codes::LIGHT_RESOLVER_PLAN_INVALID,
                    format!("light claim `{claim_ref}` does not exist"),
                )
            })?;
            validate_activation(claim.activation(), &machines)?;
            let ClaimValue::Bool(value) = claim.value() else {
                return Err(invalid_light_claim(claim.id()));
            };
            if !value {
                return Err(invalid_light_claim(claim.id()));
            }
            projected_claims.push(LightClaim::new(
                claim.id().clone(),
                project_activation(claim.activation()),
                *value,
                entity.source_span().clone(),
            ));
        }
        subjects.push(LightSubject::new(
            subject.entity().clone(),
            projected_claims,
        )?);
    }
    ProjectedLightResolverPlan::new(true, consumers, subjects)
}

fn validate_light_resolver_shape(ir: &WorldIr) -> Result<(), Diagnostic> {
    let plan = ir.light_resolver();
    if plan.law() != LightCompositionLaw::Union {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::LIGHT_RESOLVER_PLAN_INVALID,
            "light resolver must compose active claims by union",
        ));
    }
    let expected_consumers: BTreeSet<_> = [
        ProjectionConsumer::Diagnostics,
        ProjectionConsumer::Persistence,
        ProjectionConsumer::Simulation,
    ]
    .into_iter()
    .collect();
    if plan.consumers().iter().copied().collect::<BTreeSet<_>>() != expected_consumers {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::LIGHT_RESOLVER_PLAN_INVALID,
            "light resolver must name exactly simulation, persistence, and diagnostics consumers",
        ));
    }
    let expected_subjects: BTreeSet<EntityId> = ir
        .entities()
        .iter()
        .filter(|entity| {
            entity
                .expansion()
                .claims()
                .iter()
                .any(|claim| claim.capability() == CapabilityKind::EmitsLight)
        })
        .map(|entity| entity.id().clone())
        .collect();
    let declared_subjects: BTreeSet<EntityId> = plan
        .subjects()
        .iter()
        .map(|subject| subject.entity().clone())
        .collect();
    if expected_subjects != declared_subjects {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::LIGHT_RESOLVER_PLAN_INVALID,
            "light resolver subjects do not match entities with light claims",
        ));
    }
    Ok(())
}

fn invalid_light_claim(claim: &ClaimRef) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::LIGHT_CLAIM_INVALID,
        format!("light claim `{claim}` must supply the positive boolean value `true`"),
    )
}

fn project_entities(
    ir: &WorldIr,
    persisted_only: bool,
) -> Result<Vec<ProjectedEntity>, Diagnostic> {
    ir.entities()
        .iter()
        .filter(|entity| {
            !persisted_only
                || entity
                    .expansion()
                    .capabilities()
                    .contains(&CapabilityKind::Persisted)
        })
        .map(|entity| {
            ProjectedEntity::new(
                entity.id().clone(),
                project_binding(entity.binding()),
                entity
                    .expansion()
                    .machines()
                    .iter()
                    .map(|machine| machine.namespace().clone())
                    .collect(),
            )
        })
        .collect()
}

fn project_binding(binding: &Binding) -> RuntimeBinding {
    match binding {
        Binding::Cell(cell) => RuntimeBinding::Cell(project_cell(*cell)),
        Binding::Face { cell, direction } => RuntimeBinding::Face {
            cell: project_cell(*cell),
            direction: match direction {
                Direction::North => ProjectedDirection::North,
                Direction::East => ProjectedDirection::East,
                Direction::South => ProjectedDirection::South,
                Direction::West => ProjectedDirection::West,
                Direction::Up => ProjectedDirection::Up,
                Direction::Down => ProjectedDirection::Down,
            },
        },
        Binding::Region { min, max } => RuntimeBinding::Region {
            min: project_cell(*min),
            max: project_cell(*max),
        },
    }
}

fn movement_plan(ir: &WorldIr) -> Result<ProjectedMovementResolverPlan, Diagnostic> {
    validate_resolver_shape(ir)?;
    let machines: BTreeMap<&NamespaceId, &nomos_schema::MachineTemplate> = ir
        .entities()
        .iter()
        .flat_map(|entity| entity.expansion().machines())
        .map(|machine| (machine.namespace(), machine))
        .collect();
    let entities: BTreeMap<&EntityId, &nomos_schema::IrEntity> = ir
        .entities()
        .iter()
        .map(|entity| (entity.id(), entity))
        .collect();
    let mut projected_subjects = Vec::new();
    for subject in ir.movement_resolver().subjects() {
        let entity = entities.get(subject.entity()).ok_or_else(|| {
            Diagnostic::new(
                nomos_core::diagnostic::codes::RESOLVER_CLAIM_ENTITY_MISMATCH,
                format!(
                    "movement resolver subject `{}` has no world entity",
                    subject.entity()
                ),
            )
        })?;
        let expected_connectivity = crate::resolver::derive_connectivity(entity.binding())?;
        if subject.connectivity() != &expected_connectivity {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RESOLVER_CONNECTIVITY_INVALID,
                format!(
                    "movement connectivity for `{}` does not match its lattice binding",
                    subject.entity()
                ),
            ));
        }
        let claims: BTreeMap<&ClaimRef, &ClaimTemplate> = entity
            .expansion()
            .claims()
            .iter()
            .map(|claim| (claim.id(), claim))
            .collect();
        let expected: BTreeSet<ClaimRef> = claims
            .values()
            .filter(|claim| is_movement_capability(claim.capability()))
            .map(|claim| claim.id().clone())
            .collect();
        let declared: BTreeSet<ClaimRef> = subject.claims().iter().cloned().collect();
        if declared != expected {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
                format!(
                    "movement resolver subject `{}` does not name exactly its movement claims",
                    subject.entity()
                ),
            ));
        }
        let mut projected_claims = Vec::new();
        for claim_ref in subject.claims() {
            let claim = claims.get(claim_ref).ok_or_else(|| {
                Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
                    format!("movement claim `{claim_ref}` does not exist"),
                )
            })?;
            validate_activation(claim.activation(), &machines)?;
            let activation = project_activation(claim.activation());
            projected_claims.push(match (claim.capability(), claim.value()) {
                (CapabilityKind::BlocksGround, ClaimValue::Bool(value)) => MovementClaim::blocker(
                    claim.id().clone(),
                    activation,
                    *value,
                    entity.source_span().clone(),
                ),
                (CapabilityKind::TraversalCostGround, ClaimValue::Uint(cost)) if *cost > 0 => {
                    MovementClaim::traversal_cost(
                        claim.id().clone(),
                        activation,
                        *cost,
                        entity.source_span().clone(),
                    )?
                }
                (CapabilityKind::BlocksGround | CapabilityKind::TraversalCostGround, _) => {
                    return Err(Diagnostic::new(
                        nomos_core::diagnostic::codes::RESOLVER_CLAIM_VALUE_INVALID,
                        format!(
                            "movement claim `{}` has the wrong value kind or a zero cost",
                            claim.id()
                        ),
                    ));
                }
                _ => {
                    return Err(Diagnostic::new(
                        nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
                        format!(
                            "non-movement claim `{}` entered the movement plan",
                            claim.id()
                        ),
                    ));
                }
            });
        }
        projected_subjects.push(MovementSubject::new(
            subject.entity().clone(),
            project_connectivity(subject.connectivity()),
            projected_claims,
        )?);
    }
    let coherence = &ir.movement_resolver().coherence()[0];
    ProjectedMovementResolverPlan::new(
        coherence.channel().clone(),
        coherence.base_cost(),
        true,
        true,
        true,
        coherence.requires_connectivity(),
        projected_subjects,
    )
}

fn validate_resolver_shape(ir: &WorldIr) -> Result<(), Diagnostic> {
    let plan = ir.movement_resolver();
    let expected_laws: BTreeSet<_> = [
        MovementCompositionLaw::AnyActiveBlocker,
        MovementCompositionLaw::MaximumActiveCost,
    ]
    .into_iter()
    .collect();
    if plan.laws().iter().copied().collect::<BTreeSet<_>>() != expected_laws {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
            "movement resolver must declare exactly any-active blockers and maximum-active costs",
        ));
    }
    let [coherence] = plan.coherence() else {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
            "movement resolver must declare exactly one ground coherence rule",
        ));
    };
    if coherence.channel().as_str() != "ground"
        || coherence.base_cost() == 0
        || !coherence.requires_connectivity()
    {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
            "ground coherence must require connectivity and a positive base cost",
        ));
    }
    let expected_subjects: BTreeSet<EntityId> = ir
        .entities()
        .iter()
        .filter(|entity| {
            entity
                .expansion()
                .claims()
                .iter()
                .any(|claim| is_movement_capability(claim.capability()))
        })
        .map(|entity| entity.id().clone())
        .collect();
    let declared_subjects: BTreeSet<EntityId> = plan
        .subjects()
        .iter()
        .map(|subject| subject.entity().clone())
        .collect();
    if expected_subjects != declared_subjects {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
            "movement resolver subjects do not match entities with movement claims",
        ));
    }
    Ok(())
}

fn validate_activation(
    activation: &ClaimActivation,
    machines: &BTreeMap<&NamespaceId, &nomos_schema::MachineTemplate>,
) -> Result<(), Diagnostic> {
    match activation {
        ClaimActivation::Always => Ok(()),
        ClaimActivation::StateEquals { namespace, state } => {
            let machine = machines.get(namespace).ok_or_else(|| {
                Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_ACTIVATION_NAMESPACE_MISSING,
                    format!("claim activation namespace `{namespace}` does not exist"),
                )
            })?;
            if !machine.states().contains(state) {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_ACTIVATION_STATE_MISSING,
                    format!("claim activation state `{state}` does not exist in `{namespace}`"),
                ));
            }
            Ok(())
        }
        ClaimActivation::Any(children) | ClaimActivation::All(children) => {
            if children.is_empty() {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
                    "claim activation groups must not be empty",
                ));
            }
            for child in children {
                validate_activation(child, machines)?;
            }
            Ok(())
        }
        ClaimActivation::Not(child) => validate_activation(child, machines),
    }
}

fn project_activation(activation: &ClaimActivation) -> ProjectedActivation {
    match activation {
        ClaimActivation::Always => ProjectedActivation::Always,
        ClaimActivation::StateEquals { namespace, state } => ProjectedActivation::StateEquals {
            namespace: namespace.clone(),
            state: state.clone(),
        },
        ClaimActivation::Any(children) => {
            ProjectedActivation::Any(children.iter().map(project_activation).collect())
        }
        ClaimActivation::All(children) => {
            ProjectedActivation::All(children.iter().map(project_activation).collect())
        }
        ClaimActivation::Not(child) => {
            ProjectedActivation::Not(Box::new(project_activation(child)))
        }
    }
}

fn project_connectivity(connectivity: &GroundConnectivity) -> MovementConnectivity {
    match connectivity {
        GroundConnectivity::FaceAdjacent { first, second } => MovementConnectivity::FaceAdjacent {
            first: project_cell(*first),
            second: project_cell(*second),
        },
        GroundConnectivity::Region { min, max } => MovementConnectivity::Region {
            min: project_cell(*min),
            max: project_cell(*max),
        },
    }
}

fn project_cell(cell: nomos_schema::Cell) -> LatticeCell {
    LatticeCell::new(cell.x(), cell.y(), cell.z())
}

fn is_movement_capability(capability: CapabilityKind) -> bool {
    matches!(
        capability,
        CapabilityKind::BlocksGround | CapabilityKind::TraversalCostGround
    )
}

fn validate_machine_states(
    namespace: &NamespaceId,
    states: &[Ident],
    initial: &Ident,
    transitions: &[TransitionDefinition],
) -> Result<(), Diagnostic> {
    let states: BTreeSet<&Ident> = states.iter().collect();
    if !states.contains(initial) {
        return Err(missing_state(namespace, initial, "initial"));
    }
    for transition in transitions {
        if !states.contains(transition.source()) {
            return Err(missing_state(namespace, transition.source(), "source"));
        }
        if !states.contains(transition.target()) {
            return Err(missing_state(namespace, transition.target(), "target"));
        }
    }
    Ok(())
}

fn command_requirement(
    input: &TransitionInput,
    credential: Option<&nomos_core::CatalogValueId>,
) -> Result<CommandRequirement, Diagnostic> {
    match input {
        TransitionInput::None => Ok(CommandRequirement::None),
        TransitionInput::ResolvedEntityCredential => credential
            .cloned()
            .map(CommandRequirement::Credential)
            .ok_or_else(|| {
                Diagnostic::new(
                    nomos_core::diagnostic::codes::TRANSITION_INPUT_INVALID,
                    "credential-gated transition belongs to an entity with no resolved credential",
                )
            }),
        TransitionInput::Damage { .. } => Err(Diagnostic::new(
            nomos_core::diagnostic::codes::TRANSITION_INPUT_INVALID,
            "an external command cannot accept an internal damage payload",
        )),
    }
}

fn event_payload(input: &TransitionInput) -> Result<EventPayload, Diagnostic> {
    match input {
        TransitionInput::Damage { channel, amount } => Ok(EventPayload::Damage {
            channel: channel.clone(),
            amount: *amount,
        }),
        TransitionInput::None | TransitionInput::ResolvedEntityCredential => Err(Diagnostic::new(
            nomos_core::diagnostic::codes::TRANSITION_INPUT_INVALID,
            "an internal Gate K handler requires a typed damage payload",
        )),
    }
}

fn validate_references(plan: &SimulationPlan) -> Result<(), Diagnostic> {
    let machines: BTreeMap<&NamespaceId, &MachineDefinition> = plan
        .machines()
        .iter()
        .map(|machine| (machine.namespace(), machine))
        .collect();
    for edge in plan.causal_edges() {
        let Some(source) = machines.get(edge.source_namespace()) else {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::TRANSITION_SOURCE_NAMESPACE_MISSING,
                format!(
                    "interaction source namespace `{}` does not exist",
                    edge.source_namespace()
                ),
            ));
        };
        if !source.states().contains(edge.entered_state()) {
            return Err(missing_state(
                edge.source_namespace(),
                edge.entered_state(),
                "on_enter trigger",
            ));
        }
        let Some(target) = machines.get(edge.target_namespace()) else {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::INTERACTION_TARGET_NAMESPACE_MISSING,
                format!(
                    "interaction target namespace `{}` does not exist",
                    edge.target_namespace()
                ),
            ));
        };
        if !target.handlers().iter().any(|handler| {
            handler.name() == edge.target_handler() && handler.payload() == edge.payload()
        }) {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::INTERACTION_HANDLER_MISSING,
                format!(
                    "interaction target `{}` has no matching `{}` handler",
                    edge.target_namespace(),
                    edge.target_handler()
                ),
            ));
        }
    }
    Ok(())
}

fn reject_cycles(plan: &SimulationPlan) -> Result<(), Diagnostic> {
    let machines: BTreeMap<&NamespaceId, &MachineDefinition> = plan
        .machines()
        .iter()
        .map(|machine| (machine.namespace(), machine))
        .collect();
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in plan.causal_edges() {
        let source = event_node(edge.source_namespace(), edge.entered_state());
        graph.entry(source.clone()).or_default();
        let target = machines
            .get(edge.target_namespace())
            .expect("reference validation precedes cycle validation");
        for handler in target.handlers().iter().filter(|handler| {
            handler.name() == edge.target_handler() && handler.payload() == edge.payload()
        }) {
            let entered = event_node(edge.target_namespace(), handler.target());
            graph.entry(entered.clone()).or_default();
            graph.entry(source.clone()).or_default().insert(entered);
        }
    }

    let mut indegree: BTreeMap<String, usize> =
        graph.keys().cloned().map(|node| (node, 0)).collect();
    for targets in graph.values() {
        for target in targets {
            *indegree
                .get_mut(target)
                .expect("all graph targets are registered") += 1;
        }
    }
    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(node, _)| node.clone())
        .collect();
    let mut visited = 0;
    while let Some(node) = ready.pop_first() {
        visited += 1;
        for target in &graph[&node] {
            let count = indegree
                .get_mut(target)
                .expect("all graph targets have indegree");
            *count -= 1;
            if *count == 0 {
                ready.insert(target.clone());
            }
        }
    }
    if visited != graph.len() {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::INTERACTION_CYCLE,
            "causal interaction graph contains a cycle and Gate K defines no fixed point",
        ));
    }
    Ok(())
}

fn event_node(namespace: &NamespaceId, state: &Ident) -> String {
    format!("{namespace}#on_enter#{state}")
}

fn missing_state(namespace: &NamespaceId, state: &Ident, role: &str) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::TRANSITION_STATE_MISSING,
        format!("{role} state `{state}` does not exist in machine `{namespace}`"),
    )
}
