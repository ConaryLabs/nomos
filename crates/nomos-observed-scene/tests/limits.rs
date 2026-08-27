mod common;

use std::collections::BTreeMap;

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName};
use nomos_observed_scene::{
    ActionMarker, ActorPose, ObservedScene, Presence, TerrainRole, codes, compile,
};

type Object = BTreeMap<FieldName, CanonicalValue>;

fn object_mut(value: &mut CanonicalValue) -> &mut Object {
    match value {
        CanonicalValue::Object(object) => object,
        _ => panic!("expected object"),
    }
}

fn field_mut<'a>(object: &'a mut Object, name: &str) -> &'a mut CanonicalValue {
    let key = object
        .keys()
        .find(|key| key.as_str() == name)
        .expect("field")
        .clone();
    object.get_mut(&key).expect("field value")
}

fn array_mut(value: &mut CanonicalValue) -> &mut Vec<CanonicalValue> {
    match value {
        CanonicalValue::Array(items) => items,
        _ => panic!("expected array"),
    }
}

fn mutate(bytes: &[u8], change: impl FnOnce(&mut Object)) -> Vec<u8> {
    let mut value = parse_canonical(bytes).expect("canonical fixture");
    change(object_mut(&mut value));
    value.to_canonical_bytes()
}

fn reject(bytes: &[u8], code: nomos_observed_scene::ObservedCode) {
    let error = ObservedScene::from_bytes(bytes).expect_err("mutation must fail");
    assert_eq!(error.code(), code, "{error}");
}

fn find_actor_dead(plan: &nomos_observed_scene::ScenePlan) -> &nomos_observed_scene::ActorPlan {
    plan.actors()
        .iter()
        .find(|row| row.copied_actor().id().as_str() == "actor_dead")
        .expect("actor_dead")
}

#[test]
fn every_collection_limit_has_a_valid_edge_and_a_rejected_crossing() {
    let first = common::scene_one();
    let maximum = common::maximum();

    let zero_actions = mutate(&first, |root| array_mut(field_mut(root, "actions")).clear());
    assert!(ObservedScene::from_bytes(&zero_actions).is_ok());
    assert!(ObservedScene::from_bytes(&maximum).is_ok());

    let no_layers = mutate(&first, |root| {
        array_mut(field_mut(root, "terrain_layers")).clear();
    });
    reject(&no_layers, codes::BOUND_INVALID);
    let nine_layers = mutate(&maximum, |root| {
        let rows = array_mut(field_mut(root, "terrain_layers"));
        rows.push(rows[7].clone());
    });
    reject(&nine_layers, codes::BOUND_INVALID);

    let no_actors = mutate(&first, |root| array_mut(field_mut(root, "actors")).clear());
    reject(&no_actors, codes::BOUND_INVALID);
    let sixty_five_actors = mutate(&maximum, |root| {
        let rows = array_mut(field_mut(root, "actors"));
        rows.push(rows[63].clone());
    });
    reject(&sixty_five_actors, codes::BOUND_INVALID);

    let one_twenty_nine_actions = mutate(&maximum, |root| {
        let rows = array_mut(field_mut(root, "actions"));
        rows.push(rows[127].clone());
    });
    reject(&one_twenty_nine_actions, codes::BOUND_INVALID);

    let empty_layer = mutate(&first, |root| {
        let layers = array_mut(field_mut(root, "terrain_layers"));
        let layer = object_mut(&mut layers[0]);
        array_mut(field_mut(layer, "cells")).clear();
    });
    reject(&empty_layer, codes::BOUND_INVALID);
    let oversized_layer = mutate(&maximum, |root| {
        let layers = array_mut(field_mut(root, "terrain_layers"));
        let layer = object_mut(&mut layers[0]);
        let cells = array_mut(field_mut(layer, "cells"));
        while cells.len() < 1025 {
            cells.push(cells[0].clone());
        }
    });
    reject(&oversized_layer, codes::BOUND_INVALID);
}

#[test]
fn minimum_crop_shared_cells_and_shared_actor_cells_are_legal() {
    let minimum = mutate(&common::scene_one(), |root| {
        let crop = object_mut(field_mut(root, "crop"));
        *field_mut(crop, "height") = CanonicalValue::Int(1);
        *field_mut(crop, "width") = CanonicalValue::Int(1);
        for layer in array_mut(field_mut(root, "terrain_layers")) {
            let layer = object_mut(layer);
            *field_mut(layer, "cells") =
                CanonicalValue::Array(vec![CanonicalValue::object_declared([
                    ("x", CanonicalValue::Int(0)),
                    ("y", CanonicalValue::Int(0)),
                ])]);
        }
        for actor in array_mut(field_mut(root, "actors")) {
            let cell = object_mut(field_mut(object_mut(actor), "cell"));
            *field_mut(cell, "x") = CanonicalValue::Int(0);
            *field_mut(cell, "y") = CanonicalValue::Int(0);
        }
    });
    let observed = ObservedScene::from_bytes(&minimum).expect("minimum scene");
    assert_eq!(observed.crop().width(), 1);
    assert_eq!(observed.crop().height(), 1);
    assert!(
        observed
            .actors()
            .iter()
            .all(|actor| actor.cell().x() == 0 && actor.cell().y() == 0)
    );
    assert!(
        observed
            .terrain_layers()
            .iter()
            .all(|layer| layer.cells()[0].x() == 0 && layer.cells()[0].y() == 0)
    );
}

