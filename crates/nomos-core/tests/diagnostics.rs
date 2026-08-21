//! The structured diagnostic shape of `KERNEL.md` section 9.

use nomos_core::diagnostic::codes;
use nomos_core::{Diagnostic, RepairClass, SourcePath, SourceSpan};

#[test]
fn every_declared_code_is_well_formed_and_unique() {
    let mut seen = std::collections::BTreeSet::new();
    for code in codes::ALL {
        assert!(
            code.is_well_formed(),
            "`{code}` is not an `EK` + 4-digit code"
        );
        assert!(seen.insert(code.as_str()), "`{code}` is declared twice");
    }
}

#[test]
fn a_diagnostic_carries_code_message_span_and_repairs() {
    let span = SourceSpan::new(
        SourcePath::new("fixtures/gaol.nomos").unwrap(),
        42,
        51,
        7,
        3,
    )
    .unwrap();
    let diagnostic = Diagnostic::new(codes::ID_SHAPE_INVALID, "`north gate` is not an entity id")
        .with_span(span.clone())
        .with_repair(RepairClass::UseSupportedIdentifierShape);

    assert_eq!(diagnostic.code().as_str(), "EK0104");
    assert_eq!(diagnostic.span(), Some(&span));
    assert_eq!(
        diagnostic.repairs(),
        [RepairClass::UseSupportedIdentifierShape]
    );
    assert_eq!(
        diagnostic.to_string(),
        "EK0104: `north gate` is not an entity id (fixtures/gaol.nomos:7:3)"
    );
}

#[test]
fn repairs_are_stable_regardless_of_the_order_they_were_added() {
    let forward = Diagnostic::new(codes::PACKAGE_MEMBER_MISSING, "gone")
        .with_repair(RepairClass::SupplyMissingMember)
        .with_repair(RepairClass::RebuildFromSource);
    let backward = Diagnostic::new(codes::PACKAGE_MEMBER_MISSING, "gone")
        .with_repair(RepairClass::RebuildFromSource)
        .with_repair(RepairClass::SupplyMissingMember)
        .with_repair(RepairClass::RebuildFromSource);
    assert_eq!(forward.repairs(), backward.repairs());
    assert_eq!(forward.to_canonical(), backward.to_canonical());
}

#[test]
fn absolute_paths_cannot_enter_a_diagnostic() {
    // Section 7 excludes absolute paths from hashed material. Making them
    // unrepresentable is cheaper than filtering them later.
    for illegal in [
        "/home/someone/gaol.nomos",
        "",
        "../outside.nomos",
        "C:\\gaol.nomos",
    ] {
        assert_eq!(
            SourcePath::new(illegal).unwrap_err().code().as_str(),
            "EK0102",
            "`{illegal}` must be refused"
        );
    }
    assert!(SourcePath::new("fixtures/gaol.nomos").is_ok());
}

#[test]
fn an_inverted_span_is_refused() {
    let path = SourcePath::new("fixtures/gaol.nomos").unwrap();
    assert!(SourceSpan::new(path.clone(), 9, 4, 1, 1).is_err());
    assert!(SourceSpan::new(path.clone(), 0, 0, 0, 1).is_err());
    assert!(SourceSpan::new(path, 0, 0, 1, 1).is_ok());
}

#[test]
fn diagnostics_render_as_canonical_json() {
    let diagnostic = Diagnostic::new(codes::CANONICAL_NOT_CANONICAL, "not canonical")
        .with_span(SourceSpan::new(SourcePath::new("a.nomos").unwrap(), 1, 2, 3, 4).unwrap())
        .with_repair(RepairClass::EmitCanonicalBytes);
    let bytes = diagnostic.to_canonical().to_canonical_bytes();
    assert_eq!(
        String::from_utf8(bytes).unwrap(),
        r#"{"code":"EK0303","message":"not canonical","repairs":["emit_canonical_bytes"],"span":{"byte_end":2,"byte_start":1,"column":4,"line":3,"path":"a.nomos"}}"#
    );
}
