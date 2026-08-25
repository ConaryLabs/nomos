//! Issue #132: the three cases where the deleted JavaScript was wrong.
//!
//! `experiments/executable-gaol/src/build-plan.mjs:111-121` diverged from
//! `docs/movement.md`'s law in three ways that the four committed areas never
//! reach, so the twenty-scenario comparison could not see them:
//!
//! 1. the blocker filter ignored `claim.value`, so an active `value: false`
//!    blocker blocked (`build-plan.mjs:114`);
//! 2. `Math.max(base_cost, ...costs)` floored the cost at `base_cost`, so an
//!    active cost below the base cost was raised to it (`build-plan.mjs:118`);
//! 3. a traversable subject's `reasons` listed every active claim rather than
//!    only the maximum-cost claims (`build-plan.mjs:119`).
//!
//! This suite builds a `SimulationPlan` that reaches all three, runs the real
//! `nomos_sim::effective_facts` over it — the same resolver pair R1-1 accepted,
//! with no test double anywhere — writes the resulting document exactly as
//! `nomos effective-facts` would, and compiles a plan from it. In each case the
//! kernel's answer is the expected one and the JavaScript's is asserted to be
//! wrong.
//!
//! `nomos-projection` and `nomos-sim` are dev-dependencies for this file alone.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use nomos_core::{CanonicalValue, ClaimRef, EntityId, Ident, Sha256Digest, SourcePath, SourceSpan};
use nomos_projection::{
    LatticeCell, MovementClaim, MovementConnectivity, MovementResolverPlan, MovementSubject,
    ProjectedActivation, SimulationPlan,
};
use nomos_render_plan::json::Json;
use nomos_sim::{PersistedRuntimeState, SimulationState, effective_facts};

/// The resolver's base cost, chosen above one of the claim costs so that
/// divergence 2 is reachable.
const BASE_COST: u32 = 3;

/// Distinguishes the temporary workspaces of concurrently running tests.
static COUNTER: AtomicU32 = AtomicU32::new(0);

#[test]
fn the_plan_carries_the_kernel_disposition_cost_and_reasons() {
    let facts = kernel_facts();
    let workspace = Workspace::new(&facts);
    let compiled = nomos_render_plan::compile(workspace.inputs()).expect("the fixture compiles");
    let plan = nomos_render_plan::json::parse(&compiled.bytes).unwrap();
    let movement = plan
        .get("scenarios")
        .and_then(Json::as_array)
        .and_then(|scenarios| scenarios.first())
        .and_then(|scenario| scenario.get("movement"))
        .expect("the plan carries one scenario's movement facts");

    // 1. An active `blocks_ground` claim with `value: false` does not block.
    //    build-plan.mjs:114 filtered on `capability === "blocks_ground"` alone
    //    and would have emitted `blocked` with this claim as its reason.
    let gate = subject(movement, "escape_gate").expect("the gate is a subject");
    assert_eq!(disposition(gate), "traversable");
    assert_eq!(cost(gate), Some(i64::from(BASE_COST)));
    assert_eq!(reasons(gate), Vec::<String>::new());

    // 2. An active cost below `base_cost` is the cost. build-plan.mjs:118
    //    computed `Math.max(base_cost, ...costs)` and would have said 3.
    let floor = subject(movement, "sunken_floor").expect("the floor is a subject");
    assert_eq!(disposition(floor), "traversable");
    assert_eq!(cost(floor), Some(1));
    assert_eq!(
        reasons(floor),
        vec!["sunken_floor.region#traversal_cost_ground".to_owned()]
    );

    // 3. Two active costs of different value: the maximum wins and only the
    //    maximum-cost claim is a reason. build-plan.mjs:119 listed both.
    let channel = subject(movement, "deep_channel").expect("the channel is a subject");
    assert_eq!(disposition(channel), "traversable");
    assert_eq!(cost(channel), Some(5));
    assert_eq!(
        reasons(channel),
        vec!["deep_channel.deep#traversal_cost_ground".to_owned()],
        "the shallow claim is active but is not a maximum-cost reason"
    );
}

