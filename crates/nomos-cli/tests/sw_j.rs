//! SW-J end-to-end proof for immutable filesystem runtime execution.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nomos_cli::{initial_state_from_package, open_compiled_world, open_run_bundle};
use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName, Sha256Digest};
use nomos_sim::RunStatus;

static COUNTER: AtomicU32 = AtomicU32::new(0);
const SOURCE: &[u8] = include_bytes!("../../../fixtures/gaol.nomos");
const COMMANDS: &[u8] = include_bytes!("../../../fixtures/gaol.commands");
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
        .join("sw-j-cli")
        .join(format!("{}-{nonce}-{label}-{index}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("gaol.nomos"), SOURCE).unwrap();
    fs::write(root.join("gaol.commands"), COMMANDS).unwrap();
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
    parse_canonical(&output.stdout[..output.stdout.len() - 1]).unwrap()
}

fn object(value: &CanonicalValue) -> &std::collections::BTreeMap<FieldName, CanonicalValue> {
    let CanonicalValue::Object(fields) = value else {
        panic!("expected canonical object")
    };
    fields
}

fn object_mut(
    value: &mut CanonicalValue,
) -> &mut std::collections::BTreeMap<FieldName, CanonicalValue> {
    let CanonicalValue::Object(fields) = value else {
        panic!("expected canonical object")
    };
    fields
}

fn field<'a>(value: &'a CanonicalValue, name: &'static str) -> &'a CanonicalValue {
    object(value).get(&FieldName::declared(name)).unwrap()
}

fn compile(cwd: &Path, source: &str, output: &str) {
    let compiled = run(cwd, ["compile", source, "--out", output]);
    assert_exit(&compiled, 0);
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

fn assert_exact_run_files(root: &Path) {
    let names = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        names,
        RUN_FILES
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    );
}

fn assert_no_staging_entries(root: &Path) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        assert!(
            !entry.file_name().to_string_lossy().contains(".staging-"),
            "staging entry survived at {}",
            entry.path().display()
        );
        if entry.file_type().unwrap().is_dir() {
            assert_no_staging_entries(&entry.path());
        }
    }
}

#[test]
fn run_and_command_publish_reopenable_deterministic_evidence() {
    let cwd = fresh_workspace("completed");
    compile(&cwd, "gaol.nomos", "build/gaol.world");
    let package_before = directory_bytes(&cwd.join("build/gaol.world"));
    let commands_before = fs::read(cwd.join("gaol.commands")).unwrap();

    let first = run(
        &cwd,
        [
            "run",
            "build/gaol.world",
            "--commands",
            "gaol.commands",
            "--out",
            "runs/first",
        ],
    );
    let second = run(
        &cwd,
        [
            "run",
            "build/gaol.world",
            "--commands",
            "gaol.commands",
            "--out",
            "runs/second",
        ],
    );
    assert_exit(&first, 0);
    assert_exit(&second, 0);
    assert_eq!(
        directory_bytes(&cwd.join("runs/first")),
        directory_bytes(&cwd.join("runs/second"))
    );
    assert_exact_run_files(&cwd.join("runs/first"));

    let report = canonical_stdout(&first);
    assert_eq!(field(&report, "command"), &CanonicalValue::text("run"));
    assert_eq!(field(&report, "status"), &CanonicalValue::text("completed"));
    assert_eq!(
        field(&report, "committed_command_count"),
        &CanonicalValue::Int(5)
    );
    let CanonicalValue::Array(artifacts) = field(&report, "artifacts") else {
        panic!("runtime artifacts must be an array")
    };
    assert_eq!(artifacts.len(), 6);

    let world = open_compiled_world(&cwd.join("build/gaol.world")).unwrap();
    let opened = open_run_bundle(&cwd.join("runs/first"), &world).unwrap();
    let expected_initial = nomos_sim::PersistedRuntimeState::new(
        world.simulation(),
        initial_state_from_package(&world).unwrap(),
    )
    .unwrap();
    assert_eq!(opened.initial(), &expected_initial);
    assert_eq!(opened.result().status(), RunStatus::Completed);
    assert_eq!(opened.command_log().rows().len(), 5);
    assert_eq!(opened.causal_receipts().receipts().len(), 5);
    assert_eq!(opened.state_hashes().rows().len(), 6);
    assert_eq!(opened.initial().state().tick(), 0);
    assert_eq!(opened.final_state().state().tick(), 5);

    let state_path = cwd.join("runs/first/final-state.json");
    let state_before = fs::read(&state_path).unwrap();
    let command = run(
        &cwd,
        [
            "command",
            "build/gaol.world",
            "--state",
            "runs/first/final-state.json",
            "close north_gate",
            "--out",
            "runs/after-close",
        ],
    );
    assert_exit(&command, 0);
    let command_report = canonical_stdout(&command);
    assert_eq!(
        field(&command_report, "command"),
        &CanonicalValue::text("command")
    );
    let reopened = open_run_bundle(&cwd.join("runs/after-close"), &world).unwrap();
    assert_eq!(reopened.initial().state().tick(), 5);
    assert_eq!(reopened.final_state().state().tick(), 6);
    assert_eq!(reopened.command_log().rows().len(), 1);

    assert_eq!(
        directory_bytes(&cwd.join("build/gaol.world")),
        package_before
    );
    assert_eq!(
        fs::read(cwd.join("gaol.commands")).unwrap(),
        commands_before
    );
    assert_eq!(fs::read(state_path).unwrap(), state_before);
}

