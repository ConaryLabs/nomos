//! Planted workspace-boundary violations.
//!
//! `docs/workspace.md` records the receipt pattern: copy the workspace, plant
//! the violation in the copy, and point `--manifest-path` at the copy, so the
//! real workspace is never disturbed. These tests run that pattern in a
//! temporary directory and read the violations directly, which is the same
//! evidence the command prints, without a nested build.
//!
//! The planted cases are the ones `RUNTIME.md` section 3 turns into rules: an
//! undeclared member, a declared member depending on a kernel crate, a kernel
//! crate depending back, an R1 crate depending on an undeclared member, and a
//! cycle between two R1 crates. Only the copy ever contains the planted crates,
//! and their names are deliberately ones the workspace will never use, so that
//! a real R1 member joining `R1_CRATES` can never turn a planted violation into
//! an accepted one. Where a test needs its planted member *declared*, it passes
//! the shipped `R1_CRATES` plus the planted names, so the workspace's real
//! members stay declared too.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::boundary::{Graph, R1_CRATES, Violation};
use crate::load_graph;

/// A planted R1 member. The name is not, and will not become, a workspace
/// member: a planted violation must stay a violation when a real R1 crate is
/// declared.
const PLANTED_R1: &str = "nomos-planted-r1";

/// A second planted R1 member: a cycle needs two crates.
const PLANTED_PEER: &str = "nomos-planted-peer";

/// The shipped declared members plus the planted ones.
fn declared_with<'a>(planted: &[&'a str]) -> Vec<&'a str> {
    R1_CRATES
        .iter()
        .copied()
        .chain(planted.iter().copied())
        .collect()
}

/// A copy of this workspace in a temporary directory, removed on drop.
struct Planted {
    root: PathBuf,
}

impl Planted {
    /// Copies the manifests, the lock file, and the member trees — everything
    /// `cargo metadata` reads — into a fresh temporary directory.
    fn copy_of_the_workspace(label: &str) -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask is a member of the workspace it checks")
            .to_owned();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "nomos-boundary-{label}-{}-{nanos}",
            std::process::id()
        ));

        fs::create_dir_all(&root).expect("create the copy");
        for file in ["Cargo.toml", "Cargo.lock"] {
            fs::copy(source.join(file), root.join(file)).expect("copy a workspace file");
        }
        for tree in ["crates", "xtask"] {
            copy_tree(&source.join(tree), &root.join(tree));
        }
        Self { root }
    }

    fn manifest(&self) -> PathBuf {
        self.root.join("Cargo.toml")
    }

    /// Writes a new crate under `crates/` and adds it to the member list.
    fn plant_crate(&self, name: &str, dependencies: &[&str], dev_dependencies: &[&str]) {
        self.plant_crate_in("crates", name, dependencies, dev_dependencies);
    }

    /// Writes a new crate under `tree/` and adds it to the member list.
    fn plant_crate_in(
        &self,
        tree: &str,
        name: &str,
        dependencies: &[&str],
        dev_dependencies: &[&str],
    ) {
        let directory = self.root.join(tree).join(name);
        fs::create_dir_all(directory.join("src")).expect("create the planted crate");
        let section = |names: &[&str]| {
            names
                .iter()
                .map(|dependency| format!("{dependency} = {}\n", path_dependency(dependency)))
                .collect::<String>()
        };
        let manifest = format!(
            "[package]\n\
             name = \"{name}\"\n\
             version.workspace = true\n\
             edition.workspace = true\n\
             rust-version.workspace = true\n\
             license.workspace = true\n\
             repository.workspace = true\n\
             publish.workspace = true\n\
             \n\
             [dependencies]\n{}\n[dev-dependencies]\n{}",
            section(dependencies),
            section(dev_dependencies),
        );
        fs::write(directory.join("Cargo.toml"), manifest).expect("write the planted manifest");
        fs::write(
            directory.join("src/lib.rs"),
            "//! Planted by a boundary test. Never built.\n",
        )
        .expect("write the planted source");

        let manifest = self.manifest();
        let text = fs::read_to_string(&manifest).expect("read the copied workspace manifest");
        let planted = text.replacen(
            "    \"xtask\",\n",
            &format!("    \"xtask\",\n    \"{tree}/{name}\",\n"),
            1,
        );
        assert_ne!(
            planted, text,
            "the workspace manifest no longer lists `xtask` as a member"
        );
        fs::write(&manifest, planted).expect("write the copied workspace manifest");
    }

    /// Adds one dependency line to a member of the copy.
    fn plant_dependency(&self, member: &str, dependency: &str) {
        let manifest = self.root.join("crates").join(member).join("Cargo.toml");
        let text = fs::read_to_string(&manifest).expect("read a member manifest");
        let planted = text.replacen(
            "[dependencies]\n",
            &format!(
                "[dependencies]\n{dependency} = {}\n",
                path_dependency(dependency)
            ),
            1,
        );
        assert_ne!(
            planted, text,
            "`{member}` has no `[dependencies]` section to plant into"
        );
        fs::write(&manifest, planted).expect("write a member manifest");
    }

    fn graph(&self) -> Graph {
        load_graph(Some(&self.manifest().to_string_lossy())).expect("read the planted metadata")
    }
}

impl Drop for Planted {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A path dependency on another crate of the copy, from a crate in `crates/`.
fn path_dependency(name: &str) -> String {
    format!("{{ path = \"../{name}\" }}")
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create a directory in the copy");
    for entry in fs::read_dir(from).expect("read a directory of the workspace") {
        let entry = entry.expect("read a directory entry");
        // A build directory is not part of what `cargo metadata` reads.
        if entry.file_name() == "target" {
            continue;
        }
        let target = to.join(entry.file_name());
        if entry.file_type().expect("read a file type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy a file");
        }
    }
}

