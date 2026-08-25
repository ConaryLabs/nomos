//! A complete, minimal input set on disk, for the acceptance tests.
//!
//! Every document here is built with `nomos_core::CanonicalValue` and written
//! as canonical bytes, so the fixtures are produced the same way the kernel
//! produces the real ones rather than by pasting JSON text. The one exception
//! is `area.json`, which is deliberately written as hand-authored,
//! pretty-printed, camelCase text with decimal values, because that is what the
//! compiler has to accept until R1-3 replaces it.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nomos_core::CanonicalValue;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A temporary directory holding one complete input set.
pub struct Fixture {
    pub root: PathBuf,
}

/// How the fixture spells entity ids and machine namespaces.
pub type Rename = fn(&str) -> String;

/// The identity a document declares, so a test can plant a mismatch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Identity {
    /// The identity the compiler expects.
    Correct,
    /// The right name at the wrong version.
    WrongVersion,
    /// A different schema name entirely.
    WrongName,
}

impl Identity {
    fn spell(self, name: &str, version: u64) -> CanonicalValue {
        match self {
            Self::Correct => schema(name, version),
            Self::WrongVersion => schema(name, version + 1),
            Self::WrongName => schema("nomos.not_the_catalog", version),
        }
    }
}

/// The fixture's tuning knobs.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Applied to every entity id and every machine namespace prefix.
    pub rename: Rename,
    /// The identity the catalog declares.
    pub catalog_identity: Identity,
    /// The identity every facts document declares.
    pub facts_identity: Identity,
    /// Whether the world directory also holds unreadable World IR, compiler
    /// receipts, a manifest, and a `.nomos` source.
    pub poison_world: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            rename: identity_rename,
            catalog_identity: Identity::Correct,
            facts_identity: Identity::Correct,
            poison_world: false,
        }
    }
}

fn identity_rename(name: &str) -> String {
    name.to_owned()
}

/// A rename that changes every entity id and machine namespace beyond
/// recognition, including destroying the `.access` suffix convention
/// `build-plan.mjs:25` classified doors by.
pub fn scramble(name: &str) -> String {
    let mut out = String::from("zz9_");
    for segment in name.split('.') {
        if !out.ends_with('_') {
            out.push('.');
        }
        out.push_str(&segment.chars().rev().collect::<String>());
        out.push_str("_q");
    }
    out
}

pub fn schema(name: &str, version: u64) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("name", CanonicalValue::text(name)),
        ("version", CanonicalValue::Uint(version)),
    ])
}

fn write_canonical(path: &Path, value: &CanonicalValue) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, value.to_canonical_bytes()).unwrap();
}

fn source_span(line: u64) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("byte_end", CanonicalValue::Uint(line * 100 + 40)),
        ("byte_start", CanonicalValue::Uint(line * 100)),
        ("column", CanonicalValue::Uint(1)),
        ("line", CanonicalValue::Uint(line)),
        ("path", CanonicalValue::text("fixtures/plan.nomos")),
    ])
}

impl Fixture {
    /// Builds a fixture with the default options.
    pub fn new(label: &str) -> Self {
        Self::with(label, Options::default())
    }

