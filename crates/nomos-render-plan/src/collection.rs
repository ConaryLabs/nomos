//! The area collection: schema identity, route chain, and canonical bytes.
//!
//! This is the owner file for `nomos.area_collection@1`, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.
//!
//! # What it replaces
//!
//! `experiments/executable-gaol/src/build-collection.mjs`, deleted in the change
//! that lands this module. That file declared
//! `nomos.experiment.area_collection@2` (`build-collection.mjs:88`) and was the
//! only authority for the route graph: which area starts the run, which gate
//! leads where, and that the chain visits every declared area exactly once.
//! `docs/review/nomos-viewer.md` finding 2 recorded the hole — an accepted app
//! binding an identity whose only declaration was quarantined JavaScript — and
//! issue #152 is its repair.
//!
//! Under `RUNTIME.md` section 2 the study is a specification and a comparison
//! target, never a source of truth, so every check below names the study line it
//! reproduces. `docs/review/area-collection.md` records the differences.
//!
//! # What it reads
//!
//! Rendering plans, and nothing else. Each `--plans` input is a
//! [`crate::plan::rendering_plan_schema`] document, and that one constant is
//! also what the emitted `visual_grammar.rendering_plan_schema` publishes — so
//! when the plan's version moves, the collection follows it in one place rather
//! than in a second copy of the string.
//!
//! # What it emits
//!
//! One `CanonicalValue`, written through `nomos_core`'s encoder exactly as the
//! plan is. There is no encoder here and no floating-point value anywhere on the
//! path: every number it copies is an integer the plan already carried.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use nomos_core::CanonicalValue;
use nomos_core::canonical::keyed_array;
use nomos_core::hash::Sha256Digest;
use nomos_core::id::SchemaId;

use crate::error::{PlanError, PlanResult, codes};
use crate::plan::rendering_plan_schema;
use crate::read::{self, Shape};

/// The area collection's schema identity.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn area_collection_schema() -> SchemaId {
    SchemaId::new("nomos.area_collection", 1).expect("the area-collection schema id is a literal")
}

/// One rendering plan named on the command line.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlanInput {
    /// The plan document.
    pub path: PathBuf,
    /// The directory the plan was found in, when a `--plans` directory was
    /// scanned for it. `build-collection.mjs:44` requires that name to be the
    /// area's own identity; a plan named directly on the command line carries no
    /// such name to check.
    pub directory: Option<String>,
}

/// A compiled collection.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompiledCollection {
    /// Canonical bytes under `nomos.area_collection@1`.
    pub bytes: Vec<u8>,
    /// Area count, for the command's status line.
    pub area_count: usize,
    /// The start area's identity, for the command's status line.
    pub start_area: String,
    /// The shared visual grammar's digest, for the command's status line.
    pub grammar_digest: String,
}

/// A lattice cell, as the plan spells one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Cell {
    x: i64,
    y: i64,
    z: i64,
}

impl Cell {
    fn to_canonical(self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("x", CanonicalValue::Int(self.x)),
            ("y", CanonicalValue::Int(self.y)),
            ("z", CanonicalValue::Int(self.z)),
        ])
    }
}

/// One area, as the collection reads it out of that area's plan.
#[derive(Clone, PartialEq, Eq, Debug)]
struct Area {
    id: String,
    label: String,
    start: bool,
    gate: String,
    to_area: Option<String>,
    entry: Option<Cell>,
    digest: String,
    grammar: CanonicalValue,
    path: PathBuf,
}

