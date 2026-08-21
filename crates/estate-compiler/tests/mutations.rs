//! Planted source and ownership violations required by KERNEL.md section 9.

use estate_compiler::{compile_source, diagnostics, link_source};
use estate_core::{Diagnostic, SchemaId, SourcePath, SourceSpan};
use estate_schema::{SourceDocument, Spanned};

const SOURCE: &str = include_str!("../../../fixtures/gaol.estate");
const PATH: &str = "fixtures/mutated-gaol.estate";

fn reject(source: &str, expected: &str) -> Diagnostic {
    let diagnostic = compile_source(source, SourcePath::new(PATH).unwrap()).unwrap_err();
    assert_eq!(diagnostic.code().as_str(), expected, "{diagnostic}");
    let span = diagnostic.span().expect("source rejection needs a span");
    assert_eq!(span.path().as_str(), PATH);
    assert!(span.position().0 > 0);
    diagnostic
}

fn in_door(field: &str) -> String {
    SOURCE.replacen(
        "  credential credential/gaoler_key\nend",
        &format!("  credential credential/gaoler_key\n  {field}\nend"),
        1,
    )
}

#[test]
fn dangling_entity_id_fails_closed() {
    reject(
        &format!("{SOURCE}\nrelation north_gate owns missing_gate\n"),
        diagnostics::DANGLING_ENTITY.as_str(),
    );
}

#[test]
fn unapproved_relation_kind_fails_closed() {
    reject(
        &format!("{SOURCE}\nrelation north_gate illuminates brazier_02\n"),
        diagnostics::UNAPPROVED_RELATION_KIND.as_str(),
    );
}

#[test]
fn dangling_catalog_value_fails_closed() {
    reject(
        &SOURCE.replacen(
            "credential credential/gaoler_key",
            "credential credential/missing_key",
            1,
        ),
        diagnostics::DANGLING_CATALOG_VALUE.as_str(),
    );
}

#[test]
fn catalog_namespaces_remain_typed() {
    let source = SOURCE
        .replacen(
            "catalog credential/gaoler_key",
            "catalog credential/gaoler_key\ncatalog material/gaoler_key",
            1,
        )
        .replacen(
            "credential credential/gaoler_key",
            "credential material/gaoler_key",
            1,
        );
    reject(&source, diagnostics::CATALOG_NAMESPACE_MISMATCH.as_str());
}

#[test]
fn relation_encoded_as_a_lattice_property_fails_closed() {
    reject(
        &in_door("lattice_relation owns brazier_02"),
        diagnostics::RELATION_IN_LATTICE.as_str(),
    );
}

#[test]
fn authored_raw_transform_fails_closed() {
    reject(
        &in_door("transform 1 0 0 0 1 0 0 0 1"),
        diagnostics::RAW_TRANSFORM_AUTHORED.as_str(),
    );
}

#[test]
fn authored_derived_fact_fails_closed() {
    reject(
        &in_door("derived movement_disposition_ground traversable"),
        diagnostics::DERIVED_FACT_AUTHORED.as_str(),
    );
}

#[test]
fn second_canonical_fact_owner_fails_closed() {
    reject(
        &format!("{SOURCE}\nfact_owner spatial.anchor graph\n"),
        diagnostics::DUPLICATE_FACT_OWNER.as_str(),
    );
}

#[test]
fn duplicate_symbols_and_fields_fail_closed() {
    reject(
        &format!("catalog credential/gaoler_key\n{SOURCE}"),
        diagnostics::SOURCE_SCHEMA_REQUIRED.as_str(),
    );
    reject(
        &SOURCE.replacen(
            "catalog credential/gaoler_key",
            "catalog credential/gaoler_key\ncatalog credential/gaoler_key",
            1,
        ),
        diagnostics::DUPLICATE_CATALOG_VALUE.as_str(),
    );
    reject(
        &format!(
            "{SOURCE}\nentity north_gate primitive/extinguishable_light\n  anchor cell 0 0 0\nend\n"
        ),
        diagnostics::DUPLICATE_ENTITY.as_str(),
    );
    reject(
        &in_door("anchor face 6 0 0 north"),
        diagnostics::DUPLICATE_FIELD.as_str(),
    );
}

#[test]
fn primitive_catalog_and_field_shapes_fail_closed() {
    reject(
        &SOURCE.replacen(
            "primitive/extinguishable_light",
            "primitive/unapproved_torch",
            1,
        ),
        diagnostics::UNAPPROVED_PRIMITIVE.as_str(),
    );
    reject(
        &SOURCE.replacen("  credential credential/gaoler_key\n", "", 1),
        diagnostics::REQUIRED_FIELD_MISSING.as_str(),
    );
    reject(
        &SOURCE.replacen("anchor cell 3 1 0", "anchor face 3 1 0 north", 1),
        diagnostics::FIELD_NOT_ALLOWED.as_str(),
    );
}

#[test]
fn inverted_region_bounds_fail_closed() {
    reject(
        &SOURCE.replacen("anchor region 2 2 0 4 3 0", "anchor region 4 2 0 2 3 0", 1),
        diagnostics::REGION_BOUNDS_INVALID.as_str(),
    );
}

#[test]
fn parser_errors_also_carry_stable_codes_and_spans() {
    reject(
        &SOURCE.replacen("anchor cell 3 1 0", "anchor cell +3 1 0", 1),
        diagnostics::SOURCE_INTEGER_INVALID.as_str(),
    );
    reject(
        &in_door("made_up_field oatmeal"),
        diagnostics::SOURCE_UNKNOWN_STATEMENT.as_str(),
    );
    reject(
        SOURCE.trim_end_matches("end\n"),
        diagnostics::SOURCE_UNCLOSED_ENTITY.as_str(),
    );
}

#[test]
fn source_schema_is_mandatory_exactly_once() {
    reject(
        &SOURCE.replacen("schema estate.source@1\n", "", 1),
        diagnostics::SOURCE_SCHEMA_REQUIRED.as_str(),
    );
    reject(
        &SOURCE.replacen(
            "schema estate.source@1",
            "schema estate.source@1\nschema estate.source@1",
            1,
        ),
        diagnostics::SOURCE_SCHEMA_REQUIRED.as_str(),
    );
    reject(
        &SOURCE.replacen("estate.source@1", "estate.source@2", 1),
        diagnostics::SOURCE_SCHEMA_UNSUPPORTED.as_str(),
    );
}

#[test]
fn linker_rechecks_schema_for_callers_that_supply_an_ast_directly() {
    let path = SourcePath::new(PATH).unwrap();
    let span = SourceSpan::new(path, 0, 23, 1, 1).unwrap();
    let document = SourceDocument::new(
        Spanned::new(SchemaId::new("estate.source", 2).unwrap(), span),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let diagnostic = link_source(&document).unwrap_err();
    assert_eq!(diagnostic.code(), diagnostics::SOURCE_SCHEMA_UNSUPPORTED);
    assert!(diagnostic.span().is_some());
}

#[test]
fn every_compiler_code_is_well_formed_and_globally_unique() {
    let mut seen = std::collections::BTreeSet::new();
    for code in estate_core::diagnostic::codes::ALL
        .iter()
        .chain(diagnostics::ALL)
    {
        assert!(code.is_well_formed(), "`{code}` is malformed");
        assert!(seen.insert(code.as_str()), "`{code}` is declared twice");
    }
}
