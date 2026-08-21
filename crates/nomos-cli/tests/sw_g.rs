//! SW-G vertical proof for stable World IR and complete immutable packages.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use nomos_compiler::{compile_world_package, validate_compiled_package};
use nomos_core::canonical::read::parse_canonical;
use nomos_core::package::{MemberName, WorldPackage};
use nomos_core::{CanonicalValue, FieldName, Sha256Digest, SourcePath};

static COUNTER: AtomicU32 = AtomicU32::new(0);

const SOURCE: &str = include_str!("../../../fixtures/gaol.nomos");

fn fresh_path(label: &str) -> PathBuf {
    let index = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("sw-g-packages")
        .join(std::process::id().to_string());
    fs::create_dir_all(&root).unwrap();
    root.join(format!("{label}-{index}.world"))
}

fn compile() -> nomos_compiler::CompiledWorld {
    compile_world_package(SOURCE, SourcePath::new("fixtures/gaol.nomos").unwrap()).unwrap()
}

fn member(name: &str) -> MemberName {
    MemberName::new(name).unwrap()
}

#[test]
fn complete_package_is_deterministic_and_initializes_the_same_state() {
    let first = compile();
    let second = compile();
    assert_eq!(first, second);
    assert_eq!(
        first
            .registry()
            .entries()
            .iter()
            .map(|entry| (entry.artifact(), entry.owner().as_str()))
            .collect::<Vec<_>>(),
        [
            ("compiler-receipts.json", "nomos-compiler"),
            ("diagnostics.json", "nomos-projection"),
            ("manifest.json", "nomos-core"),
            ("navigation.json", "nomos-projection"),
            ("persistence.json", "nomos-projection"),
            ("schemas.json", "nomos-schema"),
            ("simulation.json", "nomos-projection"),
            ("world-ir.json", "nomos-schema"),
        ]
    );
    let produced = nomos_compiler::produced_schemas()
        .into_iter()
        .map(|schema| schema.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut expected_produced = first
        .registry()
        .entries()
        .iter()
        .filter(|entry| entry.artifact() != "manifest.json")
        .map(|entry| entry.schema().to_string())
        .collect::<std::collections::BTreeSet<_>>();
    expected_produced.insert("nomos.world_ir.construction@3".to_owned());
    assert_eq!(produced, expected_produced);
    assert_eq!(
        nomos_compiler::consumed_schemas()
            .into_iter()
            .map(|schema| schema.to_string())
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "nomos.source@1".to_owned(),
            "nomos.world_ir.construction@3".to_owned(),
            "nomos.world_ir@1".to_owned(),
        ]
        .into_iter()
        .collect()
    );
    for entry in first.registry().entries() {
        if entry.artifact() == "manifest.json" {
            assert_eq!(entry.owner().as_str(), "nomos-core");
        } else {
            assert!(
                produced.contains(&entry.schema().to_string()),
                "{} is registered but absent from produced_schemas",
                entry.schema()
            );
        }
    }

    let first_path = fresh_path("deterministic-a");
    let second_path = fresh_path("deterministic-b");
    let first_package = nomos_cli::write_compiled_world(&first, &first_path).unwrap();
    let second_package = nomos_cli::write_compiled_world(&second, &second_path).unwrap();
    assert_eq!(first_package.manifest(), second_package.manifest());
    assert_eq!(
        first_package.manifest().digest().to_hex(),
        include_str!("../../nomos-compiler/tests/golden/gaol-package-nomos-v1.sha256").trim()
    );
    assert_eq!(
        first_package
            .manifest()
            .members()
            .iter()
            .map(|record| record.name().as_str())
            .collect::<Vec<_>>(),
        [
            "compiler-receipts.json",
            "diagnostics.json",
            "navigation.json",
            "persistence.json",
            "schemas.json",
            "simulation.json",
            "world-ir.json",
        ]
    );
    for record in first_package.manifest().members() {
        assert_eq!(
            first_package.member_bytes(record.name()),
            second_package.member_bytes(record.name())
        );
    }

    let first_state = nomos_sim::SimulationState::initialize(first.simulation()).unwrap();
    let second_state = nomos_sim::SimulationState::initialize(second.simulation()).unwrap();
    assert_eq!(
        first_state.to_canonical_bytes(),
        second_state.to_canonical_bytes()
    );
    assert_eq!(first_state.state_hash(), second_state.state_hash());
    let resolved = nomos_sim::resolve_movement(first.simulation(), &first_state).unwrap();
    for row in first.stable_ir().movement_v1() {
        match resolved.get(row.entity()).unwrap() {
            nomos_projection::MovementDisposition::Blocked { .. } => {
                assert!(row.blocked_ground());
                assert_eq!(row.traversal_cost_ground(), None);
            }
            nomos_projection::MovementDisposition::Traversable { cost, .. } => {
                assert!(!row.blocked_ground());
                assert_eq!(row.traversal_cost_ground(), Some(*cost));
            }
        }
    }
    let reopened = nomos_cli::open_compiled_world(&first_path).unwrap();
    let package_state = nomos_cli::initial_state_from_package(&reopened).unwrap();
    assert_eq!(
        package_state.to_canonical_bytes(),
        first_state.to_canonical_bytes()
    );
    assert_eq!(package_state.state_hash(), first_state.state_hash());
}

