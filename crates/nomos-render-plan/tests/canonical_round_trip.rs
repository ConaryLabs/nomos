//! The plan is `nomos-core`'s canonical bytes, not a second encoder's.
//!
//! R1-2 shipped `crates/nomos-render-plan/src/doc.rs`, a private
//! reimplementation of the `KERNEL.md` section 7 byte profile, because
//! `nomos.rendering_plan@1` carried camelCase names, dotted object keys, and
//! decimal values that `nomos_core::CanonicalValue` cannot hold. Issue #144
//! records that as a drift risk of the same class as a shadow resolver, and
//! `tests/canonical_profile.rs` was the agreement test that contained it.
//!
//! `nomos.rendering_plan@2` was designed to fit inside `CanonicalValue`, and
//! `@3` still is, so the second encoder is deleted and there is nothing left to
//! compare against. What replaces the agreement test is the stronger property
//! issue #144 asks for: the kernel's own strict reader accepts the emitted
//! plan, and re-encoding what it read reproduces the bytes exactly.

mod common;

use common::Fixture;
use nomos_core::CanonicalValue;
use nomos_core::canonical::read::parse_canonical;

/// Strips the single trailing `LF` the writer appends.
///
/// `crate::plan::compile` appends it so the artifact ends in a newline like
/// every other file in the tree; `parse_canonical` refuses insignificant
/// bytes, so the round trip has to account for exactly that one. `read.rs`
/// makes the same allowance for `nomos effective-facts`.
fn body(bytes: &[u8]) -> &[u8] {
    match bytes.split_last() {
        Some((b'\n', rest)) => rest,
        _ => bytes,
    }
}

#[test]
fn parse_canonical_round_trips_the_emitted_plan() {
    let fixture = Fixture::new("round-trip");
    let compiled = nomos_render_plan::compile(fixture.inputs()).expect("the fixture compiles");

    let value = parse_canonical(body(&compiled.bytes))
        .expect("the plan is canonical bytes under nomos-core's own strict reader");
    assert_eq!(
        value.to_canonical_bytes(),
        body(&compiled.bytes),
        "re-encoding what the kernel reader read does not reproduce the plan"
    );
    assert_eq!(
        compiled.bytes.last(),
        Some(&b'\n'),
        "the artifact ends in exactly one newline"
    );
}

#[test]
fn the_plan_carries_no_value_outside_the_kernel_profile() {
    let fixture = Fixture::new("profile");
    let compiled = nomos_render_plan::compile(fixture.inputs()).expect("the fixture compiles");
    let value = parse_canonical(body(&compiled.bytes)).expect("the plan is canonical");

    // Two properties the widened profile existed for, now asserted absent.
    // A field name outside `[a-z][a-z0-9_]*` cannot survive `parse_canonical`
    // at all, so reaching this point already proves it; walking the document
    // proves the second, that no number is fractional — `CanonicalValue` has
    // no float variant, so the check is that every number is an integer
    // variant rather than that its text has no dot.
    let mut integers = 0_usize;
    let mut fields = 0_usize;
    walk(&value, &mut integers, &mut fields);
    assert!(integers > 0, "the plan carries integers");
    assert!(fields > 0, "the plan carries fields");
}

fn walk(value: &CanonicalValue, integers: &mut usize, fields: &mut usize) {
    match value {
        CanonicalValue::Int(_) | CanonicalValue::Uint(_) => *integers += 1,
        CanonicalValue::Array(items) => {
            for item in items {
                walk(item, integers, fields);
            }
        }
        CanonicalValue::Object(object) => {
            for (name, item) in object {
                *fields += 1;
                let bytes = name.as_str().as_bytes();
                assert!(
                    bytes[0].is_ascii_lowercase()
                        && bytes.iter().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_'
                        }),
                    "plan field `{name}` is outside nomos_core::FieldName's grammar"
                );
                walk(item, integers, fields);
            }
        }
        _ => {}
    }
}

#[test]
fn compiling_twice_is_byte_identical() {
    let fixture = Fixture::new("twice");
    let first = nomos_render_plan::compile(fixture.inputs()).expect("first compile");
    let second = nomos_render_plan::compile(fixture.inputs()).expect("second compile");
    assert_eq!(first.bytes, second.bytes);
}

#[test]
fn the_schema_identity_literals_are_valid() {
    assert_eq!(
        nomos_render_plan::rendering_plan_schema().to_string(),
        "nomos.rendering_plan@3"
    );
    assert_eq!(
        nomos_render_plan::presentation_source_schema().to_string(),
        "nomos.presentation_source@2"
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
