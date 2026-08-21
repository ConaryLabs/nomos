//! SW-G stable World IR promotion proof.

use nomos_compiler::{
    COMPILER_VERSION, PRIMITIVE_CATALOG_VERSION, compile_simulation_plan, compile_source,
    compile_world, promote_world_ir,
};
use nomos_core::canonical::read::is_canonical;
use nomos_core::{EntityId, Sha256Digest, SourcePath};
use nomos_schema::StableGroundMovementV1;

const SOURCE: &str = include_str!("../../../fixtures/gaol.nomos");
const PATH: &str = "fixtures/gaol.nomos";

fn world() -> nomos_schema::StableWorldIr {
    compile_world(SOURCE, SourcePath::new(PATH).unwrap()).unwrap()
}

#[test]
fn stable_v1_is_distinct_complete_and_preserves_construction_evidence() {
    let construction = compile_source(SOURCE, SourcePath::new(PATH).unwrap()).unwrap();
    let construction_bytes = construction.to_canonical_bytes();
    let stable = promote_world_ir(&construction).unwrap();

    assert_eq!(stable.schema(), &nomos_schema::stable_world_ir_schema());
    assert_eq!(stable.schema().name().to_string(), "nomos.world_ir");
    assert_eq!(stable.schema().version(), 1);
    assert_eq!(stable.compiler_version(), COMPILER_VERSION);
    assert_eq!(
        stable.primitive_catalog_version(),
        PRIMITIVE_CATALOG_VERSION
    );
    assert_eq!(construction.to_canonical_bytes(), construction_bytes);
    assert_eq!(
        construction.schema(),
        &nomos_schema::construction_world_ir_schema()
    );

    let stable_bytes = stable.to_canonical_bytes();
    assert_ne!(stable_bytes, construction_bytes);
    assert!(is_canonical(&stable_bytes));
    assert!(!stable_bytes.ends_with(b"\n"));
    assert!(
        !stable_bytes
            .windows("movement_disposition_ground".len())
            .any(|window| window == b"movement_disposition_ground")
    );
    assert_eq!(
        Sha256Digest::of_bytes(&stable_bytes).to_hex(),
        include_str!("golden/gaol-world-ir-nomos-v1.sha256").trim()
    );
}

#[test]
fn stable_v1_records_the_exact_initial_movement_shape() {
    let stable = world();
    assert_eq!(stable.movement_v1().len(), 2);
    let water = stable
        .movement_v1()
        .iter()
        .find(|row| row.entity().to_string() == "flooded_section")
        .unwrap();
    assert!(!water.blocked_ground());
    assert_eq!(water.traversal_cost_ground(), Some(3));
    let gate = stable
        .movement_v1()
        .iter()
        .find(|row| row.entity().to_string() == "north_gate")
        .unwrap();
    assert!(gate.blocked_ground());
    assert_eq!(gate.traversal_cost_ground(), None);

    let plan = compile_simulation_plan(&stable).unwrap();
    let initial = nomos_projection::simulation_schema();
    assert_eq!(plan.schema(), &initial);
}

#[test]
fn stable_movement_rows_fail_closed() {
    let entity = EntityId::parse("north_gate").unwrap();
    assert_eq!(
        StableGroundMovementV1::new(entity.clone(), true, Some(1))
            .unwrap_err()
            .code()
            .as_str(),
        "EK0909"
    );
    assert_eq!(
        StableGroundMovementV1::new(entity, false, None)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0909"
    );
}

#[test]
fn stable_bytes_are_repeatable() {
    assert_eq!(world().to_canonical_bytes(), world().to_canonical_bytes());
}
