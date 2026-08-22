//! SW-K end-to-end proof for strict deterministic replay.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nomos_cli::{initial_state_from_package, open_compiled_world, open_run_bundle};
use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName};
use nomos_sim::{
    CommandRequest, CommandScript, PersistedRuntimeState, ReplayLog, RunStatus, execute_requests,
};

static COUNTER: AtomicU32 = AtomicU32::new(0);
const SOURCE: &[u8] = include_bytes!("../../../fixtures/gaol.nomos");
const COMMANDS: &[u8] = include_bytes!("../../../fixtures/gaol.commands");
const REPLAY: &[u8] = include_bytes!("../../../fixtures/gaol.replay");
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
        .join("sw-k-cli")
        .join(format!("{}-{nonce}-{label}-{index}", std::process::id()));
    fs::create_dir_all(root.join("fixtures")).unwrap();
    fs::write(root.join("fixtures/gaol.nomos"), SOURCE).unwrap();
    fs::write(root.join("gaol.commands"), COMMANDS).unwrap();
    fs::write(root.join("gaol.replay"), REPLAY).unwrap();
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

fn compile(cwd: &Path, source: &str, output: &str) {
    let compiled = run(cwd, ["compile", source, "--out", output]);
    assert_exit(&compiled, 0);
}

fn replay(cwd: &Path, package: &str, log: &str, output: &str) -> Output {
    run(cwd, ["replay", package, "--log", log, "--out", output])
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

fn write_value(root: &Path, name: &str, value: &CanonicalValue) {
    fs::write(root.join(name), value.to_canonical_bytes()).unwrap();
}

fn valid_typed_execution(
    cwd: &Path,
) -> (nomos_compiler::OpenedCompiledWorld, nomos_sim::RunExecution) {
    compile(cwd, "fixtures/gaol.nomos", "gaol.world");
    let world = open_compiled_world(&cwd.join("gaol.world")).unwrap();
    let initial = PersistedRuntimeState::new(
        world.simulation(),
        initial_state_from_package(&world).unwrap(),
    )
    .unwrap();
    let script = CommandScript::from_bytes(COMMANDS).unwrap();
    let execution = execute_requests(
        world.simulation(),
        world.package_digest(),
        initial,
        script.requests(),
    )
    .unwrap();
    (world, execution)
}

#[test]
fn checked_in_replay_is_exactly_derived_and_strictly_decoded() {
    let cwd = fresh_workspace("typed");
    let (world, execution) = valid_typed_execution(&cwd);
    let derived = ReplayLog::from_execution(&execution).unwrap();
    assert_eq!(derived.to_canonical_bytes(), REPLAY);

    let decoded = ReplayLog::from_canonical_bytes(REPLAY).unwrap();
    assert_eq!(decoded, derived);
    assert_eq!(decoded.input_package_digest(), world.package_digest());
    assert_eq!(
        decoded.runtime_semantics_digest(),
        execution.initial().runtime_semantics_digest()
    );
    assert_eq!(
        decoded.initial_state_hash(),
        execution.initial().state_hash()
    );
    assert_eq!(
        decoded.expected_final_state_hash(),
        execution.final_state().state_hash()
    );
    assert_eq!(decoded.expected_command_log().rows().len(), 5);

    let rejected = execute_requests(
        world.simulation(),
        world.package_digest(),
        execution.initial().clone(),
        &[CommandRequest::from_line("close north_gate").unwrap()],
    )
    .unwrap();
    assert_eq!(
        ReplayLog::from_execution(&rejected)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0822"
    );

    let mut trailing = REPLAY.to_vec();
    trailing.push(b'\n');
    assert_eq!(
        ReplayLog::from_canonical_bytes(&trailing)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0822"
    );

    let original = parse_canonical(REPLAY).unwrap();
    let mut unknown = original.clone();
    object_mut(&mut unknown).insert(FieldName::declared("unknown"), CanonicalValue::Null);
    assert_eq!(
        ReplayLog::from_canonical_bytes(&unknown.to_canonical_bytes())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0822"
    );

    let mut wrong_schema = original.clone();
    let schema = object_mut(&mut wrong_schema)
        .get_mut(&FieldName::declared("schema"))
        .unwrap();
    object_mut(schema).insert(FieldName::declared("version"), CanonicalValue::Int(2));
    assert_eq!(
        ReplayLog::from_canonical_bytes(&wrong_schema.to_canonical_bytes())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0822"
    );

    let mut empty = original.clone();
    let log = object_mut(&mut empty)
        .get_mut(&FieldName::declared("expected_command_log"))
        .unwrap();
    object_mut(log).insert(
        FieldName::declared("rows"),
        CanonicalValue::Array(Vec::new()),
    );
    assert_eq!(
        ReplayLog::from_canonical_bytes(&empty.to_canonical_bytes())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0822"
    );

    let mut reordered = original.clone();
    let log = object_mut(&mut reordered)
        .get_mut(&FieldName::declared("expected_command_log"))
        .unwrap();
    let CanonicalValue::Array(rows) = object_mut(log)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("expected replay rows")
    };
    rows.swap(0, 1);
    assert_eq!(
        ReplayLog::from_canonical_bytes(&reordered.to_canonical_bytes())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0822"
    );
}

