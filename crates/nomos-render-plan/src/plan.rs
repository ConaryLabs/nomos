//! The rendering plan: schema identity, assembly, and canonical bytes.
//!
//! This is the owner file for `nomos.rendering_plan@3`, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.
//!
//! # Why `@2`, and then `@3`
//!
//! `@1` reproduced the study's document shape so that R1-2's consumers changed
//! only their schema-string check. That shape was camelCase, keyed two of its
//! objects by dotted identifiers, and carried the decimal presentation values
//! `area.json` authored — three things `nomos_core::CanonicalValue` cannot
//! express — so R1-2 had to ship a private canonical encoder in `src/doc.rs`
//! (issue #144), a second implementation of the `KERNEL.md` section 7 byte
//! profile in the accepted tree.
//!
//! `@2` is designed to fit inside `CanonicalValue` with no widening at all, and
//! `doc.rs` is deleted:
//!
//! - every field name is snake_case, so it is a `nomos_core::FieldName`;
//! - the two dotted-key objects become arrays of declared-field pairs —
//!   `projection_digests` as `{file, digest}` and `scenarios[].machine_states`
//!   as `{namespace, state}`, which is how the kernel already spells the same
//!   collection in a run bundle's `final-state.json`;
//! - the two entity-keyed objects, `movement` and `effective_light`, become the
//!   `{entity, ...}` arrays `nomos.effective_facts@2` itself uses, ordered by
//!   `nomos_core::canonical::keyed_array` so that the stable-ID ordering rule
//!   and duplicate-identity refusal come from the kernel rather than from here;
//! - heights are integer `vertical_step` counts, so no decimal survives.
//!
//! `docs/review/presentation-source.md` section 2 is the full delta from `@1`,
//! with a reason for each of the twenty changes.
//!
//! `@3` is two changes, made together so the four fixtures are regenerated
//! once (`docs/review/nomos-play.md` section 6):
//!
//! - `actors[]` gains `role`, `player` or `pursuer`, which is what
//!   `crates/nomos-play` reads to decide which actor a command moves and which
//!   one the pursuit rule steps. It retires the ownership audit's items 7 and
//!   21, where an actor's identity string was the only role signal.
//! - `entities[]` loses `visual_assembly` and `material_family`. Both were
//!   renderer-catalog data assigned per kind by a table in
//!   `crates/nomos-render-plan/src/catalog.rs`, whose own comment said the
//!   correct change was to move them out; issue #153 is that move, and this
//!   crate now names no assembly and no material family at all.
//!
//! No drawn field changes, so the SVG frames and the contact sheet are
//! byte-identical across the bump. `RUNTIME.md` revision 2, authorized by
//! `docs/decisions/0018`, is what makes the plan digests free to move while the
//! drawn artifacts are the thing held fixed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use nomos_core::CanonicalValue;
use nomos_core::canonical::keyed_array;
use nomos_core::id::SchemaId;

use crate::catalog::EntityCatalog;
use crate::error::{PlanError, PlanResult, codes};
use crate::facts::EffectiveFacts;
use crate::read;
use crate::runs::{self, ScenarioRun};
use crate::source::{self, PresentationSource};
use crate::world;

/// The rendering plan's schema identity.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn rendering_plan_schema() -> SchemaId {
    SchemaId::new("nomos.rendering_plan", 3).expect("the rendering-plan schema id is a literal")
}

/// The one objective kind the bounded profile declares.
///
/// The study authored `objective: {kind, target}` in every `area.json` and the
/// compiler forced `target == primaryGate == exit.gate`, so two of the three
/// carried no information (the audit's "Double authorities" item 5). The
/// objective is now derived here from the single authored `route.exit.gate`,
/// and its kind is this constant rather than a string content repeats.
const OBJECTIVE_KIND: &str = "exit_via";

/// The compiler's declared inputs.
///
/// Every one is a document or a directory of documents. There is deliberately
/// no field for `.nomos` source, Canonical World IR, or compiler receipts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Inputs<'a> {
    /// The `nomos.entity_catalog@1` document.
    pub catalog: &'a Path,
    /// A directory of `nomos.effective_facts@2` documents, one per scenario,
    /// named `<scenario>.json`.
    pub facts: &'a Path,
    /// The per-scenario run bundles.
    pub runs: &'a Path,
    /// The compiled world package, opened for four projection members only.
    pub world: &'a Path,
    /// The `nomos.presentation_source@2` document.
    pub source: &'a Path,
}