/// Compiles an area collection from rendering plans alone.
///
/// # Errors
///
/// Returns the first `RP####` rejection the inputs produce. Nothing is written.
pub fn build(inputs: &[PlanInput]) -> PlanResult<CompiledCollection> {
    // `build-collection.mjs:28`: two areas are the least that can prove
    // anything, because the whole point of the document is what two independent
    // areas share and how one reaches the other.
    if inputs.len() < 2 {
        return Err(PlanError::new(
            codes::COLLECTION_ROUTE_INVALID,
            format!(
                "the consistency proof requires at least two areas; {} named",
                inputs.len()
            ),
        ));
    }

    let mut areas: Vec<Area> = Vec::with_capacity(inputs.len());
    for input in inputs {
        areas.push(read_area(input)?);
    }
    // `build-collection.mjs:26` sorted by directory name, which its line 44
    // forces to equal the area identity. The identity is what is actually
    // ordered, so it is what this orders by.
    areas.sort_by(|left, right| left.id.cmp(&right.id));

    // `build-collection.mjs:42` built its `byId` map with `new Map(...)`, which
    // resolves a repeated identity by silently keeping the last plan. A repeated
    // identity is refused here instead; `docs/review/area-collection.md`
    // records the strengthening.
    let mut by_id: BTreeMap<&str, &Area> = BTreeMap::new();
    for area in &areas {
        if let Some(previous) = by_id.insert(area.id.as_str(), area) {
            return Err(PlanError::new(
                codes::COLLECTION_ROUTE_INVALID,
                format!(
                    "area `{}` is declared twice: {} and {}",
                    area.id,
                    previous.path.display(),
                    area.path.display()
                ),
            )
            .at(&area.path));
        }
    }

    // `build-collection.mjs:40,45-47`: one visual grammar, byte-identical across
    // every area, or the build fails. The grammar of the first area in identity
    // order is the one every other area is compared against.
    let grammar = areas[0].grammar.clone();
    let grammar_bytes = grammar.to_canonical_bytes();
    for area in &areas {
        if area.grammar.to_canonical_bytes() != grammar_bytes {
            return Err(PlanError::new(
                codes::COLLECTION_GRAMMAR_DIVERGED,
                format!(
                    "area `{}` diverges from the shared visual grammar declared by `{}`",
                    area.id, areas[0].id
                ),
            )
            .at(&area.path));
        }
    }

    for area in &areas {
        // `build-collection.mjs:52-54`. Each area validates its own arrival cell
        // against its own bounds and its own masses, inside the compiler. What is
        // left for the collection is the one check no single area can make: that
        // the area a gate leads to exists and can actually receive an arrival.
        if area.start != area.entry.is_none() {
            return Err(PlanError::new(
                codes::COLLECTION_ROUTE_INVALID,
                format!(
                    "area `{}` must declare an arrival cell if and only if it is not the start area",
                    area.id
                ),
            )
            .at(&area.path));
        }
        if let Some(to_area) = &area.to_area {
            // `build-collection.mjs:56-57`.
            let Some(target) = by_id.get(to_area.as_str()) else {
                return Err(PlanError::new(
                    codes::COLLECTION_ROUTE_INVALID,
                    format!("area `{}` targets unknown area `{to_area}`", area.id),
                )
                .at(&area.path));
            };
            // `build-collection.mjs:58-60`.
            if target.entry.is_none() {
                return Err(PlanError::new(
                    codes::COLLECTION_ROUTE_INVALID,
                    format!(
                        "area `{}` leads to `{to_area}`, which declares no arrival cell",
                        area.id
                    ),
                )
                .at(&area.path));
            }
        }
    }

    // `build-collection.mjs:64-66`.
    let starts: Vec<&Area> = areas.iter().filter(|area| area.start).collect();
    if starts.len() != 1 {
        return Err(PlanError::new(
            codes::COLLECTION_ROUTE_INVALID,
            format!(
                "the area collection requires exactly one start area; {} declared{}",
                starts.len(),
                named(starts.iter().map(|area| area.id.as_str()))
            ),
        ));
    }
    let start_area = starts[0].id.clone();

    // Not in `build-collection.mjs`, which left this implied by its every-area
    // check at line 85: a second area with no destination could only ever be
    // unreachable, and would be reported as "does not visit every declared
    // area". The chain has to terminate somewhere, so the collection says where
    // rather than describing the symptom.
    let exits: Vec<&Area> = areas.iter().filter(|area| area.to_area.is_none()).collect();
    if exits.len() != 1 {
        return Err(PlanError::new(
            codes::COLLECTION_ROUTE_INVALID,
            format!(
                "the route chain must terminate at exactly one area declaring no destination; \
                 {} declare none{}",
                exits.len(),
                named(exits.iter().map(|area| area.id.as_str()))
            ),
        ));
    }

    // `build-collection.mjs:67-85`: one walk from the start area, refusing a
    // cycle, and visiting every declared area.
    let mut route = Vec::with_capacity(areas.len());
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    let mut current = start_area.as_str();
    loop {
        if !visited.insert(current) {
            return Err(PlanError::new(
                codes::COLLECTION_ROUTE_INVALID,
                format!("the area route cycles at `{current}`"),
            ));
        }
        let area = by_id[current];
        let entry = match &area.to_area {
            // `build-collection.mjs:79-81`: the arrival cell is the
            // destination's own declaration, read here so a consumer can follow
            // one edge without loading the next plan first.
            Some(to_area) => by_id[to_area.as_str()]
                .entry
                .map_or(CanonicalValue::Null, Cell::to_canonical),
            None => CanonicalValue::Null,
        };
        route.push(CanonicalValue::object_declared([
            ("entry", entry),
            ("from_area", CanonicalValue::text(area.id.clone())),
            ("gate", CanonicalValue::text(area.gate.clone())),
            (
                "to_area",
                area.to_area
                    .clone()
                    .map_or(CanonicalValue::Null, CanonicalValue::text),
            ),
        ]));
        match &area.to_area {
            Some(to_area) => current = to_area.as_str(),
            None => break,
        }
    }
    // `build-collection.mjs:85`.
    if visited.len() != areas.len() {
        return Err(PlanError::new(
            codes::COLLECTION_ROUTE_INVALID,
            format!(
                "the area route visits {} of {} declared areas; unvisited{}",
                visited.len(),
                areas.len(),
                named(
                    areas
                        .iter()
                        .map(|area| area.id.as_str())
                        .filter(|id| !visited.contains(id))
                )
            ),
        ));
    }

    let digest = Sha256Digest::of_bytes(&grammar_bytes).to_hex();
    let document = assemble(&areas, &start_area, grammar, &digest, route)?;
    let mut bytes = document.to_canonical_bytes();
    bytes.push(b'\n');
    Ok(CompiledCollection {
        bytes,
        area_count: areas.len(),
        start_area,
        grammar_digest: digest,
    })
}