#[test]
fn the_compiled_plan_matches_the_kernel_document_field_for_field() {
    // Nothing in the compiler re-derives a fact: every movement value in the
    // plan is the value the kernel document carries, with `cost: null` on a
    // blocked subject the only spelling difference.
    let facts = kernel_facts();
    let document = nomos_core::canonical::read::parse_canonical(&facts[..facts.len() - 1]).unwrap();
    let workspace = Workspace::new(&facts);
    let compiled = nomos_render_plan::compile(workspace.inputs()).unwrap();
    let plan = nomos_render_plan::json::parse(&compiled.bytes).unwrap();
    let movement = plan
        .get("scenarios")
        .and_then(Json::as_array)
        .and_then(|scenarios| scenarios.first())
        .and_then(|scenario| scenario.get("movement"))
        .unwrap();

    let CanonicalValue::Object(root) = &document else {
        panic!("kernel document is an object")
    };
    let ground = root
        .get(&nomos_core::FieldName::declared("effective_facts"))
        .and_then(|facts| match facts {
            CanonicalValue::Object(fields) => {
                fields.get(&nomos_core::FieldName::declared("ground_movement"))
            }
            _ => None,
        })
        .and_then(|value| match value {
            CanonicalValue::Array(items) => Some(items),
            _ => None,
        })
        .expect("the kernel document carries ground movement facts");
    assert_eq!(ground.len(), 3);

    for fact in ground {
        let CanonicalValue::Object(fields) = fact else {
            panic!("fact is an object")
        };
        let CanonicalValue::Text(entity) = &fields[&nomos_core::FieldName::declared("entity")]
        else {
            panic!("entity is text")
        };
        let CanonicalValue::Object(kernel) =
            &fields[&nomos_core::FieldName::declared("disposition")]
        else {
            panic!("disposition is an object")
        };
        let planned = subject(movement, entity).expect("every subject reaches the plan");
        let CanonicalValue::Text(kind) = &kernel[&nomos_core::FieldName::declared("kind")] else {
            panic!("kind is text")
        };
        assert_eq!(disposition(planned), kind.as_str());
        let kernel_cost = kernel
            .get(&nomos_core::FieldName::declared("cost"))
            .map(|value| match value {
                CanonicalValue::Uint(cost) => i64::try_from(*cost).expect("a cost fits an i64"),
                CanonicalValue::Int(cost) => *cost,
                other => panic!("cost is an integer, found {other:?}"),
            });
        assert_eq!(cost(planned), kernel_cost);
    }
}

/// Runs the real resolver pair over a plan reaching all three divergences and
/// returns the bytes `nomos effective-facts` would have written.
fn kernel_facts() -> Vec<u8> {
    let span = SourceSpan::new(
        SourcePath::new("fixtures/divergence.txt").unwrap(),
        0,
        1,
        1,
        1,
    )
    .unwrap();

    // An active blocker whose value is false.
    let gate = MovementSubject::new(
        EntityId::parse("escape_gate").unwrap(),
        MovementConnectivity::FaceAdjacent {
            first: LatticeCell::new(1, 0, 0),
            second: LatticeCell::new(1, -1, 0),
        },
        vec![MovementClaim::blocker(
            ClaimRef::parse("escape_gate.portal#blocks_ground").unwrap(),
            ProjectedActivation::Always,
            false,
            span.clone(),
        )],
    )
    .unwrap();

    // One active cost below the resolver's base cost.
    let floor = MovementSubject::new(
        EntityId::parse("sunken_floor").unwrap(),
        MovementConnectivity::Region {
            min: LatticeCell::new(2, 2, 0),
            max: LatticeCell::new(3, 3, 0),
        },
        vec![
            MovementClaim::traversal_cost(
                ClaimRef::parse("sunken_floor.region#traversal_cost_ground").unwrap(),
                ProjectedActivation::Always,
                1,
                span.clone(),
            )
            .unwrap(),
        ],
    )
    .unwrap();

    // Two active costs of different value.
    let channel = MovementSubject::new(
        EntityId::parse("deep_channel").unwrap(),
        MovementConnectivity::Region {
            min: LatticeCell::new(5, 2, 0),
            max: LatticeCell::new(6, 3, 0),
        },
        vec![
            MovementClaim::traversal_cost(
                ClaimRef::parse("deep_channel.shallow#traversal_cost_ground").unwrap(),
                ProjectedActivation::Always,
                2,
                span.clone(),
            )
            .unwrap(),
            MovementClaim::traversal_cost(
                ClaimRef::parse("deep_channel.deep#traversal_cost_ground").unwrap(),
                ProjectedActivation::Always,
                5,
                span,
            )
            .unwrap(),
        ],
    )
    .unwrap();

    let resolver = MovementResolverPlan::new(
        Ident::new("ground").unwrap(),
        BASE_COST,
        true,
        true,
        true,
        true,
        vec![gate, floor, channel],
    )
    .unwrap();
    let plan = SimulationPlan::new(Vec::new(), Vec::new())
        .unwrap()
        .with_movement_resolver(resolver);
    let state = SimulationState::initialize(&plan).unwrap();
    let persisted = PersistedRuntimeState::new(&plan, state).unwrap();
    let document =
        effective_facts(&plan, &persisted, Sha256Digest::of_bytes(b"divergence")).unwrap();
    let mut bytes = document.to_canonical_bytes();
    bytes.push(b'\n');
    bytes
}

