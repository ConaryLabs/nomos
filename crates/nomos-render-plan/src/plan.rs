//! The rendering plan: schema identity, assembly, and canonical bytes.
//!
//! This is the owner file for `nomos.rendering_plan@1`, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.
//!
//! The document's field names and structure are those of the study's
//! `nomos.experiment.rendering_plan@1`, unchanged apart from the identity
//! itself, so `render-core.mjs`, `play-state.mjs`, `webgl-renderer.mjs`,
//! `build-collection.mjs`, and the viewer keep working with only their
//! schema-string checks updated. What changed is where every field comes from:
//! kinds from the entity catalog rather than from a namespace suffix,
//! dispositions from `nomos.effective_facts@1` rather than from a second
//! activation evaluator, and the bytes from a canonical encoder rather than
//! from `JSON.stringify(plan, null, 2)`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use nomos_core::id::SchemaId;

use crate::area::{self, AreaSource};
use crate::catalog::EntityCatalog;
use crate::doc::PlanValue;
use crate::error::{PlanError, PlanResult, codes};
use crate::facts::EffectiveFacts;
use crate::read;
use crate::runs::{self, ScenarioRun};
use crate::world;

/// The rendering plan's schema identity.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn rendering_plan_schema() -> SchemaId {
    SchemaId::new("nomos.rendering_plan", 1).expect("the rendering-plan schema id is a literal")
}

/// The fixed camera and palette identities the plan republishes.
///
/// `docs/review/executable-gaol-ownership-audit.md` section 2 item 1 and
/// section 4's last row record these as renderer-catalog constants re-typed
/// into every content artifact, and section 2 item 9 records `palette` as a
/// string no consumer dereferences. R1-2 reproduces them unchanged; R1-3 and
/// R1-4 own moving them to the renderer that should hold them. Their prior site
/// is `experiments/executable-gaol/src/build-plan.mjs:177-178`.
mod look {
    pub const CAMERA_IDENTITY: &str = "gaol_oblique_01";
    pub const CAMERA_PROJECTION: &str = "fixed_oblique";
    pub const CAMERA_WIDTH: u64 = 1200;
    pub const CAMERA_HEIGHT: u64 = 540;
    pub const TILE_WIDTH: u64 = 96;
    pub const TILE_HEIGHT: u64 = 50;
    pub const PALETTE: &str = "gaol_bounded_01";
    pub const UI_ANCHORS: [&str; 4] = ["vitals", "abilities", "gate_state", "water_cost"];
}

/// The compiler's declared inputs.
///
/// Every one is a document or a directory of documents. There is deliberately
/// no field for `.nomos` source, Canonical World IR, or compiler receipts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Inputs<'a> {
    /// The `nomos.entity_catalog@1` document.
    pub catalog: &'a Path,
    /// A directory of `nomos.effective_facts@1` documents, one per scenario,
    /// named `<scenario>.json`.
    pub facts: &'a Path,
    /// The per-scenario run bundles.
    pub runs: &'a Path,
    /// The compiled world package, opened for four projection members only.
    pub world: &'a Path,
    /// The presentation source.
    pub area: &'a Path,
}

/// A compiled plan.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompiledPlan {
    /// Canonical bytes under `nomos.rendering_plan@1`.
    pub bytes: Vec<u8>,
    /// Entity count, for the compiler's status line.
    pub entity_count: usize,
    /// Scenario count, for the compiler's status line.
    pub scenario_count: usize,
    /// Derived interaction-edge count, for the compiler's status line.
    pub interaction_count: usize,
}

