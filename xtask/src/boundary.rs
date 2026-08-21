//! The `KERNEL.md` section 10 dependency-boundary check.
//!
//! Acceptance 15 requires automated checks that prove the dependency graph and
//! the forbidden-dependency rules, rather than a reviewer reading six
//! `Cargo.toml` files and hoping. This module reads `cargo metadata` and
//! refuses:
//!
//! 1. a workspace member that is not a declared kernel crate or declared
//!    tooling;
//! 2. an edge between kernel crates that section 10 does not permit — including
//!    dev-dependency edges, because a dev-dependency also lets a crate name the
//!    other's types;
//! 3. a cycle among kernel crates;
//! 4. a forbidden dependency — renderer, windowing, audio, networking, watcher,
//!    or hot-reload — anywhere reachable from a kernel crate, transitively;
//! 5. tooling reaching into the kernel graph it checks.
//!
//! Rule 5 is why the checker is a separate workspace member rather than a
//! subcommand of `estate-cli`. If it lived inside the kernel, its own JSON and
//! process-spawning dependencies would sit inside the graph it polices, and the
//! forbidden list would need exceptions for the checker. Section 10 lists the
//! crates *Gate K uses*; `xtask` builds no kernel artifact, is not reachable
//! from any kernel crate, and is asserted to stay that way by rule 5.
//!
//! # What this check cannot see
//!
//! Section 10 also forbids "canonical schema types defined in more than one
//! crate". That is not visible in `cargo metadata` — it is a property of the
//! source, not the graph — and it is enforced structurally instead: the
//! Canonical World IR type is defined in `estate-schema`, and every crate that
//! must not see it is missing the edge that would let it. This check proves the
//! missing edges; SW-C owns the type itself.

use std::collections::{BTreeMap, BTreeSet};

use crate::json::Value;

/// The six kernel crates named by section 10.
pub const KERNEL_CRATES: [&str; 6] = [
    "estate-core",
    "estate-schema",
    "estate-projection",
    "estate-compiler",
    "estate-sim",
    "estate-cli",
];

/// Workspace members that are tooling rather than kernel crates.
pub const TOOLING_CRATES: [&str; 1] = ["xtask"];

/// The permitted edges, verbatim from section 10.
pub const PERMITTED_EDGES: [(&str, &[&str]); 6] = [
    ("estate-core", &[]),
    ("estate-schema", &["estate-core"]),
    ("estate-projection", &["estate-core"]),
    (
        "estate-compiler",
        &["estate-core", "estate-schema", "estate-projection"],
    ),
    ("estate-sim", &["estate-core", "estate-projection"]),
    (
        "estate-cli",
        &[
            "estate-core",
            "estate-compiler",
            "estate-sim",
            "estate-projection",
        ],
    ),
];

/// Exact crate names that may not appear anywhere in the kernel graph.
///
/// Derived from section 10's forbidden list — "any `wgpu`, windowing,
/// renderer, audio, networking, watcher, or hot-reload dependency anywhere in
/// Gate K" — and from section 12's non-goals.
pub const FORBIDDEN_NAMES: &[(&str, &str)] = &[
    ("wgpu", "renderer"),
    ("ash", "renderer"),
    ("vulkano", "renderer"),
    ("glium", "renderer"),
    ("glow", "renderer"),
    ("metal", "renderer"),
    ("naga", "renderer"),
    ("pixels", "renderer"),
    ("softbuffer", "renderer"),
    ("skia-safe", "renderer"),
    ("femtovg", "renderer"),
    ("winit", "windowing"),
    ("sdl2", "windowing"),
    ("glutin", "windowing"),
    ("raw-window-handle", "windowing"),
    ("tao", "windowing"),
    ("wry", "windowing"),
    ("egui", "windowing"),
    ("eframe", "windowing"),
    ("iced", "windowing"),
    ("druid", "windowing"),
    ("fltk", "windowing"),
    ("minifb", "windowing"),
    ("ggez", "engine"),
    ("macroquad", "engine"),
    ("piston", "engine"),
    ("nannou", "engine"),
    ("three-d", "engine"),
    ("rodio", "audio"),
    ("cpal", "audio"),
    ("kira", "audio"),
    ("symphonia", "audio"),
    ("oboe", "audio"),
    ("alsa", "audio"),
    ("awedio", "audio"),
    ("tokio", "networking"),
    ("async-std", "networking"),
    ("smol", "networking"),
    ("mio", "networking"),
    ("socket2", "networking"),
    ("hyper", "networking"),
    ("reqwest", "networking"),
    ("ureq", "networking"),
    ("curl", "networking"),
    ("tonic", "networking"),
    ("quinn", "networking"),
    ("axum", "networking"),
    ("actix-web", "networking"),
    ("warp", "networking"),
    ("rocket", "networking"),
    ("tungstenite", "networking"),
    ("libp2p", "networking"),
    ("zmq", "networking"),
    ("rustls", "networking"),
    ("native-tls", "networking"),
    ("openssl", "networking"),
    ("notify", "watcher"),
    ("hotwatch", "watcher"),
    ("watchexec", "watcher"),
    ("inotify", "watcher"),
    ("libloading", "hot-reload"),
    ("dlopen", "hot-reload"),
    ("dlopen2", "hot-reload"),
    ("hot-lib-reloader", "hot-reload"),
];