#[test]
fn runtime_rejections_publish_empty_or_partial_evidence_and_stop() {
    let cwd = fresh_workspace("rejected");
    compile(&cwd, "gaol.nomos", "gaol.world");
    fs::write(
        cwd.join("first.commands"),
        b"schema nomos.command_script@1\nclose north_gate\n",
    )
    .unwrap();
    fs::write(
        cwd.join("later.commands"),
        b"schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nunlock north_gate with credential/gaoler_key\nopen north_gate\n",
    )
    .unwrap();

    let first = run(
        &cwd,
        [
            "run",
            "gaol.world",
            "--commands",
            "first.commands",
            "--out",
            "first.run",
        ],
    );
    let later = run(
        &cwd,
        [
            "run",
            "gaol.world",
            "--commands",
            "later.commands",
            "--out",
            "later.run",
        ],
    );
    assert_exit(&first, 1);
    assert_exit(&later, 1);
    assert_eq!(
        field(&canonical_stdout(&first), "status"),
        &CanonicalValue::text("rejected")
    );

    let world = open_compiled_world(&cwd.join("gaol.world")).unwrap();
    let first = open_run_bundle(&cwd.join("first.run"), &world).unwrap();
    assert_eq!(first.result().status(), RunStatus::Rejected);
    assert_eq!(first.result().committed_command_count(), 0);
    assert_eq!(
        first.initial().state_hash(),
        first.final_state().state_hash()
    );
    assert_eq!(
        first.result().rejection_diagnostic().unwrap().as_str(),
        "EK0804"
    );

    let later = open_run_bundle(&cwd.join("later.run"), &world).unwrap();
    assert_eq!(later.result().status(), RunStatus::Rejected);
    assert_eq!(later.result().committed_command_count(), 1);
    assert_eq!(later.command_log().rows().len(), 1);
    assert_eq!(later.final_state().state().tick(), 1);
}