    /// Builds a fixture.
    pub fn with(label: &str, options: Options) -> Self {
        let index = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("render-plan")
            .join(format!("{}-{nonce}-{label}-{index}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self { root };
        fixture.write_world(options.poison_world);
        fixture.write_catalog(options);
        fixture.write_scenarios(options);
        fixture.write_area(options);
        fixture
    }

    pub fn catalog(&self) -> PathBuf {
        self.root.join("entity-catalog.json")
    }

    pub fn facts(&self) -> PathBuf {
        self.root.join("facts")
    }

    pub fn runs(&self) -> PathBuf {
        self.root.join("runs")
    }

    pub fn world(&self) -> PathBuf {
        self.root.join("world")
    }

    pub fn area(&self) -> PathBuf {
        self.root.join("area.json")
    }

    pub fn out(&self) -> PathBuf {
        self.root.join("rendering-plan.json")
    }

    pub fn inputs(&self) -> nomos_render_plan::Inputs<'_> {
        Box::leak(Box::new(Paths {
            catalog: self.catalog(),
            facts: self.facts(),
            runs: self.runs(),
            world: self.world(),
            area: self.area(),
        }))
        .as_inputs()
    }

    fn write_world(&self, poison: bool) {
        let world = self.world();
        for (file, name, version) in [
            ("simulation.json", "nomos.projection.simulation", 3),
            ("navigation.json", "nomos.projection.navigation", 1),
            ("persistence.json", "nomos.projection.persistence", 1),
            ("diagnostics.json", "nomos.projection.diagnostics", 1),
        ] {
            write_canonical(
                &world.join(file),
                &CanonicalValue::object_declared([
                    ("entities", CanonicalValue::Array(Vec::new())),
                    ("schema", schema(name, version)),
                ]),
            );
        }
        if poison {
            // Anything that opened one of these would fail: they are neither
            // canonical bytes nor JSON nor `.nomos` source. The compile has to
            // succeed anyway.
            for file in [
                "world-ir.json",
                "compiler-receipts.json",
                "manifest.json",
                "schemas.json",
                "world.nomos",
            ] {
                fs::write(world.join(file), b"\x00 not a document \xff").unwrap();
            }
        }
    }

    fn write_catalog(&self, options: Options) {
        let rename = options.rename;
        let entities = CanonicalValue::Array(vec![
            catalog_entity(
                rename,
                "flooded_section",
                "primitive/shallow_water_region",
                &["authority", "persisted", "region", "traversal_cost_ground"],
                CanonicalValue::object_declared([
                    ("kind", CanonicalValue::text("region")),
                    ("max", cell(4, 3)),
                    ("min", cell(2, 2)),
                ]),
                &[],
                &[(
                    "flooded_section.region#traversal_cost_ground",
                    "traversal_cost_ground",
                    "movement",
                    14,
                )],
                true,
                false,
            ),
            catalog_entity(
                rename,
                "north_gate",
                "primitive/iron_barred_door",
                &[
                    "authority",
                    "blocks_ground",
                    "boundary",
                    "interactable",
                    "machine",
                    "persisted",
                    "portal",
                ],
                CanonicalValue::object_declared([
                    ("cell", cell(5, 0)),
                    ("direction", CanonicalValue::text("north")),
                    ("kind", CanonicalValue::text("face")),
                ]),
                &[
                    ("north_gate.access", &["closed", "locked", "open"], "locked"),
                    ("north_gate.ward", &["sealed", "unsealed"], "sealed"),
                ],
                &[
                    (
                        "north_gate.portal#blocks_ground",
                        "blocks_ground",
                        "movement",
                        4,
                    ),
                    (
                        "north_gate.ward#blocks_ground",
                        "blocks_ground",
                        "movement",
                        4,
                    ),
                ],
                true,
                false,
            ),
            catalog_entity(
                rename,
                "watch_brazier",
                "primitive/extinguishable_light",
                &[
                    "authority",
                    "emits_light",
                    "interactable",
                    "machine",
                    "persisted",
                ],
                CanonicalValue::object_declared([
                    ("cell", cell(3, 1)),
                    ("kind", CanonicalValue::text("cell")),
                ]),
                &[("watch_brazier.emission", &["extinguished", "lit"], "lit")],
                &[(
                    "watch_brazier.emission#emits_light",
                    "emits_light",
                    "light",
                    18,
                )],
                false,
                true,
            ),
        ]);
        write_canonical(
            &self.catalog(),
            &CanonicalValue::object_declared([
                ("entities", entities),
                (
                    "schema",
                    options.catalog_identity.spell("nomos.entity_catalog", 1),
                ),
                (
                    "world",
                    CanonicalValue::object_declared([
                        ("manifest_digest", CanonicalValue::text("0".repeat(64))),
                        ("world_ir_schema", CanonicalValue::text("nomos.world_ir@2")),
                    ]),
                ),
            ]),
        );
    }

    fn write_scenarios(&self, options: Options) {
        let rename = options.rename;
        let gate = rename("north_gate");
        let water = rename("flooded_section");
        let brazier = rename("watch_brazier");
        let baseline_hash = "a".repeat(64);
        let breached_hash = "b".repeat(64);

        // 01-baseline: the gate is sealed and blocked, the brazier is lit.
        self.write_scenario(
            "01-baseline",
            "rejected",
            0,
            &[],
            &[
                (format!("{gate}.access"), "locked"),
                (format!("{gate}.ward"), "sealed"),
                (format!("{brazier}.emission"), "lit"),
            ],
            &baseline_hash,
            1,
            &[
                (
                    water.clone(),
                    "traversable",
                    Some(3),
                    vec![format!("{water}.region#traversal_cost_ground")],
                ),
                (
                    gate.clone(),
                    "blocked",
                    None,
                    vec![
                        format!("{gate}.portal#blocks_ground"),
                        format!("{gate}.ward#blocks_ground"),
                    ],
                ),
            ],
            &[(brazier.clone(), true)],
            options,
        );

        // 02-unsealed: one committed command unseals the ward, so the gate
        // resolves traversable and an interaction edge exists from 01.
        self.write_scenario(
            "02-unsealed",
            "completed",
            1,
            &[(&gate, "unseal", &baseline_hash, &breached_hash)],
            &[
                (format!("{gate}.access"), "locked"),
                (format!("{gate}.ward"), "unsealed"),
                (format!("{brazier}.emission"), "lit"),
            ],
            &breached_hash,
            2,
            &[
                (
                    water.clone(),
                    "traversable",
                    Some(3),
                    vec![format!("{water}.region#traversal_cost_ground")],
                ),
                (gate.clone(), "traversable", Some(1), vec![]),
            ],
            &[(brazier, true)],
            options,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn write_scenario(
        &self,
        id: &str,
        status: &str,
        committed: u64,
        rows: &[(&str, &str, &str, &str)],
        machines: &[(String, &str)],
        state_hash: &str,
        tick: u64,
        movement: &[(String, &str, Option<u64>, Vec<String>)],
        light: &[(String, bool)],
        options: Options,
    ) {
        let dir = self.runs().join(id);
        write_canonical(
            &dir.join("result.json"),
            &CanonicalValue::object_declared([
                ("committed_command_count", CanonicalValue::Uint(committed)),
                ("schema", schema("nomos.run_result", 1)),
                ("status", CanonicalValue::text(status)),
            ]),
        );
        write_canonical(
            &dir.join("final-state.json"),
            &CanonicalValue::object_declared([
                ("schema", schema("nomos.persisted_runtime_state", 2)),
                (
                    "state",
                    CanonicalValue::object_declared([
                        (
                            "machines",
                            CanonicalValue::Array(
                                machines
                                    .iter()
                                    .map(|(namespace, state)| {
                                        CanonicalValue::object_declared([
                                            ("namespace", CanonicalValue::text(namespace.clone())),
                                            ("state", CanonicalValue::text(*state)),
                                        ])
                                    })
                                    .collect(),
                            ),
                        ),
                        ("schema", schema("nomos.runtime_state", 2)),
                        ("tick", CanonicalValue::Uint(tick)),
                    ]),
                ),
                ("state_hash", CanonicalValue::text(state_hash)),
            ]),
        );
        write_canonical(
            &dir.join("command-log.json"),
            &CanonicalValue::object_declared([
                (
                    "rows",
                    CanonicalValue::Array(
                        rows.iter()
                            .enumerate()
                            .map(|(ordinal, (entity, action, input, result))| {
                                CanonicalValue::object_declared([
                                    ("input_state_hash", CanonicalValue::text(*input)),
                                    ("ordinal", CanonicalValue::Uint(ordinal as u64)),
                                    (
                                        "request",
                                        CanonicalValue::object_declared([
                                            ("action", CanonicalValue::text(*action)),
                                            ("argument", CanonicalValue::Null),
                                            ("entity", CanonicalValue::text(*entity)),
                                        ]),
                                    ),
                                    ("resulting_state_hash", CanonicalValue::text(*result)),
                                ])
                            })
                            .collect(),
                    ),
                ),
                ("schema", schema("nomos.command_log", 1)),
            ]),
        );

        let ground = CanonicalValue::Array(
            movement
                .iter()
                .map(|(entity, kind, cost, reasons)| {
                    let mut fields = vec![
                        ("kind", CanonicalValue::text(*kind)),
                        (
                            "reasons",
                            CanonicalValue::Array(
                                reasons
                                    .iter()
                                    .map(|reason| CanonicalValue::text(reason.clone()))
                                    .collect(),
                            ),
                        ),
                    ];
                    if let Some(cost) = cost {
                        fields.push(("cost", CanonicalValue::Uint(*cost)));
                    }
                    CanonicalValue::object_declared([
                        ("disposition", CanonicalValue::object_declared(fields)),
                        ("entity", CanonicalValue::text(entity.clone())),
                    ])
                })
                .collect(),
        );
        let emission = CanonicalValue::Array(
            light
                .iter()
                .map(|(entity, emitting)| {
                    CanonicalValue::object_declared([
                        ("emitting", CanonicalValue::Bool(*emitting)),
                        ("entity", CanonicalValue::text(entity.clone())),
                        ("reasons", CanonicalValue::Array(Vec::new())),
                    ])
                })
                .collect(),
        );
        write_canonical(
            &self.facts().join(format!("{id}.json")),
            &CanonicalValue::object_declared([
                ("command", CanonicalValue::text("effective-facts")),
                (
                    "effective_facts",
                    CanonicalValue::object_declared([
                        ("ground_movement", ground),
                        ("light_emission", emission),
                    ]),
                ),
                (
                    "schema",
                    options.facts_identity.spell("nomos.effective_facts", 1),
                ),
                ("state_hash", CanonicalValue::text(state_hash)),
                ("status", CanonicalValue::text("completed")),
                ("tick", CanonicalValue::Uint(tick)),
            ]),
        );
    }

    fn write_area(&self, options: Options) {
        let rename = options.rename;
        let gate = rename("north_gate");
        let brazier = rename("watch_brazier");
        // Hand-authored presentation source: pretty-printed, camelCase, and
        // carrying the decimal transforms the audit's section 4 lists.
        let text = format!(
            r#"{{
  "id": "test-area",
  "label": "Test Area",
  "start": true,
  "primaryGate": "{gate}",
  "objective": {{ "kind": "exit_via", "target": "{gate}" }},
  "pursuitLight": "{brazier}",
  "forensicScenario": "02-unsealed",
  "exit": {{ "gate": "{gate}", "toArea": null }},
  "architecture": {{
    "bounds": {{ "width": 9, "height": 6 }},
    "wallHeight": 4.5,
    "style": {{
      "assembly": "visual/beveled_masonry",
      "materialFamily": "stone_bounded",
      "trimFamily": "broad_mortar"
    }},
    "masses": [
      {{ "id": "pier", "min": {{ "x": 2, "y": 1 }}, "max": {{ "x": 3, "y": 2 }}, "height": 3.2 }}
    ]
  }},
  "actors": [
    {{ "id": "player", "assembly": "visual/player_silhouette", "anchor": {{ "kind": "cell", "cell": {{ "x": 7, "y": 4, "z": 0 }} }} }},
    {{ "id": "gaoler", "assembly": "visual/gaoler_silhouette", "anchor": {{ "kind": "cell", "cell": {{ "x": 4, "y": 3, "z": 0 }} }} }}
  ],
  "effects": [
    {{ "id": "ward_crescent", "assembly": "visual/cyan_crescent", "anchorEntity": "{gate}", "presentationAnchor": {{ "x": 4.9, "y": 3.8, "z": 0 }} }}
  ]
}}
"#
        );
        fs::write(self.area(), text).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Owned paths, so `Inputs` can borrow them for the lifetime of a test.
pub struct Paths {
    pub catalog: PathBuf,
    pub facts: PathBuf,
    pub runs: PathBuf,
    pub world: PathBuf,
    pub area: PathBuf,
}

impl Paths {
    pub fn as_inputs(&self) -> nomos_render_plan::Inputs<'_> {
        nomos_render_plan::Inputs {
            catalog: &self.catalog,
            facts: &self.facts,
            runs: &self.runs,
            world: &self.world,
            area: &self.area,
        }
    }
}

fn cell(x: i64, y: i64) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("x", CanonicalValue::Int(x)),
        ("y", CanonicalValue::Int(y)),
        ("z", CanonicalValue::Int(0)),
    ])
}