/// ` (a, b)`, for a diagnostic that names the areas it counted.
fn named<'a>(ids: impl Iterator<Item = &'a str>) -> String {
    let ids: Vec<&str> = ids.collect();
    if ids.is_empty() {
        return String::new();
    }
    format!(" ({})", ids.join(", "))
}

fn assemble(
    areas: &[Area],
    start_area: &str,
    grammar: CanonicalValue,
    digest: &str,
    route: Vec<CanonicalValue>,
) -> PlanResult<CanonicalValue> {
    // `build-collection.mjs:89-93` spelled the same thing as
    // `{digest, ...grammar}`: the digest is over the grammar the areas agreed
    // on, and it is published inside the object it summarises.
    let CanonicalValue::Object(mut visual_grammar) = grammar else {
        return Err(PlanError::new(
            codes::DOCUMENT_SHAPE,
            "the visual grammar is not an object",
        ));
    };
    visual_grammar.insert(
        nomos_core::FieldName::declared("digest"),
        CanonicalValue::text(digest),
    );

    let rows = areas.iter().map(|area| {
        (
            area.id.clone(),
            CanonicalValue::object_declared([
                (
                    "entry",
                    area.entry.map_or(CanonicalValue::Null, Cell::to_canonical),
                ),
                (
                    "exit",
                    CanonicalValue::object_declared([
                        ("gate", CanonicalValue::text(area.gate.clone())),
                        (
                            "to_area",
                            area.to_area
                                .clone()
                                .map_or(CanonicalValue::Null, CanonicalValue::text),
                        ),
                    ]),
                ),
                ("id", CanonicalValue::text(area.id.clone())),
                ("label", CanonicalValue::text(area.label.clone())),
                (
                    "plan",
                    CanonicalValue::object_declared([
                        // The published file name, derived from the identity
                        // rather than from where the plan was read: the
                        // collection names what a consumer will fetch, and the
                        // input path is the compiler's business alone.
                        ("file", CanonicalValue::text(format!("{}.json", area.id))),
                        ("sha256", CanonicalValue::text(area.digest.clone())),
                    ]),
                ),
                ("start", CanonicalValue::Bool(area.start)),
            ]),
        )
    });

    Ok(CanonicalValue::object_declared([
        ("areas", stable(rows)?),
        ("route", CanonicalValue::Array(route)),
        (
            "schema",
            CanonicalValue::text(area_collection_schema().to_string()),
        ),
        ("start_area", CanonicalValue::text(start_area)),
        ("visual_grammar", CanonicalValue::Object(visual_grammar)),
    ]))
}

/// Builds one stable-ID-ordered array, as `plan.rs` does.
fn stable(items: impl IntoIterator<Item = (String, CanonicalValue)>) -> PlanResult<CanonicalValue> {
    keyed_array(items).map_err(|diagnostic| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("collection is not stably keyed: {diagnostic}"),
        )
    })
}