/// Compiles a rendering plan from documents alone.
///
/// # Errors
///
/// Returns the first `RP####` rejection any input produces. Nothing is written.
pub fn compile(inputs: Inputs<'_>) -> PlanResult<CompiledPlan> {
    let catalog_document = read::read_document(inputs.catalog)?;
    let catalog = EntityCatalog::decode(&catalog_document, inputs.catalog)?;
    let kinds: BTreeMap<String, &'static str> = catalog
        .entities()
        .iter()
        .map(|entity| (entity.id.clone(), entity.kind.as_str()))
        .collect();
    let area = area::read_area(inputs.area, &|entity| kinds.get(entity).copied())?;

    let projections = world::read_projections(inputs.world)?;
    let scenario_runs = runs::read_runs(inputs.runs)?;
    let facts = read_facts(inputs.facts, &scenario_runs)?;

    let state_hashes: BTreeMap<String, String> = facts
        .iter()
        .map(|(id, fact)| (id.clone(), fact.state_hash.clone()))
        .collect();
    let edges = runs::interaction_edges(&scenario_runs, &state_hashes);

    let mut entities = Vec::new();
    for entity in catalog.entities() {
        let provenance = entity
            .claims
            .iter()
            .filter(|claim| claim.resolver == "movement")
            .map(|claim| {
                PlanValue::object([
                    ("claim", PlanValue::text(claim.id.clone())),
                    ("source", PlanValue::from_canonical(&claim.source)),
                ])
            })
            .collect();
        entities.push(PlanValue::object([
            ("id", PlanValue::text(entity.id.clone())),
            ("kind", PlanValue::text(entity.kind.as_str())),
            (
                "visualAssembly",
                PlanValue::text(entity.kind.visual_assembly()),
            ),
            (
                "materialFamily",
                PlanValue::text(entity.kind.material_family()),
            ),
            ("anchor", PlanValue::from_canonical(&entity.binding)),
            (
                "machineNamespaces",
                PlanValue::Array(
                    entity
                        .machine_namespaces
                        .iter()
                        .map(|namespace| PlanValue::text(namespace.clone()))
                        .collect(),
                ),
            ),
            ("provenance", PlanValue::Array(provenance)),
        ]));
    }

    let mut scenarios = Vec::new();
    for run in &scenario_runs {
        let fact = &facts[&run.id];
        let movement = PlanValue::keyed_object(fact.movement.iter().map(|(entity, resolved)| {
            (
                entity.clone(),
                PlanValue::object([
                    ("disposition", PlanValue::text(resolved.disposition.clone())),
                    // A blocked subject's cost is spelled `null`: the kernel's
                    // Blocked variant carries no cost key, and the plan has
                    // always published one. Presentation, not semantics —
                    // `RUNTIME.md` section 5 R1-1 names this the only
                    // normalization in the comparison.
                    (
                        "cost",
                        resolved.cost.map_or(PlanValue::Null, PlanValue::Uint),
                    ),
                    (
                        "reasons",
                        PlanValue::Array(
                            resolved
                                .reasons
                                .iter()
                                .map(|reason| PlanValue::text(reason.clone()))
                                .collect(),
                        ),
                    ),
                ]),
            )
        }))?;
        let effective_light = PlanValue::keyed_object(
            fact.light
                .iter()
                .map(|(entity, emitting)| (entity.clone(), PlanValue::Bool(*emitting))),
        )?;
        let machine_states = PlanValue::keyed_object(
            run.machine_states
                .iter()
                .map(|(namespace, state)| (namespace.clone(), PlanValue::text(state.clone()))),
        )?;
        scenarios.push(PlanValue::object([
            ("id", PlanValue::text(run.id.clone())),
            ("label", PlanValue::text(scenario_label(&run.id))),
            ("tick", PlanValue::Uint(fact.tick)),
            ("stateHash", PlanValue::text(fact.state_hash.clone())),
            ("machineStates", machine_states),
            ("movement", movement),
            ("effectiveLight", effective_light),
        ]));
    }

    let interactions = edges
        .iter()
        .map(|edge| {
            PlanValue::object([
                ("id", PlanValue::text(edge.id.clone())),
                ("fromScenario", PlanValue::text(edge.from_scenario.clone())),
                ("toScenario", PlanValue::text(edge.to_scenario.clone())),
                ("targetEntity", PlanValue::text(edge.target_entity.clone())),
                ("action", PlanValue::text(edge.action.clone())),
                (
                    "inputStateHash",
                    PlanValue::text(edge.input_state_hash.clone()),
                ),
                (
                    "resultingStateHash",
                    PlanValue::text(edge.resulting_state_hash.clone()),
                ),
            ])
        })
        .collect();

    let plan = assemble(&area, &projections, entities, scenarios, interactions)?;
    let mut bytes = plan.to_canonical_bytes();
    bytes.push(b'\n');
    Ok(CompiledPlan {
        entity_count: catalog.entities().len(),
        scenario_count: scenario_runs.len(),
        interaction_count: edges.len(),
        bytes,
    })
}

