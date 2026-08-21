//! SW-D runtime proof for immutable atomic transaction preparation.

use nomos_core::{CatalogValueId, EntityId, Ident, NamespaceId};
use nomos_projection::{
    CausalEdge, Command, CommandArgument, CommandRequirement, CommandTransition, EventHandler,
    EventPayload, LatticeCell, MachineDefinition, Phase, ProjectedEntity, RuntimeBinding,
    SimulationPlan,
};
use nomos_sim::{
    SimulationState, TransitionCause, commit_transaction, commit_transaction_with_budget,
    prepare_transaction,
};

fn ident(value: &str) -> Ident {
    Ident::new(value).unwrap()
}

fn namespace(entity: &str, local: &str) -> NamespaceId {
    NamespaceId::new(EntityId::parse(entity).unwrap(), ident(local))
}

fn credential(value: &str) -> CatalogValueId {
    CatalogValueId::parse(value).unwrap()
}

fn command(namespace: NamespaceId, action: &str, argument: CommandArgument) -> Command {
    Command::new(namespace, ident(action), argument)
}

#[test]
fn ignite_stages_local_then_target_owned_fire_damage_exactly_once() {
    let plan = gate_plan();
    let state = SimulationState::initialize(&plan).unwrap();
    let original = state.clone();
    let before = state.to_canonical_bytes();
    let prepared = prepare_transaction(
        &plan,
        &state,
        &command(
            namespace("north_gate", "combustion"),
            "ignite",
            CommandArgument::None,
        ),
    )
    .unwrap();

    assert_eq!(
        state.to_canonical_bytes(),
        before,
        "input state is immutable"
    );
    assert_eq!(state, original);
    assert_eq!(prepared.steps().len(), 2);
    assert_eq!(prepared.steps()[0].phase(), Phase::Local);
    assert_eq!(
        prepared.steps()[0].namespace().to_string(),
        "north_gate.combustion"
    );
    assert_eq!(prepared.steps()[0].to().as_str(), "burning");
    assert_eq!(prepared.steps()[1].phase(), Phase::Causal);
    assert_eq!(
        prepared.steps()[1].namespace().to_string(),
        "north_gate.integrity"
    );
    assert_eq!(prepared.steps()[1].to().as_str(), "destroyed");
    assert!(matches!(
        prepared.steps()[1].cause(),
        TransitionCause::Event { handler, payload }
            if handler.as_str() == "apply_damage"
                && payload == &EventPayload::Damage {
                    channel: ident("fire"),
                    amount: 2,
                }
    ));
    assert_eq!(
        prepared
            .after()
            .machine(&namespace("north_gate", "integrity"))
            .unwrap()
            .as_str(),
        "destroyed"
    );
}

