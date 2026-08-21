//! SW-C happy-path proof against the exact Gate K base fixture.

use estate_compiler::compile_source;
use estate_core::canonical::read::is_canonical;
use estate_core::{SourcePath, id::StableId};
use estate_schema::{CapabilityKind, ClaimActivation, ClaimValue, FactOwner};

const SOURCE: &str = include_str!("../../../fixtures/gaol.estate");
const PATH: &str = "fixtures/gaol.estate";

fn compile() -> estate_schema::WorldIr {
    compile_source(SOURCE, SourcePath::new(PATH).unwrap()).unwrap()
}

#[test]
fn the_base_fixture_is_one_screen_and_names_exactly_the_contract_entities() {
    assert!(SOURCE.lines().count() <= 20);
    let ir = compile();
    let ids: Vec<String> = ir
        .entities()
        .iter()
        .map(|entity| entity.id().canonical_string())
        .collect();
    assert_eq!(ids, ["brazier_02", "flooded_section", "north_gate"]);

    let kinds: Vec<String> = ir
        .entities()
        .iter()
        .map(|entity| entity.primitive().canonical_string())
        .collect();
    assert_eq!(
        kinds,
        [
            "primitive/extinguishable_light",
            "primitive/shallow_water_region",
            "primitive/iron_barred_door",
        ]
    );
    assert_eq!(
        ir.catalog_values()
            .iter()
            .map(StableId::canonical_string)
            .collect::<Vec<_>>(),
        ["credential/gaoler_key"]
    );
}

#[test]
fn the_catalog_credential_never_becomes_a_fourth_entity() {
    let ir = compile();
    assert_eq!(ir.entities().len(), 3);
    let door = ir
        .entities()
        .iter()
        .find(|entity| entity.id().to_string() == "north_gate")
        .unwrap();
    assert_eq!(
        door.credential().unwrap().to_string(),
        "credential/gaoler_key"
    );
}

#[test]
fn approved_primitives_expand_into_machines_capabilities_and_claims() {
    let ir = compile();
    let door = entity(&ir, "north_gate");
    let machine_names: Vec<&str> = door
        .expansion()
        .machines()
        .iter()
        .map(|machine| machine.namespace().local_name().as_str())
        .collect();
    assert_eq!(machine_names, ["access", "combustion", "integrity", "ward"]);
    assert!(
        door.expansion()
            .capabilities()
            .contains(&CapabilityKind::Portal)
    );
    assert_eq!(door.expansion().claims().len(), 2);
    let portal_blocker = door
        .expansion()
        .claims()
        .iter()
        .find(|claim| claim.id().namespace().local_name().as_str() == "portal")
        .unwrap();
    assert!(matches!(
        portal_blocker.activation(),
        ClaimActivation::Not(child)
            if matches!(child.as_ref(), ClaimActivation::Any(children) if children.len() == 2)
    ));

    let water = entity(&ir, "flooded_section");
    let cost = &water.expansion().claims()[0];
    assert_eq!(cost.capability(), CapabilityKind::TraversalCostGround);
    assert_eq!(cost.value(), &ClaimValue::Uint(3));

    let light = entity(&ir, "brazier_02");
    assert_eq!(light.expansion().machines().len(), 1);
    assert_eq!(light.expansion().machines()[0].initial().as_str(), "lit");
    assert_eq!(
        light.expansion().claims()[0].capability(),
        CapabilityKind::EmitsLight
    );
}

#[test]
fn ownership_receipts_name_graph_lattice_and_linker_authority() {
    let ir = compile();
    let receipt = |fact: &str| {
        ir.ownership_receipts()
            .iter()
            .find(|receipt| receipt.fact() == fact)
            .unwrap()
    };
    assert_eq!(
        receipt("entity.north_gate.identity").owner(),
        FactOwner::Graph
    );
    assert_eq!(
        receipt("entity.north_gate.spatial_anchor").owner(),
        FactOwner::Lattice
    );
    assert_eq!(
        receipt("entity.north_gate.spatial_binding").owner(),
        FactOwner::WorldLinker
    );
    assert_eq!(
        receipt("entity.north_gate.credential").owner(),
        FactOwner::WorldLinker
    );
}

#[test]
fn world_ir_bytes_are_canonical_and_repeatable() {
    let ir = compile();
    assert_eq!(ir.schema(), &estate_schema::construction_world_ir_schema());
    assert_eq!(
        ir.schema().name().to_string(),
        "estate.world_ir.construction"
    );
    assert_eq!(ir.schema().version(), 3);
    let first = ir.to_canonical_bytes();
    let second = compile().to_canonical_bytes();
    assert_eq!(first, second);
    assert!(is_canonical(&first));
    let encoded_schema = br#""schema":{"name":"estate.world_ir.construction","version":3}"#;
    assert!(
        first
            .windows(encoded_schema.len())
            .any(|window| window == encoded_schema)
    );
    assert_eq!(
        estate_core::hash::Sha256Digest::of_bytes(&first).to_hex(),
        include_str!("golden/gaol-world-ir-construction-v3.sha256").trim()
    );
    assert!(!first.ends_with(b"\n"));
}

#[test]
fn source_line_endings_do_not_change_what_resolves() {
    let crlf = SOURCE.replace('\n', "\r\n");
    let ir = compile_source(&crlf, SourcePath::new(PATH).unwrap()).unwrap();
    assert_eq!(ir.entities().len(), 3);
    assert_eq!(ir.catalog_values().len(), 1);
}

#[test]
fn a_valid_graph_relation_links_only_after_both_entities_resolve() {
    let source = format!("{SOURCE}\nrelation north_gate owns brazier_02\n");
    let ir = compile_source(&source, SourcePath::new(PATH).unwrap()).unwrap();
    assert_eq!(ir.relations().len(), 1);
    assert_eq!(ir.relations()[0].subject().to_string(), "north_gate");
    assert_eq!(ir.relations()[0].kind().as_str(), "owns");
    assert_eq!(ir.relations()[0].object().to_string(), "brazier_02");
}

fn entity<'a>(ir: &'a estate_schema::WorldIr, id: &str) -> &'a estate_schema::IrEntity {
    ir.entities()
        .iter()
        .find(|entity| entity.id().to_string() == id)
        .unwrap()
}