/// Reads one rendering plan into the facts the collection publishes about it.
fn read_area(input: &PlanInput) -> PlanResult<Area> {
    let path = input.path.as_path();
    let bytes = std::fs::read(path)
        .map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(path))?;
    let document = read::parse_document(&bytes, path)?;
    // One constant, read from the plan's own owner file: the collection accepts
    // exactly the plan identity `nomos-render-plan` emits, so a version move
    // carries both ends at once.
    let expected = rendering_plan_schema();
    read::bind_schema(&document, &expected, path)?;

    let area = required(&document, "area", "area", path)?;
    let id = required_text(area, "id", "area.id", path)?.to_owned();
    // `build-collection.mjs:44`.
    if let Some(directory) = &input.directory
        && directory != &id
    {
        return Err(PlanError::new(
            codes::COLLECTION_ROUTE_INVALID,
            format!("directory `{directory}` does not match plan area identity `{id}`"),
        )
        .at(path));
    }
    let label = required_text(area, "label", "area.label", path)?.to_owned();
    let start = required_bool(area, "start", "area.start", path)?;

    let objective = required(&document, "objective", "objective", path)?;
    let gate = required_text(objective, "gate", "objective.gate", path)?.to_owned();

    let route = required(&document, "route", "route", path)?;
    let to_area = match required(route, "to_area", "route.to_area", path)? {
        CanonicalValue::Null => None,
        value => Some(
            value
                .as_text()
                .ok_or_else(|| {
                    PlanError::new(
                        codes::DOCUMENT_SHAPE,
                        "field `route.to_area` is neither a string nor null",
                    )
                    .at(path)
                })?
                .to_owned(),
        ),
    };
    let entry = match route.get("entry") {
        Some(value) => Some(cell(value, "route.entry", path)?),
        None => None,
    };

    Ok(Area {
        id,
        label,
        start,
        gate,
        to_area,
        entry,
        digest: Sha256Digest::of_bytes(&bytes).to_hex(),
        grammar: visual_grammar(&document, path, &expected)?,
        path: path.to_path_buf(),
    })
}

/// The visual grammar every area is required to share.
///
/// `build-collection.mjs:31-38`, field for field:
///
/// | This | The study |
/// | --- | --- |
/// | `rendering_plan_schema` | `:32`, `plan.schema` — read here from the one constant the identity was bound against |
/// | `projection_schemas` | `:33`, copied verbatim |
/// | `architecture_style` | `:34`, `plan.architecture.style`, copied verbatim |
/// | `entity_assemblies` | `:35`, the unique `(kind, visual_assembly, material_family)` rows |
/// | `actor_assemblies` | `:36`, the unique `actors[].assembly` values |
/// | `effect_assemblies` | `:37`, the unique `effects[].assembly` values |
///
/// The study's `uniqueRows` (`:30`) deduplicated by JSON text and sorted; here
/// the deduplication and the ordering are both a `BTreeSet`, and the entity row
/// is a declared-field object rather than a three-element array, because a
/// positional triple has no field names and `CanonicalValue` gives no reason to
/// keep one. `docs/review/area-collection.md` records it.
fn visual_grammar(
    document: &CanonicalValue,
    path: &Path,
    expected: &SchemaId,
) -> PlanResult<CanonicalValue> {
    let architecture = required(document, "architecture", "architecture", path)?;
    let style = required(architecture, "style", "architecture.style", path)?.clone();
    let projection_schemas =
        required(document, "projection_schemas", "projection_schemas", path)?.clone();

    let mut entities = BTreeSet::new();
    for entity in required_array(document, "entities", "entities", path)? {
        entities.insert((
            required_text(entity, "kind", "entities[].kind", path)?.to_owned(),
            required_text(
                entity,
                "visual_assembly",
                "entities[].visual_assembly",
                path,
            )?
            .to_owned(),
            required_text(
                entity,
                "material_family",
                "entities[].material_family",
                path,
            )?
            .to_owned(),
        ));
    }
    let entity_assemblies = entities
        .into_iter()
        .map(|(kind, visual_assembly, material_family)| {
            CanonicalValue::object_declared([
                ("kind", CanonicalValue::text(kind)),
                ("material_family", CanonicalValue::text(material_family)),
                ("visual_assembly", CanonicalValue::text(visual_assembly)),
            ])
        })
        .collect();

    Ok(CanonicalValue::object_declared([
        ("actor_assemblies", assemblies(document, "actors", path)?),
        ("architecture_style", style),
        ("effect_assemblies", assemblies(document, "effects", path)?),
        (
            "entity_assemblies",
            CanonicalValue::Array(entity_assemblies),
        ),
        ("projection_schemas", projection_schemas),
        (
            "rendering_plan_schema",
            CanonicalValue::text(expected.to_string()),
        ),
    ]))
}

