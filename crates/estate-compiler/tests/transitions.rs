//! SW-D compiler proof for executable transitions and causal interactions.

use estate_compiler::{compile_simulation_plan, compile_source};
use estate_core::{EntityId, Ident, NamespaceId, PrimitiveKindId, SourcePath, SourceSpan};
use estate_projection::{CommandRequirement, EventPayload, Phase, SimulationPlan};
use estate_schema::{
    Binding, Cell, InteractionDefinition, InteractionPhase, InteractionTrigger, IrEntity,
    MachineTemplate, PrimitiveExpansion, TransitionDefinition, TransitionInput, TransitionTrigger,
    WorldIr, source_schema,
};

const SOURCE: &str = include_str!("../../../fixtures/gaol.estate");

fn ident(value: &str) -> Ident {
    Ident::new(value).unwrap()
}

fn namespace(entity: &str, local: &str) -> NamespaceId {
    NamespaceId::new(EntityId::parse(entity).unwrap(), ident(local))
}

fn fixture_ir() -> WorldIr {
    compile_source(SOURCE, SourcePath::new("fixtures/gaol.estate").unwrap()).unwrap()
}

#[test]
fn catalog_declares_the_exact_external_commands_and_internal_damage_handler() {
    let ir = fixture_ir();
    let mut external = Vec::new();
    let mut internal = Vec::new();
    for entity in ir.entities() {
        for machine in entity.expansion().machines() {
            for transition in machine.transitions() {
                let qualified = format!(
                    "{}.{}",
                    machine.namespace().local_name(),
                    transition.trigger().action()
                );
                if transition.trigger().is_external() {
                    external.push(qualified);
                } else {
                    internal.push(qualified);
                }
            }
        }
    }
    external.sort();
    internal.sort();
    assert_eq!(
        external,
        [
            "access.close",
            "access.open",
            "access.unlock",
            "combustion.ignite",
            "emission.extinguish",
            "ward.unseal",
        ]
    );
    assert_eq!(internal, ["integrity.apply_damage"]);
}

#[test]
fn construction_ir_encodes_the_exact_on_enter_damage_edge() {
    let ir = fixture_ir();
    let door = ir
        .entities()
        .iter()
        .find(|entity| entity.id().to_string() == "north_gate")
        .unwrap();
    let [edge] = door.expansion().interactions() else {
        panic!("the door has exactly one causal edge")
    };
    assert_eq!(edge.phase(), InteractionPhase::Causal);
    assert_eq!(
        edge.trigger().namespace().to_string(),
        "north_gate.combustion"
    );
    assert_eq!(edge.trigger().state().as_str(), "burning");
    assert_eq!(edge.target_namespace().to_string(), "north_gate.integrity");
    assert_eq!(edge.target_handler().as_str(), "apply_damage");
    assert_eq!(
        edge.payload(),
        &TransitionInput::Damage {
            channel: ident("fire"),
            amount: 2,
        }
    );
}

#[test]
fn projection_resolves_credentials_and_is_order_stable() {
    let plan = compile_simulation_plan(&fixture_ir()).unwrap();
    assert_eq!(plan.schema(), &estate_projection::simulation_schema());
    let access = plan
        .machines()
        .iter()
        .find(|machine| machine.namespace().to_string() == "north_gate.access")
        .unwrap();
    let unlock = access
        .commands()
        .iter()
        .find(|command| command.action().as_str() == "unlock")
        .unwrap();
    assert_eq!(
        unlock.requirement(),
        &CommandRequirement::Credential(
            estate_core::CatalogValueId::parse("credential/gaoler_key").unwrap()
        )
    );
    let [edge] = plan.causal_edges() else {
        panic!("the simulation projection contains one edge")
    };
    assert_eq!(edge.phase(), Phase::Causal);
    assert_eq!(
        edge.payload(),
        &EventPayload::Damage {
            channel: ident("fire"),
            amount: 2,
        }
    );

    let mut machines = plan.machines().to_vec();
    let mut edges = plan.causal_edges().to_vec();
    machines.reverse();
    edges.reverse();
    let reversed = SimulationPlan::new(machines, edges).unwrap();
    assert_eq!(plan.to_canonical_bytes(), reversed.to_canonical_bytes());
}

#[test]
fn construction_ir_transition_and_interaction_order_is_insertion_independent() {
    let machine_namespace = namespace("order_fixture", "machine");
    let first_transition = TransitionDefinition::new(
        TransitionTrigger::Command {
            action: ident("advance"),
            input: TransitionInput::None,
        },
        ident("a"),
        ident("b"),
    );
    let second_transition = TransitionDefinition::new(
        TransitionTrigger::Command {
            action: ident("retreat"),
            input: TransitionInput::None,
        },
        ident("b"),
        ident("a"),
    );
    let first_edge = edge(
        &machine_namespace,
        "a",
        "first_handler",
        TransitionInput::Damage {
            channel: ident("fire"),
            amount: 1,
        },
    );
    let second_edge = edge(
        &machine_namespace,
        "b",
        "second_handler",
        TransitionInput::Damage {
            channel: ident("fire"),
            amount: 2,
        },
    );
    let build = |transitions, interactions| {
        let machine = MachineTemplate::new(
            machine_namespace.clone(),
            vec![ident("a"), ident("b")],
            ident("a"),
        )
        .with_transitions(transitions)
        .unwrap();
        let expansion = PrimitiveExpansion::new(Vec::new(), vec![machine], Vec::new())
            .unwrap()
            .with_interactions(interactions)
            .unwrap();
        world(EntityId::parse("order_fixture").unwrap(), expansion)
    };
    let forward = build(
        vec![first_transition.clone(), second_transition.clone()],
        vec![first_edge.clone(), second_edge.clone()],
    );
    let reversed = build(
        vec![second_transition, first_transition],
        vec![second_edge, first_edge],
    );
    assert_eq!(forward.to_canonical_bytes(), reversed.to_canonical_bytes());
}

