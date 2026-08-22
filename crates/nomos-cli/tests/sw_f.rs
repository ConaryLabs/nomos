//! SW-F vertical acceptance proof across compiler, projections, and runtime.

use nomos_compiler::{
    compile_diagnostics_plan, compile_navigation_plan, compile_persistence_plan,
    compile_simulation_plan, compile_world, validate_light_projections,
};
use nomos_core::hash::Sha256Digest;
use nomos_core::{CatalogValueId, EntityId, Ident, NamespaceId, SourcePath};
use nomos_projection::{
    Command, CommandArgument, Phase, SimulationPlan, diagnostics_schema, persistence_schema,
    simulation_schema,
};
use nomos_sim::{EffectiveFactRef, SimulationState, commit_transaction, resolve_light};

const SOURCE: &str = include_str!("../../../fixtures/gaol.nomos");

fn ident(value: &str) -> Ident {
    Ident::new(value).unwrap()
}

fn entity(value: &str) -> EntityId {
    EntityId::parse(value).unwrap()
}

fn namespace(entity: &str, local: &str) -> NamespaceId {
    NamespaceId::new(self::entity(entity), ident(local))
}

fn command(entity: &str, namespace: &str, action: &str) -> Command {
    Command::new(
        self::namespace(entity, namespace),
        ident(action),
        CommandArgument::None,
    )
}

fn fixture_plan() -> SimulationPlan {
    let ir = compile_world(SOURCE, SourcePath::new("fixtures/gaol.nomos").unwrap()).unwrap();
    validate_light_projections(&ir).unwrap();
    compile_simulation_plan(&ir).unwrap()
}