#[test]
fn usage_environment_existing_output_and_cross_semantics_fail_closed() {
    let cwd = fresh_workspace("failures");
    compile(&cwd, "gaol.nomos", "one.world");
    for args in [["run", "--help"], ["command", "--help"]] {
        let help = run(&cwd, args);
        assert_exit(&help, 0);
        assert!(!help.stdout.starts_with(b"{"));
    }
    let initial = run(
        &cwd,
        [
            "run",
            "one.world",
            "--commands",
            "gaol.commands",
            "--out",
            "initial.run",
        ],
    );
    assert_exit(&initial, 0);

    for args in [
        vec!["run", "one.world", "gaol.commands", "--out", "bad.run"],
        vec![
            "command",
            "one.world",
            "--state",
            "initial.run/final-state.json",
            "--out",
            "bad.run",
            "close north_gate",
        ],
    ] {
        assert_exit(&run(&cwd, args), 2);
    }
    assert_exit(
        &run(
            &cwd,
            [
                "run",
                "one.world",
                "--commands",
                "missing.commands",
                "--out",
                "missing.run",
            ],
        ),
        3,
    );
    assert!(!cwd.join("missing.run").exists());

    let existing = cwd.join("existing.run");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("sentinel"), b"unchanged").unwrap();
    let existing_before = directory_bytes(&existing);
    let rejected = run(
        &cwd,
        [
            "run",
            "one.world",
            "--commands",
            "gaol.commands",
            "--out",
            "existing.run",
        ],
    );
    assert_exit(&rejected, 1);
    assert_eq!(directory_bytes(&existing), existing_before);

    let package_before = directory_bytes(&cwd.join("one.world"));
    let collision = run(
        &cwd,
        [
            "run",
            "one.world",
            "--commands",
            "gaol.commands",
            "--out",
            "one.world",
        ],
    );
    assert_exit(&collision, 1);
    assert_eq!(directory_bytes(&cwd.join("one.world")), package_before);

    let nested_run = run(
        &cwd,
        [
            "run",
            "one.world",
            "--commands",
            "gaol.commands",
            "--out",
            "one.world/nested.run",
        ],
    );
    assert_exit(&nested_run, 1);
    let nested_report = canonical_stdout(&nested_run);
    let CanonicalValue::Array(diagnostics) = field(&nested_report, "diagnostics") else {
        panic!("rejection report must contain diagnostics")
    };
    assert_eq!(
        field(&diagnostics[0], "code"),
        &CanonicalValue::text("EK0821")
    );
    assert_eq!(directory_bytes(&cwd.join("one.world")), package_before);
    assert!(!cwd.join("one.world/nested.run").exists());

    let nested_command = run(
        &cwd,
        [
            "command",
            "one.world",
            "--state",
            "initial.run/final-state.json",
            "close north_gate",
            "--out",
            "one.world/nested-command.run",
        ],
    );
    assert_exit(&nested_command, 1);
    assert_eq!(directory_bytes(&cwd.join("one.world")), package_before);
    assert!(!cwd.join("one.world/nested-command.run").exists());

    let state_bundle_before = directory_bytes(&cwd.join("initial.run"));
    let nested_in_state_bundle = run(
        &cwd,
        [
            "command",
            "one.world",
            "--state",
            "initial.run/final-state.json",
            "close north_gate",
            "--out",
            "initial.run/nested-command.run",
        ],
    );
    assert_exit(&nested_in_state_bundle, 1);
    assert_eq!(
        directory_bytes(&cwd.join("initial.run")),
        state_bundle_before
    );
    assert!(!cwd.join("initial.run/nested-command.run").exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        symlink("one.world", cwd.join("package-alias")).unwrap();
        let aliased_nested = run(
            &cwd,
            [
                "run",
                "one.world",
                "--commands",
                "gaol.commands",
                "--out",
                "package-alias/aliased.run",
            ],
        );
        assert_exit(&aliased_nested, 1);
        assert_eq!(directory_bytes(&cwd.join("one.world")), package_before);
        assert!(!cwd.join("one.world/aliased.run").exists());

        symlink("initial.run", cwd.join("run-alias")).unwrap();
        let aliased_state_bundle = run(
            &cwd,
            [
                "command",
                "one.world",
                "--state",
                "run-alias/final-state.json",
                "close north_gate",
                "--out",
                "run-alias/aliased-command.run",
            ],
        );
        assert_exit(&aliased_state_bundle, 1);
        assert_eq!(
            directory_bytes(&cwd.join("initial.run")),
            state_bundle_before
        );
        assert!(!cwd.join("initial.run/aliased-command.run").exists());
    }

    let changed = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("credential/gaoler_key", "credential/other_key");
    fs::write(cwd.join("changed.nomos"), changed).unwrap();
    compile(&cwd, "changed.nomos", "two.world");
    let cross = run(
        &cwd,
        [
            "command",
            "two.world",
            "--state",
            "initial.run/final-state.json",
            "close north_gate",
            "--out",
            "cross.run",
        ],
    );
    assert_exit(&cross, 1);
    assert!(!cwd.join("cross.run").exists());

    let state_path = cwd.join("initial.run/final-state.json");
    let state_before = fs::read(&state_path).unwrap();
    let state_collision = run(
        &cwd,
        [
            "command",
            "one.world",
            "--state",
            "initial.run/final-state.json",
            "close north_gate",
            "--out",
            "initial.run/final-state.json",
        ],
    );
    assert_exit(&state_collision, 1);
    assert_eq!(fs::read(state_path).unwrap(), state_before);

    let invalid_request = run(
        &cwd,
        [
            "command",
            "one.world",
            "--state",
            "initial.run/final-state.json",
            "close  north_gate",
            "--out",
            "invalid-request.run",
        ],
    );
    assert_exit(&invalid_request, 1);
    assert!(!cwd.join("invalid-request.run").exists());
    assert_no_staging_entries(&cwd);
}

