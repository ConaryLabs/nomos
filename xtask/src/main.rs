//! Workspace tooling for signed-world.
//!
//! ```text
//! cargo xtask boundary [--manifest-path <path/to/Cargo.toml>]
//! ```
//!
//! `boundary` proves `KERNEL.md` section 10 (acceptance 15) against the
//! resolved dependency graph. `--manifest-path` points the check at another
//! workspace, which is how a planted-violation receipt is produced without
//! disturbing this one.
//!
//! Exit codes follow `KERNEL.md` section 8: `0` clean, `1` violations found,
//! `2` invalid usage, `3` the environment prevented the check.

mod boundary;
mod json;

use std::process::{Command, ExitCode};

use boundary::Graph;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut manifest_path: Option<String> = None;
    let mut task: Option<String> = None;

    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--manifest-path" => {
                index += 1;
                let Some(path) = arguments.get(index) else {
                    return usage("`--manifest-path` needs a path");
                };
                manifest_path = Some(path.clone());
            }
            "-h" | "--help" => {
                println!("{}", HELP);
                return ExitCode::from(0);
            }
            other if other.starts_with('-') => {
                return usage(format!("unknown option `{other}`"));
            }
            other => {
                if task.is_some() {
                    return usage(format!("unexpected extra argument `{other}`"));
                }
                task = Some(other.to_owned());
            }
        }
        index += 1;
    }

    match task.as_deref() {
        Some("boundary") => run_boundary(manifest_path.as_deref()),
        Some(other) => usage(format!("unknown task `{other}`")),
        None => usage("no task given"),
    }
}

const HELP: &str = "\
cargo xtask boundary [--manifest-path <path>]

Proves the KERNEL.md section 10 dependency boundaries against the resolved
cargo dependency graph: workspace membership, permitted edges, absence of
cycles, forbidden renderer/windowing/audio/networking/watcher/hot-reload
dependencies, and tooling isolation.

Exit codes: 0 clean, 1 violations, 2 invalid usage, 3 environment failure.";

fn usage(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("error: {message}\n\n{HELP}");
    ExitCode::from(2)
}

fn run_boundary(manifest_path: Option<&str>) -> ExitCode {
    let mut command = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()));
    command
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        // All features, so a dependency hidden behind a feature flag cannot
        // hide from the forbidden list.
        .arg("--all-features");
    if let Some(path) = manifest_path {
        command.arg("--manifest-path").arg(path);
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("error: could not run `cargo metadata`: {error}");
            return ExitCode::from(3);
        }
    };
    if !output.status.success() {
        eprintln!(
            "error: `cargo metadata` failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return ExitCode::from(3);
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let metadata = match json::parse(&text) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: could not read `cargo metadata` output: {error}");
            return ExitCode::from(3);
        }
    };
    let graph = match Graph::from_metadata(&metadata) {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: could not build the dependency graph: {error}");
            return ExitCode::from(3);
        }
    };

    let violations = graph.check();
    if violations.is_empty() {
        println!("boundary: clean");
        println!(
            "  kernel crates      {}",
            boundary::KERNEL_CRATES.join(", ")
        );
        println!(
            "  tooling crates     {}",
            boundary::TOOLING_CRATES.join(", ")
        );
        println!(
            "  rules checked      membership, permitted-edges, cycles, \
             forbidden-dependency, tooling-isolation"
        );
        println!(
            "  forbidden entries  {} exact names, {} prefixes",
            boundary::FORBIDDEN_NAMES.len(),
            boundary::FORBIDDEN_PREFIXES.len()
        );
        return ExitCode::from(0);
    }

    eprintln!("boundary: {} violation(s)", violations.len());
    for violation in &violations {
        eprintln!("  [{}] {}", violation.rule, violation.detail);
    }
    ExitCode::from(1)
}

#[cfg(test)]
mod tests {
    use crate::boundary::{FORBIDDEN_NAMES, KERNEL_CRATES, PERMITTED_EDGES, forbidden_category};

    #[test]
    fn permitted_edges_cover_every_kernel_crate_exactly_once() {
        assert_eq!(PERMITTED_EDGES.len(), KERNEL_CRATES.len());
        for (crate_name, _) in PERMITTED_EDGES {
            assert!(KERNEL_CRATES.contains(&crate_name));
        }
    }

    #[test]
    fn permitted_edges_only_name_kernel_crates_and_form_a_dag() {
        // The declared table itself must be acyclic, or the check would be
        // proving the graph against a contradiction.
        let rank = |name: &str| {
            KERNEL_CRATES
                .iter()
                .position(|candidate| *candidate == name)
                .expect("permitted edges name only kernel crates")
        };
        let order = [
            "estate-core",
            "estate-schema",
            "estate-projection",
            "estate-compiler",
            "estate-sim",
            "estate-cli",
        ];
        for (crate_name, dependencies) in PERMITTED_EDGES {
            for dependency in dependencies {
                let _ = rank(dependency);
                let source = order.iter().position(|name| *name == crate_name).unwrap();
                let target = order.iter().position(|name| *name == *dependency).unwrap();
                assert!(
                    target < source,
                    "`{crate_name}` -> `{dependency}` would make the declared table cyclic"
                );
            }
        }
    }

    #[test]
    fn the_forbidden_list_covers_every_category_section_10_names() {
        let categories: Vec<&str> = FORBIDDEN_NAMES.iter().map(|(_, kind)| *kind).collect();
        for required in [
            "renderer",
            "windowing",
            "audio",
            "networking",
            "watcher",
            "hot-reload",
        ] {
            assert!(
                categories.contains(&required),
                "section 10 forbids {required} dependencies; the list must cover them"
            );
        }
        assert_eq!(forbidden_category("wgpu"), Some("renderer"));
        assert_eq!(forbidden_category("tokio-tungstenite"), Some("networking"));
        assert_eq!(forbidden_category("bevy_ecs"), Some("engine"));
        assert_eq!(forbidden_category("estate-core"), None);
    }
}