fn assemble(
    area: &AreaSource,
    projections: &[world::ProjectionFacts],
    entities: Vec<PlanValue>,
    scenarios: Vec<PlanValue>,
    interactions: Vec<PlanValue>,
) -> PlanResult<PlanValue> {
    let projection_digests = PlanValue::keyed_object(
        projections
            .iter()
            .map(|facts| (facts.file.to_owned(), PlanValue::text(facts.digest.clone()))),
    )?;
    Ok(PlanValue::object([
        (
            "schema",
            PlanValue::text(rendering_plan_schema().to_string()),
        ),
        ("deterministic", PlanValue::Bool(true)),
        (
            "area",
            PlanValue::object([
                ("id", PlanValue::text(area.id.clone())),
                ("label", PlanValue::text(area.label.clone())),
                ("start", PlanValue::Bool(area.start)),
            ]),
        ),
        (
            "projectionSchemas",
            PlanValue::Array(
                projections
                    .iter()
                    .map(|facts| PlanValue::from_canonical(&facts.schema))
                    .collect(),
            ),
        ),
        ("projectionDigests", projection_digests),
        (
            "camera",
            PlanValue::object([
                ("identity", PlanValue::text(look::CAMERA_IDENTITY)),
                ("projection", PlanValue::text(look::CAMERA_PROJECTION)),
                ("width", PlanValue::Uint(look::CAMERA_WIDTH)),
                ("height", PlanValue::Uint(look::CAMERA_HEIGHT)),
                ("tileWidth", PlanValue::Uint(look::TILE_WIDTH)),
                ("tileHeight", PlanValue::Uint(look::TILE_HEIGHT)),
            ]),
        ),
        ("palette", PlanValue::text(look::PALETTE)),
        ("architecture", PlanValue::from_area(&area.architecture)?),
        ("entities", PlanValue::Array(entities)),
        ("actors", PlanValue::from_area(&area.actors)?),
        ("effects", PlanValue::from_area(&area.effects)?),
        (
            "presentation",
            PlanValue::object([
                ("primaryGate", PlanValue::text(area.primary_gate.clone())),
                ("objective", PlanValue::from_area(&area.objective)?),
                ("pursuitLight", PlanValue::text(area.pursuit_light.clone())),
                (
                    "forensicScenario",
                    PlanValue::text(area.forensic_scenario.clone()),
                ),
                ("exit", PlanValue::from_area(&area.exit)?),
            ]),
        ),
        (
            "uiAnchors",
            PlanValue::Array(
                look::UI_ANCHORS
                    .iter()
                    .copied()
                    .map(PlanValue::text)
                    .collect(),
            ),
        ),
        ("scenarios", PlanValue::Array(scenarios)),
        ("interactions", PlanValue::Array(interactions)),
    ]))
}

/// The scenario's display label.
///
/// `build-plan.mjs:133` derived it from the scenario directory name by
/// stripping a numeric prefix and replacing hyphens with spaces.
/// `docs/review/executable-gaol-ownership-audit.md` section 3 item 14 records
/// that as convention-derived. R1-2 reproduces it; R1-3's typed presentation
/// source is where an authored scenario label belongs.
fn scenario_label(id: &str) -> String {
    let digits = id.bytes().take_while(u8::is_ascii_digit).count();
    let stripped = match id.as_bytes().get(digits) {
        Some(b'-') if digits > 0 => &id[digits + 1..],
        _ => id,
    };
    stripped.replace('-', " ")
}

fn read_facts(
    facts_dir: &Path,
    scenario_runs: &[ScenarioRun],
) -> PlanResult<BTreeMap<String, EffectiveFacts>> {
    let mut documents = BTreeMap::new();
    for run in scenario_runs {
        let path: PathBuf = facts_dir.join(format!("{}.json", run.id));
        if !path.is_file() {
            return Err(PlanError::new(
                codes::SCENARIO_SET_MISMATCH,
                format!(
                    "run bundle `{}` has no effective-fact document in the facts directory",
                    run.id
                ),
            )
            .at(&path));
        }
        let document = read::read_document(&path)?;
        documents.insert(run.id.clone(), EffectiveFacts::decode(&document, &path)?);
    }

    let entries = std::fs::read_dir(facts_dir).map_err(|error| {
        PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(facts_dir)
    })?;
    for entry in entries {
        let entry =
            entry.map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(scenario) = name.strip_suffix(".json") else {
            continue;
        };
        if !documents.contains_key(scenario) {
            return Err(PlanError::new(
                codes::SCENARIO_SET_MISMATCH,
                format!("effective-fact document `{name}` has no run bundle"),
            )
            .at(facts_dir));
        }
    }
    Ok(documents)
}
