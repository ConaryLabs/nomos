//! SW-G stable World IR promotion proof.

use nomos_compiler::{
    COMPILER_VERSION, PRIMITIVE_CATALOG_VERSION, compile_simulation_plan, compile_source,
    compile_world, promote_world_ir,
};
use nomos_core::canonical::read::is_canonical;
use nomos_core::{CanonicalValue, EntityId, FieldName, Sha256Digest, SourcePath};
use nomos_schema::{StableGroundMovementV1, StableMovementDispositionGround};

const SOURCE: &str = include_str!("../../../fixtures/gaol.nomos");
const PATH: &str = "fixtures/gaol.nomos";

fn world() -> nomos_schema::StableWorldIr {
    compile_world(SOURCE, SourcePath::new(PATH).unwrap()).unwrap()
}

#[test]
fn stable_v2_is_distinct_complete_and_preserves_construction_evidence() {
    let construction = compile_source(SOURCE, SourcePath::new(PATH).unwrap()).unwrap();
    let construction_bytes = construction.to_canonical_bytes();
    let stable = promote_world_ir(&construction).unwrap();

    assert_eq!(stable.schema(), &nomos_schema::stable_world_ir_schema());
    assert_eq!(stable.schema().name().to_string(), "nomos.world_ir");
    assert_eq!(stable.schema().version(), 2);
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
        stable_bytes
            .windows("movement_disposition_ground".len())
            .any(|window| window == b"movement_disposition_ground")
    );
    assert_eq!(
        Sha256Digest::of_bytes(&stable_bytes).to_hex(),
        include_str!("golden/gaol-world-ir-nomos-v2.sha256").trim()
    );
}

