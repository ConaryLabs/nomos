//! Acceptance proof for the read-only entity-catalog projection (issue #138).
//!
//! The design record is `docs/review/entity-catalog.md`, which maps each
//! acceptance bullet to the test below that proves it.
//!
//! The point of the command is that no downstream tool has to infer an entity's
//! kind from a naming convention — `build-plan.mjs:25` classifies doors by
//! `machine.endsWith(".access")` only because no kernel command says what an
//! entity *is*. These tests hold the catalog to the two facts that removes:
//! every entity's `primitive` is its source declaration, and its `capabilities`
//! are World IR's `expansion.capabilities`. The source is read *here*, to know
//! what to expect; the command itself never opens it, which
//! `the_catalog_reads_no_source` proves by deleting it.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName};

static COUNTER: AtomicU32 = AtomicU32::new(0);

const GAOL: &str = include_str!("../../../fixtures/gaol.nomos");
const COMMANDS: &[u8] = include_bytes!("../../../fixtures/gaol.commands");

fn fresh_workspace(label: &str) -> PathBuf {
    let index = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("entity-catalog")
        .join(format!("{}-{nonce}-{label}-{index}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
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

fn text_set(value: &CanonicalValue) -> BTreeSet<&str> {
    array(value).iter().map(text).collect()
}

fn json(value: &CanonicalValue) -> String {
    String::from_utf8(value.to_canonical_bytes()).unwrap()
}

fn listing(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut entries: Vec<(PathBuf, Vec<u8>)> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    entries.sort();
    entries
}

fn diagnostic_code(output: &Output) -> String {
    let value = canonical_stdout(output);
    let diagnostic = &array(field(&value, "diagnostics"))[0];
    let CanonicalValue::Text(code) = field(diagnostic, "code") else {
        panic!("diagnostic code must be text")
    };
    code.clone()
}

/// Writes one source and compiles it into `world/` under a fresh workspace.
fn compiled(label: &str, source: &str) -> PathBuf {
    let root = fresh_workspace(label);
    fs::write(root.join("world.nomos"), source).unwrap();
    assert_exit(&run(&root, ["compile", "world.nomos", "--out", "world"]), 0);
    root
}

fn catalog(root: &Path) -> CanonicalValue {
    let output = run(root, ["entity-catalog", "world"]);
    assert_exit(&output, 0);
    canonical_stdout(&output)
}

/// The `entity <id> <primitive/kind>` declarations of a `.nomos` source.
///
/// This is the *test's* reading of the source, which is the whole point: the
/// expected value comes from the declaration a human wrote, not from anything
/// the command computed.
fn declared_primitives(source: &str) -> BTreeMap<&str, &str> {
    source
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            match (words.next(), words.next(), words.next(), words.next()) {
                (Some("entity"), Some(id), Some(primitive), None) => Some((id, primitive)),
                _ => None,
            }
        })
        .collect()
}

/// `entities[].expansion.capabilities` from the package's World IR member.
fn world_ir_capabilities(root: &Path) -> BTreeMap<String, BTreeSet<String>> {
    let ir = parse_canonical(&fs::read(root.join("world/world-ir.json")).unwrap()).unwrap();
    array(field(&ir, "entities"))
        .iter()
        .map(|entity| {
            let capabilities = text_set(field(field(entity, "expansion"), "capabilities"))
                .into_iter()
                .map(str::to_owned)
                .collect();
            (text(field(entity, "id")).to_owned(), capabilities)
        })
        .collect()
}

#[test]
fn the_catalog_names_its_schema_and_the_world_it_catalogued() {
    let root = compiled("identity", GAOL);
    let document = catalog(&root);

    assert_eq!(text(field(&document, "command")), "entity-catalog");
    assert_eq!(text(field(&document, "status")), "completed");
    assert_eq!(text(field(&document, "schema")), "nomos.entity_catalog@1");

    // The document binds the exact package it was built from, so a consumer
    // cannot silently pair a catalog with a different world.
    let world = field(&document, "world");
    assert_eq!(text(field(world, "world_ir_schema")), "nomos.world_ir@2");
    let inspect = run(&root, ["inspect", "world"]);
    assert_exit(&inspect, 0);
    assert_eq!(
        text(field(world, "manifest_digest")),
        text(field(&canonical_stdout(&inspect), "manifest_digest"))
    );
}

#[test]
fn every_entity_carries_its_declared_primitive_and_world_ir_capabilities() {
    for (path, source) in common::worlds() {
        let root = compiled("primitives", &source);
        let document = catalog(&root);
        let entities = array(field(&document, "entities"));

        let declared = declared_primitives(&source);
        assert!(
            !declared.is_empty(),
            "`{path}` declares no entity; the assertion below would be vacuous"
        );
        assert_eq!(
            entities.len(),
            declared.len(),
            "`{path}`: catalog and source disagree about how many entities exist"
        );

        let capabilities = world_ir_capabilities(&root);
        let mut previous: Option<&str> = None;
        for entity in entities {
            let id = text(field(entity, "id"));
            assert!(
                previous.is_none_or(|earlier| earlier < id),
                "`{path}`: catalog entities are not in strict entity order"
            );
            previous = Some(id);

            // The acceptance criterion: the kind is the one declared in source.
            assert_eq!(
                text(field(entity, "primitive")),
                declared[id],
                "`{path}`: entity `{id}` is not catalogued as its declared primitive"
            );
            // ... and the capability set is World IR's, verbatim as a set. The
            // catalog sorts arrays by wire spelling; World IR emits the same
            // members in `CapabilityKind` declaration order.
            assert_eq!(
                text_set(field(entity, "capabilities"))
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<BTreeSet<_>>(),
                capabilities[id],
                "`{path}`: entity `{id}` does not carry World IR's expansion.capabilities"
            );
            assert!(
                !capabilities[id].is_empty(),
                "`{path}`: entity `{id}` has no capabilities; the assertion above is vacuous"
            );
        }
    }
}

#[test]
fn resolver_subjects_claims_and_machines_come_from_the_plans() {
    let root = compiled("subjects", GAOL);
    let document = catalog(&root);
    let entities: BTreeMap<&str, &CanonicalValue> = array(field(&document, "entities"))
        .iter()
        .map(|entity| (text(field(entity, "id")), entity))
        .collect();
    assert_eq!(
        entities.keys().copied().collect::<Vec<_>>(),
        ["brazier_02", "flooded_section", "north_gate"]
    );

    // The door is a movement subject with two blocking claims, four machines,
    // and a face binding; it emits no light.
    let gate = entities["north_gate"];
    assert_eq!(json(field(gate, "movement_subject")), "true");
    assert_eq!(json(field(gate, "light_subject")), "false");
    assert_eq!(
        json(field(gate, "binding")),
        r#"{"cell":{"x":5,"y":0,"z":0},"direction":"north","kind":"face"}"#
    );
    assert_eq!(
        json(field(gate, "machines")),
        concat!(
            r#"[{"initial":"locked","namespace":"north_gate.access","#,
            r#""states":["closed","locked","open"]},"#,
            r#"{"initial":"cold","namespace":"north_gate.combustion","#,
            r#""states":["burning","cold","spent"]},"#,
            r#"{"initial":"intact","namespace":"north_gate.integrity","#,
            r#""states":["damaged","destroyed","intact"]},"#,
            r#"{"initial":"sealed","namespace":"north_gate.ward","#,
            r#""states":["sealed","unsealed"]}]"#,
        )
    );
    let claims = array(field(gate, "claims"));
    assert_eq!(claims.len(), 2);
    for claim in claims {
        assert_eq!(text(field(claim, "resolver")), "movement");
        assert_eq!(text(field(claim, "capability")), "blocks_ground");
        // The span is the entity declaration's, verbatim from the resolver
        // plan: catalog claims are compiler expansions of a sealed primitive
        // and have no source text of their own.
        assert_eq!(
            json(field(claim, "source")),
            concat!(
                r#"{"byte_end":162,"byte_start":53,"column":1,"line":4,"#,
                r#""path":"world.nomos"}"#,
            )
        );
    }
    assert_eq!(
        array(field(gate, "claims"))
            .iter()
            .map(|claim| text(field(claim, "id")))
            .collect::<Vec<_>>(),
        [
            "north_gate.portal#blocks_ground",
            "north_gate.ward#blocks_ground"
        ]
    );

    // The water region is a movement subject with a cost claim and no machine.
    let water = entities["flooded_section"];
    assert_eq!(json(field(water, "movement_subject")), "true");
    assert_eq!(json(field(water, "light_subject")), "false");
    assert_eq!(json(field(water, "machines")), "[]");
    assert_eq!(
        text(field(&array(field(water, "claims"))[0], "capability")),
        "traversal_cost_ground"
    );

    // The brazier is a light subject and not a movement subject.
    let brazier = entities["brazier_02"];
    assert_eq!(json(field(brazier, "movement_subject")), "false");
    assert_eq!(json(field(brazier, "light_subject")), "true");
    assert_eq!(
        json(field(brazier, "claims")),
        concat!(
            r#"[{"capability":"emits_light","id":"brazier_02.emission#emits_light","#,
            r#""resolver":"light","source":{"byte_end":323,"byte_start":251,"#,
            r#""column":1,"line":13,"path":"world.nomos"}}]"#,
        )
    );
}

#[test]
fn the_catalog_reads_no_source() {
    let root = compiled("no-source", GAOL);
    let before = catalog(&root);

    // The command is a projection over the compiled package. Removing the
    // source it was compiled from must change nothing at all.
    fs::remove_file(root.join("world.nomos")).unwrap();
    let after = catalog(&root);
    assert_eq!(json(&after), json(&before));
    assert!(json(&after).contains("nomos.entity_catalog@1"));
}

#[test]
fn the_catalog_mutates_no_input() {
    let root = compiled("immutable", GAOL);
    fs::write(root.join("gaol.commands"), COMMANDS).unwrap();
    assert_exit(
        &run(
            &root,
            [
                "run",
                "world",
                "--commands",
                "gaol.commands",
                "--out",
                "runs/gaol",
            ],
        ),
        0,
    );

    let package_before = listing(&root.join("world"));
    // A run bundle is a closed six-file set whose strict reopener fails closed
    // on an extra entry; the catalog must never add a seventh.
    let bundle_before = listing(&root.join("runs/gaol"));
    assert_eq!(bundle_before.len(), 6, "run bundle is not the six-file set");
    let mut workspace_before: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    workspace_before.sort();

    let _ = catalog(&root);

    assert_eq!(listing(&root.join("world")), package_before);
    assert_eq!(listing(&root.join("runs/gaol")), bundle_before);
    let mut workspace_after: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    workspace_after.sort();
    assert_eq!(
        workspace_after, workspace_before,
        "the catalog wrote an artifact"
    );
}

#[test]
fn the_argument_grammar_is_exact() {
    let root = compiled("grammar", GAOL);

    let help = run(&root, ["entity-catalog", "--help"]);
    assert_exit(&help, 0);
    assert!(String::from_utf8_lossy(&help.stdout).contains("nomos entity-catalog <world/>"));
    assert!(
        String::from_utf8_lossy(&run(&root, ["--help"]).stdout).contains("nomos entity-catalog")
    );

    // A missing argument, an option in the package position, and any extra
    // argument are usage errors rather than partial work.
    for args in [
        vec!["entity-catalog"],
        vec!["entity-catalog", "--world", "world"],
        vec!["entity-catalog", "world", "extra"],
        vec![
            "entity-catalog",
            "world",
            "--state",
            "runs/gaol/final-state.json",
        ],
    ] {
        assert_exit(&run(&root, args), 2);
    }

    // An absolute or escaping path is refused before any filesystem access.
    let escaping = run(&root, ["entity-catalog", "../world"]);
    assert_exit(&escaping, 1);
    assert_eq!(diagnostic_code(&escaping), "EK0002");

    // A world that is not there is a rejected world, exactly as `inspect` and
    // `explain-entity` report it.
    let absent = run(&root, ["entity-catalog", "absent.world"]);
    assert_exit(&absent, 1);
    assert_eq!(diagnostic_code(&absent), "EK0405");
}

#[test]
fn the_same_world_produces_byte_identical_output() {
    let root = compiled("determinism", GAOL);

    // Ten invocations against one package must agree on every byte of stdout,
    // not merely on the facts they encode.
    let mut outputs = Vec::new();
    for _ in 0..10 {
        let output = run(&root, ["entity-catalog", "world"]);
        assert_exit(&output, 0);
        outputs.push(output.stdout);
    }

    // Guard against a vacuous pass: ten empty stdouts are also "identical".
    let first = &outputs[0];
    assert!(
        first.len() > 1024,
        "stdout is implausibly short: {}",
        first.len()
    );
    assert_eq!(first.last(), Some(&b'\n'));
    assert!(
        String::from_utf8_lossy(first).contains(r#""schema":"nomos.entity_catalog@1""#),
        "stdout does not carry the catalog: {}",
        String::from_utf8_lossy(first)
    );

    for (index, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            String::from_utf8_lossy(output),
            String::from_utf8_lossy(first),
            "run {index} diverged from the first run"
        );
    }

    // A second compilation of the same source at the same path must produce the
    // same bytes, so nothing about the package's location on disk reaches
    // canonical output. The source path itself is inside the document, in claim
    // source spans, so "the same source" means the same bytes at the same path.
    assert_exit(
        &run(&root, ["compile", "world.nomos", "--out", "copy.world"]),
        0,
    );
    let copy = run(&root, ["entity-catalog", "copy.world"]);
    assert_exit(&copy, 0);
    assert_eq!(
        String::from_utf8_lossy(&copy.stdout),
        String::from_utf8_lossy(first),
        "a second compilation of the same source produced different bytes"
    );

    // The same bytes compiled from a different path are a different world, and
    // the catalog says so rather than quietly reporting the first world's spans.
    fs::create_dir_all(root.join("nested")).unwrap();
    fs::copy(root.join("world.nomos"), root.join("nested/world.nomos")).unwrap();
    assert_exit(
        &run(
            &root,
            ["compile", "nested/world.nomos", "--out", "nested.world"],
        ),
        0,
    );
    let moved = run(&root, ["entity-catalog", "nested.world"]);
    assert_exit(&moved, 0);
    assert_ne!(
        String::from_utf8_lossy(&moved.stdout),
        String::from_utf8_lossy(first),
        "the source path appears in claim spans and must reach the document"
    );
}
