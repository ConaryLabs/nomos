//! The one documented equivalence normalization.
//!
//! `RUNTIME.md` section 5 R1-2 accepts the plan when it equals the committed
//! fixture "under one documented normalization, or every difference is recorded
//! with its cause". This is that normalization stated as executable properties;
//! `experiments/executable-gaol/compare-rendering-plan.sh` is the same rule
//! applied to the four committed areas, and its header states it in prose.
//!
//! The rule has exactly three clauses, and each has a test here:
//!
//! 1. Parse both documents as JSON, so key order and insignificant whitespace
//!    are not differences.
//! 2. Ignore the `schema` field on both sides.
//! 3. Normalize nothing else — array order counts, and `null` is a value, never
//!    the same thing as an absent key.

mod common;

use common::normalized_differences;

#[test]
fn key_order_and_whitespace_are_not_differences() {
    let pretty = br#"{
  "b": 2,
  "a": [1, 2, {"y": true, "x": null}]
}"#;
    let compact = br#"{"a":[1,2,{"x":null,"y":true}],"b":2}"#;
    assert!(normalized_differences(pretty, compact).is_empty());
}

#[test]
fn the_schema_field_is_ignored() {
    let js = br#"{"schema":"nomos.experiment.rendering_plan@1","deterministic":true}"#;
    let rust = br#"{"deterministic":true,"schema":"nomos.rendering_plan@3"}"#;
    assert!(normalized_differences(js, rust).is_empty());
    // Only at the top level, and only that name: a nested `schema` still counts.
    let left = br#"{"area":{"schema":1}}"#;
    let right = br#"{"area":{"schema":2}}"#;
    assert_eq!(normalized_differences(left, right).len(), 1);
}

#[test]
fn a_null_cost_is_a_value_and_not_an_absent_key() {
    let with_null = br#"{"movement":{"gate":{"cost":null,"disposition":"blocked"}}}"#;
    let without = br#"{"movement":{"gate":{"disposition":"blocked"}}}"#;
    let differences = normalized_differences(with_null, without);
    assert_eq!(differences.len(), 1);
    assert!(differences[0].contains("cost"), "{differences:?}");
    // And `null` is not `0`, and not `false`.
    assert_eq!(
        normalized_differences(br#"{"cost":null}"#, br#"{"cost":0}"#).len(),
        1
    );
    assert_eq!(
        normalized_differences(br#"{"cost":null}"#, br#"{"cost":false}"#).len(),
        1
    );
}

#[test]
fn array_order_is_a_difference() {
    let left = br#"{"reasons":["a","b"]}"#;
    let right = br#"{"reasons":["b","a"]}"#;
    assert_eq!(normalized_differences(left, right).len(), 2);
    assert_eq!(
        normalized_differences(br#"{"reasons":["a"]}"#, br#"{"reasons":["a","b"]}"#).len(),
        1
    );
}

#[test]
fn there_is_no_number_spelling_clause_left_to_normalize() {
    // `@1` carried decimals, so the comparison had to say that `4.50` and
    // `4.5` were one value spelled two ways. `@2` retired them and `@3` keeps
    // the plan integer-only, and the reader refuses a decimal outright, so the
    // clause is gone rather than merely unused: no pair of distinct lexemes
    // denoting one number is left for it to normalize.
    let refusal = nomos_render_plan::json::parse(br#"{"h":4.5}"#)
        .expect_err("a decimal is refused, not normalized");
    assert_eq!(refusal.code().as_str(), "RP0205");
    assert_eq!(
        normalized_differences(br#"{"h":45}"#, br#"{"h":46}"#).len(),
        1
    );
    assert!(normalized_differences(br#"{"h":45}"#, br#"{"h":45}"#).is_empty());
}

#[test]
fn every_difference_names_its_path() {
    let left = br#"{"scenarios":[{"movement":{"gate":{"cost":1}}}]}"#;
    let right = br#"{"scenarios":[{"movement":{"gate":{"cost":3}}}]}"#;
    let differences = normalized_differences(left, right);
    assert_eq!(differences.len(), 1);
    assert_eq!(differences[0], "$.scenarios[0].movement.gate.cost: 1 != 3");
}
