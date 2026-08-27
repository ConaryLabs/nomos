mod common;

use nomos_observed_scene::{ObservedScene, codes};

fn rejected(bytes: &[u8], code: nomos_observed_scene::ObservedCode) {
    let error = ObservedScene::from_bytes(bytes).expect_err("mutation must fail");
    assert_eq!(error.code(), code, "{error}");
}

#[test]
fn malformed_noncanonical_schema_and_field_phases_are_distinct() {
    let base = common::scene_one();
    rejected(b"{", codes::INPUT_MALFORMED);
    let mut whitespace = base.clone();
    whitespace.insert(1, b' ');
    rejected(&whitespace, codes::INPUT_NOT_CANONICAL);
    rejected(
        &common::replace_once(&base, "nomos.observed_scene@1", "nomos.observed_scene@2"),
        codes::SCHEMA_MISMATCH,
    );
    rejected(
        &common::replace_once(&base, "\"schema\":\"nomos.observed_scene@1\",", ""),
        codes::SCHEMA_MISMATCH,
    );
    rejected(
        &common::replace_once(&base, "\"width\":6", "\"width\":6,\"wrong\":0"),
        codes::FIELD_INVALID,
    );
    rejected(
        &common::replace_once(&base, "\"height\":6", "\"height\":true"),
        codes::FIELD_INVALID,
    );
    rejected(
        &common::replace_once(&base, "\"height\":6", "\"height\":6,\"height\":6"),
        codes::FIELD_INVALID,
    );
}

#[test]
fn every_bound_identity_order_and_reference_class_fails_closed() {
    let base = common::scene_one();
    for mutation in [
        common::replace_once(&base, "\"height\":6", "\"height\":0"),
        common::replace_once(&base, "\"width\":6", "\"width\":33"),
        common::replace_once(&base, "\"x\":4,\"y\":3,\"z\":0", "\"x\":6,\"y\":3,\"z\":0"),
        common::replace_once(&base, "\"z\":0", "\"z\":1"),
    ] {
        rejected(&mutation, codes::BOUND_INVALID);
    }
    rejected(
        &common::replace_once(&base, "\"id\":\"actor_all\"", "\"id\":\"Actor_all\""),
        codes::IDENTITY_INVALID,
    );
    rejected(
        &common::replace_once(&base, "\"id\":\"actor_controlled\"", "\"id\":\"actor_all\""),
        codes::IDENTITY_INVALID,
    );
    rejected(
        &common::replace_once(
            &base,
            "\"target_actor\":\"actor_dead\"",
            "\"target_actor\":\"actor_void\"",
        ),
        codes::TARGET_DANGLING,
    );
    let out_of_order =
        common::replace_once(&base, "\"id\":\"route_layer\"", "\"id\":\"a_route_layer\"");
    rejected(&out_of_order, codes::INPUT_NOT_CANONICAL);
}

#[test]
fn closed_enums_and_independent_boolean_types_are_not_coerced() {
    let base = common::scene_one();
    for mutation in [
        common::replace_once(&base, "\"role\":\"calm_ground\"", "\"role\":\"water\""),
        common::replace_once(
            &base,
            "\"life_state\":\"living\"",
            "\"life_state\":\"sleeping\"",
        ),
        common::replace_once(
            &base,
            "\"availability\":\"disabled\"",
            "\"availability\":\"hidden\"",
        ),
        common::replace_once(&base, "\"controlled\":true", "\"controlled\":1"),
        common::replace_once(&base, "\"hostile\":true", "\"hostile\":null"),
        common::replace_once(&base, "\"protected\":true", "\"protected\":\"yes\""),
    ] {
        rejected(&mutation, codes::FIELD_INVALID);
    }
}

#[test]
fn cell_and_identity_arrays_are_semantically_canonical() {
    let base = common::scene_one();
    let duplicate = common::replace_once(
        &base,
        "{\"x\":0,\"y\":0},{\"x\":1,\"y\":0}",
        "{\"x\":0,\"y\":0},{\"x\":0,\"y\":0}",
    );
    rejected(&duplicate, codes::BOUND_INVALID);
    let reversed = common::replace_once(
        &base,
        "{\"x\":0,\"y\":0},{\"x\":1,\"y\":0}",
        "{\"x\":1,\"y\":0},{\"x\":0,\"y\":0}",
    );
    rejected(&reversed, codes::INPUT_NOT_CANONICAL);
}