/// One movement subject, found by entity in the plan's stable-ID array.
///
/// `nomos.rendering_plan@3` spells `movement` the way
/// `nomos.effective_facts@1` does — an array of `{entity, ...}` rows ordered
/// by entity — rather than as an entity-keyed object, so that no object key
/// comes from data.
fn subject<'a>(movement: &'a Json, entity: &str) -> Option<&'a Json> {
    movement
        .as_array()?
        .iter()
        .find(|row| row.get("entity").and_then(Json::as_text) == Some(entity))
}

fn disposition(subject: &Json) -> &str {
    subject
        .get("disposition")
        .and_then(Json::as_text)
        .expect("a movement subject carries a disposition")
}

fn cost(subject: &Json) -> Option<i64> {
    match subject.get("cost").expect("cost is always present") {
        Json::Null => None,
        Json::Integer(value) => Some(*value),
        other => panic!("cost is null or an integer, found {other:?}"),
    }
}

fn reasons(subject: &Json) -> Vec<String> {
    subject
        .get("reasons")
        .and_then(Json::as_array)
        .expect("a movement subject carries reasons")
        .iter()
        .map(|reason| reason.as_text().expect("a reason is text").to_owned())
        .collect()
}

/// The input set around one kernel-produced facts document.
struct Workspace {
    root: PathBuf,
    paths: common::Paths,
}

