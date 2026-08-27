mod common;

use std::collections::BTreeSet;

use nomos_core::Sha256Digest;
use nomos_observed_scene::{
    ActionMarker, ActorPose, Availability, LifeState, MaterialFamily, ObservedScene, Presence,
    ScenePlan, TerrainAssembly, TerrainRole, compile, input, plan,
};

#[test]
fn the_crate_owns_exactly_the_two_r2_schema_spellings() {
    assert_eq!(input::SCHEMA, "nomos.observed_scene@1");
    assert_eq!(plan::SCHEMA, "nomos.observed_scene_plan@1");
    let source =
        std::fs::read_to_string(common::root().join("crates/nomos-observed-scene/src/input.rs"))
            .expect("read input owner");
    let output =
        std::fs::read_to_string(common::root().join("crates/nomos-observed-scene/src/plan.rs"))
            .expect("read plan owner");
    assert_eq!(source.matches("nomos.observed_scene@1").count(), 3);
    assert_eq!(output.matches("nomos.observed_scene_plan@1").count(), 3);
}

#[test]
fn first_scene_round_trips_and_compiles_to_the_committed_plan() {
    let source = common::scene_one();
    let observed = ObservedScene::from_bytes(&source).expect("strict input");
    assert_eq!(observed.to_canonical_bytes(), source);

    let compiled = compile(&source).expect("compile scene");
    assert_eq!(compiled.source_sha256(), Sha256Digest::of_bytes(&source));
    assert_eq!(compiled.to_canonical_bytes(), common::plan_one());
    let reopened = ScenePlan::from_bytes(&common::plan_one()).expect("strict plan");
    assert_eq!(reopened, compiled);
    assert_eq!(reopened.to_canonical_bytes(), common::plan_one());
}

#[test]
fn every_finite_mapping_is_exact_and_composes() {
    let plan = compile(&common::scene_one()).expect("compile scene");
    let terrain: Vec<_> = plan
        .terrain_layers()
        .iter()
        .map(|row| {
            (
                row.role(),
                row.assembly(),
                row.material_family(),
                row.stack(),
            )
        })
        .collect();
    assert_eq!(
        terrain,
        [
            (
                TerrainRole::CalmGround,
                TerrainAssembly::CalmGround,
                MaterialFamily::GroundMuted,
                0,
            ),
            (
                TerrainRole::TraversableRoute,
                TerrainAssembly::TraversableRoute,
                MaterialFamily::RouteWorn,
                10,
            ),
            (
                TerrainRole::StructureFootprint,
                TerrainAssembly::StructureFootprint,
                MaterialFamily::StructureStone,
                20,
            ),
        ]
    );
    let all = plan
        .actors()
        .iter()
        .find(|row| row.copied_actor().id().as_str() == "actor_all")
        .expect("all-flags actor");
    assert_eq!(all.pose(), ActorPose::UprightLiving);
    assert_eq!(all.controlled_marker(), Presence::Present);
    assert_eq!(all.hostile_outline(), Presence::Present);
    assert_eq!(all.protection_ring(), Presence::Present);
    assert_eq!(
        plan.actions()
            .iter()
            .map(|row| (row.copied_action().availability(), row.marker()))
            .collect::<Vec<_>>(),
        [
            (Availability::Disabled, ActionMarker::Disabled),
            (Availability::Enabled, ActionMarker::Enabled),
        ]
    );
}

#[test]
fn the_maximum_fixture_is_the_exact_contract_workload() {
    let source = common::maximum();
    assert_eq!(source.len(), 98_421);
    assert_eq!(
        Sha256Digest::of_bytes(&source).to_hex(),
        "fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909"
    );
    let observed = ObservedScene::from_bytes(&source).expect("strict maximum input");
    assert_eq!(observed.crop().width(), 32);
    assert_eq!(observed.crop().height(), 32);
    assert_eq!(observed.terrain_layers().len(), 8);
    assert_eq!(
        observed
            .terrain_layers()
            .iter()
            .map(|layer| layer.cells().len())
            .sum::<usize>(),
        4096
    );
    assert_eq!(observed.actors().len(), 64);
    assert_eq!(observed.actions().len(), 128);

    let combinations: BTreeSet<_> = observed
        .actors()
        .iter()
        .map(|actor| {
            (
                actor.life_state(),
                actor.controlled(),
                actor.hostile(),
                actor.protected(),
            )
        })
        .collect();
    assert_eq!(combinations.len(), 16);
    assert!(combinations.contains(&(LifeState::Living, false, false, false)));
    assert!(combinations.contains(&(LifeState::Dead, true, true, true)));
    let plan = compile(&source).expect("compile maximum scene");
    ScenePlan::from_bytes(&plan.to_canonical_bytes()).expect("strict maximum plan");
}

#[test]
fn the_plan_reader_refuses_every_copied_fact_selection_disagreement() {
    let plan = common::plan_one();
    let cases = [
        (
            "\"assembly\":\"terrain/calm_ground\"",
            "\"assembly\":\"terrain/traversable_route\"",
        ),
        (
            "\"material_family\":\"ground_muted\"",
            "\"material_family\":\"route_worn\"",
        ),
        ("\"stack\":0", "\"stack\":10"),
        ("\"pose\":\"upright_living\"", "\"pose\":\"prone_dead\""),
        (
            "\"controlled_marker\":\"present\"",
            "\"controlled_marker\":\"absent\"",
        ),
        (
            "\"hostile_outline\":\"present\"",
            "\"hostile_outline\":\"absent\"",
        ),
        (
            "\"protection_ring\":\"present\"",
            "\"protection_ring\":\"absent\"",
        ),
        (
            "\"marker\":\"action/disabled\"",
            "\"marker\":\"action/enabled\"",
        ),
    ];
    for (before, after) in cases {
        let mutation = common::replace_once(&plan, before, after);
        let error = ScenePlan::from_bytes(&mutation).expect_err(before);
        assert_eq!(
            error.code(),
            nomos_observed_scene::codes::FIELD_INVALID,
            "{before}"
        );
    }
}
