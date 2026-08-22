//! SW-H end-to-end proof for the filesystem authoring CLI.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nomos_core::CanonicalValue;
use nomos_core::canonical::read::parse_canonical;

static COUNTER: AtomicU32 = AtomicU32::new(0);
const SOURCE: &[u8] = include_bytes!("../../../fixtures/gaol.nomos");

fn fresh_workspace(label: &str) -> PathBuf {
    let index = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("sw-h-cli")
        .join(format!("{}-{nonce}-{label}-{index}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("gaol.nomos"), SOURCE).unwrap();
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
    assert_eq!(output.status.code(), Some(expected));
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

fn object(
    value: &CanonicalValue,
) -> &std::collections::BTreeMap<nomos_core::FieldName, CanonicalValue> {
    let CanonicalValue::Object(fields) = value else {
        panic!("expected canonical object")
    };
    fields
}

fn field<'a>(value: &'a CanonicalValue, name: &'static str) -> &'a CanonicalValue {
    object(value)
        .get(&nomos_core::FieldName::declared(name))
        .unwrap()
}

fn package_bytes(root: &Path) -> Vec<(String, Vec<u8>)> {
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
fn exact_help_and_usage_are_stable() {
    let cwd = fresh_workspace("grammar");
    for args in [
        vec!["--help"],
        vec!["validate", "--help"],
        vec!["compile", "--help"],
        vec!["inspect", "--help"],
    ] {
        let first = run(&cwd, &args);
        let second = run(&cwd, &args);
        assert_exit(&first, 0);
        assert_eq!(first.stdout, second.stdout);
        assert!(!first.stdout.starts_with(b"{"));
    }

    for args in [
        vec![],
        vec!["run"],
        vec!["--version"],
        vec!["validate"],
        vec!["validate", "--help", "gaol.nomos"],
        vec!["compile", "gaol.nomos", "--out=one.world"],
        vec!["compile", "--out", "one.world", "gaol.nomos"],
        vec!["inspect", "--", "one.world"],
    ] {
        let output = run(&cwd, &args);
        assert_exit(&output, 2);
        let value = canonical_stdout(&output);
        assert_eq!(field(&value, "status"), &CanonicalValue::text("rejected"));
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_argv_is_invalid_usage_without_lossy_echo() {
    use std::os::unix::ffi::OsStringExt;

    let cwd = fresh_workspace("non-utf8-argv");
    let output = run(
        &cwd,
        [OsString::from("validate"), OsString::from_vec(vec![0xff])],
    );
    assert_exit(&output, 2);
    let value = canonical_stdout(&output);
    assert_eq!(
        field(&value, "diagnostics"),
        &CanonicalValue::Array(vec![CanonicalValue::object_declared([
            ("code", CanonicalValue::text("EK0001")),
            (
                "message",
                CanonicalValue::text("argument 2 is not UTF-8 and cannot name a Nomos operation",),
            ),
            ("repairs", CanonicalValue::Array(Vec::new())),
        ])])
    );
}

#[test]
fn validate_compile_and_inspect_are_immutable_and_deterministic() {
    let cwd = fresh_workspace("roundtrip");
    let source_before = fs::read(cwd.join("gaol.nomos")).unwrap();
    let entries_before = fs::read_dir(&cwd).unwrap().count();

    let first_validate = run(&cwd, ["validate", "gaol.nomos"]);
    let second_validate = run(&cwd, ["validate", "gaol.nomos"]);
    assert_exit(&first_validate, 0);
    assert_eq!(first_validate.stdout, second_validate.stdout);
    let validated = canonical_stdout(&first_validate);
    assert_eq!(
        field(&validated, "command"),
        &CanonicalValue::text("validate")
    );
    assert_eq!(fs::read_dir(&cwd).unwrap().count(), entries_before);

    let first_compile = run(
        &cwd,
        ["compile", "gaol.nomos", "--out", "build/first.world"],
    );
    let second_compile = run(
        &cwd,
        ["compile", "gaol.nomos", "--out", "build/second.world"],
    );
    assert_exit(&first_compile, 0);
    assert_exit(&second_compile, 0);
    let first_result = canonical_stdout(&first_compile);
    let second_result = canonical_stdout(&second_compile);
    assert_eq!(
        field(&first_result, "manifest_digest"),
        field(&second_result, "manifest_digest")
    );
    assert_eq!(
        package_bytes(&cwd.join("build/first.world")),
        package_bytes(&cwd.join("build/second.world"))
    );

    let first_inspect = run(&cwd, ["inspect", "build/first.world"]);
    let second_inspect = run(&cwd, ["inspect", "build/first.world"]);
    assert_exit(&first_inspect, 0);
    assert_eq!(first_inspect.stdout, second_inspect.stdout);
    let inspection = canonical_stdout(&first_inspect);
    let CanonicalValue::Array(entities) = field(&inspection, "entities") else {
        panic!("inspection entities must be an array")
    };
    assert_eq!(entities.len(), 3);
    assert_eq!(
        field(&entities[0], "id"),
        &CanonicalValue::text("brazier_02")
    );
    assert_eq!(
        field(&entities[1], "id"),
        &CanonicalValue::text("flooded_section")
    );
    assert_eq!(
        field(&entities[2], "id"),
        &CanonicalValue::text("north_gate")
    );
    for entity in entities {
        for name in ["primitive", "source", "capabilities", "machines", "claims"] {
            let _ = field(entity, name);
        }
    }

    assert_eq!(fs::read(cwd.join("gaol.nomos")).unwrap(), source_before);
    assert!(fs::read_dir(cwd.join("build")).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".staging-")
    }));
}

#[test]
fn semantic_usage_and_environment_failures_use_distinct_exits() {
    let cwd = fresh_workspace("failures");

    let missing_source = run(&cwd, ["validate", "missing.nomos"]);
    assert_exit(&missing_source, 3);
    assert_eq!(
        field(&canonical_stdout(&missing_source), "status"),
        &CanonicalValue::text("rejected")
    );

    fs::create_dir(cwd.join("source-directory")).unwrap();
    let source_directory = run(&cwd, ["validate", "source-directory"]);
    assert_exit(&source_directory, 3);

    fs::write(cwd.join("non-utf8.nomos"), [0xff]).unwrap();
    let non_utf8_source = run(&cwd, ["validate", "non-utf8.nomos"]);
    assert_exit(&non_utf8_source, 1);

    let escaped_source = run(&cwd, ["validate", "../gaol.nomos"]);
    assert_exit(&escaped_source, 1);
    let escaped_output = run(&cwd, ["compile", "gaol.nomos", "--out", "../escaped.world"]);
    assert_exit(&escaped_output, 1);

    fs::write(cwd.join("invalid.nomos"), b"not nomos source\n").unwrap();
    let invalid_compile = run(
        &cwd,
        ["compile", "invalid.nomos", "--out", "build/rejected.world"],
    );
    assert_exit(&invalid_compile, 1);
    assert!(!cwd.join("build/rejected.world").exists());
    assert!(
        !cwd.join("build").exists()
            || fs::read_dir(cwd.join("build")).unwrap().all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".staging-"))
    );

    let missing_package = run(&cwd, ["inspect", "missing.world"]);
    assert_exit(&missing_package, 1);
}

#[test]
fn tampering_extra_entries_and_existing_outputs_fail_closed() {
    let cwd = fresh_workspace("immutability");
    let compile = run(
        &cwd,
        ["compile", "gaol.nomos", "--out", "build/evidence.world"],
    );
    assert_exit(&compile, 0);

    let existing = cwd.join("build/existing.world");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("sentinel"), b"do not replace").unwrap();
    let before = package_bytes(&existing);
    let rejected = run(
        &cwd,
        ["compile", "gaol.nomos", "--out", "build/existing.world"],
    );
    assert_exit(&rejected, 1);
    assert_eq!(package_bytes(&existing), before);

    let world = cwd.join("build/evidence.world");
    fs::write(world.join("extra.json"), b"{}").unwrap();
    let extra = run(&cwd, ["inspect", "build/evidence.world"]);
    assert_exit(&extra, 1);
    fs::remove_file(world.join("extra.json")).unwrap();

    let mut bytes = fs::read(world.join("world-ir.json")).unwrap();
    bytes.push(b'\n');
    fs::write(world.join("world-ir.json"), bytes).unwrap();
    let tampered = run(&cwd, ["inspect", "build/evidence.world"]);
    assert_exit(&tampered, 1);
}

#[cfg(unix)]
#[test]
fn source_symlinks_are_followed_but_package_root_symlinks_are_refused() {
    use std::os::unix::fs::symlink;

    let cwd = fresh_workspace("symlinks");
    symlink("gaol.nomos", cwd.join("source-link.nomos")).unwrap();
    let validate = run(&cwd, ["validate", "source-link.nomos"]);
    assert_exit(&validate, 0);
    assert_eq!(
        field(&canonical_stdout(&validate), "source"),
        &CanonicalValue::text("source-link.nomos")
    );

    let compile = run(&cwd, ["compile", "gaol.nomos", "--out", "evidence.world"]);
    assert_exit(&compile, 0);
    symlink("evidence.world", cwd.join("alias.world")).unwrap();
    let inspect = run(&cwd, ["inspect", "alias.world"]);
    assert_exit(&inspect, 1);
}