#[test]
fn compiler_rejects_dangling_namespaces_states_and_handlers() {
    let cases = [
        (
            invalid_ir(
                namespace("north_gate", "missing"),
                ident("burning"),
                namespace("north_gate", "integrity"),
                ident("apply_damage"),
            ),
            "EK0701",
        ),
        (
            invalid_ir(
                namespace("north_gate", "combustion"),
                ident("burning"),
                namespace("north_gate", "missing"),
                ident("apply_damage"),
            ),
            "EK0702",
        ),
        (
            invalid_ir(
                namespace("north_gate", "combustion"),
                ident("spent"),
                namespace("north_gate", "integrity"),
                ident("apply_damage"),
            ),
            "EK0703",
        ),
        (
            invalid_ir(
                namespace("north_gate", "combustion"),
                ident("burning"),
                namespace("north_gate", "integrity"),
                ident("missing"),
            ),
            "EK0706",
        ),
    ];
    for (ir, code) in cases {
        assert_eq!(
            compile_simulation_plan(&ir).unwrap_err().code().as_str(),
            code
        );
    }
}

#[test]
fn compiler_rejects_every_causal_cycle() {
    let entity_id = EntityId::parse("cycle_fixture").unwrap();
    let machine_namespace = namespace("cycle_fixture", "machine");
    let payload = TransitionInput::Damage {
        channel: ident("fire"),
        amount: 2,
    };
    let machine = MachineTemplate::new(
        machine_namespace.clone(),
        vec![ident("a"), ident("b")],
        ident("a"),
    )
    .with_transitions(vec![
        event("flip", payload.clone(), "a", "b"),
        event("flip", payload.clone(), "b", "a"),
    ])
    .unwrap();
    let expansion = PrimitiveExpansion::new(Vec::new(), vec![machine], Vec::new())
        .unwrap()
        .with_interactions(vec![
            edge(&machine_namespace, "a", "flip", payload.clone()),
            edge(&machine_namespace, "b", "flip", payload),
        ])
        .unwrap();
    let ir = world(entity_id, expansion);
    assert_eq!(
        compile_simulation_plan(&ir).unwrap_err().code().as_str(),
        "EK0707"
    );
}

#[test]
fn compiler_rejects_transition_states_absent_from_the_machine() {
    let entity_id = EntityId::parse("bad_transition").unwrap();
    let machine = MachineTemplate::new(
        namespace("bad_transition", "machine"),
        vec![ident("a"), ident("b")],
        ident("a"),
    )
    .with_transitions(vec![TransitionDefinition::new(
        TransitionTrigger::Command {
            action: ident("advance"),
            input: TransitionInput::None,
        },
        ident("missing"),
        ident("b"),
    )])
    .unwrap();
    let expansion = PrimitiveExpansion::new(Vec::new(), vec![machine], Vec::new()).unwrap();
    assert_eq!(
        compile_simulation_plan(&world(entity_id, expansion))
            .unwrap_err()
            .code()
            .as_str(),
        "EK0703"
    );
}

fn invalid_ir(
    source_namespace: NamespaceId,
    source_state: Ident,
    target_namespace: NamespaceId,
    target_handler: Ident,
) -> WorldIr {
    let entity_id = EntityId::parse("north_gate").unwrap();
    let payload = TransitionInput::Damage {
        channel: ident("fire"),
        amount: 2,
    };
    let combustion = MachineTemplate::new(
        namespace("north_gate", "combustion"),
        vec![ident("cold"), ident("burning")],
        ident("cold"),
    );
    let integrity = MachineTemplate::new(
        namespace("north_gate", "integrity"),
        vec![ident("intact"), ident("destroyed")],
        ident("intact"),
    )
    .with_transitions(vec![event(
        "apply_damage",
        payload.clone(),
        "intact",
        "destroyed",
    )])
    .unwrap();
    let expansion = PrimitiveExpansion::new(Vec::new(), vec![combustion, integrity], Vec::new())
        .unwrap()
        .with_interactions(vec![InteractionDefinition::new(
            InteractionTrigger::OnEnter {
                namespace: source_namespace,
                state: source_state,
            },
            InteractionPhase::Causal,
            target_namespace,
            target_handler,
            payload,
        )])
        .unwrap();
    world(entity_id, expansion)
}

fn world(entity_id: EntityId, expansion: PrimitiveExpansion) -> WorldIr {
    let entity = IrEntity::new(
        entity_id,
        PrimitiveKindId::parse("primitive/iron_barred_door").unwrap(),
        Binding::Cell(Cell::new(0, 0, 0)),
        None,
        expansion,
        SourceSpan::new(
            SourcePath::new("tests/transition.estate").unwrap(),
            0,
            1,
            1,
            1,
        )
        .unwrap(),
    );
    WorldIr::new(
        source_schema(),
        Vec::new(),
        vec![entity],
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
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

fn edge(
    namespace: &NamespaceId,
    state: &str,
    handler: &str,
    payload: TransitionInput,
) -> InteractionDefinition {
    InteractionDefinition::new(
        InteractionTrigger::OnEnter {
            namespace: namespace.clone(),
            state: ident(state),
        },
        InteractionPhase::Causal,
        namespace.clone(),
        ident(handler),
        payload,
    )
}