#[test]
fn repeated_ignite_is_rejected_without_refiring_or_mutating_state() {
    let plan = gate_plan();
    let initial = SimulationState::initialize(&plan).unwrap();
    let after_first = prepare_transaction(
        &plan,
        &initial,
        &command(
            namespace("north_gate", "combustion"),
            "ignite",
            CommandArgument::None,
        ),
    )
    .unwrap()
    .into_after();
    let original = after_first.clone();
    let before_retry = after_first.to_canonical_bytes();
    let rejected = prepare_transaction(
        &plan,
        &after_first,
        &command(
            namespace("north_gate", "combustion"),
            "ignite",
            CommandArgument::None,
        ),
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0804");
    assert_eq!(after_first.to_canonical_bytes(), before_retry);
    assert_eq!(after_first, original);
}

#[test]
fn credential_typing_and_argument_free_commands_fail_closed() {
    let plan = gate_plan();
    let state = SimulationState::initialize(&plan).unwrap();
    let access = namespace("north_gate", "access");
    for argument in [
        CommandArgument::None,
        CommandArgument::Credential(credential("credential/wrong_key")),
        CommandArgument::Event(fire_damage()),
    ] {
        let rejected =
            prepare_transaction(&plan, &state, &command(access.clone(), "unlock", argument))
                .unwrap_err();
        assert_eq!(rejected.code().as_str(), "EK0805");
    }

    let unlocked = prepare_transaction(
        &plan,
        &state,
        &command(
            access.clone(),
            "unlock",
            CommandArgument::Credential(credential("credential/gaoler_key")),
        ),
    )
    .unwrap()
    .into_after();
    let rejected = prepare_transaction(
        &plan,
        &unlocked,
        &command(
            access,
            "open",
            CommandArgument::Credential(credential("credential/gaoler_key")),
        ),
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0805");
}

#[test]
fn unlock_open_close_unseal_and_extinguish_change_only_their_machine() {
    let plan = gate_plan();
    let initial = SimulationState::initialize(&plan).unwrap();
    let access = namespace("north_gate", "access");
    let ward = namespace("north_gate", "ward");
    let emission = namespace("brazier_02", "emission");

    let unlocked = prepare_transaction(
        &plan,
        &initial,
        &command(
            access.clone(),
            "unlock",
            CommandArgument::Credential(credential("credential/gaoler_key")),
        ),
    )
    .unwrap()
    .into_after();
    assert_eq!(unlocked.machine(&access).unwrap().as_str(), "closed");
    let opened = prepare_transaction(
        &plan,
        &unlocked,
        &command(access.clone(), "open", CommandArgument::None),
    )
    .unwrap()
    .into_after();
    assert_eq!(opened.machine(&access).unwrap().as_str(), "open");
    assert_eq!(opened.machine(&ward).unwrap().as_str(), "sealed");
    let closed = prepare_transaction(
        &plan,
        &opened,
        &command(access.clone(), "close", CommandArgument::None),
    )
    .unwrap()
    .into_after();
    assert_eq!(closed.machine(&access).unwrap().as_str(), "closed");
    let unsealed = prepare_transaction(
        &plan,
        &closed,
        &command(ward.clone(), "unseal", CommandArgument::None),
    )
    .unwrap()
    .into_after();
    assert_eq!(unsealed.machine(&ward).unwrap().as_str(), "unsealed");
    assert_eq!(unsealed.machine(&access).unwrap().as_str(), "closed");
    let extinguished = prepare_transaction(
        &plan,
        &unsealed,
        &command(emission.clone(), "extinguish", CommandArgument::None),
    )
    .unwrap()
    .into_after();
    assert_eq!(
        extinguished.machine(&emission).unwrap().as_str(),
        "extinguished"
    );
    assert_eq!(extinguished.machine(&ward).unwrap().as_str(), "unsealed");
}

#[test]
fn undeclared_actions_and_external_internal_handlers_are_distinct() {
    let plan = gate_plan();
    let state = SimulationState::initialize(&plan).unwrap();
    let integrity = namespace("north_gate", "integrity");
    let undeclared = prepare_transaction(
        &plan,
        &state,
        &command(integrity.clone(), "repair", CommandArgument::None),
    )
    .unwrap_err();
    assert_eq!(undeclared.code().as_str(), "EK0802");
    let internal = prepare_transaction(
        &plan,
        &state,
        &command(
            integrity,
            "apply_damage",
            CommandArgument::Event(fire_damage()),
        ),
    )
    .unwrap_err();
    assert_eq!(internal.code().as_str(), "EK0803");
}

#[test]
fn missing_event_targets_and_handlers_discard_the_staged_local_change() {
    for (plan, code) in [
        (broken_event_plan(true), "EK0806"),
        (broken_event_plan(false), "EK0807"),
    ] {
        let state = SimulationState::initialize(&plan).unwrap();
        let original = state.clone();
        let before = state.to_canonical_bytes();
        let rejected = commit_transaction(
            &plan,
            &state,
            &command(
                namespace("source", "machine"),
                "start",
                CommandArgument::None,
            ),
        )
        .unwrap_err();
        assert_eq!(rejected.code().as_str(), code);
        assert_eq!(state.to_canonical_bytes(), before);
        assert_eq!(state, original);
    }
}

#[test]
fn transition_budget_stops_a_malicious_cyclic_projection_atomically() {
    let plan = cyclic_plan();
    let state = SimulationState::initialize(&plan).unwrap();
    let original = state.clone();
    let before = state.to_canonical_bytes();
    let rejected = commit_transaction_with_budget(
        &plan,
        &state,
        &command(
            namespace("cycle", "machine"),
            "start",
            CommandArgument::None,
        ),
        4,
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0808");
    assert_eq!(state.to_canonical_bytes(), before);
    assert_eq!(state, original);
}

fn gate_plan() -> SimulationPlan {
    let access = machine(
        namespace("north_gate", "access"),
        &["locked", "closed", "open"],
        "locked",
        vec![
            external(
                "unlock",
                CommandRequirement::Credential(credential("credential/gaoler_key")),
                "locked",
                "closed",
            ),
            external("open", CommandRequirement::None, "closed", "open"),
            external("close", CommandRequirement::None, "open", "closed"),
        ],
        Vec::new(),
    );
    let integrity = machine(
        namespace("north_gate", "integrity"),
        &["intact", "damaged", "destroyed"],
        "intact",
        Vec::new(),
        vec![handler("apply_damage", "intact", "destroyed")],
    );
    let combustion = machine(
        namespace("north_gate", "combustion"),
        &["cold", "burning", "spent"],
        "cold",
        vec![external(
            "ignite",
            CommandRequirement::None,
            "cold",
            "burning",
        )],
        Vec::new(),
    );
    let ward = machine(
        namespace("north_gate", "ward"),
        &["sealed", "unsealed"],
        "sealed",
        vec![external(
            "unseal",
            CommandRequirement::None,
            "sealed",
            "unsealed",
        )],
        Vec::new(),
    );
    let emission = machine(
        namespace("brazier_02", "emission"),
        &["lit", "extinguished"],
        "lit",
        vec![external(
            "extinguish",
            CommandRequirement::None,
            "lit",
            "extinguished",
        )],
        Vec::new(),
    );
    complete_plan(
        SimulationPlan::new(
            vec![access, integrity, combustion, ward, emission],
            vec![CausalEdge::new(
                namespace("north_gate", "combustion"),
                ident("burning"),
                Phase::Causal,
                namespace("north_gate", "integrity"),
                ident("apply_damage"),
                fire_damage(),
            )],
        )
        .unwrap(),
    )
}

fn broken_event_plan(missing_target: bool) -> SimulationPlan {
    let source = machine(
        namespace("source", "machine"),
        &["a", "b"],
        "a",
        vec![external("start", CommandRequirement::None, "a", "b")],
        Vec::new(),
    );
    let mut machines = vec![source];
    if !missing_target {
        machines.push(machine(
            namespace("target", "machine"),
            &["a", "b"],
            "a",
            Vec::new(),
            Vec::new(),
        ));
    }
    complete_plan(
        SimulationPlan::new(
            machines,
            vec![CausalEdge::new(
                namespace("source", "machine"),
                ident("b"),
                Phase::Causal,
                namespace("target", "machine"),
                ident("handle"),
                fire_damage(),
            )],
        )
        .unwrap(),
    )
}

fn cyclic_plan() -> SimulationPlan {
    let machine_namespace = namespace("cycle", "machine");
    let machine = machine(
        machine_namespace.clone(),
        &["a", "b"],
        "a",
        vec![external("start", CommandRequirement::None, "a", "b")],
        vec![
            handler_from("flip", "a", "b"),
            handler_from("flip", "b", "a"),
        ],
    );
    complete_plan(
        SimulationPlan::new(
            vec![machine],
            vec![
                CausalEdge::new(
                    machine_namespace.clone(),
                    ident("a"),
                    Phase::Causal,
                    machine_namespace.clone(),
                    ident("flip"),
                    fire_damage(),
                ),
                CausalEdge::new(
                    machine_namespace.clone(),
                    ident("b"),
                    Phase::Causal,
                    machine_namespace,
                    ident("flip"),
                    fire_damage(),
                ),
            ],
        )
        .unwrap(),
    )
}

fn complete_plan(plan: SimulationPlan) -> SimulationPlan {
    let mut by_entity = std::collections::BTreeMap::<EntityId, Vec<NamespaceId>>::new();
    for machine in plan.machines() {
        by_entity
            .entry(machine.namespace().entity().clone())
            .or_default()
            .push(machine.namespace().clone());
    }
    let entities = by_entity
        .into_iter()
        .map(|(entity, machines)| {
            ProjectedEntity::new(
                entity,
                RuntimeBinding::Cell(LatticeCell::new(0, 0, 0)),
                machines,
            )
            .unwrap()
        })
        .collect();
    plan.with_entities(entities).unwrap()
}

fn machine(
    namespace: NamespaceId,
    states: &[&str],
    initial: &str,
    commands: Vec<CommandTransition>,
    handlers: Vec<EventHandler>,
) -> MachineDefinition {
    MachineDefinition::new(
        namespace,
        states.iter().map(|state| ident(state)).collect(),
        ident(initial),
        commands,
        handlers,
    )
    .unwrap()
}

fn external(
    action: &str,
    requirement: CommandRequirement,
    source: &str,
    target: &str,
) -> CommandTransition {
    CommandTransition::new(ident(action), requirement, ident(source), ident(target))
}

fn handler(name: &str, source: &str, target: &str) -> EventHandler {
    handler_from(name, source, target)
}

fn handler_from(name: &str, source: &str, target: &str) -> EventHandler {
    EventHandler::new(ident(name), fire_damage(), ident(source), ident(target))
}

fn fire_damage() -> EventPayload {
    EventPayload::Damage {
        channel: ident("fire"),
        amount: 2,
    }
}