#[test]
fn package_publication_preserves_inputs_and_existing_evidence() {
    let compiled = compile();
    let source_before = SOURCE.as_bytes().to_vec();
    let path = fresh_path("immutable");
    let first = nomos_cli::write_compiled_world(&compiled, &path).unwrap();
    let manifest_before = fs::read(path.join("manifest.json")).unwrap();
    let rejected = nomos_cli::write_compiled_world(&compiled, &path).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0401");
    assert_eq!(SOURCE.as_bytes(), source_before);
    assert_eq!(
        fs::read(path.join("manifest.json")).unwrap(),
        manifest_before
    );
    assert_eq!(
        nomos_cli::open_compiled_world(&path).unwrap().manifest(),
        first.manifest()
    );

    let invalid_path = fresh_path("compile-rejected");
    assert!(
        nomos_cli::compile_and_write_world(
            "schema nomos.source@1\nentity broken",
            SourcePath::new("bad.nomos").unwrap(),
            &invalid_path,
        )
        .is_err()
    );
    assert!(!invalid_path.exists());
}

#[test]
fn semantic_opener_rejects_member_set_schema_and_version_mutations() {
    let compiled = compile();

    for missing_name in [
        "compiler-receipts.json",
        "diagnostics.json",
        "navigation.json",
        "persistence.json",
        "schemas.json",
        "simulation.json",
        "world-ir.json",
    ] {
        let mut missing = compiled.members().unwrap();
        missing.retain(|(name, _)| name.as_str() != missing_name);
        assert_semantic_rejection(
            &format!("missing-{}", missing_name.trim_end_matches(".json")),
            missing,
            "EK0411",
        );
    }

    let mut extra = compiled.members().unwrap();
    extra.push((
        member("extra.json"),
        CanonicalValue::object_declared([("extra", CanonicalValue::Bool(true))])
            .to_canonical_bytes(),
    ));
    assert_semantic_rejection("extra", extra, "EK0411");

    let mut relabelled = compiled.members().unwrap();
    replace_bytes(
        &mut relabelled,
        "world-ir.json",
        compiled.stable_ir().construction().to_canonical_bytes(),
    );
    assert_semantic_rejection("construction-relabel", relabelled, "EK0412");

    let wrong_schema = mutate_member(&compiled, "simulation.json", |fields| {
        fields.insert(
            FieldName::declared("schema"),
            nomos_projection::navigation_schema().to_canonical(),
        );
    });
    assert_semantic_rejection("wrong-schema", wrong_schema, "EK0412");

    let omitted_version = mutate_member(&compiled, "world-ir.json", |fields| {
        fields.remove(&FieldName::declared("compiler_version"));
    });
    assert_semantic_rejection("missing-version", omitted_version, "EK0412");

    let omitted_provenance = mutate_member(&compiled, "world-ir.json", |fields| {
        fields.remove(&FieldName::declared("ownership_receipts"));
    });
    assert_semantic_rejection("missing-provenance", omitted_provenance, "EK0412");

    let v2_shape = mutate_member(&compiled, "world-ir.json", |fields| {
        fields.insert(
            FieldName::declared("movement_disposition_ground"),
            CanonicalValue::text("blocked"),
        );
    });
    assert_semantic_rejection("v2-smuggled", v2_shape, "EK0412");
}

