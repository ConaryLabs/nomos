//! `nomos.presentation_source@1` refuses what it says it refuses.
//!
//! `RUNTIME.md` section 5 R1-3 requires two of these outright — "the accepted
//! source is versioned, and a version mismatch is refused with a stable
//! diagnostic" and "a schema test rejects a source file carrying a raw
//! floating-point transform" — and issue #146 makes the second exhaustive:
//! *any* `.`-bearing or exponent number anywhere in the source.
//!
//! Each test edits one thing in a source that otherwise compiles, so a pass
//! cannot come from the file being broken some other way. The unedited text is
//! asserted to compile first, in [`the_unedited_source_compiles`].

mod common;

use std::fs;

use common::Fixture;
use nomos_render_plan::PlanResult;

/// Compiles the fixture after rewriting its presentation source.
fn compile_with(label: &str, edit: impl Fn(&str) -> String) -> PlanResult<()> {
    let fixture = Fixture::new(label);
    let original = fs::read_to_string(fixture.source()).unwrap();
    let edited = edit(&original);
    assert_ne!(edited, original, "the edit changed nothing");
    fs::write(fixture.source(), edited).unwrap();
    nomos_render_plan::compile(fixture.inputs()).map(|_| ())
}

/// The code and message of the rejection an edit produces.
fn refusal(label: &str, edit: impl Fn(&str) -> String) -> (String, String) {
    let error = compile_with(label, edit).expect_err("the edited source is refused");
    (error.code().as_str().to_owned(), error.message().to_owned())
}