/// A compiled plan.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompiledPlan {
    /// Canonical bytes under `nomos.rendering_plan@3`.
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
    let kinds: BTreeMap<String, crate::catalog::EntityKind> = catalog
        .entities()
        .iter()
        .map(|entity| (entity.id.clone(), entity.kind))
        .collect();
    let presentation = source::read_source(inputs.source, &|entity| kinds.get(entity).copied())?;

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
                CanonicalValue::object_declared([
                    ("claim", CanonicalValue::text(claim.id.clone())),
                    ("source", claim.source.clone()),
                ])
            })
            .collect();
        entities.push(CanonicalValue::object_declared([
            ("anchor", entity.binding.clone()),
            ("id", CanonicalValue::text(entity.id.clone())),
            ("kind", CanonicalValue::text(entity.kind.as_str())),
            (
                "machine_namespaces",
                CanonicalValue::Array(
                    entity
                        .machine_namespaces
                        .iter()
                        .map(|namespace| CanonicalValue::text(namespace.clone()))
                        .collect(),
                ),
            ),
            ("provenance", CanonicalValue::Array(provenance)),
        ]));
    }

    let mut scenarios = Vec::new();
    for run in &scenario_runs {
        let fact = &facts[&run.id];
        let movement = stable(fact.movement.iter().map(|(entity, resolved)| {
            (
                entity.clone(),
                CanonicalValue::object_declared([
                    // A blocked subject's cost is spelled `null`: the kernel's
                    // Blocked variant carries no cost key, and the plan has
                    // always published one. Presentation, not semantics —
                    // `RUNTIME.md` section 5 R1-1 names this the only
                    // normalization in the comparison.
                    (
                        "cost",
                        resolved
                            .cost
                            .map_or(CanonicalValue::Null, CanonicalValue::Uint),
                    ),
                    (
                        "disposition",
                        CanonicalValue::text(resolved.disposition.clone()),
                    ),
                    ("entity", CanonicalValue::text(entity.clone())),
                    (
                        "reasons",
                        CanonicalValue::Array(
                            resolved
                                .reasons
                                .iter()
                                .map(|reason| CanonicalValue::text(reason.clone()))
                                .collect(),
                        ),
                    ),
                ]),
            )
        }))?;
        let effective_light = stable(fact.light.iter().map(|(entity, emitting)| {
            (
                entity.clone(),
                CanonicalValue::object_declared([
                    ("emitting", CanonicalValue::Bool(*emitting)),
                    ("entity", CanonicalValue::text(entity.clone())),
                ]),
            )
        }))?;
        let machine_states = stable(run.machine_states.iter().map(|(namespace, state)| {
            (
                namespace.clone(),
                CanonicalValue::object_declared([
                    ("namespace", CanonicalValue::text(namespace.clone())),
                    ("state", CanonicalValue::text(state.clone())),
                ]),
            )
        }))?;
        scenarios.push(CanonicalValue::object_declared([
            ("effective_light", effective_light),
            ("id", CanonicalValue::text(run.id.clone())),
            ("label", CanonicalValue::text(scenario_label(&run.id))),
            ("machine_states", machine_states),
            ("movement", movement),
            ("state_hash", CanonicalValue::text(fact.state_hash.clone())),
            ("tick", CanonicalValue::Uint(fact.tick)),
        ]));
    }

    let interactions = edges
        .iter()
        .map(|edge| {
            CanonicalValue::object_declared([
                ("action", CanonicalValue::text(edge.action.clone())),
                (
                    "from_scenario",
                    CanonicalValue::text(edge.from_scenario.clone()),
                ),
                ("id", CanonicalValue::text(edge.id.clone())),
                (
                    "input_state_hash",
                    CanonicalValue::text(edge.input_state_hash.clone()),
                ),
                (
                    "resulting_state_hash",
                    CanonicalValue::text(edge.resulting_state_hash.clone()),
                ),
                (
                    "target_entity",
                    CanonicalValue::text(edge.target_entity.clone()),
                ),
                (
                    "to_scenario",
                    CanonicalValue::text(edge.to_scenario.clone()),
                ),
            ])
        })
        .collect();

    let plan = assemble(
        &presentation,
        &projections,
        entities,
        scenarios,
        interactions,
    );
    let mut bytes = plan.to_canonical_bytes();
    bytes.push(b'\n');
    Ok(CompiledPlan {
        entity_count: catalog.entities().len(),
        scenario_count: scenario_runs.len(),
        interaction_count: edges.len(),
        bytes,
    })
}

/// Builds one stable-ID-ordered array.
///
/// `nomos_core::canonical::keyed_array` is `KERNEL.md` section 7's ordering
/// rule as a function: entity collections are arrays ordered by stable entity
/// ID, machine collections by canonical namespace ID, and a repeated ID is
/// refused rather than silently resolved.
fn stable(items: impl IntoIterator<Item = (String, CanonicalValue)>) -> PlanResult<CanonicalValue> {
    keyed_array(items).map_err(|diagnostic| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("plan collection is not stably keyed: {diagnostic}"),
        )
    })
}

fn cell(value: source::Cell) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("x", CanonicalValue::Int(value.x)),
        ("y", CanonicalValue::Int(value.y)),
        ("z", CanonicalValue::Int(value.z)),
    ])
}

fn corner(value: source::Corner) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("x", CanonicalValue::Int(value.x)),
        ("y", CanonicalValue::Int(value.y)),
    ])
}

