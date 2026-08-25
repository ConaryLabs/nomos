//! The input boundary: what the compiler is allowed to read, and nothing else.
//!
//! `RUNTIME.md` section 5 R1-2's first acceptance bullet: a temp directory
//! containing only the allowed inputs compiles successfully; the crate has no
//! dependency on `nomos-schema` or `nomos-compiler`; no code path names
//! `world-ir.json`, `compiler-receipts.json`, or `.nomos`.

mod common;

use std::path::Path;
use std::process::Command;

use common::{Fixture, Options};

#[test]
fn a_directory_of_only_the_declared_inputs_compiles() {
    let fixture = Fixture::new("declared-inputs");
    let compiled = nomos_render_plan::compile(fixture.inputs()).expect("declared inputs compile");
    assert_eq!(compiled.entity_count, 3);
    assert_eq!(compiled.scenario_count, 2);
    assert_eq!(
        compiled.interaction_count, 1,
        "the second scenario adds exactly one command to the first"
    );
    let text = String::from_utf8(compiled.bytes.clone()).unwrap();
    assert!(text.contains("\"schema\":\"nomos.rendering_plan@1\""));
    assert!(text.ends_with('\n'), "the plan file is newline-terminated");
}

#[test]
fn world_ir_compiler_receipts_and_source_are_never_opened() {
    // The world directory carries unreadable World IR, compiler receipts, a
    // manifest, a schemas member, and a `.nomos` source. Any code path that
    // opened one of them would fail every reader in this crate.
    let fixture = Fixture::with(
        "poisoned-world",
        Options {
            poison_world: true,
            ..Options::default()
        },
    );
    let poisoned =
        nomos_render_plan::compile(fixture.inputs()).expect("only the four projections are read");
    let clean = Fixture::new("clean-world");
    let clean = nomos_render_plan::compile(clean.inputs()).expect("clean world compiles");
    // The digests differ only because the two temp worlds are distinct files;
    // everything the poison could have touched is identical.
    assert_eq!(poisoned.entity_count, clean.entity_count);
    assert_eq!(poisoned.scenario_count, clean.scenario_count);
}

#[test]
fn no_code_path_names_a_forbidden_input() {
    // Comments are stripped first: the modules deliberately *document* which
    // package members they refuse to open, and the acceptance criterion is
    // about code paths, not prose.
    let source = executable_source();
    assert!(!source.is_empty(), "the crate has source files");
    for forbidden in ["world-ir.json", "compiler-receipts.json", ".nomos"] {
        for (path, text) in &source {
            assert!(
                !text.contains(forbidden),
                "{path} names the forbidden input {forbidden}"
            );
        }
    }
}

#[test]
fn no_code_path_holds_a_floating_point_type() {
    for (path, text) in executable_source() {
        for float in ["f32", "f64"] {
            assert!(
                !text.contains(float),
                "{path} names {float}; the compiler carries presentation numbers as exact decimals"
            );
        }
    }
}

#[test]
fn the_build_dependency_graph_is_nomos_core_only() {
    // Declarations only; the manifest's comments name the crates it refuses.
    let manifest = std::fs::read_to_string(crate_root().join("Cargo.toml")).unwrap();
    let mut section = String::new();
    let mut build = Vec::new();
    let mut dev = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            section = line.to_owned();
            continue;
        }
        match section.as_str() {
            "[dependencies]" => build.push(line.to_owned()),
            "[dev-dependencies]" => dev.push(line.to_owned()),
            _ => {}
        }
    }
    assert_eq!(build, vec!["nomos-core.workspace = true".to_owned()]);
    assert_eq!(
        dev,
        vec![
            "nomos-projection.workspace = true".to_owned(),
            "nomos-sim.workspace = true".to_owned(),
        ]
    );
    for line in build.iter().chain(&dev) {
        for forbidden in ["nomos-schema", "nomos-compiler", "nomos-cli"] {
            assert!(!line.contains(forbidden), "{line} reaches {forbidden}");
        }
        // RUNTIME.md section 4: zero third-party dependencies in this crate.
        assert!(
            line.starts_with("nomos-") && line.contains("workspace = true"),
            "third-party or unpinned dependency declared: {line}"
        );
    }
}

#[test]
fn compiling_twice_is_byte_identical() {
    let fixture = Fixture::new("determinism");
    let first = nomos_render_plan::compile(fixture.inputs()).unwrap();
    let second = nomos_render_plan::compile(fixture.inputs()).unwrap();
    assert_eq!(first.bytes, second.bytes);
    assert!(
        common::normalized_differences(&first.bytes, &second.bytes).is_empty(),
        "byte identity implies normalized identity"
    );
}

#[test]
fn the_binary_writes_the_plan_and_a_canonical_status() {
    let fixture = Fixture::new("binary");
    let output = Command::new(env!("CARGO_BIN_EXE_nomos-render-plan"))
        .args([
            "--catalog".as_ref(),
            fixture.catalog().as_os_str(),
            "--facts".as_ref(),
            fixture.facts().as_os_str(),
            "--runs".as_ref(),
            fixture.runs().as_os_str(),
            "--world".as_ref(),
            fixture.world().as_os_str(),
            "--area".as_ref(),
            fixture.area().as_os_str(),
            "--out".as_ref(),
            fixture.out().as_os_str(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"command\":\"render-plan\""), "{stdout}");
    assert!(stdout.contains("\"status\":\"completed\""), "{stdout}");
    let written = std::fs::read(fixture.out()).unwrap();
    let compiled = nomos_render_plan::compile(fixture.inputs()).unwrap();
    assert_eq!(written, compiled.bytes);
}

#[test]
fn a_missing_argument_is_refused_with_a_stable_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_nomos-render-plan"))
        .arg("--catalog")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"code\":\"RP0106\""), "{stdout}");
    assert!(stdout.contains("\"status\":\"rejected\""), "{stdout}");
}

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under `src`, with line comments removed.
fn executable_source() -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                out.push((path.display().to_string(), strip_comments(&text)));
            }
        }
    }
    let mut out = Vec::new();
    walk(&crate_root().join("src"), &mut out);
    out.sort();
    out
}

/// Drops `//`-to-end-of-line comments, tracking string literals so a `//`
/// inside a string is kept. The crate uses no block comments; a stray one
/// would only make this check stricter.
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let bytes = line.as_bytes();
        let mut in_string = false;
        let mut escaped = false;
        let mut cut = line.len();
        let mut index = 0;
        while index < bytes.len() {
            let byte = bytes[index];
            if in_string {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    in_string = false;
                }
            } else if byte == b'"' {
                in_string = true;
            } else if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
                cut = index;
                break;
            }
            index += 1;
        }
        out.push_str(&line[..cut]);
        out.push('\n');
    }
    out
}