/// Name prefixes that may not appear anywhere in the kernel graph.
pub const FORBIDDEN_PREFIXES: &[(&str, &str)] = &[
    ("wgpu-", "renderer"),
    ("winit-", "windowing"),
    ("bevy", "engine"),
    ("tokio-", "networking"),
    ("async-net", "networking"),
    ("async-io", "networking"),
    ("gfx-", "renderer"),
    ("sdl2-", "windowing"),
];

/// One boundary violation.
#[derive(Debug)]
pub struct Violation {
    pub rule: &'static str,
    pub detail: String,
}

impl Violation {
    fn new(rule: &'static str, detail: impl Into<String>) -> Self {
        Self {
            rule,
            detail: detail.into(),
        }
    }
}

/// The dependency graph, reduced to what the boundary rules care about.
#[derive(Debug)]
pub struct Graph {
    /// Package id to package name.
    names: BTreeMap<String, String>,
    /// Package id to resolved dependency package ids.
    edges: BTreeMap<String, BTreeSet<String>>,
    /// Names of the workspace members.
    members: BTreeSet<String>,
    /// Workspace member name to package id.
    member_ids: BTreeMap<String, String>,
}

impl Graph {
    /// Reduces `cargo metadata` output to a graph.
    pub fn from_metadata(metadata: &Value) -> Result<Self, String> {
        let mut names = BTreeMap::new();
        for package in metadata.field_array("packages") {
            let id = package
                .field_str("id")
                .ok_or("a package has no `id`")?
                .to_owned();
            let name = package
                .field_str("name")
                .ok_or("a package has no `name`")?
                .to_owned();
            names.insert(id, name);
        }

        let mut edges = BTreeMap::new();
        let resolve = metadata
            .get("resolve")
            .ok_or("`cargo metadata` produced no `resolve` graph")?;
        for node in resolve.field_array("nodes") {
            let id = node.field_str("id").ok_or("a resolve node has no `id`")?;
            let mut targets = BTreeSet::new();
            for dependency in node.field_array("deps") {
                if let Some(package) = dependency.field_str("pkg") {
                    targets.insert(package.to_owned());
                }
            }
            edges.insert(id.to_owned(), targets);
        }

        let mut members = BTreeSet::new();
        let mut member_ids = BTreeMap::new();
        for member in metadata.field_array("workspace_members") {
            let id = member
                .as_str()
                .ok_or("a workspace member is not a string")?;
            let name = names
                .get(id)
                .ok_or_else(|| format!("workspace member `{id}` has no package entry"))?;
            members.insert(name.clone());
            member_ids.insert(name.clone(), id.to_owned());
        }

        Ok(Self {
            names,
            edges,
            members,
            member_ids,
        })
    }

    fn name_of<'a>(&'a self, id: &'a str) -> &'a str {
        self.names.get(id).map_or(id, String::as_str)
    }

    fn direct_dependency_names(&self, member: &str) -> BTreeSet<&str> {
        self.member_ids
            .get(member)
            .and_then(|id| self.edges.get(id))
            .map(|targets| targets.iter().map(|id| self.name_of(id)).collect())
            .unwrap_or_default()
    }