impl Workspace {
    fn new(facts: &[u8]) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("render-plan-divergence")
            .join(format!(
                "{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("facts")).unwrap();
        fs::create_dir_all(root.join("runs/01-divergence")).unwrap();
        fs::create_dir_all(root.join("world")).unwrap();
        fs::write(root.join("facts/01-divergence.json"), facts).unwrap();

        for (file, name, version) in [
            ("simulation.json", "nomos.projection.simulation", 3),
            ("navigation.json", "nomos.projection.navigation", 1),
            ("persistence.json", "nomos.projection.persistence", 1),
            ("diagnostics.json", "nomos.projection.diagnostics", 1),
        ] {
            write(
                &root.join("world").join(file),
                &CanonicalValue::object_declared([("schema", common::schema(name, version))]),
            );
        }

        let state_hash = state_hash_of(facts);
        write(
            &root.join("runs/01-divergence/result.json"),
            &CanonicalValue::object_declared([
                ("committed_command_count", CanonicalValue::Uint(0)),
                ("schema", common::schema("nomos.run_result", 1)),
                ("status", CanonicalValue::text("completed")),
            ]),
        );
        write(
            &root.join("runs/01-divergence/final-state.json"),
            &CanonicalValue::object_declared([
                ("schema", common::schema("nomos.persisted_runtime_state", 2)),
                (
                    "state",
                    CanonicalValue::object_declared([
                        ("machines", CanonicalValue::Array(Vec::new())),
                        ("schema", common::schema("nomos.runtime_state", 2)),
                        ("tick", CanonicalValue::Uint(0)),
                    ]),
                ),
                ("state_hash", CanonicalValue::text(state_hash)),
            ]),
        );
        write(
            &root.join("runs/01-divergence/command-log.json"),
            &CanonicalValue::object_declared([
                ("rows", CanonicalValue::Array(Vec::new())),
                ("schema", common::schema("nomos.command_log", 1)),
            ]),
        );

        write(&root.join("entity-catalog.json"), &catalog());
        fs::write(root.join("presentation.json"), SOURCE).unwrap();

        let paths = common::Paths {
            catalog: root.join("entity-catalog.json"),
            facts: root.join("facts"),
            runs: root.join("runs"),
            world: root.join("world"),
            source: root.join("presentation.json"),
        };
        Self { root, paths }
    }

    fn inputs(&self) -> nomos_render_plan::Inputs<'_> {
        self.paths.as_inputs()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write(path: &Path, value: &CanonicalValue) {
    fs::write(path, value.to_canonical_bytes()).unwrap();
}

fn state_hash_of(facts: &[u8]) -> String {
    let document = nomos_core::canonical::read::parse_canonical(&facts[..facts.len() - 1]).unwrap();
    let CanonicalValue::Object(fields) = &document else {
        panic!("the kernel document is an object")
    };
    let CanonicalValue::Text(hash) = &fields[&nomos_core::FieldName::declared("state_hash")] else {
        panic!("state_hash is text")
    };
    hash.clone()
}

fn catalog() -> CanonicalValue {
    let entity = |id: &str, primitive: &str, capabilities: &[&str], claims: &[&str]| {
        CanonicalValue::object_declared([
            (
                "binding",
                CanonicalValue::object_declared([
                    (
                        "cell",
                        CanonicalValue::object_declared([
                            ("x", CanonicalValue::Int(1)),
                            ("y", CanonicalValue::Int(1)),
                            ("z", CanonicalValue::Int(0)),
                        ]),
                    ),
                    ("kind", CanonicalValue::text("cell")),
                ]),
            ),
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
                        .map(|claim| {
                            CanonicalValue::object_declared([
                                ("capability", CanonicalValue::text("traversal_cost_ground")),
                                ("id", CanonicalValue::text(*claim)),
                                ("resolver", CanonicalValue::text("movement")),
                                (
                                    "source",
                                    CanonicalValue::object_declared([
                                        ("byte_end", CanonicalValue::Uint(1)),
                                        ("byte_start", CanonicalValue::Uint(0)),
                                        ("column", CanonicalValue::Uint(1)),
                                        ("line", CanonicalValue::Uint(1)),
                                        ("path", CanonicalValue::text("fixtures/divergence.txt")),
                                    ]),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("id", CanonicalValue::text(id)),
            ("light_subject", CanonicalValue::Bool(false)),
            ("machines", CanonicalValue::Array(Vec::new())),
            ("movement_subject", CanonicalValue::Bool(true)),
            ("primitive", CanonicalValue::text(primitive)),
        ])
    };
    CanonicalValue::object_declared([
        (
            "entities",
            CanonicalValue::Array(vec![
                entity(
                    "deep_channel",
                    "primitive/shallow_water_region",
                    &["authority", "persisted", "region", "traversal_cost_ground"],
                    &[
                        "deep_channel.deep#traversal_cost_ground",
                        "deep_channel.shallow#traversal_cost_ground",
                    ],
                ),
                entity(
                    "escape_gate",
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
                    &[],
                ),
                entity(
                    "sunken_floor",
                    "primitive/shallow_water_region",
                    &["authority", "persisted", "region", "traversal_cost_ground"],
                    &["sunken_floor.region#traversal_cost_ground"],
                ),
                entity(
                    "watch_brazier",
                    "primitive/extinguishable_light",
                    &["authority", "emits_light", "machine", "persisted"],
                    &[],
                ),
            ]),
        ),
        ("schema", CanonicalValue::text("nomos.entity_catalog@1")),
        (
            "world",
            CanonicalValue::object_declared([
                ("manifest_digest", CanonicalValue::text("0".repeat(64))),
                ("world_ir_schema", CanonicalValue::text("nomos.world_ir@2")),
            ]),
        ),
    ])
}

const SOURCE: &str = r#"{
  "schema": "nomos.presentation_source@2",
  "area": { "id": "divergence", "label": "Divergence", "start": true },
  "route": { "exit": { "gate": "escape_gate", "to_area": null } },
  "pursuit": { "light": "watch_brazier" },
  "architecture": {
    "bounds": { "width": 9, "height": 6 },
    "wall_height_steps": 45,
    "style": { "assembly": "visual/beveled_masonry", "material_family": "stone_bounded", "trim_family": "broad_mortar" },
    "masses": []
  },
  "actors": [
    { "id": "player", "role": "player", "assembly": "visual/player_silhouette", "cell": { "x": 1, "y": 1, "z": 0 } },
    { "id": "gaoler", "role": "pursuer", "assembly": "visual/gaoler_silhouette", "cell": { "x": 4, "y": 3, "z": 0 } }
  ],
  "effects": []
}
"#;