#[test]
fn extinguish_commits_versioned_state_hash_and_typed_projection_receipt() {
    let ir = compile_world(SOURCE, SourcePath::new("fixtures/gaol.nomos").unwrap()).unwrap();
    let plan = compile_simulation_plan(&ir).unwrap();
    let persistence = compile_persistence_plan(&ir).unwrap();
    let diagnostics = compile_diagnostics_plan(&ir).unwrap();
    let navigation = compile_navigation_plan(&ir).unwrap();
    assert_eq!(plan.schema(), &simulation_schema());
    assert_eq!(persistence.schema(), &persistence_schema());
    assert_eq!(diagnostics.schema(), &diagnostics_schema());
    assert_eq!(navigation.schema(), &nomos_projection::navigation_schema());
    assert_eq!(
        plan.light_resolver().to_canonical_bytes(),
        persistence.light_resolver().to_canonical_bytes()
    );
    assert_eq!(
        persistence.light_resolver().to_canonical_bytes(),
        diagnostics.light_resolver().to_canonical_bytes()
    );

    let initial = SimulationState::initialize(&plan).unwrap();
    assert_eq!(initial.schema(), &nomos_sim::runtime_state_schema());
    assert_eq!(initial.tick(), 0);
    assert_eq!(initial.entities().len(), 3);
    let initial_light = resolve_light(&plan, &initial).unwrap();
    let brazier = initial_light.get(&entity("brazier_02")).unwrap();
    assert!(brazier.emitting());
    assert_eq!(brazier.reasons().len(), 1);

    let input_bytes = initial.to_canonical_bytes();
    let input_copy = initial.clone();
    let committed = commit_transaction(
        &plan,
        &initial,
        &command("brazier_02", "emission", "extinguish"),
    )
    .unwrap();
    assert_eq!(initial, input_copy);
    assert_eq!(initial.to_canonical_bytes(), input_bytes);
    assert_eq!(committed.snapshot().tick(), 1);
    assert_eq!(
        committed
            .snapshot()
            .machine(&namespace("brazier_02", "emission"))
            .unwrap()
            .as_str(),
        "extinguished"
    );
    assert_eq!(
        committed
            .snapshot()
            .machine(&namespace("north_gate", "ward"))
            .unwrap()
            .as_str(),
        "sealed",
        "unrelated machine state is preserved"
    );

    let receipt = committed.receipt();
    assert_eq!(receipt.schema(), &nomos_sim::causal_receipt_schema());
    assert!(
        receipt
            .light_before()
            .get(&entity("brazier_02"))
            .unwrap()
            .emitting()
    );
    assert!(
        !receipt
            .light_after()
            .get(&entity("brazier_02"))
            .unwrap()
            .emitting()
    );
    assert_eq!(receipt.tick(), 1);
    assert_eq!(receipt.state_hash(), committed.state_hash());
    let targets = receipt
        .projection_deltas()
        .iter()
        .map(|delta| {
            assert!(matches!(
                delta.fact(),
                EffectiveFactRef::EmitsLight { entity } if entity == &self::entity("brazier_02")
            ));
            delta.projection().clone()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        targets,
        [
            diagnostics_schema(),
            persistence_schema(),
            simulation_schema()
        ]
    );

    let snapshot_bytes = committed.snapshot().to_canonical_bytes();
    assert!(nomos_core::canonical::read::parse_canonical(&snapshot_bytes).is_ok());
    assert_eq!(
        committed.state_hash().to_hex(),
        Sha256Digest::of_bytes(&snapshot_bytes).to_hex()
    );
    let snapshot_text = String::from_utf8(snapshot_bytes).unwrap();
    for excluded in [
        "fixtures/gaol.nomos",
        "source_span",
        "display",
        "build_path",
        "projection_cache",
        "cosmetic",
    ] {
        assert!(!snapshot_text.contains(excluded));
    }
}

#[test]
fn exact_command_sequence_is_byte_deterministic_from_one_initial_snapshot() {
    let plan = fixture_plan();
    let initial = SimulationState::initialize(&plan).unwrap();
    let first = run_sequence(&plan, &initial);
    let second = run_sequence(&plan, &initial);
    assert_eq!(first, second);
    assert_eq!(
        first
            .iter()
            .map(|(_, receipt, _)| Sha256Digest::of_bytes(receipt).to_hex())
            .collect::<Vec<_>>(),
        [
            "137aae97ca284b1d086d36b03967df22eccdb8c0787464b56d56eebf1f0b0fad",
            "02ae95579f61814b68a1735a08ab85c64a1662e81a52de61a8ca6961941fe538",
            "7105dd0ffdde41eef147b55115f4b617845db8f11c7ba161f2970bfd919a349f",
            "f41a0505df7937e99622f1e63fe910c4a926d0a571c4702b34d7f749aaba6842",
        ]
    );
    assert_eq!(
        first
            .iter()
            .map(|(_, _, state_hash)| state_hash.as_str())
            .collect::<Vec<_>>(),
        [
            "b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b",
            "0545806ca38848850f9e9ca674fa4891066ed57b72fa7a957fd66332cae7314b",
            "7843d53de4c8bb543a8ea11603d5ada260b549b18a5d5076e74e0e29b81a6e79",
            "1fa9f01245beae92a25aa96b3b74a4c8e5138ad4fbde545d3d765fc6597c1fde",
        ]
    );
    assert_eq!(initial.tick(), 0);
}

#[test]
fn rejected_commit_exposes_no_evidence_and_cannot_mutate_input() {
    let plan = fixture_plan();
    let initial = SimulationState::initialize(&plan).unwrap();
    let bytes = initial.to_canonical_bytes();
    for rejected in [
        Command::new(
            namespace("north_gate", "access"),
            ident("unlock"),
            CommandArgument::Credential(CatalogValueId::parse("credential/wrong_key").unwrap()),
        ),
        command("north_gate", "access", "missing"),
        command("north_gate", "integrity", "apply_damage"),
    ] {
        assert!(commit_transaction(&plan, &initial, &rejected).is_err());
        assert_eq!(initial.to_canonical_bytes(), bytes);
        assert_eq!(initial.tick(), 0);
    }

    let once = commit_transaction(
        &plan,
        &initial,
        &command("brazier_02", "emission", "extinguish"),
    )
    .unwrap();
    let committed_bytes = once.snapshot().to_canonical_bytes();
    let illegal = commit_transaction(
        &plan,
        once.snapshot(),
        &command("brazier_02", "emission", "extinguish"),
    )
    .unwrap_err();
    assert_eq!(illegal.code().as_str(), "EK0804");
    assert_eq!(once.snapshot().to_canonical_bytes(), committed_bytes);
}

#[test]
fn state_hash_tampering_is_detected_and_causal_steps_remain_ordered() {
    let plan = fixture_plan();
    let initial = SimulationState::initialize(&plan).unwrap();
    let extinguished = commit_transaction(
        &plan,
        &initial,
        &command("brazier_02", "emission", "extinguish"),
    )
    .unwrap();
    extinguished
        .snapshot()
        .verify_hash(extinguished.state_hash())
        .unwrap();

    let ignited = commit_transaction(
        &plan,
        extinguished.snapshot(),
        &command("north_gate", "combustion", "ignite"),
    )
    .unwrap();
    let mismatch = ignited
        .snapshot()
        .verify_hash(extinguished.state_hash())
        .unwrap_err();
    assert_eq!(mismatch.code().as_str(), "EK0810");
    assert_eq!(ignited.receipt().steps().len(), 2);
    assert_eq!(ignited.receipt().steps()[0].phase(), Phase::Local);
    assert_eq!(ignited.receipt().steps()[1].phase(), Phase::Causal);
}

fn run_sequence(
    plan: &SimulationPlan,
    initial: &SimulationState,
) -> Vec<(Vec<u8>, Vec<u8>, String)> {
    let commands = [
        Command::new(
            namespace("north_gate", "access"),
            ident("unlock"),
            CommandArgument::Credential(CatalogValueId::parse("credential/gaoler_key").unwrap()),
        ),
        command("north_gate", "access", "open"),
        command("north_gate", "ward", "unseal"),
        command("brazier_02", "emission", "extinguish"),
    ];
    let mut current = initial.clone();
    let mut evidence = Vec::new();
    for command in commands {
        let committed = commit_transaction(plan, &current, &command).unwrap();
        evidence.push((
            committed.snapshot().to_canonical_bytes(),
            committed.receipt().to_canonical_bytes(),
            committed.state_hash().to_hex(),
        ));
        current = committed.into_snapshot();
    }
    evidence
}