#[allow(clippy::too_many_arguments)]
fn catalog_entity(
    rename: Rename,
    id: &str,
    primitive: &str,
    capabilities: &[&str],
    binding: CanonicalValue,
    machines: &[(&str, &[&str], &str)],
    claims: &[(&str, &str, &str, u64)],
    movement_subject: bool,
    light_subject: bool,
) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("binding", binding),
        (
            "capabilities",
            CanonicalValue::Array(
                capabilities
                    .iter()
                    .map(|it| CanonicalValue::text(*it))
                    .collect(),
            ),
        ),
        (
            "claims",
            CanonicalValue::Array(
                claims
                    .iter()
                    .map(|(claim, capability, resolver, line)| {
                        CanonicalValue::object_declared([
                            ("capability", CanonicalValue::text(*capability)),
                            ("id", CanonicalValue::text(rename_claim(rename, claim))),
                            ("resolver", CanonicalValue::text(*resolver)),
                            ("source", source_span(*line)),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("id", CanonicalValue::text(rename(id))),
        ("light_subject", CanonicalValue::Bool(light_subject)),
        (
            "machines",
            CanonicalValue::Array(
                machines
                    .iter()
                    .map(|(namespace, states, initial)| {
                        CanonicalValue::object_declared([
                            ("initial", CanonicalValue::text(*initial)),
                            ("namespace", CanonicalValue::text(rename(namespace))),
                            (
                                "states",
                                CanonicalValue::Array(
                                    states.iter().map(|it| CanonicalValue::text(*it)).collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("movement_subject", CanonicalValue::Bool(movement_subject)),
        ("primitive", CanonicalValue::text(primitive)),
    ])
}

fn rename_claim(rename: Rename, claim: &str) -> String {
    match claim.split_once('#') {
        Some((namespace, capability)) => format!("{}#{capability}", rename(namespace)),
        None => rename(claim),
    }
}

/// The documented equivalence normalization, as a test helper.
///
/// `experiments/executable-gaol/compare-rendering-plan.sh` implements the same
/// rule for the four committed areas; this is the Rust sibling, and
/// `tests/normalization.rs` proves its properties.
///
/// The rule: parse both documents as JSON, so key order and insignificant
/// whitespace are ignored; ignore the `schema` field on both sides; compare
/// everything else exactly, including array order and including a `null` value,
/// which is never equal to an absent key.
pub fn normalized_differences(left: &[u8], right: &[u8]) -> Vec<String> {
    use nomos_render_plan::json::{self, Json};

    fn strip_schema(value: Json) -> Json {
        match value {
            Json::Object(mut fields) => {
                fields.remove("schema");
                Json::Object(fields)
            }
            other => other,
        }
    }

    fn kind(value: &Json) -> &'static str {
        match value {
            Json::Null => "null",
            Json::Bool(_) => "boolean",
            Json::Number(_) => "number",
            Json::Text(_) => "string",
            Json::Array(_) => "array",
            Json::Object(_) => "object",
        }
    }

    fn walk(path: &str, left: &Json, right: &Json, out: &mut Vec<String>) {
        if kind(left) != kind(right) {
            out.push(format!("{path}: {} != {}", kind(left), kind(right)));
            return;
        }
        match (left, right) {
            (Json::Array(left), Json::Array(right)) => {
                if left.len() != right.len() {
                    out.push(format!(
                        "{path}: array length {} != {}",
                        left.len(),
                        right.len()
                    ));
                    return;
                }
                for (index, (left, right)) in left.iter().zip(right).enumerate() {
                    walk(&format!("{path}[{index}]"), left, right, out);
                }
            }
            (Json::Object(left), Json::Object(right)) => {
                let mut keys: Vec<&String> = left.keys().chain(right.keys()).collect();
                keys.sort();
                keys.dedup();
                for key in keys {
                    match (left.get(key), right.get(key)) {
                        (Some(left), Some(right)) => {
                            walk(&format!("{path}.{key}"), left, right, out);
                        }
                        (Some(_), None) => out.push(format!("{path}.{key}: present != absent")),
                        (None, Some(_)) => out.push(format!("{path}.{key}: absent != present")),
                        (None, None) => unreachable!("key came from one of the two maps"),
                    }
                }
            }
            (Json::Number(left), Json::Number(right)) => {
                if left.units() != right.units() {
                    out.push(format!("{path}: {} != {}", left.lexeme(), right.lexeme()));
                }
            }
            (left, right) => {
                if left != right {
                    out.push(format!("{path}: {left:?} != {right:?}"));
                }
            }
        }
    }

    let left = strip_schema(json::parse(left).expect("left document is JSON"));
    let right = strip_schema(json::parse(right).expect("right document is JSON"));
    let mut differences = Vec::new();
    walk("$", &left, &right, &mut differences);
    differences
}