/// Asserts one violation of `rule` and returns its detail, printed as a receipt.
fn only_violation(violations: &[Violation], rule: &str) -> String {
    assert_eq!(
        violations.len(),
        1,
        "expected exactly one violation, found {violations:?}"
    );
    assert_eq!(violations[0].rule, rule, "{violations:?}");
    println!("PLANTED [{}] {}", violations[0].rule, violations[0].detail);
    violations[0].detail.clone()
}

#[test]
fn an_undeclared_r1_member_fails_membership() {
    let planted = Planted::copy_of_the_workspace("undeclared");
    planted.plant_crate(PLANTED_R1, &["nomos-sim"], &[]);

    // `check` uses the shipped `R1_CRATES`, which does not declare this name.
    let detail = only_violation(&planted.graph().check(), "membership");
    assert!(detail.contains(&format!("`{PLANTED_R1}`")), "{detail}");
}

#[test]
fn a_declared_r1_member_may_depend_on_a_kernel_crate() {
    let planted = Planted::copy_of_the_workspace("declared");
    planted.plant_crate(PLANTED_R1, &["nomos-sim"], &[]);

    let violations = planted.graph().check_with(&declared_with(&[PLANTED_R1]));
    println!(
        "PLANTED [declared] {PLANTED_R1} -> nomos-sim: {} violation(s)",
        violations.len()
    );
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn a_kernel_crate_depending_on_an_r1_member_fails_permitted_edges() {
    let planted = Planted::copy_of_the_workspace("kernel-edge");
    planted.plant_crate(PLANTED_R1, &[], &[]);
    planted.plant_dependency("nomos-sim", PLANTED_R1);

    let detail = only_violation(
        &planted.graph().check_with(&declared_with(&[PLANTED_R1])),
        "permitted-edges",
    );
    assert!(
        detail.contains(&format!("`nomos-sim` depends on `{PLANTED_R1}`")),
        "{detail}"
    );
}

#[test]
fn an_r1_member_depending_on_an_undeclared_member_fails_twice() {
    // The only workspace members an R1 crate can reach that are neither kernel
    // crates nor declared R1 crates are undeclared ones and, once R1-4 lands,
    // `apps/`: Cargo drops a dependency on `xtask`, which has no lib target.
    let planted = Planted::copy_of_the_workspace("undeclared-peer");
    planted.plant_crate(PLANTED_R1, &[PLANTED_PEER], &[]);
    planted.plant_crate(PLANTED_PEER, &[], &[]);

    let violations = planted.graph().check_with(&declared_with(&[PLANTED_R1]));
    for violation in &violations {
        println!("PLANTED [{}] {}", violation.rule, violation.detail);
    }
    let rules: Vec<&str> = violations.iter().map(|violation| violation.rule).collect();
    assert_eq!(rules, ["membership", "permitted-edges"], "{violations:?}");
    assert!(
        violations[0].detail.contains(PLANTED_PEER),
        "{violations:?}"
    );
    assert!(
        violations[1].detail.contains(&format!(
            "R1 crate `{PLANTED_R1}` depends on workspace member `{PLANTED_PEER}`"
        )),
        "{violations:?}"
    );
}

#[test]
fn a_cycle_between_two_r1_members_fails_cycles() {
    let planted = Planted::copy_of_the_workspace("r1-cycle");
    // Cargo refuses a cycle of normal dependencies outright, so the return edge
    // is a dev-dependency — the kind Cargo allows and this rule still refuses.
    planted.plant_crate(PLANTED_R1, &[PLANTED_PEER], &[]);
    planted.plant_crate(PLANTED_PEER, &[], &[PLANTED_R1]);

    let detail = only_violation(
        &planted
            .graph()
            .check_with(&declared_with(&[PLANTED_R1, PLANTED_PEER])),
        "cycles",
    );
    assert!(
        detail.contains(&format!("{PLANTED_R1} -> {PLANTED_PEER} -> {PLANTED_R1}")),
        "{detail}"
    );
}

#[test]
fn a_workspace_member_under_apps_fails_viewer_isolation() {
    // RUNTIME.md section 3: `apps/nomos-viewer/` consumes published artifacts
    // only, and viewer isolation "joins the checker with R1-4". The viewer is
    // JavaScript and so has no manifest; what this refuses is the change that
    // would make it reachable at all - a Cargo member placed under `apps/`.
    let planted = Planted::copy_of_the_workspace("apps-member");
    planted.plant_crate_in("apps", PLANTED_R1, &[], &[]);

    let violations = planted.graph().check_with(&declared_with(&[PLANTED_R1]));
    for violation in &violations {
        println!("PLANTED [{}] {}", violation.rule, violation.detail);
    }
    let detail = only_violation(&violations, "viewer-isolation");
    assert!(detail.contains(PLANTED_R1), "{detail}");
    assert!(detail.contains("apps/"), "{detail}");
}

#[test]
fn the_shipped_workspace_has_no_member_under_apps() {
    // The positive half: `apps/nomos-viewer/` exists in the tree and is not a
    // workspace member, so the rule above is satisfied by construction rather
    // than by there being no `apps/` at all.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is a member of the workspace it checks")
        .join("apps/nomos-viewer");
    assert!(root.is_dir(), "apps/nomos-viewer is missing");
    assert!(
        !root.join("Cargo.toml").exists(),
        "apps/nomos-viewer has grown a Cargo manifest"
    );

    let graph = crate::load_graph(None).expect("read the workspace metadata");
    let violations = graph.check();
    assert!(
        violations.iter().all(|one| one.rule != "viewer-isolation"),
        "{violations:?}"
    );
}