/// The unique `assembly` values one plan collection declares.
fn assemblies(document: &CanonicalValue, field: &str, path: &Path) -> PlanResult<CanonicalValue> {
    let mut names = BTreeSet::new();
    for row in required_array(document, field, field, path)? {
        names.insert(
            required_text(row, "assembly", &format!("{field}[].assembly"), path)?.to_owned(),
        );
    }
    Ok(CanonicalValue::Array(
        names.into_iter().map(CanonicalValue::text).collect(),
    ))
}

// The plan is nested, and `read`'s accessors name a field by the same string
// they look it up with, so a nested field would be reported as `id` rather than
// as `area.id`. These four take the leaf to look up and the dotted path to
// report, which is what a reader of the diagnostic needs.

fn required<'a>(
    value: &'a CanonicalValue,
    leaf: &str,
    at: &str,
    path: &Path,
) -> PlanResult<&'a CanonicalValue> {
    value.get(leaf).ok_or_else(|| {
        PlanError::new(codes::DOCUMENT_SHAPE, format!("field `{at}` is absent")).at(path)
    })
}

fn required_text<'a>(
    value: &'a CanonicalValue,
    leaf: &str,
    at: &str,
    path: &Path,
) -> PlanResult<&'a str> {
    required(value, leaf, at, path)?.as_text().ok_or_else(|| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("field `{at}` is not a string"),
        )
        .at(path)
    })
}

fn required_bool(value: &CanonicalValue, leaf: &str, at: &str, path: &Path) -> PlanResult<bool> {
    required(value, leaf, at, path)?.as_bool().ok_or_else(|| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("field `{at}` is not a boolean"),
        )
        .at(path)
    })
}

fn required_array<'a>(
    value: &'a CanonicalValue,
    leaf: &str,
    at: &str,
    path: &Path,
) -> PlanResult<&'a [CanonicalValue]> {
    required(value, leaf, at, path)?.as_array().ok_or_else(|| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("field `{at}` is not an array"),
        )
        .at(path)
    })
}

fn cell(value: &CanonicalValue, at: &str, path: &Path) -> PlanResult<Cell> {
    let axis = |name: &str| -> PlanResult<i64> {
        let found = value.get(name).ok_or_else(|| {
            PlanError::new(
                codes::DOCUMENT_SHAPE,
                format!("field `{at}.{name}` is absent"),
            )
            .at(path)
        })?;
        match found {
            CanonicalValue::Int(value) => Ok(*value),
            CanonicalValue::Uint(value) => i64::try_from(*value).map_err(|_| {
                PlanError::new(
                    codes::DOCUMENT_SHAPE,
                    format!("field `{at}.{name}` does not fit a signed 64-bit integer"),
                )
                .at(path)
            }),
            _ => Err(PlanError::new(
                codes::DOCUMENT_SHAPE,
                format!("field `{at}.{name}` is not an integer"),
            )
            .at(path)),
        }
    };
    Ok(Cell {
        x: axis("x")?,
        y: axis("y")?,
        z: axis("z")?,
    })
}

/// Expands one `--plans` value into the plan documents it names.
///
/// A directory is the study's published layout — one subdirectory per area, each
/// holding `rendering-plan.json` (`build-collection.mjs:20-26`) — and a file is
/// that one plan.
///
/// # Errors
///
/// Returns `RP0101` when the path cannot be read or a scanned subdirectory holds
/// no `rendering-plan.json`.
pub fn expand(value: &Path) -> PlanResult<Vec<PlanInput>> {
    if value.is_file() {
        return Ok(vec![PlanInput {
            path: value.to_path_buf(),
            directory: None,
        }]);
    }
    let entries = std::fs::read_dir(value)
        .map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(value))?;
    let mut found = BTreeMap::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()))?;
        if !entry.path().is_dir() {
            continue;
        }
        let directory = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path().join("rendering-plan.json");
        if !path.is_file() {
            return Err(PlanError::new(
                codes::INPUT_UNREADABLE,
                format!("area directory `{directory}` holds no `rendering-plan.json`"),
            )
            .at(&path));
        }
        found.insert(
            directory.clone(),
            PlanInput {
                path,
                directory: Some(directory),
            },
        );
    }
    if found.is_empty() {
        return Err(PlanError::new(
            codes::INPUT_UNREADABLE,
            "no area directory holding a `rendering-plan.json` was found",
        )
        .at(value));
    }
    Ok(found.into_values().collect())
}