#[test]
fn replay_cli_reproduces_the_exact_run_deterministically() {
    let cwd = fresh_workspace("completed");
    compile(&cwd, "fixtures/gaol.nomos", "gaol.world");
    let package_before = directory_bytes(&cwd.join("gaol.world"));
    let replay_before = fs::read(cwd.join("gaol.replay")).unwrap();

    let first = replay(&cwd, "gaol.world", "gaol.replay", "first.run");
    let second = replay(&cwd, "gaol.world", "gaol.replay", "second.run");
    let ordinary = run(
        &cwd,
        [
            "run",
            "gaol.world",
            "--commands",
            "gaol.commands",
            "--out",
            "ordinary.run",
        ],
    );
    assert_exit(&first, 0);
    assert_exit(&second, 0);
    assert_exit(&ordinary, 0);
    assert_eq!(
        directory_bytes(&cwd.join("first.run")),
        directory_bytes(&cwd.join("second.run"))
    );
    assert_eq!(
        directory_bytes(&cwd.join("first.run")),
        directory_bytes(&cwd.join("ordinary.run"))
    );
    assert_exact_run_files(&cwd.join("first.run"));

    let report = canonical_stdout(&first);
    assert_eq!(field(&report, "command"), &CanonicalValue::text("replay"));
    assert_eq!(field(&report, "status"), &CanonicalValue::text("completed"));
    assert_eq!(
        field(&report, "committed_command_count"),
        &CanonicalValue::Int(5)
    );
    assert_eq!(
        field(&report, "final_state_hash"),
        &CanonicalValue::text("3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc")
    );

    let world = open_compiled_world(&cwd.join("gaol.world")).unwrap();
    let opened = open_run_bundle(&cwd.join("first.run"), &world).unwrap();
    assert_eq!(opened.result().status(), RunStatus::Completed);
    assert_eq!(opened.command_log().rows().len(), 5);
    assert_eq!(opened.causal_receipts().receipts().len(), 5);
    assert_eq!(opened.state_hashes().rows().len(), 6);
    assert_eq!(opened.initial().state().tick(), 0);
    assert_eq!(opened.final_state().state().tick(), 5);
    assert_eq!(
        opened.command_log(),
        ReplayLog::from_canonical_bytes(REPLAY)
            .unwrap()
            .expected_command_log()
    );
    assert_eq!(directory_bytes(&cwd.join("gaol.world")), package_before);
    assert_eq!(fs::read(cwd.join("gaol.replay")).unwrap(), replay_before);
}

