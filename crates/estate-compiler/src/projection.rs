//! Validation and projection from construction IR to the simulation plan.

use std::collections::{BTreeMap, BTreeSet};

use estate_core::{Diagnostic, Ident, NamespaceId};
use estate_projection::{
    CausalEdge, CommandRequirement, CommandTransition, EventHandler, EventPayload,
    MachineDefinition, Phase, SimulationPlan,
};
use estate_schema::{
    InteractionPhase, TransitionDefinition, TransitionInput, TransitionTrigger, WorldIr,
};

pub(crate) fn simulation_plan(ir: &WorldIr) -> Result<SimulationPlan, Diagnostic> {
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

    let plan = SimulationPlan::new(machines, edges)?;
    validate_references(&plan)?;
    reject_cycles(&plan)?;
    Ok(plan)
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
    credential: Option<&estate_core::CatalogValueId>,
) -> Result<CommandRequirement, Diagnostic> {
    match input {
        TransitionInput::None => Ok(CommandRequirement::None),
        TransitionInput::ResolvedEntityCredential => credential
            .cloned()
            .map(CommandRequirement::Credential)
            .ok_or_else(|| {
                Diagnostic::new(
                    estate_core::diagnostic::codes::TRANSITION_INPUT_INVALID,
                    "credential-gated transition belongs to an entity with no resolved credential",
                )
            }),
        TransitionInput::Damage { .. } => Err(Diagnostic::new(
            estate_core::diagnostic::codes::TRANSITION_INPUT_INVALID,
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
            estate_core::diagnostic::codes::TRANSITION_INPUT_INVALID,
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
                estate_core::diagnostic::codes::TRANSITION_SOURCE_NAMESPACE_MISSING,
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
                estate_core::diagnostic::codes::INTERACTION_TARGET_NAMESPACE_MISSING,
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
                estate_core::diagnostic::codes::INTERACTION_HANDLER_MISSING,
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
            estate_core::diagnostic::codes::INTERACTION_CYCLE,
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
        estate_core::diagnostic::codes::TRANSITION_STATE_MISSING,
        format!("{role} state `{state}` does not exist in machine `{namespace}`"),
    )
}