fn architecture(value: &source::Architecture) -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "bounds",
            CanonicalValue::object_declared([
                ("height", CanonicalValue::Int(value.bounds.height)),
                ("width", CanonicalValue::Int(value.bounds.width)),
            ]),
        ),
        (
            "masses",
            CanonicalValue::Array(
                value
                    .masses
                    .iter()
                    .map(|mass| {
                        CanonicalValue::object_declared([
                            ("height_steps", CanonicalValue::Int(mass.height_steps)),
                            ("id", CanonicalValue::text(mass.id.clone())),
                            ("max", corner(mass.max)),
                            ("min", corner(mass.min)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "style",
            CanonicalValue::object_declared([
                (
                    "assembly",
                    CanonicalValue::text(value.style.assembly.clone()),
                ),
                (
                    "material_family",
                    CanonicalValue::text(value.style.material_family.clone()),
                ),
                (
                    "trim_family",
                    CanonicalValue::text(value.style.trim_family.clone()),
                ),
            ]),
        ),
        (
            "wall_height_steps",
            CanonicalValue::Int(value.wall_height_steps),
        ),
    ])
}

fn assemble(
    presentation: &PresentationSource,
    projections: &[world::ProjectionFacts],
    entities: Vec<CanonicalValue>,
    scenarios: Vec<CanonicalValue>,
    interactions: Vec<CanonicalValue>,
) -> CanonicalValue {
    // Published in the declared PROJECTION_FILES order, row for row with
    // `projection_schemas`, rather than sorted by file name: the two arrays
    // describe the same four members and must not be able to disagree about
    // which row is which.
    let projection_digests = CanonicalValue::Array(
        projections
            .iter()
            .map(|facts| {
                CanonicalValue::object_declared([
                    ("digest", CanonicalValue::text(facts.digest.clone())),
                    ("file", CanonicalValue::text(facts.file)),
                ])
            })
            .collect(),
    );

    let mut route = vec![(
        "to_area",
        presentation
            .route
            .exit
            .to_area
            .clone()
            .map_or(CanonicalValue::Null, CanonicalValue::text),
    )];
    if let Some(entry) = presentation.route.entry {
        route.push(("entry", cell(entry)));
    }

    CanonicalValue::object_declared([
        (
            "actors",
            CanonicalValue::Array(
                presentation
                    .actors
                    .iter()
                    .map(|actor| {
                        CanonicalValue::object_declared([
                            ("assembly", CanonicalValue::text(actor.assembly.clone())),
                            ("cell", cell(actor.cell)),
                            ("id", CanonicalValue::text(actor.id.clone())),
                            ("role", CanonicalValue::text(actor.role.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("architecture", architecture(&presentation.architecture)),
        (
            "area",
            CanonicalValue::object_declared([
                ("id", CanonicalValue::text(presentation.area.id.clone())),
                (
                    "label",
                    CanonicalValue::text(presentation.area.label.clone()),
                ),
                ("start", CanonicalValue::Bool(presentation.area.start)),
            ]),
        ),
        (
            "effects",
            CanonicalValue::Array(
                presentation
                    .effects
                    .iter()
                    .map(|effect| {
                        CanonicalValue::object_declared([
                            (
                                "anchor",
                                CanonicalValue::object_declared([
                                    ("entity", CanonicalValue::text(effect.anchor.entity.clone())),
                                    ("socket", CanonicalValue::text(effect.anchor.socket.clone())),
                                ]),
                            ),
                            ("assembly", CanonicalValue::text(effect.assembly.clone())),
                            ("id", CanonicalValue::text(effect.id.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("entities", CanonicalValue::Array(entities)),
        ("interactions", CanonicalValue::Array(interactions)),
        (
            "objective",
            CanonicalValue::object_declared([
                (
                    "gate",
                    CanonicalValue::text(presentation.route.exit.gate.clone()),
                ),
                ("kind", CanonicalValue::text(OBJECTIVE_KIND)),
            ]),
        ),
        ("projection_digests", projection_digests),
        (
            "projection_schemas",
            CanonicalValue::Array(
                projections
                    .iter()
                    .map(|facts| facts.schema.clone())
                    .collect(),
            ),
        ),
        (
            "pursuit",
            CanonicalValue::object_declared([(
                "light",
                CanonicalValue::text(presentation.pursuit.light.clone()),
            )]),
        ),
        ("route", CanonicalValue::object_declared(route)),
        ("scenarios", CanonicalValue::Array(scenarios)),
        (
            "schema",
            CanonicalValue::text(rendering_plan_schema().to_string()),
        ),
    ])
}

/// The scenario's display label.
///
/// `build-plan.mjs:133` derived it from the scenario directory name by
/// stripping a numeric prefix and replacing hyphens with spaces.
/// `docs/review/executable-gaol-ownership-audit.md`'s "Derived by convention"
/// item 14 records that as convention-derived. It stays derived, in Rust:
/// `docs/review/presentation-source.md` defers it to R1-5, which declares the
/// ordered scenario collection a run's authored label would attach to. A
/// scenario names a run, not an area, so `nomos.presentation_source@2` has no
/// place to put one.
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