    fn reachable_from(&self, roots: &[&str]) -> BTreeSet<&str> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut stack: Vec<&str> = roots
            .iter()
            .filter_map(|name| self.member_ids.get(*name).map(String::as_str))
            .collect();
        while let Some(id) = stack.pop() {
            if let Some(targets) = self.edges.get(id) {
                for target in targets {
                    if seen.insert(target.as_str()) {
                        stack.push(target.as_str());
                    }
                }
            }
        }
        seen.iter().map(|id| self.name_of(id)).collect()
    }

    /// Runs every boundary rule, returning the violations found.
    pub fn check(&self) -> Vec<Violation> {
        let mut violations = Vec::new();
        self.check_membership(&mut violations);
        self.check_permitted_edges(&mut violations);
        self.check_cycles(&mut violations);
        self.check_forbidden(&mut violations);
        self.check_tooling_isolation(&mut violations);
        violations
    }

    fn check_membership(&self, violations: &mut Vec<Violation>) {
        let declared: BTreeSet<&str> = KERNEL_CRATES
            .iter()
            .chain(TOOLING_CRATES.iter())
            .copied()
            .collect();
        for member in &self.members {
            if !declared.contains(member.as_str()) {
                violations.push(Violation::new(
                    "membership",
                    format!(
                        "workspace member `{member}` is neither a KERNEL.md section 10 kernel \
                         crate nor declared tooling"
                    ),
                ));
            }
        }
        for expected in KERNEL_CRATES {
            if !self.members.contains(expected) {
                violations.push(Violation::new(
                    "membership",
                    format!("kernel crate `{expected}` is missing from the workspace"),
                ));
            }
        }
    }

    fn check_permitted_edges(&self, violations: &mut Vec<Violation>) {
        for (crate_name, permitted) in PERMITTED_EDGES {
            if !self.members.contains(crate_name) {
                continue;
            }
            let permitted: BTreeSet<&str> = permitted.iter().copied().collect();
            for dependency in self.direct_dependency_names(crate_name) {
                let is_workspace_crate = self.members.contains(dependency);
                if is_workspace_crate && !permitted.contains(dependency) {
                    violations.push(Violation::new(
                        "permitted-edges",
                        format!(
                            "`{crate_name}` depends on `{dependency}`, which section 10 does not permit"
                        ),
                    ));
                }
            }
        }
    }

    fn check_cycles(&self, violations: &mut Vec<Violation>) {
        let mut state: BTreeMap<&str, u8> = BTreeMap::new();
        for crate_name in KERNEL_CRATES {
            if self.members.contains(crate_name) {
                self.visit(crate_name, &mut state, &mut Vec::new(), violations);
            }
        }
    }

    fn visit<'a>(
        &'a self,
        crate_name: &'a str,
        state: &mut BTreeMap<&'a str, u8>,
        path: &mut Vec<&'a str>,
        violations: &mut Vec<Violation>,
    ) {
        match state.get(crate_name) {
            Some(2) => return,
            Some(1) => {
                let mut cycle = path.clone();
                cycle.push(crate_name);
                violations.push(Violation::new(
                    "cycles",
                    format!("dependency cycle: {}", cycle.join(" -> ")),
                ));
                return;
            }
            _ => {}
        }
        state.insert(crate_name, 1);
        path.push(crate_name);
        for dependency in self.direct_dependency_names(crate_name) {
            if self.members.contains(dependency) {
                let dependency = self
                    .members
                    .get(dependency)
                    .map_or(dependency, String::as_str);
                self.visit(dependency, state, path, violations);
            }
        }
        path.pop();
        state.insert(crate_name, 2);
    }

    fn check_forbidden(&self, violations: &mut Vec<Violation>) {
        let reachable = self.reachable_from(&KERNEL_CRATES);
        for name in reachable {
            if let Some(category) = forbidden_category(name) {
                violations.push(Violation::new(
                    "forbidden-dependency",
                    format!(
                        "`{name}` is reachable from a kernel crate; section 10 forbids \
                         {category} dependencies anywhere in Gate K"
                    ),
                ));
            }
        }
    }

    fn check_tooling_isolation(&self, violations: &mut Vec<Violation>) {
        for tool in TOOLING_CRATES {
            if !self.members.contains(tool) {
                continue;
            }
            for dependency in self.reachable_from(&[tool]) {
                if KERNEL_CRATES.contains(&dependency) {
                    violations.push(Violation::new(
                        "tooling-isolation",
                        format!(
                            "tooling crate `{tool}` reaches kernel crate `{dependency}`; the \
                             boundary checker must stay outside the graph it checks"
                        ),
                    ));
                }
            }
        }
    }
}

/// The forbidden category a crate name falls into, if any.
pub fn forbidden_category(name: &str) -> Option<&'static str> {
    for (forbidden, category) in FORBIDDEN_NAMES {
        if name == *forbidden {
            return Some(category);
        }
    }
    for (prefix, category) in FORBIDDEN_PREFIXES {
        if name.starts_with(prefix) {
            return Some(category);
        }
    }
    None
}
