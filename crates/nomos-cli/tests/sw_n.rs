//! SW-N end-to-end proof for package-bound semantic explanations.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName, Sha256Digest};

static COUNTER: AtomicU32 = AtomicU32::new(0);
const SOURCE: &[u8] = include_bytes!("../../../fixtures/gaol.nomos");
const COMMANDS: &[u8] = include_bytes!("../../../fixtures/gaol.commands");
const SEVEN_COMMANDS: &[u8] = include_bytes!("../../../fixtures/gaol-seven.commands");
const RUN_FILES: [&str; 6] = [
    "causal-receipts.json",
    "command-log.json",
    "final-state.json",
    "initial-state.json",
    "result.json",
    "state-hashes.json",
];

fn fresh_workspace(label: &str) -> PathBuf {
    let index = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("sw-n-cli")
        .join(format!("{}-{nonce}-{label}-{index}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("gaol.nomos"), SOURCE).unwrap();
    fs::write(root.join("gaol.commands"), COMMANDS).unwrap();
    fs::write(root.join("gaol-seven.commands"), SEVEN_COMMANDS).unwrap();
    root
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
    assert_ne!(
        output.stdout.get(output.stdout.len().saturating_sub(2)),
        Some(&b'\n')
    );
    parse_canonical(&output.stdout[..output.stdout.len() - 1]).unwrap()
}

fn object(value: &CanonicalValue) -> &BTreeMap<FieldName, CanonicalValue> {
    let CanonicalValue::Object(fields) = value else {
        panic!("expected canonical object")
    };
    fields
}

fn field<'a>(value: &'a CanonicalValue, name: &'static str) -> &'a CanonicalValue {
    object(value).get(&FieldName::declared(name)).unwrap()
}

fn array(value: &CanonicalValue) -> &[CanonicalValue] {
    let CanonicalValue::Array(values) = value else {
        panic!("expected canonical array")
    };
    values
}

fn diagnostic_code(output: &Output) -> String {
    let value = canonical_stdout(output);
    let diagnostic = &array(field(&value, "diagnostics"))[0];
    let CanonicalValue::Text(code) = field(diagnostic, "code") else {
        panic!("diagnostic code must be text")
    };
    code.clone()
}

fn compile(cwd: &Path, source: &str, output: &str) {
    let compiled = run(cwd, ["compile", source, "--out", output]);
    assert_exit(&compiled, 0);
}

fn publish_run(cwd: &Path, commands: &str, output: &str) {
    let published = run(
        cwd,
        ["run", "gaol.world", "--commands", commands, "--out", output],
    );
    assert_exit(&published, 0);
}

fn directory_bytes(root: &Path) -> Vec<(String, Vec<u8>)> {
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

fn copy_run(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    for name in RUN_FILES {
        fs::copy(source.join(name), destination.join(name)).unwrap();
    }
}

fn assert_output_hash(output: &Output, expected: &str) {
    assert_eq!(Sha256Digest::of_bytes(&output.stdout).to_hex(), expected);
}

#[test]
fn explanation_help_and_grammar_are_exact() {
    let cwd = fresh_workspace("grammar");
    let root_help = run(&cwd, ["--help"]);
    assert_exit(&root_help, 0);
    let help = String::from_utf8(root_help.stdout).unwrap();
    assert!(help.contains("nomos explain-entity <world/> <entity>\n"));
    assert!(
        help.contains("nomos explain-transition <run/> <entity> --tick <tick> --world <world/>\n")
    );

    for args in [
        vec!["explain-entity", "--help"],
        vec!["explain-transition", "--help"],
    ] {
        let first = run(&cwd, &args);
        let second = run(&cwd, &args);
        assert_exit(&first, 0);
        assert_eq!(first.stdout, second.stdout);
        assert!(!first.stdout.starts_with(b"{"));
    }

    for args in [
        vec!["explain-entity"],
        vec!["explain-entity", "gaol.world"],
        vec!["explain-entity", "gaol.world", "north_gate", "extra"],
        vec!["explain-transition", "gaol.run", "north_gate"],
        vec![
            "explain-transition",
            "gaol.run",
            "north_gate",
            "--world",
            "gaol.world",
            "--tick",
            "4",
        ],
        vec![
            "explain-transition",
            "gaol.run",
            "north_gate",
            "--tick",
            "four",
            "--world",
            "gaol.world",
        ],
        vec![
            "explain-transition",
            "gaol.run",
            "north_gate",
            "--tick",
            "4",
            "--world",
            "gaol.world",
            "extra",
        ],
    ] {
        let rejected = run(&cwd, args);
        assert_exit(&rejected, 2);
        assert_eq!(diagnostic_code(&rejected), "EK0001");
    }
}

#[cfg(unix)]
#[test]
fn explanation_non_utf8_argv_is_invalid_usage() {
    use std::os::unix::ffi::OsStringExt;

    let cwd = fresh_workspace("non-utf8");
    for args in [
        vec![
            OsString::from("explain-entity"),
            OsString::from("gaol.world"),
            OsString::from_vec(vec![0xff]),
        ],
        vec![
            OsString::from("explain-transition"),
            OsString::from_vec(vec![0xff]),
            OsString::from("north_gate"),
            OsString::from("--tick"),
            OsString::from("4"),
            OsString::from("--world"),
            OsString::from("gaol.world"),
        ],
    ] {
        let rejected = run(&cwd, args);
        assert_exit(&rejected, 2);
        assert_eq!(diagnostic_code(&rejected), "EK0001");
    }
}

#[test]
fn door_water_and_light_reports_freeze_distinct_semantic_causes() {
    let cwd = fresh_workspace("entities");
    compile(&cwd, "gaol.nomos", "gaol.world");
    let package_before = directory_bytes(&cwd.join("gaol.world"));

    let door = run(&cwd, ["explain-entity", "gaol.world", "north_gate"]);
    let water = run(&cwd, ["explain-entity", "gaol.world", "flooded_section"]);
    let light = run(&cwd, ["explain-entity", "gaol.world", "brazier_02"]);
    for output in [&door, &water, &light] {
        assert_exit(output, 0);
    }
    assert_output_hash(
        &door,
        "12ca7b682e6c1a90618821ee2d790c55308d9e4873299cff1c27b8dc32221687",
    );
    assert_output_hash(
        &water,
        "21c1b7013145e7424633c3765e6d34d83c365ccd73a2e05d181d6bccb1c609b1",
    );
    assert_output_hash(
        &light,
        "62baac58f7d0b1f2f783bd717e9224d302ec6643c3fcde4738bf0c71175c78a4",
    );

    let door = canonical_stdout(&door);
    let water = canonical_stdout(&water);
    let light = canonical_stdout(&light);
    assert_eq!(
        field(field(&door, "entity"), "primitive"),
        &CanonicalValue::text("primitive/iron_barred_door")
    );
    assert_eq!(
        array(field(&door, "active_initial_claims")),
        &[
            CanonicalValue::text("north_gate.portal#blocks_ground"),
            CanonicalValue::text("north_gate.ward#blocks_ground"),
        ]
    );
    assert_eq!(
        field(
            field(field(&water, "effective_initial_facts"), "ground_movement"),
            "cost"
        ),
        &CanonicalValue::Int(3)
    );
    assert_eq!(
        field(
            field(field(&light, "effective_initial_facts"), "light_emission"),
            "emitting"
        ),
        &CanonicalValue::Bool(true)
    );
    for report in [&door, &water, &light] {
        assert!(!array(field(report, "ownership_receipts")).is_empty());
        let _ = field(field(report, "schemas"), "world_ir");
        let expansion = field(field(report, "entity"), "expansion");
        let _ = field(expansion, "capabilities");
        let _ = field(expansion, "claims");
        let _ = field(expansion, "machines");
    }
    assert_eq!(directory_bytes(&cwd.join("gaol.world")), package_before);
}

#[test]
fn required_tick_four_and_tick_seven_reports_are_exact_and_read_only() {
    let cwd = fresh_workspace("transitions");
    compile(&cwd, "gaol.nomos", "gaol.world");
    publish_run(&cwd, "gaol.commands", "gaol.run");
    publish_run(&cwd, "gaol-seven.commands", "gaol-seven.run");
    let package_before = directory_bytes(&cwd.join("gaol.world"));
    let primary_before = directory_bytes(&cwd.join("gaol.run"));
    let seven_before = directory_bytes(&cwd.join("gaol-seven.run"));
    let commands_before = fs::read(cwd.join("gaol.commands")).unwrap();
    let replay_before = include_bytes!("../../../fixtures/gaol.replay").to_vec();

    let door = run(
        &cwd,
        [
            "explain-transition",
            "gaol.run",
            "north_gate",
            "--tick",
            "4",
            "--world",
            "gaol.world",
        ],
    );
    let light = run(
        &cwd,
        [
            "explain-transition",
            "gaol-seven.run",
            "brazier_02",
            "--tick",
            "7",
            "--world",
            "gaol.world",
        ],
    );
    assert_exit(&door, 0);
    assert_exit(&light, 0);
    assert_output_hash(
        &door,
        "fd6cb5f6efe69c6c334e8a3c2d5f57883089a15e7c18d21da8870ed4275dd9ac",
    );
    assert_output_hash(
        &light,
        "608cedfd41c1c002c48ee0358afb4230589763c212a60974122332ffbb46ecc0",
    );

    let door = canonical_stdout(&door);
    assert_eq!(field(&door, "tick"), &CanonicalValue::Int(4));
    assert_eq!(
        field(field(&door, "request"), "action"),
        &CanonicalValue::text("ignite")
    );
    assert_eq!(
        array(field(field(&door, "receipt"), "transitions")).len(),
        2
    );
    let light = canonical_stdout(&light);
    assert_eq!(field(&light, "tick"), &CanonicalValue::Int(7));
    assert_eq!(
        array(field(&light, "claims_removed")),
        &[CanonicalValue::text("brazier_02.emission#emits_light")]
    );
    assert_eq!(
        array(field(field(&light, "receipt"), "projection_deltas")).len(),
        3
    );
    let receipt = field(&light, "receipt");
    let _ = field(receipt, "effective_facts_before");
    let _ = field(receipt, "effective_facts_after");
    let _ = field(receipt, "state_hash");
    let _ = field(&light, "source_mapping");

    assert_eq!(directory_bytes(&cwd.join("gaol.world")), package_before);
    assert_eq!(directory_bytes(&cwd.join("gaol.run")), primary_before);
    assert_eq!(directory_bytes(&cwd.join("gaol-seven.run")), seven_before);
    assert_eq!(
        fs::read(cwd.join("gaol.commands")).unwrap(),
        commands_before
    );
    assert_eq!(
        include_bytes!("../../../fixtures/gaol.replay"),
        &*replay_before
    );
}

#[test]
fn explanation_selection_and_verified_input_failures_are_stable() {
    let cwd = fresh_workspace("failures");
    compile(&cwd, "gaol.nomos", "gaol.world");
    publish_run(&cwd, "gaol.commands", "gaol.run");

    for (args, code) in [
        (
            vec!["explain-entity", "gaol.world", "missing_entity"],
            "EK0825",
        ),
        (vec!["explain-entity", "gaol.world", "Bad-Entity"], "EK0101"),
        (
            vec![
                "explain-transition",
                "gaol.run",
                "north_gate",
                "--tick",
                "99",
                "--world",
                "gaol.world",
            ],
            "EK0826",
        ),
        (
            vec![
                "explain-transition",
                "gaol.run",
                "flooded_section",
                "--tick",
                "4",
                "--world",
                "gaol.world",
            ],
            "EK0827",
        ),
    ] {
        let rejected = run(&cwd, args);
        assert_exit(&rejected, 1);
        assert_eq!(diagnostic_code(&rejected), code);
    }

    fs::write(cwd.join("other.nomos"), SOURCE).unwrap();
    compile(&cwd, "other.nomos", "other.world");
    let wrong_world = run(
        &cwd,
        [
            "explain-transition",
            "gaol.run",
            "north_gate",
            "--tick",
            "4",
            "--world",
            "other.world",
        ],
    );
    assert_exit(&wrong_world, 1);
    assert_eq!(diagnostic_code(&wrong_world), "EK0813");

    copy_run(&cwd.join("gaol.run"), &cwd.join("forged.run"));
    let mut forged = fs::read(cwd.join("forged.run/causal-receipts.json")).unwrap();
    forged.push(b'\n');
    fs::write(cwd.join("forged.run/causal-receipts.json"), forged).unwrap();
    let forged = run(
        &cwd,
        [
            "explain-transition",
            "forged.run",
            "north_gate",
            "--tick",
            "4",
            "--world",
            "gaol.world",
        ],
    );
    assert_exit(&forged, 1);
    assert_eq!(diagnostic_code(&forged), "EK0302");

    fs::write(cwd.join("not-a-world"), b"not a directory").unwrap();
    let special_world = run(&cwd, ["explain-entity", "not-a-world", "north_gate"]);
    assert_exit(&special_world, 1);
    assert_eq!(diagnostic_code(&special_world), "EK0409");

    fs::write(cwd.join("not-a-run"), b"not a directory").unwrap();
    let special_run = run(
        &cwd,
        [
            "explain-transition",
            "not-a-run",
            "north_gate",
            "--tick",
            "4",
            "--world",
            "gaol.world",
        ],
    );
    assert_exit(&special_run, 1);
    assert_eq!(diagnostic_code(&special_run), "EK0819");

    let missing_world = run(&cwd, ["explain-entity", "missing.world", "north_gate"]);
    assert_exit(&missing_world, 1);
    assert_eq!(diagnostic_code(&missing_world), "EK0405");
    let missing_run = run(
        &cwd,
        [
            "explain-transition",
            "missing.run",
            "north_gate",
            "--tick",
            "4",
            "--world",
            "gaol.world",
        ],
    );
    assert_exit(&missing_run, 1);
    assert_eq!(diagnostic_code(&missing_run), "EK0818");
}

#[cfg(unix)]
#[test]
fn explanation_refuses_symlinked_package_and_run_roots_and_entries() {
    use std::os::unix::fs::symlink;

    let cwd = fresh_workspace("symlinks");
    compile(&cwd, "gaol.nomos", "gaol.world");
    publish_run(&cwd, "gaol.commands", "gaol.run");
    symlink("gaol.world", cwd.join("world-link")).unwrap();
    symlink("gaol.run", cwd.join("run-link")).unwrap();

    let world = run(&cwd, ["explain-entity", "world-link", "north_gate"]);
    assert_exit(&world, 1);
    assert_eq!(diagnostic_code(&world), "EK0409");

    compile(&cwd, "gaol.nomos", "world-entry.world");
    fs::remove_file(cwd.join("world-entry.world/simulation.json")).unwrap();
    symlink(
        "../gaol.world/simulation.json",
        cwd.join("world-entry.world/simulation.json"),
    )
    .unwrap();
    let world_entry = run(&cwd, ["explain-entity", "world-entry.world", "north_gate"]);
    assert_exit(&world_entry, 1);
    assert_eq!(diagnostic_code(&world_entry), "EK0409");

    let run_link = run(
        &cwd,
        [
            "explain-transition",
            "run-link",
            "north_gate",
            "--tick",
            "4",
            "--world",
            "gaol.world",
        ],
    );
    assert_exit(&run_link, 1);
    assert_eq!(diagnostic_code(&run_link), "EK0819");

    copy_run(&cwd.join("gaol.run"), &cwd.join("run-entry.run"));
    fs::remove_file(cwd.join("run-entry.run/state-hashes.json")).unwrap();
    symlink(
        "../gaol.run/state-hashes.json",
        cwd.join("run-entry.run/state-hashes.json"),
    )
    .unwrap();
    let run_entry = run(
        &cwd,
        [
            "explain-transition",
            "run-entry.run",
            "north_gate",
            "--tick",
            "4",
            "--world",
            "gaol.world",
        ],
    );
    assert_exit(&run_entry, 1);
    assert_eq!(diagnostic_code(&run_entry), "EK0819");
}
