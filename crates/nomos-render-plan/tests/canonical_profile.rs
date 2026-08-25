//! The plan encoder against `nomos-core`'s.
//!
//! `crates/nomos-render-plan/src/doc.rs` reimplements the `KERNEL.md` section 7
//! byte profile for one document, because `nomos_core::CanonicalValue` cannot
//! hold a camelCase field name, a dotted key, or a decimal. This suite is the
//! proof that the reimplementation is the same profile: for every value both
//! types can express, the two encoders produce the same bytes.

use nomos_core::{CanonicalValue, FieldName};
use nomos_render_plan::decimal::Decimal;
use nomos_render_plan::{PlanField, PlanValue};

#[test]
fn the_two_encoders_agree_on_every_value_both_can_express() {
    let value = PlanValue::object([
        ("null_field", PlanValue::Null),
        ("true_field", PlanValue::Bool(true)),
        ("false_field", PlanValue::Bool(false)),
        ("negative", PlanValue::Int(-42)),
        ("zero", PlanValue::Int(0)),
        ("large", PlanValue::Uint(u64::MAX)),
        ("smallest", PlanValue::Int(i64::MIN)),
        (
            "escapes",
            PlanValue::text("quote \" solidus / backslash \\ tab \t newline \n bell \u{7}"),
        ),
        ("unicode", PlanValue::text("café — 日本 — \u{7f}")),
        ("empty_text", PlanValue::text("")),
        ("empty_array", PlanValue::Array(Vec::new())),
        (
            "nested",
            PlanValue::Array(vec![
                PlanValue::object([("b", PlanValue::Uint(2)), ("a", PlanValue::Uint(1))]),
                PlanValue::Array(vec![PlanValue::Null, PlanValue::Bool(false)]),
            ]),
        ),
        (
            "ordering",
            PlanValue::object([
                ("z", PlanValue::Uint(26)),
                ("a", PlanValue::Uint(1)),
                ("a_b", PlanValue::Uint(2)),
                ("ab", PlanValue::Uint(3)),
            ]),
        ),
    ]);

    let canonical = value
        .to_canonical()
        .expect("every field here is inside nomos-core's profile");
    assert_eq!(
        value.to_canonical_bytes(),
        canonical.to_canonical_bytes(),
        "the plan encoder and nomos-core disagree on the byte profile"
    );
    // And the bytes are what nomos-core's strict reader accepts.
    nomos_core::canonical::read::parse_canonical(&value.to_canonical_bytes())
        .expect("plan bytes are canonical under nomos-core's own reader");
}

#[test]
fn the_widened_values_are_exactly_the_ones_nomos_core_refuses() {
    // camelCase: legal here, refused there.
    assert!(PlanField::new("visualAssembly").is_ok());
    assert!(FieldName::new("visualAssembly").is_err());
    // A dotted key: legal here, refused there.
    assert!(PlanField::new("simulation.json").is_ok());
    assert!(PlanField::new("north_gate.access").is_ok());
    assert!(FieldName::new("simulation.json").is_err());
    // A decimal: expressible here, no variant there.
    let decimal = PlanValue::Number(Decimal::parse("4.5").unwrap());
    assert_eq!(decimal.to_canonical_bytes(), b"4.5");
    assert!(
        decimal.to_canonical().is_none(),
        "a decimal has no CanonicalValue equivalent"
    );
}

#[test]
fn the_field_name_profile_is_narrow() {
    for legal in ["a", "id", "toArea", "tile_width", "x9", "a.b.c"] {
        assert!(PlanField::new(legal).is_ok(), "{legal} should be legal");
    }
    for illegal in [
        "",
        "Area",
        "9lives",
        "_leading",
        "with space",
        "dash-ed",
        "ünicode",
    ] {
        let error = PlanField::new(illegal).unwrap_err();
        assert_eq!(error.code().as_str(), "RP0206", "{illegal}");
    }
}

#[test]
fn the_decimal_profile_is_exact_and_narrow() {
    for (lexeme, units) in [
        ("0", 0),
        ("5", 5_000_000),
        ("4.5", 4_500_000),
        ("0.7", 700_000),
        ("-2.25", -2_250_000),
        ("1200", 1_200_000_000),
    ] {
        let decimal = Decimal::parse(lexeme).unwrap();
        assert_eq!(decimal.units(), units, "{lexeme}");
        assert_eq!(decimal.lexeme(), lexeme, "the lexeme is carried verbatim");
    }
    for illegal in [
        "",
        "+1",
        "1e3",
        "1E3",
        "01",
        "1.",
        ".5",
        "1.2345678",
        "-0",
        "0x10",
        "1.2.3",
    ] {
        let error = Decimal::parse(illegal).unwrap_err();
        assert_eq!(error.code().as_str(), "RP0205", "{illegal}");
    }
    assert!(Decimal::parse("4").unwrap().is_integer());
    assert!(!Decimal::parse("4.5").unwrap().is_integer());
    assert_eq!(Decimal::parse("9").unwrap().as_i64(), Some(9));
    assert_eq!(Decimal::parse("9.5").unwrap().as_i64(), None);
    assert!(Decimal::parse("0.000001").unwrap().greater_than(0));
    assert!(!Decimal::parse("0").unwrap().greater_than(0));
    assert!(Decimal::parse("5").unwrap().at_most(5));
    assert!(!Decimal::parse("5.000001").unwrap().at_most(5));
}

#[test]
fn the_schema_identity_literal_is_valid() {
    assert_eq!(
        nomos_render_plan::rendering_plan_schema().to_string(),
        "nomos.rendering_plan@1"
    );
    assert_eq!(
        nomos_render_plan::entity_catalog_schema().to_string(),
        "nomos.entity_catalog@1"
    );
    assert_eq!(
        nomos_render_plan::effective_facts_schema().to_string(),
        "nomos.effective_facts@1"
    );
}

#[test]
fn a_duplicate_runtime_key_is_refused() {
    let error = PlanValue::keyed_object([
        ("a".to_owned(), PlanValue::Uint(1)),
        ("a".to_owned(), PlanValue::Uint(2)),
    ])
    .unwrap_err();
    assert_eq!(error.code().as_str(), "RP0105");
}

#[test]
fn a_canonical_kernel_value_round_trips_into_the_plan() {
    let canonical = CanonicalValue::object_declared([
        ("byte_end", CanonicalValue::Uint(162)),
        ("line", CanonicalValue::Uint(4)),
        ("path", CanonicalValue::text("areas/north-gaol/world.txt")),
    ]);
    let plan = PlanValue::from_canonical(&canonical);
    assert_eq!(plan.to_canonical_bytes(), canonical.to_canonical_bytes());
}
