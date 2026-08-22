//! SW-M proof for the required stable movement v1-to-v2 migration.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nomos_compiler::{compile_world_package, migrate_world_package_v1};
use nomos_core::canonical::read::parse_canonical;
use nomos_core::package::{MANIFEST_FILE, MemberName, WorldPackage};
use nomos_core::{CanonicalValue, FieldName, Sha256Digest, SourcePath};
use nomos_sim::{
    CommandScript, PersistedRuntimeState, ReplayLog, SimulationState, execute_requests,
};

static COUNTER: AtomicU32 = AtomicU32::new(0);
const SOURCE: &[u8] = include_bytes!("../../../fixtures/gaol.nomos");
const COMMANDS: &[u8] = include_bytes!("../../../fixtures/gaol.commands");
const LEGACY_FILES: [(&str, &[u8]); 8] = [
    (
        "compiler-receipts.json",
        include_bytes!("../../../fixtures/gaol-v1.world/compiler-receipts.json"),
    ),
    (
        "diagnostics.json",
        include_bytes!("../../../fixtures/gaol-v1.world/diagnostics.json"),
    ),
    (
        MANIFEST_FILE,
        include_bytes!("../../../fixtures/gaol-v1.world/manifest.json"),
    ),
    (
        "navigation.json",
        include_bytes!("../../../fixtures/gaol-v1.world/navigation.json"),
    ),
    (
        "persistence.json",
        include_bytes!("../../../fixtures/gaol-v1.world/persistence.json"),
    ),
    (
        "schemas.json",
        include_bytes!("../../../fixtures/gaol-v1.world/schemas.json"),
    ),
    (
        "simulation.json",
        include_bytes!("../../../fixtures/gaol-v1.world/simulation.json"),
    ),
    (
        "world-ir.json",
        include_bytes!("../../../fixtures/gaol-v1.world/world-ir.json"),
    ),
];

