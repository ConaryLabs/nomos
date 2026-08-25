//! Spike proof for issue #126: the composed effective-fact projection.
//!
//! The point of the command is that no consumer needs a second resolver. These
//! tests hold it to that: the projection must agree with `explain-entity`'s
//! independently composed `effective_initial_facts` for every subject, and must
//! track committed state rather than the package's initial state.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName};

static COUNTER: AtomicU32 = AtomicU32::new(0);
const SOURCE: &[u8] = include_bytes!("../../../fixtures/gaol.nomos");
const COMMANDS: &[u8] = include_bytes!("../../../fixtures/gaol.commands");

fn fresh_workspace(label: &str) -> PathBuf {
    let index = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("effective-facts")
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

fn object(value: &CanonicalValue) -> &BTreeMap<FieldName, CanonicalValue> {
    let CanonicalValue::Object(fields) = value else {
        panic!("expected canonical object")
    };
    fields
}

fn field<'a>(value: &'a CanonicalValue, name: &'static str) -> &'a CanonicalValue {
    object(value)
        .get(&FieldName::declared(name))
        .unwrap_or_else(|| panic!("missing field `{name}`"))
}

fn array(value: &CanonicalValue) -> &[CanonicalValue] {
    let CanonicalValue::Array(items) = value else {
        panic!("expected canonical array")
    };
    items
}

fn text(value: &CanonicalValue) -> &str {
    let CanonicalValue::Text(inner) = value else {
        panic!("expected canonical text")
    };
    inner.as_str()
}

fn json(value: &CanonicalValue) -> String {
    String::from_utf8(value.to_canonical_bytes()).unwrap()
}

/// Compiles the fixture world and executes the five-command script.
fn compiled_run(label: &str) -> PathBuf {
    let root = fresh_workspace(label);
    assert_exit(
        &run(&root, ["compile", "gaol.nomos", "--out", "gaol.world"]),
        0,
    );
    assert_exit(
        &run(
            &root,
            [
                "run",
                "gaol.world",
                "--commands",
                "gaol.commands",
                "--out",
                "runs/gaol",
            ],
        ),
        0,
    );
    root
}

fn effective_facts(root: &Path, state: &str) -> CanonicalValue {
    let output = run(root, ["effective-facts", "gaol.world", "--state", state]);
    assert_exit(&output, 0);
    canonical_stdout(&output)
}

#[test]
fn the_projection_names_its_schema_world_and_state() {
    let root = compiled_run("identity");
    let document = effective_facts(&root, "runs/gaol/final-state.json");

    assert_eq!(text(field(&document, "command")), "effective-facts");
    assert_eq!(text(field(&document, "status")), "completed");
    assert_eq!(
        json(field(&document, "schema")),
        r#"{"name":"nomos.effective_facts","version":1}"#
    );
    assert_eq!(json(field(&document, "tick")), "5");

    // The document binds the exact package and runtime semantics it resolved
    // against, so facts cannot be silently paired with a different world.
    let state: CanonicalValue =
        parse_canonical(&fs::read(root.join("runs/gaol/final-state.json")).unwrap()).unwrap();
    assert_eq!(
        text(field(&document, "state_hash")),
        text(field(&state, "state_hash"))
    );
    assert_eq!(
        text(field(&document, "runtime_semantics_digest")),
        text(field(&state, "runtime_semantics_digest"))
    );
    let inspect = run(&root, ["inspect", "gaol.world"]);
    assert_exit(&inspect, 0);
    assert_eq!(
        text(field(&document, "package_digest")),
        text(field(&canonical_stdout(&inspect), "manifest_digest"))
    );
}

#[test]
fn every_resolver_subject_is_composed_at_the_supplied_state() {
    let root = compiled_run("subjects");

    // Initial state: the gate is blocked by both its portal and its ward, the
    // flooded section costs 3, and the brazier is lit.
    assert_eq!(
        json(field(
            &effective_facts(&root, "runs/gaol/initial-state.json"),
            "effective_facts"
        )),
        concat!(
            r#"{"ground_movement":["#,
            r#"{"disposition":{"cost":3,"kind":"traversable","reasons":["#,
            r#""flooded_section.region#traversal_cost_ground"]},"#,
            r#""entity":"flooded_section"},"#,
            r#"{"disposition":{"kind":"blocked","reasons":["#,
            r#""north_gate.portal#blocks_ground","north_gate.ward#blocks_ground"]},"#,
            r#""entity":"north_gate"}],"#,
            r#""light_emission":[{"emitting":true,"entity":"brazier_02","reasons":["#,
            r#""brazier_02.emission#emits_light"]}]}"#,
        )
    );

    // After unlock/open/unseal/ignite/extinguish: the gate falls back to the
    // base cost with an empty reason list, and the brazier is dark with none.
    assert_eq!(
        json(field(
            &effective_facts(&root, "runs/gaol/final-state.json"),
            "effective_facts"
        )),
        concat!(
            r#"{"ground_movement":["#,
            r#"{"disposition":{"cost":3,"kind":"traversable","reasons":["#,
            r#""flooded_section.region#traversal_cost_ground"]},"#,
            r#""entity":"flooded_section"},"#,
            r#"{"disposition":{"cost":1,"kind":"traversable","reasons":[]},"#,
            r#""entity":"north_gate"}],"#,
            r#""light_emission":[{"emitting":false,"entity":"brazier_02","reasons":[]}]}"#,
        )
    );
}

#[test]
fn the_projection_agrees_with_explain_entity_at_the_initial_state() {
    let root = compiled_run("agreement");
    let document = effective_facts(&root, "runs/gaol/initial-state.json");
    let facts = field(&document, "effective_facts");

    let movement: BTreeMap<&str, &CanonicalValue> = array(field(facts, "ground_movement"))
        .iter()
        .map(|fact| (text(field(fact, "entity")), field(fact, "disposition")))
        .collect();
    let light: BTreeMap<&str, &CanonicalValue> = array(field(facts, "light_emission"))
        .iter()
        .map(|fact| (text(field(fact, "entity")), fact))
        .collect();
    assert_eq!(movement.len(), 2);
    assert_eq!(light.len(), 1);

    // `explain-entity` composes the same facts through an independent code
    // path. If a shadow resolver ever appears, these two disagree.
    for entity in ["north_gate", "flooded_section", "brazier_02"] {
        let explained = run(&root, ["explain-entity", "gaol.world", entity]);
        assert_exit(&explained, 0);
        let report = canonical_stdout(&explained);
        let initial = field(&report, "effective_initial_facts");

        let explained_movement = field(initial, "ground_movement");
        match movement.get(entity) {
            Some(disposition) => assert_eq!(json(explained_movement), json(disposition)),
            None => assert_eq!(explained_movement, &CanonicalValue::Null),
        }

        let explained_light = field(initial, "light_emission");
        match light.get(entity) {
            Some(fact) => {
                assert_eq!(
                    json(field(explained_light, "emitting")),
                    json(field(fact, "emitting"))
                );
                assert_eq!(
                    json(field(explained_light, "reasons")),
                    json(field(fact, "reasons"))
                );
            }
            None => assert_eq!(explained_light, &CanonicalValue::Null),
        }
    }
}

#[test]
fn a_state_from_another_world_is_rejected_rather_than_resolved() {
    let root = compiled_run("foreign");
    // The fixture world plus one more gate: it compiles, but its simulation
    // semantics differ, so the first world's state must not resolve against it.
    let mut other = SOURCE.to_vec();
    other.extend_from_slice(
        b"\nentity south_gate primitive/iron_barred_door\n  anchor face 1 0 0 north\n  credential credential/gaoler_key\nend\n",
    );
    fs::write(root.join("other.nomos"), &other).unwrap();
    assert_exit(
        &run(&root, ["compile", "other.nomos", "--out", "other.world"]),
        0,
    );

    let output = run(
        &root,
        [
            "effective-facts",
            "other.world",
            "--state",
            "runs/gaol/final-state.json",
        ],
    );
    assert_exit(&output, 1);
    assert_eq!(
        text(field(&canonical_stdout(&output), "status")),
        "rejected"
    );
}

#[test]
fn the_projection_mutates_no_input() {
    let root = compiled_run("immutable");
    let state = root.join("runs/gaol/final-state.json");

    let before_state = fs::read(&state).unwrap();
    let mut before_package: Vec<(PathBuf, Vec<u8>)> = fs::read_dir(root.join("gaol.world"))
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    before_package.sort();

    let _ = effective_facts(&root, "runs/gaol/final-state.json");

    assert_eq!(fs::read(&state).unwrap(), before_state);
    let mut after_package: Vec<(PathBuf, Vec<u8>)> = fs::read_dir(root.join("gaol.world"))
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    after_package.sort();
    assert_eq!(after_package, before_package);
}

#[test]
fn the_argument_grammar_is_exact() {
    let root = compiled_run("grammar");

    let help = run(&root, ["effective-facts", "--help"]);
    assert_exit(&help, 0);
    assert!(
        String::from_utf8_lossy(&help.stdout).contains("nomos effective-facts <world/> --state")
    );
    assert!(
        String::from_utf8_lossy(&run(&root, ["--help"]).stdout).contains("nomos effective-facts")
    );

    // Missing option, misspelled option, and extra arguments are usage errors.
    for args in [
        vec!["effective-facts", "gaol.world"],
        vec!["effective-facts", "gaol.world", "--state"],
        vec![
            "effective-facts",
            "gaol.world",
            "--from",
            "runs/gaol/final-state.json",
        ],
        vec![
            "effective-facts",
            "gaol.world",
            "--state",
            "runs/gaol/final-state.json",
            "--out",
            "runs/extra",
        ],
    ] {
        assert_exit(&run(&root, args), 2);
    }

    // A missing state file is an environment failure, not a rejected world.
    assert_exit(
        &run(
            &root,
            ["effective-facts", "gaol.world", "--state", "absent.json"],
        ),
        3,
    );
}

#[test]
fn the_same_world_and_state_produce_byte_identical_output() {
    let root = compiled_run("determinism");

    // R1-1: ten invocations against one package and one state must agree on
    // every byte of stdout, not merely on the facts they encode.
    let mut outputs = Vec::new();
    for _ in 0..10 {
        let output = run(
            &root,
            [
                "effective-facts",
                "gaol.world",
                "--state",
                "runs/gaol/final-state.json",
            ],
        );
        assert_exit(&output, 0);
        outputs.push(output.stdout);
    }

    // Guard against a vacuous pass: ten empty stdouts are also "identical".
    let first = &outputs[0];
    assert!(
        first.len() > 512,
        "stdout is implausibly short: {}",
        first.len()
    );
    assert_eq!(first.last(), Some(&b'\n'));
    assert!(
        String::from_utf8_lossy(first).contains(r#""schema":{"name":"nomos.effective_facts""#),
        "stdout does not carry the projection: {}",
        String::from_utf8_lossy(first)
    );

    for (index, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            String::from_utf8_lossy(output),
            String::from_utf8_lossy(first),
            "run {index} diverged from the first run"
        );
    }

    // A second freshly compiled copy of the same source must produce the same
    // bytes, so nothing about the package's location on disk reaches canonical
    // output. "Same source" means the same bytes at the same source path: the
    // path appears in claim source spans and is therefore inside the hash
    // domain, which is what makes a state fail closed against a world compiled
    // from elsewhere (see the foreign-world test above).
    assert_exit(
        &run(&root, ["compile", "gaol.nomos", "--out", "copy.world"]),
        0,
    );
    let copy = run(
        &root,
        [
            "effective-facts",
            "copy.world",
            "--state",
            "runs/gaol/final-state.json",
        ],
    );
    assert_exit(&copy, 0);
    assert_eq!(
        String::from_utf8_lossy(&copy.stdout),
        String::from_utf8_lossy(first),
        "a second compilation of the same source produced different bytes"
    );
}