#[test]
fn the_unedited_source_compiles() {
    let fixture = Fixture::new("source-baseline");
    nomos_render_plan::compile(fixture.inputs()).expect("the fixture's source compiles");
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

#[test]
fn a_version_mismatch_is_refused_with_a_stable_diagnostic() {
    let (code, message) = refusal("source-version", |text| {
        text.replace("nomos.presentation_source@1", "nomos.presentation_source@2")
    });
    assert_eq!(code, "RP0104");
    assert_eq!(
        message,
        "expected schema `nomos.presentation_source@1`, \
         found `nomos.presentation_source@2`"
    );
}

#[test]
fn a_different_schema_name_is_refused() {
    let (code, message) = refusal("source-name", |text| {
        text.replace("nomos.presentation_source@1", "nomos.area@1")
    });
    assert_eq!(code, "RP0104");
    assert!(message.contains("found `nomos.area@1`"), "{message}");
}

#[test]
fn an_absent_schema_field_is_refused_by_identity_not_by_shape() {
    let (code, message) = refusal("source-no-schema", |text| {
        text.replace("  \"schema\": \"nomos.presentation_source@1\",\n", "")
    });
    // The field set is checked before the identity, so an absent `schema` is
    // an RP0202 shape refusal that still names the missing field. What matters
    // is that it fails closed and says which field.
    assert_eq!(code, "RP0202");
    assert!(message.contains("schema"), "{message}");
}

// ---------------------------------------------------------------------------
// Floating point
// ---------------------------------------------------------------------------

#[test]
fn a_decimal_wall_height_is_refused() {
    let (code, message) = refusal("float-wall", |text| {
        text.replace("\"wall_height_steps\": 45", "\"wall_height_steps\": 4.5")
    });
    assert_eq!(code, "RP0205");
    assert!(message.contains("`4.5` carries a fraction"), "{message}");
    assert!(message.contains("integers only"), "{message}");
}

#[test]
fn a_decimal_mass_height_is_refused() {
    let (code, _) = refusal("float-mass", |text| {
        text.replace("\"height_steps\": 32", "\"height_steps\": 3.2")
    });
    assert_eq!(code, "RP0205");
}

#[test]
fn a_decimal_cell_coordinate_is_refused() {
    let (code, _) = refusal("float-cell", |text| {
        text.replace("\"x\": 7, \"y\": 4", "\"x\": 7.5, \"y\": 4")
    });
    assert_eq!(code, "RP0205");
}

#[test]
fn a_decimal_in_a_field_the_schema_does_not_know_is_still_refused() {
    // The refusal is in the reader, not in the decoder, so it fires before any
    // field is interpreted. A float smuggled into an unknown field is refused
    // as a float rather than reported as an unknown field.
    let (code, message) = refusal("float-unknown-field", |text| {
        text.replace(
            "  \"effects\": [",
            "  \"presentation_anchor\": { \"x\": 3.6 },\n  \"effects\": [",
        )
    });
    assert_eq!(code, "RP0205");
    assert!(message.contains("`3.6`"), "{message}");
}

#[test]
fn a_trailing_zero_decimal_is_refused() {
    // `45.0` is integer-valued but still a decimal literal. Issue #146 says
    // any `.`-bearing number, and that includes this one.
    let (code, message) = refusal("float-trailing-zero", |text| {
        text.replace("\"wall_height_steps\": 45", "\"wall_height_steps\": 45.0")
    });
    assert_eq!(code, "RP0205");
    assert!(message.contains("carries a fraction"), "{message}");
}

#[test]
fn an_exponent_is_refused() {
    let (code, message) = refusal("float-exponent", |text| {
        text.replace("\"wall_height_steps\": 45", "\"wall_height_steps\": 4.5e1")
    });
    assert_eq!(code, "RP0205");
    assert!(message.contains("carries a fraction"), "{message}");
}

#[test]
fn an_integer_exponent_is_refused_too() {
    let (code, message) = refusal("float-integer-exponent", |text| {
        text.replace("\"wall_height_steps\": 45", "\"wall_height_steps\": 45e0")
    });
    assert_eq!(code, "RP0205");
    assert!(message.contains("carries an exponent"), "{message}");
}

#[test]
fn a_leading_plus_and_a_leading_zero_are_refused() {
    let (code, message) = refusal("number-plus", |text| {
        text.replace("\"wall_height_steps\": 45", "\"wall_height_steps\": +45")
    });
    assert_eq!(code, "RP0205");
    assert!(message.contains("leading `+`"), "{message}");

    let (code, message) = refusal("number-zero", |text| {
        text.replace("\"wall_height_steps\": 45", "\"wall_height_steps\": 045")
    });
    assert_eq!(code, "RP0205");
    assert!(message.contains("redundant leading zero"), "{message}");
}

// ---------------------------------------------------------------------------
// Closed shape
// ---------------------------------------------------------------------------

#[test]
fn an_unknown_top_level_field_is_refused_rather_than_ignored() {
    let (code, message) = refusal("unknown-field", |text| {
        text.replace(
            "  \"effects\": [",
            "  \"camera\": \"gaol_oblique_01\",\n  \"effects\": [",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("unknown camera"), "{message}");
}

#[test]
fn a_missing_field_is_refused() {
    let (code, message) = refusal("missing-field", |text| {
        text.replace(",\n    \"start\": true", "")
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("missing start"), "{message}");
}

#[test]
fn a_reintroduced_presentation_anchor_is_refused() {
    // The audit's twelve floating-point components cannot come back even as
    // integers: `effects[].anchor` is exactly `{entity, socket}`.
    let (code, message) = refusal("anchor-coords", |text| {
        text.replace(
            "\"anchor\": { \"entity\": \"north_gate\", \"socket\": \"ward\" }",
            "\"anchor\": { \"entity\": \"north_gate\", \"socket\": \"ward\", \"x\": 3, \"y\": 3 }",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("unknown x, y"), "{message}");
}

// ---------------------------------------------------------------------------
// Sockets
// ---------------------------------------------------------------------------

#[test]
fn an_undeclared_socket_is_refused() {
    let (code, message) = refusal("socket-unknown", |text| {
        text.replace("\"socket\": \"ward\"", "\"socket\": \"hinge\"")
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("socket `hinge`"), "{message}");
    assert!(message.contains("declares: ward"), "{message}");
}

#[test]
fn a_socket_on_a_kind_that_declares_none_is_refused() {
    let (code, message) = refusal("socket-on-light", |text| {
        text.replace(
            "\"entity\": \"north_gate\"",
            "\"entity\": \"watch_brazier\"",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("declares: none"), "{message}");
}

#[test]
fn an_effect_anchored_to_an_uncompiled_entity_is_refused() {
    let (code, message) = refusal("socket-unknown-entity", |text| {
        text.replace("\"entity\": \"north_gate\"", "\"entity\": \"no_such_gate\"")
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("not a compiled entity"), "{message}");
}

// ---------------------------------------------------------------------------
// Identifier grammars
// ---------------------------------------------------------------------------

#[test]
fn an_area_id_outside_its_grammar_is_refused() {
    let (code, message) = refusal("grammar-area", |text| {
        text.replace("\"id\": \"test-area\"", "\"id\": \"Test_Area\"")
    });
    assert_eq!(code, "RP0206");
    assert!(message.contains("area.id"), "{message}");
}

#[test]
fn an_assembly_without_a_namespace_is_refused() {
    let (code, message) = refusal("grammar-assembly", |text| {
        text.replace(
            "\"assembly\": \"visual/beveled_masonry\"",
            "\"assembly\": \"beveled_masonry\"",
        )
    });
    assert_eq!(code, "RP0206");
    assert!(message.contains("architecture.style.assembly"), "{message}");
}

#[test]
fn an_entity_id_with_a_hyphen_is_refused() {
    let (code, message) = refusal("grammar-entity", |text| {
        text.replace("\"socket\": \"ward\"", "\"socket\": \"ward-mark\"")
    });
    assert_eq!(code, "RP0206");
    assert!(message.contains("effects[].anchor.socket"), "{message}");
}

// ---------------------------------------------------------------------------
// Bounded invariants, which used to be compiler magic numbers
// ---------------------------------------------------------------------------

#[test]
fn a_wall_taller_than_the_bounded_profile_is_refused() {
    let (code, message) = refusal("bound-wall", |text| {
        text.replace("\"wall_height_steps\": 45", "\"wall_height_steps\": 51")
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("1..=50 vertical steps"), "{message}");
}

#[test]
fn a_lattice_wider_than_the_bounded_profile_is_refused() {
    let (code, message) = refusal("bound-lattice", |text| {
        text.replace(
            "\"width\": 9, \"height\": 6",
            "\"width\": 10, \"height\": 6",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("1..=9 by 1..=6"), "{message}");
}

#[test]
fn a_mass_leaving_the_lattice_is_refused() {
    let (code, message) = refusal("bound-mass", |text| {
        text.replace(
            "\"max\": { \"x\": 3, \"y\": 2 }",
            "\"max\": { \"x\": 30, \"y\": 2 }",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("leaves the 9x6 lattice"), "{message}");
}

#[test]
fn an_actor_inside_masonry_is_refused() {
    let (code, message) = refusal("actor-in-masonry", |text| {
        text.replace(
            "\"x\": 7, \"y\": 4, \"z\": 0",
            "\"x\": 2, \"y\": 1, \"z\": 0",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("inside masonry mass `pier`"), "{message}");
}

#[test]
fn an_actor_outside_the_lattice_is_refused() {
    let (code, message) = refusal("actor-out-of-bounds", |text| {
        text.replace(
            "\"x\": 7, \"y\": 4, \"z\": 0",
            "\"x\": 9, \"y\": 4, \"z\": 0",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(
        message.contains("outside this area's 9x6 lattice"),
        "{message}"
    );
}

#[test]
fn a_nonzero_elevation_is_refused() {
    let (code, message) = refusal("actor-elevated", |text| {
        text.replace(
            "\"x\": 7, \"y\": 4, \"z\": 0",
            "\"x\": 7, \"y\": 4, \"z\": 1",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(
        message.contains("the bounded profile is z = 0"),
        "{message}"
    );
}

// ---------------------------------------------------------------------------
// Route
// ---------------------------------------------------------------------------

#[test]
fn a_start_area_declaring_an_entry_is_refused() {
    // Owner ruling 3: nothing arrives at the start area, so it has no arrival
    // cell to declare. The field set is checked against `area.start`, which is
    // what makes this a schema property rather than a convention.
    let (code, message) = refusal("start-with-entry", |text| {
        text.replace(
            "\"exit\": { \"gate\": \"north_gate\", \"to_area\": null }",
            "\"exit\": { \"gate\": \"north_gate\", \"to_area\": null },\n    \
             \"entry\": { \"x\": 1, \"y\": 1, \"z\": 0 }",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("unknown entry"), "{message}");
}

#[test]
fn a_non_start_area_without_an_entry_is_refused() {
    let (code, message) = refusal("non-start-without-entry", |text| {
        text.replace("\"start\": true", "\"start\": false")
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("missing entry"), "{message}");
}

#[test]
fn a_gate_that_is_not_a_compiled_door_is_refused() {
    let (code, message) = refusal("gate-not-a-door", |text| {
        text.replace("\"gate\": \"north_gate\"", "\"gate\": \"watch_brazier\"")
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("not `door`"), "{message}");
}

#[test]
fn a_pursuit_light_that_is_not_a_compiled_light_is_refused() {
    let (code, message) = refusal("light-not-a-light", |text| {
        text.replace("\"light\": \"watch_brazier\"", "\"light\": \"north_gate\"")
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("not `light`"), "{message}");
}

#[test]
fn an_area_routing_to_itself_is_refused() {
    let (code, message) = refusal("route-self", |text| {
        text.replace("\"to_area\": null", "\"to_area\": \"test-area\"")
    });
    assert_eq!(code, "RP0202");
    assert!(
        message.contains("may not name the area itself"),
        "{message}"
    );
}

// ---------------------------------------------------------------------------
// Actors
// ---------------------------------------------------------------------------

#[test]
fn a_missing_required_actor_is_refused() {
    let (code, message) = refusal("actor-missing", |text| {
        text.replace(
            "    { \"id\": \"gaoler\", \"assembly\": \"visual/gaoler_silhouette\", \
             \"cell\": { \"x\": 4, \"y\": 3, \"z\": 0 } }\n",
            "",
        )
        .replace(
            "\"cell\": { \"x\": 7, \"y\": 4, \"z\": 0 } },\n",
            "\"cell\": { \"x\": 7, \"y\": 4, \"z\": 0 } }\n",
        )
    });
    assert_eq!(code, "RP0202");
    assert!(message.contains("must declare exactly"), "{message}");
}

// ---------------------------------------------------------------------------
// The whole corpus
// ---------------------------------------------------------------------------

#[test]
fn the_committed_sources_carry_no_decimal_literal() {
    // The corpus-level statement issue #146's "no decimal literal in accepted
    // presentation source" makes, checked against the files rather than
    // against the reader.
    let areas = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../experiments/executable-gaol/areas"
    );
    let mut checked = 0_usize;
    for entry in fs::read_dir(areas).expect("the study's areas directory is readable") {
        let path = entry.unwrap().path().join("presentation.json");
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        // Every `.` in a source is inside a string (an assembly name has a
        // `/`, never a `.`; a schema identity has both). A `.` adjacent to a
        // digit outside a string is the shape a decimal takes.
        for (index, window) in text.as_bytes().windows(3).enumerate() {
            if window[1] == b'.' && window[0].is_ascii_digit() && window[2].is_ascii_digit() {
                panic!(
                    "{} carries a decimal literal at byte {index}",
                    path.display()
                );
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 4, "all four committed areas were checked");
}