#[test]
fn run_open_refuses_tampering_missing_extra_and_nested_entries() {
    let cwd = fresh_workspace("mutations");
    compile(&cwd, "gaol.nomos", "gaol.world");
    let completed = run(
        &cwd,
        [
            "run",
            "gaol.world",
            "--commands",
            "gaol.commands",
            "--out",
            "valid.run",
        ],
    );
    assert_exit(&completed, 0);
    let world = open_compiled_world(&cwd.join("gaol.world")).unwrap();

    let mut same_semantics_source = SOURCE.to_vec();
    same_semantics_source.push(b'\n');
    fs::write(cwd.join("gaol.nomos"), same_semantics_source).unwrap();
    compile(&cwd, "gaol.nomos", "same-semantics.world");
    let same_semantics = open_compiled_world(&cwd.join("same-semantics.world")).unwrap();
    assert_ne!(world.package_digest(), same_semantics.package_digest());
    assert_eq!(
        world.simulation().to_canonical_bytes(),
        same_semantics.simulation().to_canonical_bytes()
    );
    assert_eq!(
        open_run_bundle(&cwd.join("valid.run"), &same_semantics)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0816"
    );

    copy_run(&cwd.join("valid.run"), &cwd.join("tampered.run"));
    let mut bytes = fs::read(cwd.join("tampered.run/command-log.json")).unwrap();
    bytes.push(b'\n');
    fs::write(cwd.join("tampered.run/command-log.json"), bytes).unwrap();
    assert!(open_run_bundle(&cwd.join("tampered.run"), &world).is_err());

    copy_run(&cwd.join("valid.run"), &cwd.join("digest.run"));
    fs::copy(
        cwd.join("digest.run/initial-state.json"),
        cwd.join("digest.run/final-state.json"),
    )
    .unwrap();
    assert_eq!(
        open_run_bundle(&cwd.join("digest.run"), &world)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0816"
    );

    copy_run(&cwd.join("valid.run"), &cwd.join("extra.run"));
    fs::write(cwd.join("extra.run/extra.json"), b"{}").unwrap();
    assert_eq!(
        open_run_bundle(&cwd.join("extra.run"), &world)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0818"
    );

    copy_run(&cwd.join("valid.run"), &cwd.join("missing.run"));
    fs::remove_file(cwd.join("missing.run/state-hashes.json")).unwrap();
    assert_eq!(
        open_run_bundle(&cwd.join("missing.run"), &world)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0818"
    );

    copy_run(&cwd.join("valid.run"), &cwd.join("nested.run"));
    fs::create_dir(cwd.join("nested.run/nested")).unwrap();
    assert_eq!(
        open_run_bundle(&cwd.join("nested.run"), &world)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0819"
    );
}

