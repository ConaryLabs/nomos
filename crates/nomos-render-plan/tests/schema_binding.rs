//! Identity binding fails closed, naming expected and found.
//!
//! `RUNTIME.md` section 5 R1-2: "as the first accepted consumer of
//! `nomos.effective_facts@1` it binds that identity and version, and refuses a
//! mismatch with a stable diagnostic". Issue #139 extends the same requirement
//! to `nomos.entity_catalog@1`.

mod common;

use common::{Fixture, Identity, Options};

#[test]
fn the_catalog_identity_and_version_are_bound() {
    for (identity, found) in [
        (Identity::WrongVersion, "nomos.entity_catalog@2"),
        (Identity::WrongName, "nomos.not_the_catalog@1"),
    ] {
        let fixture = Fixture::with(
            "catalog-identity",
            Options {
                catalog_identity: identity,
                ..Options::default()
            },
        );
        let error = nomos_render_plan::compile(fixture.inputs()).unwrap_err();
        assert_eq!(error.code().as_str(), "RP0104");
        assert!(
            error.message().contains("nomos.entity_catalog@1"),
            "the diagnostic must name what was expected: {}",
            error.message()
        );
        assert!(
            error.message().contains(found),
            "the diagnostic must name what was found: {}",
            error.message()
        );
        assert_eq!(error.path(), Some(fixture.catalog().as_path()));
    }
}

#[test]
fn the_effective_facts_identity_and_version_are_bound() {
    let fixture = Fixture::with(
        "facts-identity",
        Options {
            facts_identity: Identity::WrongVersion,
            ..Options::default()
        },
    );
    let error = nomos_render_plan::compile(fixture.inputs()).unwrap_err();
    assert_eq!(error.code().as_str(), "RP0104");
    assert!(
        error.message().contains("nomos.effective_facts@1"),
        "{}",
        error.message()
    );
    assert!(
        error.message().contains("nomos.effective_facts@2"),
        "{}",
        error.message()
    );
}

#[test]
fn both_kernel_spellings_of_an_identity_are_accepted() {
    // `nomos effective-facts` writes `{"name": ..., "version": N}`; the
    // entity-catalog document in issue #138 writes `"name@version"`. Both are
    // the same identity and both bind.
    let fixture = Fixture::new("catalog-string-identity");
    let path = fixture.catalog();
    let text = std::fs::read_to_string(&path).unwrap();
    let rewritten = text.replace(
        r#""schema":{"name":"nomos.entity_catalog","version":1}"#,
        r#""schema":"nomos.entity_catalog@1""#,
    );
    assert_ne!(rewritten, text, "the object spelling was not found");
    std::fs::write(&path, rewritten).unwrap();
    nomos_render_plan::compile(fixture.inputs())
        .expect("the string spelling of the identity binds too");
}

#[test]
fn a_document_that_is_not_canonical_is_refused() {
    let fixture = Fixture::new("not-canonical");
    let path = fixture.catalog();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("  {text}")).unwrap();
    let error = nomos_render_plan::compile(fixture.inputs()).unwrap_err();
    assert_eq!(error.code().as_str(), "RP0102");
}

#[test]
fn a_facts_document_without_a_run_bundle_is_refused() {
    let fixture = Fixture::new("orphan-facts");
    std::fs::copy(
        fixture.facts().join("01-baseline.json"),
        fixture.facts().join("99-orphan.json"),
    )
    .unwrap();
    let error = nomos_render_plan::compile(fixture.inputs()).unwrap_err();
    assert_eq!(error.code().as_str(), "RP0204");
    assert!(
        error.message().contains("99-orphan.json"),
        "{}",
        error.message()
    );
}

#[test]
fn a_run_bundle_without_a_facts_document_is_refused() {
    let fixture = Fixture::new("orphan-run");
    std::fs::remove_file(fixture.facts().join("02-unsealed.json")).unwrap();
    let error = nomos_render_plan::compile(fixture.inputs()).unwrap_err();
    assert_eq!(error.code().as_str(), "RP0204");
    assert!(
        error.message().contains("02-unsealed"),
        "{}",
        error.message()
    );
}

#[test]
fn a_rejected_kernel_document_is_refused_before_its_identity() {
    // `nomos entity-catalog` writes `{"diagnostics":[...],"status":"rejected"}`
    // with no `schema` field when it refuses a package. Reporting "no `schema`
    // field" would throw away the kernel's own reason.
    let fixture = Fixture::new("rejected-catalog");
    std::fs::write(
        fixture.catalog(),
        br#"{"command":"entity-catalog","diagnostics":[{"code":"EK0413","message":"inconsistent"}],"status":"rejected"}"#,
    )
    .unwrap();
    let error = nomos_render_plan::compile(fixture.inputs()).unwrap_err();
    assert_eq!(error.code().as_str(), "RP0105");
    assert!(error.message().contains("rejected"), "{}", error.message());
    assert!(error.message().contains("EK0413"), "{}", error.message());
}

#[test]
fn the_catalogs_extra_command_and_status_fields_are_accepted() {
    // The landed #138 document carries `command` and `status` beside the shape
    // the issue text listed. Both are ignored; the issue's shape is a subset.
    use nomos_render_plan::json::Json;

    let fixture = Fixture::new("catalog-envelope");
    let value = nomos_render_plan::json::parse(&std::fs::read(fixture.catalog()).unwrap()).unwrap();
    let Json::Object(mut fields) = value else {
        panic!("the catalog is an object")
    };
    fields.insert(
        "command".to_owned(),
        Json::Text("entity-catalog".to_owned()),
    );
    fields.insert("status".to_owned(), Json::Text("completed".to_owned()));
    let envelope = nomos_render_plan::PlanValue::from_area(&Json::Object(fields)).unwrap();
    std::fs::write(fixture.catalog(), envelope.to_canonical_bytes()).unwrap();
    nomos_render_plan::compile(fixture.inputs())
        .expect("the kernel stdout envelope fields are ignored");
}