fn fresh_workspace(label: &str) -> PathBuf {
    let index = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("sw-m-migration")
        .join(format!("{}-{nonce}-{label}-{index}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    write_legacy(&root.join("gaol-v1.world"));
    fs::write(root.join("gaol.nomos"), SOURCE).unwrap();
    fs::write(root.join("gaol.commands"), COMMANDS).unwrap();
    root
}

fn write_legacy(root: &Path) {
    fs::create_dir_all(root).unwrap();
    for (name, bytes) in LEGACY_FILES {
        fs::write(root.join(name), bytes).unwrap();
    }
}

fn fixture_legacy_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/gaol-v1.world")
}

fn run<I, S>(cwd: &Path, args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_nomos"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

fn assert_exit(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
}

fn canonical_stdout(output: &Output) -> CanonicalValue {
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    parse_canonical(&output.stdout[..output.stdout.len() - 1]).unwrap()
}

fn object(value: &CanonicalValue) -> &BTreeMap<FieldName, CanonicalValue> {
    let CanonicalValue::Object(fields) = value else {
        panic!("expected canonical object")
    };
    fields
}

fn object_mut(value: &mut CanonicalValue) -> &mut BTreeMap<FieldName, CanonicalValue> {
    let CanonicalValue::Object(fields) = value else {
        panic!("expected canonical object")
    };
    fields
}

fn field<'a>(value: &'a CanonicalValue, name: &'static str) -> &'a CanonicalValue {
    object(value).get(&FieldName::declared(name)).unwrap()
}

fn diagnostic_code(output: &Output) -> String {
    let report = canonical_stdout(output);
    let CanonicalValue::Array(diagnostics) = field(&report, "diagnostics") else {
        panic!("rejection report must contain diagnostics")
    };
    let CanonicalValue::Text(code) = field(&diagnostics[0], "code") else {
        panic!("diagnostic code must be text")
    };
    code.clone()
}

fn tree_bytes(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut rows = fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().into_string().unwrap(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    rows
}

#[test]
fn legacy_and_migrated_semantics_normalize_to_identical_runtime_v2_evidence() {
    let migrated = migrate_world_package_v1(&fixture_legacy_path()).unwrap();
    assert_eq!(
        migrated.source_package_digest().to_hex(),
        "f1af0cc92ea44fd09ba93815bb99cc6c24517b56888f39be33a9d47b1299bab7"
    );
    assert_eq!(
        migrated.source_world_ir_digest().to_hex(),
        "555017cf5e13a33b4bb5b18bae14b7577fd1fc38abf89b1f6f475874600fa493"
    );
    assert_eq!(migrated.compiled_world().stable_ir().schema().version(), 2);
    assert_eq!(nomos_sim::runtime_state_schema().version(), 2);
    assert_eq!(nomos_sim::persisted_runtime_state_schema().version(), 2);

    let active = compile_world_package(
        std::str::from_utf8(SOURCE).unwrap(),
        SourcePath::new("fixtures/gaol.nomos").unwrap(),
    )
    .unwrap();
    assert_eq!(
        migrated.compiled_world().stable_ir().to_canonical_bytes(),
        active.stable_ir().to_canonical_bytes()
    );
    assert_eq!(
        migrated.normalized_legacy_simulation().to_canonical_bytes(),
        migrated.compiled_world().simulation().to_canonical_bytes()
    );
    assert_eq!(
        migrated.compiled_world().simulation().to_canonical_bytes(),
        active.simulation().to_canonical_bytes()
    );

    let requests = CommandScript::from_bytes(COMMANDS).unwrap();
    let legacy_initial = PersistedRuntimeState::new(
        migrated.normalized_legacy_simulation(),
        SimulationState::initialize(migrated.normalized_legacy_simulation()).unwrap(),
    )
    .unwrap();
    let active_initial = PersistedRuntimeState::new(
        active.simulation(),
        SimulationState::initialize(active.simulation()).unwrap(),
    )
    .unwrap();
    assert_eq!(legacy_initial, active_initial);
    let legacy = execute_requests(
        migrated.normalized_legacy_simulation(),
        migrated.source_package_digest(),
        legacy_initial,
        requests.requests(),
    )
    .unwrap();
    let current = execute_requests(
        active.simulation(),
        Sha256Digest::of_bytes(b"active-package-identity"),
        active_initial,
        requests.requests(),
    )
    .unwrap();
    assert_eq!(legacy.initial(), current.initial());
    assert_eq!(legacy.final_state(), current.final_state());
    assert_eq!(legacy.command_log(), current.command_log());
    assert_eq!(legacy.causal_receipts(), current.causal_receipts());
    assert_eq!(legacy.state_hashes(), current.state_hashes());
    assert_eq!(legacy.result().status(), current.result().status());
    assert_eq!(
        legacy.result().runtime_semantics_digest(),
        current.result().runtime_semantics_digest()
    );
    assert_eq!(legacy.result().artifacts(), current.result().artifacts());
    assert_eq!(
        legacy.result().first_state_hash(),
        current.result().first_state_hash()
    );
    assert_eq!(
        legacy.result().final_state_hash(),
        current.result().final_state_hash()
    );
    assert_eq!(
        legacy.result().committed_command_count(),
        current.result().committed_command_count()
    );
    assert_eq!(
        legacy.result().rejection_diagnostic(),
        current.result().rejection_diagnostic()
    );
    assert_eq!(
        legacy.final_state().state_hash().to_hex(),
        "3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc"
    );
}

#[test]
fn migrate_cli_is_deterministic_immutable_and_required_before_runtime_use() {
    let cwd = fresh_workspace("cli");
    let before = tree_bytes(&cwd.join("gaol-v1.world"));
    let help = run(&cwd, ["migrate", "--help"]);
    assert_exit(&help, 0);
    assert!(help.stdout.starts_with(b"Migrate one immutable"));
    for args in [
        vec!["migrate"],
        vec!["migrate", "gaol-v1.world", "--to=2", "--out", "bad.world"],
        vec![
            "migrate",
            "gaol-v1.world",
            "--out",
            "bad.world",
            "--to",
            "2",
        ],
    ] {
        let usage = run(&cwd, args);
        assert_exit(&usage, 2);
        assert_eq!(diagnostic_code(&usage), "EK0001");
    }
    for output in ["first.world", "second.world"] {
        let migrated = run(
            &cwd,
            ["migrate", "gaol-v1.world", "--to", "2", "--out", output],
        );
        assert_exit(&migrated, 0);
        let report = canonical_stdout(&migrated);
        assert_eq!(field(&report, "command"), &CanonicalValue::text("migrate"));
        assert_eq!(
            object(&report)
                .keys()
                .map(FieldName::as_str)
                .collect::<Vec<_>>(),
            [
                "artifacts",
                "command",
                "output",
                "source_package_digest",
                "source_world_ir_digest",
                "source_world_ir_schema",
                "status",
                "target_manifest_digest",
                "target_runtime_state_schema",
                "target_world_ir_schema",
            ]
        );
        assert_eq!(
            field(&report, "source_package_digest"),
            &CanonicalValue::text(
                "f1af0cc92ea44fd09ba93815bb99cc6c24517b56888f39be33a9d47b1299bab7"
            )
        );
        assert_eq!(
            field(&report, "source_world_ir_digest"),
            &CanonicalValue::text(
                "555017cf5e13a33b4bb5b18bae14b7577fd1fc38abf89b1f6f475874600fa493"
            )
        );
        assert_eq!(
            field(&report, "target_manifest_digest"),
            &CanonicalValue::text(
                "42af352bdbf0a3c0642d4f86f6d74384351c44fc40bbe3e3134829dcf715d17a"
            )
        );
        assert_eq!(
            field(&report, "target_runtime_state_schema"),
            &parse_canonical(
                &nomos_sim::runtime_state_schema()
                    .to_canonical()
                    .to_canonical_bytes(),
            )
            .unwrap()
        );
    }
    assert_eq!(
        tree_bytes(&cwd.join("first.world")),
        tree_bytes(&cwd.join("second.world"))
    );
    assert_eq!(tree_bytes(&cwd.join("gaol-v1.world")), before);

    for command in [
        vec!["inspect", "gaol-v1.world"],
        vec![
            "run",
            "gaol-v1.world",
            "--commands",
            "gaol.commands",
            "--out",
            "forbidden.run",
        ],
        vec![
            "command",
            "gaol-v1.world",
            "--state",
            "missing.state",
            "close north_gate",
            "--out",
            "forbidden-command.run",
        ],
        vec![
            "replay",
            "gaol-v1.world",
            "--log",
            "missing.replay",
            "--out",
            "forbidden-replay.run",
        ],
    ] {
        let rejected = run(&cwd, command);
        assert_exit(&rejected, 1);
        assert_eq!(diagnostic_code(&rejected), "EK0414");
    }
    assert!(!cwd.join("forbidden.run").exists());
    assert!(!cwd.join("forbidden-command.run").exists());
    assert!(!cwd.join("forbidden-replay.run").exists());

    let inspected = run(&cwd, ["inspect", "first.world"]);
    assert_exit(&inspected, 0);

    let ordinary = run(
        &cwd,
        [
            "run",
            "first.world",
            "--commands",
            "gaol.commands",
            "--out",
            "ordinary.run",
        ],
    );
    assert_exit(&ordinary, 0);
    let continued = run(
        &cwd,
        [
            "command",
            "first.world",
            "--state",
            "ordinary.run/final-state.json",
            "close north_gate",
            "--out",
            "continued.run",
        ],
    );
    assert_exit(&continued, 0);
    let world = nomos_cli::open_compiled_world(&cwd.join("first.world")).unwrap();
    let script = CommandScript::from_bytes(COMMANDS).unwrap();
    let initial = PersistedRuntimeState::new(
        world.simulation(),
        SimulationState::initialize(world.simulation()).unwrap(),
    )
    .unwrap();
    let execution = execute_requests(
        world.simulation(),
        world.package_digest(),
        initial,
        script.requests(),
    )
    .unwrap();
    fs::write(
        cwd.join("migrated.replay"),
        ReplayLog::from_execution(&execution)
            .unwrap()
            .to_canonical_bytes(),
    )
    .unwrap();
    let replayed = run(
        &cwd,
        [
            "replay",
            "first.world",
            "--log",
            "migrated.replay",
            "--out",
            "replay.run",
        ],
    );
    assert_exit(&replayed, 0);
    assert_eq!(
        tree_bytes(&cwd.join("ordinary.run")),
        tree_bytes(&cwd.join("replay.run"))
    );

    let unsupported = run(
        &cwd,
        ["migrate", "gaol-v1.world", "--to", "3", "--out", "no.world"],
    );
    assert_exit(&unsupported, 1);
    assert_eq!(diagnostic_code(&unsupported), "EK0415");
    assert!(!cwd.join("no.world").exists());
    for output in ["gaol-v1.world", "gaol-v1.world/nested.world"] {
        let overlap = run(
            &cwd,
            ["migrate", "gaol-v1.world", "--to", "2", "--out", output],
        );
        assert_exit(&overlap, 1);
        assert_eq!(diagnostic_code(&overlap), "EK0416");
    }
    let existing = run(
        &cwd,
        [
            "migrate",
            "gaol-v1.world",
            "--to",
            "2",
            "--out",
            "first.world",
        ],
    );
    assert_exit(&existing, 1);
    assert_eq!(diagnostic_code(&existing), "EK0401");

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("gaol-v1.world", cwd.join("legacy-alias")).unwrap();
        let input_alias = run(
            &cwd,
            [
                "migrate",
                "legacy-alias",
                "--to",
                "2",
                "--out",
                "alias-input.world",
            ],
        );
        assert_exit(&input_alias, 1);
        assert_eq!(diagnostic_code(&input_alias), "EK0409");
        let output_alias = run(
            &cwd,
            [
                "migrate",
                "gaol-v1.world",
                "--to",
                "2",
                "--out",
                "legacy-alias/nested.world",
            ],
        );
        assert_exit(&output_alias, 1);
        assert_eq!(diagnostic_code(&output_alias), "EK0416");
    }
    assert_eq!(tree_bytes(&cwd.join("gaol-v1.world")), before);
}

#[test]
fn public_migration_api_rejects_output_overlap_before_mutation() {
    let cwd = fresh_workspace("public-api-overlap");
    let input = cwd.join("gaol-v1.world");
    let before = tree_bytes(&input);

    for output in [input.clone(), input.join("nested.world")] {
        let rejected = nomos_cli::migrate_and_write_world(&input, &output).unwrap_err();
        assert_eq!(rejected.code().as_str(), "EK0416");
        assert_eq!(tree_bytes(&input), before);
        assert!(!input.join("nested.world").exists());
    }
}

#[test]
fn legacy_and_v2_shape_mutations_fail_closed_with_fresh_integrity() {
    let cwd = fresh_workspace("mutations");
    let invalid_v1 = refreshed_legacy_mutation(&cwd, "invalid-v1.world", |world| {
        let rows = array_mut(field_mut(world, "movement_v1"));
        let first = object_mut(&mut rows[0]);
        first.insert(
            FieldName::declared("traversal_cost_ground"),
            CanonicalValue::Uint(4),
        );
    });
    assert_eq!(
        migrate_world_package_v1(&invalid_v1)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0412"
    );

    let masquerade = refreshed_legacy_mutation(&cwd, "masquerade.world", |world| {
        world.insert(
            FieldName::declared("schema"),
            CanonicalValue::object_declared([
                ("name", CanonicalValue::text("nomos.world_ir.construction")),
                ("version", CanonicalValue::Uint(3)),
            ]),
        );
    });
    assert_eq!(
        migrate_world_package_v1(&masquerade)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0412"
    );

    let incomplete = refreshed_legacy_mutation(&cwd, "incomplete.world", |world| {
        array_mut(field_mut(world, "movement_v1")).pop();
    });
    assert_eq!(
        migrate_world_package_v1(&incomplete)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0412"
    );

    let duplicate = refreshed_legacy_mutation(&cwd, "duplicate.world", |world| {
        let rows = array_mut(field_mut(world, "movement_v1"));
        rows.push(rows[0].clone());
    });
    assert_eq!(
        migrate_world_package_v1(&duplicate)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0412"
    );

    let stale_projection =
        refreshed_member_mutation(&cwd, "stale.world", "simulation.json", |value| {
            object_mut(value).insert(
                FieldName::declared("schema"),
                CanonicalValue::object_declared([
                    ("name", CanonicalValue::text("nomos.projection.simulation")),
                    ("version", CanonicalValue::Uint(99)),
                ]),
            );
        });
    assert_eq!(
        migrate_world_package_v1(&stale_projection)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0413"
    );
}

fn refreshed_legacy_mutation(
    cwd: &Path,
    output: &str,
    mutate: impl FnOnce(&mut BTreeMap<FieldName, CanonicalValue>),
) -> PathBuf {
    refreshed_member_mutation(cwd, output, "world-ir.json", |value| {
        mutate(object_mut(value))
    })
}

fn refreshed_member_mutation(
    cwd: &Path,
    output: &str,
    member: &str,
    mutate: impl FnOnce(&mut CanonicalValue),
) -> PathBuf {
    let source = cwd.join("gaol-v1.world");
    let opened = WorldPackage::open(&source).unwrap();
    let mut members = opened
        .manifest()
        .members()
        .iter()
        .map(|row| {
            let bytes = opened.member_bytes(row.name()).unwrap().to_vec();
            (row.name().as_str().to_owned(), bytes)
        })
        .collect::<BTreeMap<_, _>>();
    let mut value = parse_canonical(members.get(member).unwrap()).unwrap();
    mutate(&mut value);
    let bytes = value.to_canonical_bytes();
    members.insert(member.to_owned(), bytes.clone());
    refresh_receipt_artifact(&mut members, member, Sha256Digest::of_bytes(&bytes));
    let destination = cwd.join(output);
    WorldPackage::write(
        &destination,
        members
            .into_iter()
            .map(|(name, bytes)| (MemberName::new(&name).unwrap(), bytes)),
    )
    .unwrap();
    destination
}

fn refresh_receipt_artifact(
    members: &mut BTreeMap<String, Vec<u8>>,
    member: &str,
    digest: Sha256Digest,
) {
    if member == "compiler-receipts.json" {
        return;
    }
    let mut receipt = parse_canonical(members.get("compiler-receipts.json").unwrap()).unwrap();
    let artifacts = array_mut(field_mut(object_mut(&mut receipt), "artifacts"));
    let row = artifacts
        .iter_mut()
        .find(|row| field(row, "name") == &CanonicalValue::text(member))
        .unwrap();
    object_mut(row).insert(
        FieldName::declared("sha256"),
        CanonicalValue::text(digest.to_hex()),
    );
    members.insert(
        "compiler-receipts.json".to_owned(),
        receipt.to_canonical_bytes(),
    );
}

fn field_mut<'a>(
    fields: &'a mut BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
) -> &'a mut CanonicalValue {
    fields.get_mut(&FieldName::declared(name)).unwrap()
}

fn array_mut(value: &mut CanonicalValue) -> &mut Vec<CanonicalValue> {
    let CanonicalValue::Array(values) = value else {
        panic!("expected canonical array")
    };
    values
}