#[test]
fn run_open_reexecutes_a_fully_rehashed_committed_prefix() {
    let cwd = fresh_workspace("semantic-reexecution");
    compile(&cwd, "gaol.nomos", "gaol.world");
    fs::write(
        cwd.join("unlock.commands"),
        b"schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\n",
    )
    .unwrap();
    let completed = run(
        &cwd,
        [
            "run",
            "gaol.world",
            "--commands",
            "unlock.commands",
            "--out",
            "valid.run",
        ],
    );
    assert_exit(&completed, 0);
    copy_run(&cwd.join("valid.run"), &cwd.join("forged.run"));
    let world = open_compiled_world(&cwd.join("gaol.world")).unwrap();
    let forged = cwd.join("forged.run");

    let final_path = forged.join("final-state.json");
    let final_text = String::from_utf8(fs::read(&final_path).unwrap())
        .unwrap()
        .replace("\"state\":\"closed\"", "\"state\":\"locked\"");
    let mut final_value = parse_canonical(final_text.as_bytes()).unwrap();
    let state_value = object(&final_value)
        .get(&FieldName::declared("state"))
        .unwrap();
    let forged_state = nomos_sim::SimulationState::from_canonical_bytes(
        &state_value.to_canonical_bytes(),
        world.simulation(),
    )
    .unwrap();
    let forged_hash = forged_state.state_hash();
    object_mut(&mut final_value).insert(
        FieldName::declared("state_hash"),
        CanonicalValue::text(forged_hash.to_hex()),
    );
    fs::write(&final_path, final_value.to_canonical_bytes()).unwrap();

    let receipts_path = forged.join("causal-receipts.json");
    let mut receipts_value = parse_canonical(&fs::read(&receipts_path).unwrap()).unwrap();
    let CanonicalValue::Array(receipts) = object_mut(&mut receipts_value)
        .get_mut(&FieldName::declared("receipts"))
        .unwrap()
    else {
        panic!("receipt sequence must contain an array")
    };
    object_mut(&mut receipts[0]).insert(
        FieldName::declared("state_hash"),
        CanonicalValue::text(forged_hash.to_hex()),
    );
    let receipts_bytes = receipts_value.to_canonical_bytes();
    let forged_receipts =
        nomos_sim::CausalReceiptSequence::from_canonical_bytes(&receipts_bytes).unwrap();
    let receipt_digest = forged_receipts.receipts()[0].digest();
    fs::write(&receipts_path, receipts_bytes).unwrap();

    let log_path = forged.join("command-log.json");
    let mut log_value = parse_canonical(&fs::read(&log_path).unwrap()).unwrap();
    let CanonicalValue::Array(rows) = object_mut(&mut log_value)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("command log must contain an array")
    };
    object_mut(&mut rows[0]).insert(
        FieldName::declared("causal_receipt_digest"),
        CanonicalValue::text(receipt_digest.to_hex()),
    );
    object_mut(&mut rows[0]).insert(
        FieldName::declared("resulting_state_hash"),
        CanonicalValue::text(forged_hash.to_hex()),
    );
    let log_bytes = log_value.to_canonical_bytes();
    let forged_log = nomos_sim::CommandLog::from_canonical_bytes(&log_bytes).unwrap();
    fs::write(&log_path, log_bytes).unwrap();

    let hashes_path = forged.join("state-hashes.json");
    let mut hashes_value = parse_canonical(&fs::read(&hashes_path).unwrap()).unwrap();
    let CanonicalValue::Array(rows) = object_mut(&mut hashes_value)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("state-hash sequence must contain an array")
    };
    object_mut(&mut rows[1]).insert(
        FieldName::declared("state_hash"),
        CanonicalValue::text(forged_hash.to_hex()),
    );
    let hashes_bytes = hashes_value.to_canonical_bytes();
    let forged_hashes = nomos_sim::StateHashSequence::from_canonical_bytes(&hashes_bytes).unwrap();
    fs::write(&hashes_path, hashes_bytes).unwrap();

    let initial = nomos_sim::PersistedRuntimeState::from_canonical_bytes(
        &fs::read(forged.join("initial-state.json")).unwrap(),
        world.simulation(),
    )
    .unwrap();
    let final_state = nomos_sim::PersistedRuntimeState::from_canonical_bytes(
        &fs::read(&final_path).unwrap(),
        world.simulation(),
    )
    .unwrap();
    let result = nomos_sim::RunResult::completed(
        world.package_digest(),
        &initial,
        &final_state,
        &forged_log,
        &forged_receipts,
        &forged_hashes,
    )
    .unwrap();
    fs::write(forged.join("result.json"), result.to_canonical_bytes()).unwrap();

    assert_eq!(
        open_run_bundle(&forged, &world)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0816"
    );
    assert_ne!(
        Sha256Digest::of_bytes(&fs::read(forged.join("result.json")).unwrap()),
        Sha256Digest::of_bytes(&fs::read(cwd.join("valid.run/result.json")).unwrap())
    );
}

#[cfg(unix)]
#[test]
fn run_open_refuses_root_and_entry_symlinks_and_special_files() {
    use std::os::unix::fs::symlink;

    let cwd = fresh_workspace("entry-types");
    compile(&cwd, "gaol.nomos", "gaol.world");
    let completed = run(
        &cwd,
        [
            "run",
            "gaol.world",
            "--commands",
            "gaol.commands",
            "--out",
            "valid.run",
        ],
    );
    assert_exit(&completed, 0);
    let world = open_compiled_world(&cwd.join("gaol.world")).unwrap();

    symlink("valid.run", cwd.join("alias.run")).unwrap();
    for alias in [cwd.join("alias.run"), cwd.join("alias.run/")] {
        assert_eq!(
            open_run_bundle(&alias, &world).unwrap_err().code().as_str(),
            "EK0819"
        );
    }

    copy_run(&cwd.join("valid.run"), &cwd.join("symlink-entry.run"));
    fs::remove_file(cwd.join("symlink-entry.run/state-hashes.json")).unwrap();
    symlink(
        "../valid.run/state-hashes.json",
        cwd.join("symlink-entry.run/state-hashes.json"),
    )
    .unwrap();
    assert_eq!(
        open_run_bundle(&cwd.join("symlink-entry.run"), &world)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0819"
    );

    assert_eq!(
        open_run_bundle(Path::new("/dev/null"), &world)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0819"
    );
}