#[test]
fn stable_v2_records_the_exact_initial_movement_shape_and_reasons() {
    let stable = world();
    assert_eq!(stable.movement_v2().len(), 2);
    let water = stable
        .movement_v2()
        .iter()
        .find(|row| row.entity().to_string() == "flooded_section")
        .unwrap();
    assert!(!water.movement_disposition_ground().is_blocked());
    assert_eq!(water.movement_disposition_ground().cost(), Some(3));
    assert_eq!(water.movement_disposition_ground().reasons().len(), 1);
    let gate = stable
        .movement_v2()
        .iter()
        .find(|row| row.entity().to_string() == "north_gate")
        .unwrap();
    assert!(gate.movement_disposition_ground().is_blocked());
    assert_eq!(gate.movement_disposition_ground().cost(), None);
    assert_eq!(gate.movement_disposition_ground().reasons().len(), 2);

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
    assert_eq!(
        StableMovementDispositionGround::blocked(Vec::new())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0909"
    );
    assert_eq!(
        StableMovementDispositionGround::traversable(0, Vec::new())
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

#[test]
fn stable_v2_decoder_rejects_zero_cost_with_fully_canonical_bytes() {
    let mut value =
        nomos_core::canonical::read::parse_canonical(&world().to_canonical_bytes()).unwrap();
    let CanonicalValue::Object(root) = &mut value else {
        panic!("stable World IR is an object")
    };
    let CanonicalValue::Array(rows) = root.get_mut(&FieldName::declared("movement_v2")).unwrap()
    else {
        panic!("stable-v2 movement is an array")
    };
    let CanonicalValue::Object(row) = &mut rows[0] else {
        panic!("stable-v2 movement row is an object")
    };
    let CanonicalValue::Object(disposition) = row
        .get_mut(&FieldName::declared("movement_disposition_ground"))
        .unwrap()
    else {
        panic!("stable-v2 movement disposition is an object")
    };
    disposition.insert(FieldName::declared("cost"), CanonicalValue::Uint(0));
    assert_eq!(
        nomos_schema::StableWorldIr::from_canonical_bytes(&value.to_canonical_bytes())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0412"
    );
}

#[test]
fn stable_v2_decoder_rejects_variants_reasons_and_subject_coverage() {
    assert_v2_decode_rejected(|root| {
        disposition_mut(root, "north_gate")
            .insert(FieldName::declared("kind"), CanonicalValue::text("unknown"));
    });
    assert_v2_decode_rejected(|root| {
        disposition_mut(root, "north_gate").insert(
            FieldName::declared("reasons"),
            CanonicalValue::Array(Vec::new()),
        );
    });
    assert_v2_decode_rejected(|root| {
        let CanonicalValue::Array(reasons) = disposition_mut(root, "north_gate")
            .get_mut(&FieldName::declared("reasons"))
            .unwrap()
        else {
            panic!("movement reasons are an array")
        };
        reasons.push(reasons[0].clone());
    });
    assert_v2_decode_rejected(|root| {
        let CanonicalValue::Array(reasons) = disposition_mut(root, "north_gate")
            .get_mut(&FieldName::declared("reasons"))
            .unwrap()
        else {
            panic!("movement reasons are an array")
        };
        reasons.swap(0, 1);
    });
    assert_v2_decode_rejected(|root| {
        let CanonicalValue::Array(reasons) = disposition_mut(root, "north_gate")
            .get_mut(&FieldName::declared("reasons"))
            .unwrap()
        else {
            panic!("movement reasons are an array")
        };
        reasons[0] = CanonicalValue::text("north_gate.portal#emits_light");
    });
    assert_v2_decode_rejected(|root| {
        let disposition = disposition_mut(root, "flooded_section");
        disposition.remove(&FieldName::declared("cost"));
        disposition.insert(FieldName::declared("kind"), CanonicalValue::text("blocked"));
    });
    assert_v2_decode_rejected(|root| {
        let disposition = disposition_mut(root, "north_gate");
        disposition.insert(FieldName::declared("cost"), CanonicalValue::Uint(1));
        disposition.insert(
            FieldName::declared("kind"),
            CanonicalValue::text("traversable"),
        );
    });
    assert_v2_decode_rejected(|root| {
        movement_rows_mut(root).pop();
    });
    assert_v2_decode_rejected(|root| {
        let rows = movement_rows_mut(root);
        rows.push(rows[0].clone());
    });
}

fn assert_v2_decode_rejected(
    mutate: impl FnOnce(&mut std::collections::BTreeMap<FieldName, CanonicalValue>),
) {
    let mut value =
        nomos_core::canonical::read::parse_canonical(&world().to_canonical_bytes()).unwrap();
    let CanonicalValue::Object(root) = &mut value else {
        panic!("stable World IR is an object")
    };
    mutate(root);
    assert_eq!(
        nomos_schema::StableWorldIr::from_canonical_bytes(&value.to_canonical_bytes())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0412"
    );
}

fn movement_rows_mut(
    root: &mut std::collections::BTreeMap<FieldName, CanonicalValue>,
) -> &mut Vec<CanonicalValue> {
    let CanonicalValue::Array(rows) = root.get_mut(&FieldName::declared("movement_v2")).unwrap()
    else {
        panic!("stable-v2 movement is an array")
    };
    rows
}

fn disposition_mut<'a>(
    root: &'a mut std::collections::BTreeMap<FieldName, CanonicalValue>,
    entity: &str,
) -> &'a mut std::collections::BTreeMap<FieldName, CanonicalValue> {
    let row = movement_rows_mut(root)
        .iter_mut()
        .find(|row| {
            let CanonicalValue::Object(fields) = row else {
                return false;
            };
            fields.get(&FieldName::declared("entity")) == Some(&CanonicalValue::text(entity))
        })
        .unwrap();
    let CanonicalValue::Object(fields) = row else {
        unreachable!()
    };
    let CanonicalValue::Object(disposition) = fields
        .get_mut(&FieldName::declared("movement_disposition_ground"))
        .unwrap()
    else {
        panic!("stable-v2 movement disposition is an object")
    };
    disposition
}
