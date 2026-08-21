//! The frozen hash-domain fixture.
//!
//! `KERNEL.md` section 7 makes SHA-256 over canonical bytes the identity of
//! authoritative state. This file pins that behaviour to a literal, so any
//! change to canonicalisation or hashing — a different escape spelling, a
//! different key order, a different integer rendering — breaks the build
//! instead of silently producing a world with a new identity.
//!
//! # This fixture is not the runtime-state schema
//!
//! It is shaped like the Gate K initial state so it exercises the ordering
//! rules that matter (entities by stable entity ID, machines by canonical
//! namespace ID, integer traversal cost, a catalog reference that is not an
//! entity), but it is declared under its own schema name and is frozen
//! forever. The real runtime-state envelope belongs to `estate-sim` and will
//! change as the kernel grows; if this fixture tracked it, a legitimate schema
//! change would look identical to a canonicalisation regression. It must not.
//!
//! The `HASH <name> <hex>` lines this test prints are the input to CI's
//! determinism step, which runs the test twice and diffs the lines.

use estate_core::canonical::keyed_array;
use estate_core::id::{CatalogValueId, EntityId, NamespaceId, SchemaId, StableId};
use estate_core::{CanonicalValue, StateHash};

/// The exact canonical bytes of the frozen fixture, committed as an
/// inspectable artifact so a reviewer can `diff` rather than trust.
const GOLDEN_BYTES: &[u8] = include_bytes!("golden/hash-domain-fixture.json");

/// The frozen hash of those bytes.
const GOLDEN_HASH: &str = "09ef5bc23dd2e47109dec91aea083e4f883b3c0ff8e021f86dd127c06c94faf8";

fn entity(id: &str, credential: Option<&str>) -> (EntityId, CanonicalValue) {
    let id = EntityId::parse(id).unwrap();
    let mut fields = vec![("id", id.to_canonical())];
    if let Some(credential) = credential {
        fields.push((
            "credential",
            CatalogValueId::parse(credential).unwrap().to_canonical(),
        ));
    }
    (id, CanonicalValue::object_declared(fields))
}

fn machine(namespace: &str, state: &str) -> (NamespaceId, CanonicalValue) {
    let namespace = NamespaceId::parse(namespace).unwrap();
    (
        namespace.clone(),
        CanonicalValue::object_declared([
            ("namespace", namespace.to_canonical()),
            ("state", CanonicalValue::text(state)),
        ]),
    )
}

/// Builds the fixture from deliberately unsorted input, so the ordering rules
/// are doing the work rather than the author being tidy.
fn fixture() -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "entities",
            keyed_array([
                entity("north_gate", Some("credential/gaoler_key")),
                entity("brazier_02", None),
                entity("flooded_section", None),
            ]),
        ),
        (
            "lattice_bindings",
            CanonicalValue::Array(vec![CanonicalValue::object_declared([
                (
                    "entity",
                    EntityId::parse("flooded_section").unwrap().to_canonical(),
                ),
                ("ground_traversal_cost", CanonicalValue::Uint(3)),
                ("region", CanonicalValue::text("flooded_section_region")),
            ])]),
        ),
        (
            "machines",
            keyed_array([
                machine("north_gate.ward", "sealed"),
                machine("brazier_02.emission", "lit"),
                machine("north_gate.access", "locked"),
                machine("north_gate.combustion", "cold"),
                machine("north_gate.integrity", "intact"),
            ]),
        ),
        (
            "schema",
            SchemaId::new("estate.hash_domain_fixture", 1)
                .unwrap()
                .to_canonical(),
        ),
        ("tick", CanonicalValue::Uint(0)),
    ])
}

#[test]
fn the_frozen_fixture_hash_does_not_move() {
    let bytes = fixture().to_canonical_bytes();
    let hash = StateHash::of_envelope(&fixture());

    // The harness leaves `test <name> ... ` open on the current line, so the
    // first HASH line needs its own line start or CI's `grep '^HASH '` silently
    // loses it.
    println!();
    println!("HASH hash_domain_fixture {}", hash.to_hex());
    println!(
        "HASH hash_domain_fixture_empty_object {}",
        StateHash::of_envelope(&CanonicalValue::object([])).to_hex()
    );
    println!(
        "HASH hash_domain_fixture_entities {}",
        StateHash::of_envelope(&fixture().get_entities()).to_hex()
    );

    assert_eq!(
        String::from_utf8(bytes.clone()).unwrap(),
        String::from_utf8(GOLDEN_BYTES.to_vec()).unwrap(),
        "canonical bytes moved; if this is intended, the change is a contract \
         change and needs a decision record"
    );
    assert_eq!(hash.to_hex(), GOLDEN_HASH);
    assert!(!bytes.ends_with(b"\n"));
}

#[test]
fn declaration_order_cannot_change_the_hash() {
    let once = StateHash::of_envelope(&fixture());
    let again = StateHash::of_envelope(&fixture());
    assert_eq!(once, again);
}

trait Entities {
    fn get_entities(&self) -> CanonicalValue;
}

impl Entities for CanonicalValue {
    fn get_entities(&self) -> CanonicalValue {
        match self {
            CanonicalValue::Object(fields) => fields
                .iter()
                .find(|(name, _)| name.as_str() == "entities")
                .map(|(_, value)| value.clone())
                .unwrap_or(CanonicalValue::Null),
            _ => CanonicalValue::Null,
        }
    }
}
