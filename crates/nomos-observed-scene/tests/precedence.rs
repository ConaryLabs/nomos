mod common;

use std::ffi::OsString;
use std::fs;

use nomos_observed_scene::{ObservedScene, codes, execute};

fn rejected(bytes: &[u8]) -> nomos_observed_scene::ObservedError {
    ObservedScene::from_bytes(bytes).expect_err("multi-fault input must fail")
}

fn command_code(stdout: &[u8]) -> &str {
    let text = std::str::from_utf8(stdout).expect("diagnostic UTF-8");
    &text[text.find("OS").expect("diagnostic code")..][..6]
}

#[test]
fn semantic_phases_follow_the_frozen_precedence() {
    let base = common::scene_one();

    let mut noncanonical_schema =
        common::replace_once(&base, "nomos.observed_scene@1", "nomos.observed_scene@2");
    noncanonical_schema.insert(1, b' ');
    assert_eq!(
        rejected(&noncanonical_schema).code(),
        codes::INPUT_NOT_CANONICAL
    );

    let mut schema_and_field =
        common::replace_once(&base, "nomos.observed_scene@1", "nomos.observed_scene@2");
    schema_and_field = common::replace_once(
        &schema_and_field,
        "{\"actions\"",
        "{\"aardvark\":0,\"actions\"",
    );
    assert_eq!(rejected(&schema_and_field).code(), codes::SCHEMA_MISMATCH);

    let mut field_and_bound = common::replace_once(&base, "\"height\":6", "\"height\":0");
    field_and_bound =
        common::replace_once(&field_and_bound, "\"width\":6", "\"width\":6,\"wrong\":0");
    assert_eq!(rejected(&field_and_bound).code(), codes::FIELD_INVALID);

    let mut bound_and_identity = common::replace_once(&base, "\"height\":6", "\"height\":0");
    bound_and_identity = common::replace_once(
        &bound_and_identity,
        "\"id\":\"actor_all\"",
        "\"id\":\"Actor_all\"",
    );
    assert_eq!(rejected(&bound_and_identity).code(), codes::BOUND_INVALID);

    let mut identity_and_order =
        common::replace_once(&base, "\"id\":\"actor_all\"", "\"id\":\"Actor_all\"");
    identity_and_order = common::replace_once(
        &identity_and_order,
        "\"id\":\"route_layer\"",
        "\"id\":\"a_route_layer\"",
    );
    assert_eq!(
        rejected(&identity_and_order).code(),
        codes::IDENTITY_INVALID
    );

    let mut order_and_reference =
        common::replace_once(&base, "\"id\":\"route_layer\"", "\"id\":\"a_route_layer\"");
    order_and_reference = common::replace_once(
        &order_and_reference,
        "\"target_actor\":\"actor_dead\"",
        "\"target_actor\":\"actor_void\"",
    );
    assert_eq!(
        rejected(&order_and_reference).code(),
        codes::INPUT_NOT_CANONICAL
    );
}

#[test]
fn faults_within_one_phase_walk_canonical_paths_lexically() {
    let base = common::scene_one();
    let mut fields = common::replace_once(&base, "\"controlled\":true", "\"controlled\":null");
    fields = common::replace_once(&fields, "\"height\":6", "\"height\":null");
    let error = rejected(&fields);
    assert_eq!(error.code(), codes::FIELD_INVALID);
    assert!(
        error.message().contains("$.actors[0].controlled"),
        "{error}"
    );

    let mut bounds =
        common::replace_once(&base, "\"x\":2,\"y\":2,\"z\":0", "\"x\":7,\"y\":2,\"z\":0");
    bounds = common::replace_once(&bounds, "\"height\":6", "\"height\":0");
    let error = rejected(&bounds);
    assert_eq!(error.code(), codes::BOUND_INVALID);
    assert!(error.message().contains("$.actors[0].cell.x"), "{error}");

    let mut identities = common::replace_once(
        &base,
        "\"target_actor\":\"actor_dead\"",
        "\"target_actor\":\"Actor_target\"",
    );
    identities = common::replace_once(
        &identities,
        "\"id\":\"action_enabled\"",
        "\"id\":\"Action_later\"",
    );
    let error = rejected(&identities);
    assert_eq!(error.code(), codes::IDENTITY_INVALID);
    assert!(error.message().contains("Actor_target"), "{error}");

    let mut too_many_actions = common::replace_once(
        &common::maximum(),
        "],\"actors\"",
        ",{\"availability\":\"enabled\",\"id\":\"z\",\"target_actor\":\"a\"}],\"actors\"",
    );
    too_many_actions = common::replace_once(
        &too_many_actions,
        "\"width\":32",
        "\"width\":18446744073709551615",
    );
    let error = rejected(&too_many_actions);
    assert_eq!(error.code(), codes::BOUND_INVALID);
    assert!(error.message().contains("$.actions"), "{error}");

    let mut crop_paths =
        common::replace_once(&base, "\"height\":6", "\"height\":18446744073709551615");
    crop_paths = common::replace_once(&crop_paths, "\"width\":6", "\"width\":18446744073709551615");
    let error = rejected(&crop_paths);
    assert_eq!(error.code(), codes::BOUND_INVALID);
    assert!(error.message().contains("$.crop.height"), "{error}");
}

#[test]
fn the_signed_minimum_dimension_is_rejected_without_panicking() {
    let mutation = common::replace_once(
        &common::scene_one(),
        "\"width\":6",
        "\"width\":-9223372036854775808",
    );

    let error = rejected(&mutation);

    assert_eq!(error.code(), codes::BOUND_INVALID);
    assert!(error.message().contains("$.actors[0].cell.x"), "{error}");
}

#[test]
fn filesystem_phases_precede_content_in_the_declared_order() {
    let root = common::fresh_dir("precedence");
    let output = root.join("out.json");
    fs::write(&output, b"immutable").expect("existing output");

    let missing = root.join("missing.json");
    let missing_result = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        missing.as_os_str().to_owned(),
        OsString::from("--out"),
        output.as_os_str().to_owned(),
    ]);
    assert_eq!(
        command_code(missing_result.stdout()),
        codes::INPUT_UNREADABLE.as_str()
    );

    let malformed = root.join("malformed.json");
    fs::write(&malformed, b"{").expect("malformed input");
    let existing_result = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        malformed.as_os_str().to_owned(),
        OsString::from("--out"),
        output.as_os_str().to_owned(),
    ]);
    assert_eq!(
        command_code(existing_result.stdout()),
        codes::OUTPUT_UNAVAILABLE.as_str()
    );
    assert_eq!(fs::read(&output).expect("unchanged"), b"immutable");
}