#[test]
fn replay_identity_evidence_and_output_failures_publish_nothing() {
    let cwd = fresh_workspace("failures");
    compile(&cwd, "fixtures/gaol.nomos", "gaol.world");
    let package_before = directory_bytes(&cwd.join("gaol.world"));
    let replay_before = fs::read(cwd.join("gaol.replay")).unwrap();

    let help = run(&cwd, ["replay", "--help"]);
    assert_exit(&help, 0);
    assert!(!help.stdout.starts_with(b"{"));
    assert_exit(
        &run(
            &cwd,
            ["replay", "gaol.world", "gaol.replay", "--out", "bad.run"],
        ),
        2,
    );
    let missing = replay(&cwd, "gaol.world", "missing.replay", "missing.run");
    assert_exit(&missing, 3);
    assert!(!cwd.join("missing.run").exists());

    let mut noncanonical = replay_before.clone();
    noncanonical.push(b'\n');
    fs::write(cwd.join("noncanonical.replay"), noncanonical).unwrap();
    let malformed = replay(&cwd, "gaol.world", "noncanonical.replay", "malformed.run");
    assert_exit(&malformed, 1);
    assert_eq!(diagnostic_code(&malformed), "EK0822");
    assert!(!cwd.join("malformed.run").exists());

    let existing = cwd.join("existing.run");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("sentinel"), b"unchanged").unwrap();
    let existing_before = directory_bytes(&existing);
    let collision = replay(&cwd, "gaol.world", "gaol.replay", "existing.run");
    assert_exit(&collision, 1);
    assert_eq!(directory_bytes(&existing), existing_before);
    let input_collision = replay(&cwd, "gaol.world", "gaol.replay", "gaol.replay");
    assert_exit(&input_collision, 1);
    assert_eq!(fs::read(cwd.join("gaol.replay")).unwrap(), replay_before);

    let nested = replay(&cwd, "gaol.world", "gaol.replay", "gaol.world/nested.run");
    assert_exit(&nested, 1);
    assert_eq!(diagnostic_code(&nested), "EK0821");
    assert!(!cwd.join("gaol.world/nested.run").exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        symlink("gaol.world", cwd.join("package-alias")).unwrap();
        let aliased = replay(
            &cwd,
            "gaol.world",
            "gaol.replay",
            "package-alias/aliased.run",
        );
        assert_exit(&aliased, 1);
        assert_eq!(diagnostic_code(&aliased), "EK0821");
        assert!(!cwd.join("gaol.world/aliased.run").exists());
    }

    let original = parse_canonical(REPLAY).unwrap();
    for (name, field_name) in [
        ("wrong-package.replay", "input_package_digest"),
        ("wrong-semantics.replay", "runtime_semantics_digest"),
    ] {
        let mut value = original.clone();
        object_mut(&mut value).insert(
            FieldName::declared(field_name),
            CanonicalValue::text(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
        );
        write_value(&cwd, name, &value);
        let output = format!("{name}.run");
        let rejected = replay(&cwd, "gaol.world", name, &output);
        assert_exit(&rejected, 1);
        assert_eq!(diagnostic_code(&rejected), "EK0823");
        assert!(!cwd.join(output).exists());
    }

    let mut wrong_initial = original.clone();
    object_mut(&mut wrong_initial).insert(
        FieldName::declared("initial_state_hash"),
        CanonicalValue::text("0000000000000000000000000000000000000000000000000000000000000000"),
    );
    let log = object_mut(&mut wrong_initial)
        .get_mut(&FieldName::declared("expected_command_log"))
        .unwrap();
    let CanonicalValue::Array(rows) = object_mut(log)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("expected replay rows")
    };
    object_mut(&mut rows[0]).insert(
        FieldName::declared("input_state_hash"),
        CanonicalValue::text("0000000000000000000000000000000000000000000000000000000000000000"),
    );
    write_value(&cwd, "wrong-initial.replay", &wrong_initial);
    let rejected = replay(
        &cwd,
        "gaol.world",
        "wrong-initial.replay",
        "wrong-initial.run",
    );
    assert_exit(&rejected, 1);
    assert_eq!(diagnostic_code(&rejected), "EK0823");
    assert!(!cwd.join("wrong-initial.run").exists());

    let mut wrong_final = original.clone();
    object_mut(&mut wrong_final).insert(
        FieldName::declared("expected_final_state_hash"),
        CanonicalValue::text("0000000000000000000000000000000000000000000000000000000000000000"),
    );
    let log = object_mut(&mut wrong_final)
        .get_mut(&FieldName::declared("expected_command_log"))
        .unwrap();
    let CanonicalValue::Array(rows) = object_mut(log)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("expected replay rows")
    };
    object_mut(rows.last_mut().unwrap()).insert(
        FieldName::declared("resulting_state_hash"),
        CanonicalValue::text("0000000000000000000000000000000000000000000000000000000000000000"),
    );
    write_value(&cwd, "wrong-final.replay", &wrong_final);
    ReplayLog::from_canonical_bytes(&fs::read(cwd.join("wrong-final.replay")).unwrap()).unwrap();
    let rejected = replay(&cwd, "gaol.world", "wrong-final.replay", "wrong-final.run");
    assert_exit(&rejected, 1);
    assert_eq!(diagnostic_code(&rejected), "EK0824");
    assert!(!cwd.join("wrong-final.run").exists());

    let mut forged = original.clone();
    let log = object_mut(&mut forged)
        .get_mut(&FieldName::declared("expected_command_log"))
        .unwrap();
    let CanonicalValue::Array(rows) = object_mut(log)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("expected replay rows")
    };
    let resolved = object_mut(&mut rows[0])
        .get_mut(&FieldName::declared("resolved_command"))
        .unwrap();
    object_mut(resolved).insert(
        FieldName::declared("namespace"),
        CanonicalValue::text("north_gate.ward"),
    );
    write_value(&cwd, "forged.replay", &forged);
    ReplayLog::from_canonical_bytes(&fs::read(cwd.join("forged.replay")).unwrap()).unwrap();
    let rejected = replay(&cwd, "gaol.world", "forged.replay", "forged.run");
    assert_exit(&rejected, 1);
    assert_eq!(diagnostic_code(&rejected), "EK0824");
    assert!(!cwd.join("forged.run").exists());

    let changed = String::from_utf8(SOURCE.to_vec()).unwrap() + "\n";
    fs::write(cwd.join("changed.nomos"), changed).unwrap();
    compile(&cwd, "changed.nomos", "changed.world");
    let wrong_package = replay(&cwd, "changed.world", "gaol.replay", "wrong-package.run");
    assert_exit(&wrong_package, 1);
    assert_eq!(diagnostic_code(&wrong_package), "EK0823");
    assert!(!cwd.join("wrong-package.run").exists());

    assert_eq!(directory_bytes(&cwd.join("gaol.world")), package_before);
    assert_eq!(fs::read(cwd.join("gaol.replay")).unwrap(), replay_before);
}