#[test]
fn required_role_coverage_and_the_sixty_four_byte_identity_edge_are_exact() {
    let missing_role = mutate(&common::maximum(), |root| {
        let layers = array_mut(field_mut(root, "terrain_layers"));
        for layer in layers {
            let layer = object_mut(layer);
            if matches!(field_mut(layer, "role"), CanonicalValue::Text(role) if role == "structure_footprint")
            {
                *field_mut(layer, "role") = CanonicalValue::text("calm_ground");
            }
        }
    });
    reject(&missing_role, codes::BOUND_INVALID);

    let too_long = mutate(&common::scene_one(), |root| {
        let scene = object_mut(field_mut(root, "scene"));
        *field_mut(scene, "id") = CanonicalValue::text(format!("s{}", "0".repeat(64)));
    });
    reject(&too_long, codes::IDENTITY_INVALID);
    assert_eq!(
        ObservedScene::from_bytes(&common::maximum())
            .expect("maximum")
            .scene()
            .id()
            .as_str()
            .len(),
        64
    );
}

#[test]
fn each_actor_fact_and_action_availability_selects_independently() {
    let base = common::scene_one();
    let original = compile(&base).expect("base compile");
    let actor = find_actor_dead;

    let controlled = compile(&common::replace_once(
        &base,
        "\"controlled\":false,\"hostile\":false,\"id\":\"actor_dead\"",
        "\"controlled\":true,\"hostile\":false,\"id\":\"actor_dead\"",
    ))
    .expect("controlled mutation");
    assert_eq!(actor(&controlled).controlled_marker(), Presence::Present);
    assert_eq!(
        actor(&controlled).hostile_outline(),
        actor(&original).hostile_outline()
    );
    assert_eq!(
        actor(&controlled).protection_ring(),
        actor(&original).protection_ring()
    );
    assert_eq!(actor(&controlled).pose(), actor(&original).pose());

    let hostile = compile(&common::replace_once(
        &base,
        "\"controlled\":false,\"hostile\":false,\"id\":\"actor_dead\"",
        "\"controlled\":false,\"hostile\":true,\"id\":\"actor_dead\"",
    ))
    .expect("hostile mutation");
    assert_eq!(actor(&hostile).hostile_outline(), Presence::Present);
    assert_eq!(
        actor(&hostile).controlled_marker(),
        actor(&original).controlled_marker()
    );
    assert_eq!(
        actor(&hostile).protection_ring(),
        actor(&original).protection_ring()
    );

    let protected = compile(&common::replace_once(
        &base,
        "\"life_state\":\"dead\",\"protected\":false",
        "\"life_state\":\"dead\",\"protected\":true",
    ))
    .expect("protected mutation");
    assert_eq!(actor(&protected).protection_ring(), Presence::Present);
    assert_eq!(
        actor(&protected).controlled_marker(),
        actor(&original).controlled_marker()
    );
    assert_eq!(
        actor(&protected).hostile_outline(),
        actor(&original).hostile_outline()
    );

    let living = compile(&common::replace_once(
        &base,
        "\"id\":\"actor_dead\",\"life_state\":\"dead\"",
        "\"id\":\"actor_dead\",\"life_state\":\"living\"",
    ))
    .expect("life mutation");
    assert_eq!(actor(&living).pose(), ActorPose::UprightLiving);
    assert_eq!(actor(&original).pose(), ActorPose::ProneDead);
    assert_eq!(
        actor(&living).controlled_marker(),
        actor(&original).controlled_marker()
    );
    assert_eq!(
        actor(&living).hostile_outline(),
        actor(&original).hostile_outline()
    );
    assert_eq!(
        actor(&living).protection_ring(),
        actor(&original).protection_ring()
    );

    let enabled = compile(&common::replace_once(
        &base,
        "\"availability\":\"disabled\",\"id\":\"action_disabled\"",
        "\"availability\":\"enabled\",\"id\":\"action_disabled\"",
    ))
    .expect("availability mutation");
    assert_eq!(enabled.actions()[0].marker(), ActionMarker::Enabled);
    assert_eq!(
        enabled.actions()[1].marker(),
        original.actions()[1].marker()
    );
}

#[test]
fn one_role_mutation_changes_only_its_declared_terrain_selection() {
    let source = common::maximum();
    let original = compile(&source).expect("maximum compile");
    let mutation = common::replace_once(
        &source,
        "\"role\":\"calm_ground\"",
        "\"role\":\"traversable_route\"",
    );
    let changed = compile(&mutation).expect("role mutation");
    assert_eq!(
        changed.terrain_layers()[0].role(),
        TerrainRole::TraversableRoute
    );
    assert_ne!(
        changed.terrain_layers()[0].assembly(),
        original.terrain_layers()[0].assembly()
    );
    assert_eq!(
        changed.terrain_layers()[1..],
        original.terrain_layers()[1..]
    );
}
