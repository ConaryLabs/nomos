//! Stable identity: typed, non-interchangeable, and ordered the way section 7
//! requires for hashing.

use estate_core::id::{
    CatalogValueId, ClaimRef, EntityId, NamespaceId, PrimitiveKindId, SchemaId, StableId,
};
use estate_core::ident::{Ident, SEPARATORS};

#[test]
fn the_base_fixture_ids_parse() {
    for entity in ["north_gate", "flooded_section", "brazier_02"] {
        assert_eq!(EntityId::parse(entity).unwrap().to_string(), entity);
    }
    let key = CatalogValueId::parse("credential/gaoler_key").unwrap();
    assert_eq!(key.catalog().as_str(), "credential");
    assert_eq!(key.name().as_str(), "gaoler_key");
    assert_eq!(key.to_string(), "credential/gaoler_key");
}

#[test]
fn a_catalog_value_cannot_be_spelled_as_an_entity() {
    // Acceptance 2: `credential/gaoler_key` must resolve without becoming a
    // fourth entity. The `/` is not a legal identifier byte, so the entity
    // parser refuses it outright.
    let rejected = EntityId::parse("credential/gaoler_key").unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0101");

    // And a bare name is not a legal catalog value either: the namespaces are
    // separate in both directions.
    let rejected = CatalogValueId::parse("north_gate").unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0104");
}

#[test]
fn identifier_validation_is_fail_closed() {
    for illegal in [
        "",
        "North_gate",
        "north gate",
        "north-gate",
        "2gate",
        "_gate",
        "gate!",
        "gaté",
    ] {
        assert!(
            EntityId::parse(illegal).is_err(),
            "`{illegal}` must be refused"
        );
    }
    // Section 7 requires NFC-normalised identifiers. Refusing every non-ASCII
    // identifier is how that is guaranteed without a Unicode table: the two
    // spellings of "é" are both refused rather than one silently normalising
    // into the other.
    assert!(EntityId::parse("caf\u{e9}").is_err());
    assert!(EntityId::parse("cafe\u{301}").is_err());
}

#[test]
fn separators_sort_below_every_identifier_byte() {
    // This is the invariant that makes field ordering and canonical-string
    // ordering the same ordering, which is what lets section 7 say "arrays
    // ordered by stable ID" without ambiguity.
    for separator in SEPARATORS {
        for byte in 0_u8..=127 {
            if Ident::is_legal_byte(byte) {
                assert!(
                    separator < byte,
                    "separator {separator:#04x} must sort below identifier byte {byte:#04x}"
                );
            }
        }
    }
}

#[test]
fn namespace_ids_order_by_canonical_string_and_by_parts_alike() {
    let mut ids: Vec<NamespaceId> = [
        "north_gate.ward",
        "north_gate.access",
        "brazier_02.emission",
        "north_gate.integrity",
        "north_gate.combustion",
        // Adversarial: a longer entity whose name is a prefix of another.
        "north_gate_two.access",
    ]
    .iter()
    .map(|text| NamespaceId::parse(text).unwrap())
    .collect();
    ids.sort();

    let by_parts: Vec<String> = ids.iter().map(NamespaceId::canonical_string).collect();
    let mut by_string = by_parts.clone();
    by_string.sort();
    assert_eq!(
        by_parts, by_string,
        "sorting by parts and sorting by canonical string must agree"
    );

    assert_eq!(
        by_parts,
        vec![
            "brazier_02.emission",
            "north_gate.access",
            "north_gate.combustion",
            "north_gate.integrity",
            "north_gate.ward",
            "north_gate_two.access",
        ]
    );
}

#[test]
fn claim_refs_carry_the_semantic_namespace_that_raises_them() {
    let reason = ClaimRef::parse("north_gate.ward#blocks_ground").unwrap();
    assert_eq!(reason.namespace().entity().to_string(), "north_gate");
    assert_eq!(reason.namespace().local_name().as_str(), "ward");
    assert_eq!(reason.capability().as_str(), "blocks_ground");
    assert_eq!(reason.to_string(), "north_gate.ward#blocks_ground");

    let built = ClaimRef::new(
        NamespaceId::new(
            EntityId::parse("north_gate").unwrap(),
            Ident::new("ward").unwrap(),
        ),
        Ident::new("blocks_ground").unwrap(),
    );
    assert_eq!(built, reason);

    assert!(ClaimRef::parse("north_gate.ward").is_err());
    assert!(ClaimRef::parse("north_gate#blocks_ground").is_err());
}

#[test]
fn schema_versions_start_at_one() {
    assert_eq!(
        SchemaId::new("estate.source", 0)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0105"
    );
    let schema = SchemaId::new("estate.source", 2).unwrap();
    assert_eq!(schema.to_string(), "estate.source@2");
    assert_eq!(
        schema.to_canonical().to_canonical_bytes(),
        br#"{"name":"estate.source","version":2}"#.to_vec()
    );
}

#[test]
fn primitive_kinds_are_not_catalog_values() {
    let primitive = PrimitiveKindId::parse("primitive/iron_barred_door").unwrap();
    assert_eq!(primitive.name().as_str(), "iron_barred_door");
    assert_eq!(primitive.to_string(), "primitive/iron_barred_door");
    assert!(PrimitiveKindId::parse("credential/gaoler_key").is_err());
}

#[test]
fn schema_ids_parse_their_wire_spelling() {
    assert_eq!(
        SchemaId::parse("estate.source@1").unwrap(),
        SchemaId::new("estate.source", 1).unwrap()
    );
    for illegal in ["estate.source", "estate.source@", "estate.source@x"] {
        assert!(SchemaId::parse(illegal).is_err());
    }
}