#[test]
fn semantic_opener_rejects_receipt_ownership_and_projection_disagreement() {
    let compiled = compile();

    let stale_projection = mutate_member(&compiled, "simulation.json", |fields| {
        fields.insert(
            FieldName::declared("light_resolver"),
            CanonicalValue::object_declared([("stale", CanonicalValue::Bool(true))]),
        );
    });
    assert_semantic_rejection("stale-projection", stale_projection, "EK0413");

    let movement_subject = mutate_member(&compiled, "world-ir.json", |fields| {
        let CanonicalValue::Array(rows) = fields
            .get_mut(&FieldName::declared("movement_v1"))
            .expect("stable movement rows exist")
        else {
            panic!("stable movement rows are an array")
        };
        let CanonicalValue::Object(first) = &mut rows[0] else {
            panic!("stable movement row is an object")
        };
        first.insert(
            FieldName::declared("entity"),
            CanonicalValue::text("brazier_02"),
        );
    });
    assert_semantic_rejection("movement-subject-mismatch", movement_subject, "EK0413");

    let invalid_initial = mutate_member(&compiled, "simulation.json", |fields| {
        let CanonicalValue::Array(machines) = fields
            .get_mut(&FieldName::declared("machines"))
            .expect("simulation machines exist")
        else {
            panic!("simulation machines are an array")
        };
        let CanonicalValue::Object(first) = &mut machines[0] else {
            panic!("simulation machine is an object")
        };
        first.insert(
            FieldName::declared("initial"),
            CanonicalValue::text("missing"),
        );
    });
    assert_semantic_rejection("invalid-initial-state", invalid_initial, "EK0412");

    let ownership = mutate_member(&compiled, "schemas.json", |fields| {
        let CanonicalValue::Array(entries) = fields
            .get_mut(&FieldName::declared("entries"))
            .expect("schema registry entries exist")
        else {
            panic!("schema registry entries are an array")
        };
        let CanonicalValue::Object(first) = &mut entries[0] else {
            panic!("schema registry row is an object")
        };
        first.insert(
            FieldName::declared("owner"),
            CanonicalValue::text("nomos-core"),
        );
    });
    assert_semantic_rejection("wrong-owner", ownership, "EK0412");

    let receipts = mutate_member(&compiled, "compiler-receipts.json", |fields| {
        fields.remove(&FieldName::declared("passes"));
    });
    assert_semantic_rejection("malformed-receipts", receipts, "EK0412");
}

fn mutate_member(
    compiled: &nomos_compiler::CompiledWorld,
    name: &str,
    mutate: impl FnOnce(&mut std::collections::BTreeMap<FieldName, CanonicalValue>),
) -> Vec<(MemberName, Vec<u8>)> {
    let mut members = compiled.members().unwrap();
    let (_, bytes) = members
        .iter_mut()
        .find(|(member, _)| member.as_str() == name)
        .unwrap();
    let CanonicalValue::Object(mut fields) = parse_canonical(bytes).unwrap() else {
        panic!("compiled member is an object")
    };
    mutate(&mut fields);
    *bytes = CanonicalValue::Object(fields).to_canonical_bytes();
    members
}

fn replace_bytes(members: &mut [(MemberName, Vec<u8>)], name: &str, replacement: Vec<u8>) {
    members
        .iter_mut()
        .find(|(member, _)| member.as_str() == name)
        .unwrap()
        .1 = replacement;
}

fn assert_semantic_rejection(
    label: &str,
    members: Vec<(MemberName, Vec<u8>)>,
    expected_code: &str,
) {
    let path = fresh_path(label);
    let generic = WorldPackage::write(&path, members).unwrap();
    let rejected = validate_compiled_package(&generic).unwrap_err();
    assert_eq!(rejected.code().as_str(), expected_code, "{label}");
    assert_eq!(
        nomos_cli::open_compiled_world(&path)
            .unwrap_err()
            .code()
            .as_str(),
        expected_code,
        "{label}"
    );
}

#[test]
fn member_hashes_are_exact_canonical_bytes() {
    let compiled = compile();
    for (name, bytes) in compiled.members().unwrap() {
        let path = fresh_path(name.as_str());
        let package = WorldPackage::write(&path, vec![(name.clone(), bytes.clone())]).unwrap();
        assert_eq!(
            package.manifest().members()[0].digest(),
            Sha256Digest::of_bytes(&bytes)
        );
    }
}
