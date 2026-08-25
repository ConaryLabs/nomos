//! The `nomos-render-plan` command, in its two modes.
//!
//! ```text
//! nomos-render-plan --catalog <entity-catalog.json> --facts <dir> \
//!                   --runs <dir> --world <world/> \
//!                   --source <presentation.json> --out <plan.json>
//!
//! nomos-render-plan collection --plans <dir-or-plan> [--plans …] \
//!                              --out <areas.json>
//! ```
//!
//! The first word selects the mode: `collection` compiles the area collection
//! from plans already compiled, and anything else is the plan compiler's own
//! flags, unchanged. There is no `plan` subcommand, so the invocation R1-2
//! landed keeps working exactly as written.
//!
//! Every argument is required and every one is a document or a directory of
//! documents. On success the output is written to `--out` as canonical bytes
//! followed by one `LF`, and a canonical status document is written to stdout.
//! On failure nothing is written, a canonical rejection document goes to
//! stdout, and the exit status is 1 — the same stdout-only, fail-closed shape
//! the kernel's own commands use, so a pipeline sees a non-zero status rather
//! than a half-written artifact.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use nomos_core::CanonicalValue;
use nomos_render_plan::collection::{self, PlanInput};
use nomos_render_plan::error::{PlanError, PlanResult, codes};
use nomos_render_plan::plan::{self, Inputs};

const USAGE: &str = "usage: nomos-render-plan --catalog <entity-catalog.json> --facts <dir> \
                     --runs <dir> --world <world/> --source <presentation.json> \
                     --out <plan.json>";

const COLLECTION_USAGE: &str =
    "usage: nomos-render-plan collection --plans <dir-or-plan> [--plans …] --out <areas.json>";

/// The `command` field each mode's stdout document carries.
const PLAN_COMMAND: &str = "render-plan";
const COLLECTION_COMMAND: &str = "area-collection";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let collection_mode = arguments.first().map(String::as_str) == Some("collection");
    let command = if collection_mode {
        COLLECTION_COMMAND
    } else {
        PLAN_COMMAND
    };
    let outcome = if collection_mode {
        run_collection(&arguments[1..])
    } else {
        run(arguments.into_iter())
    };
    match outcome {
        Ok(document) => {
            println!("{}", render(&document));
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!("{}", render(&rejection(&error, command)));
            ExitCode::FAILURE
        }
    }
}

fn run_collection(arguments: &[String]) -> PlanResult<CanonicalValue> {
    let arguments = CollectionArguments::parse(arguments)?;
    let mut inputs: Vec<PlanInput> = Vec::new();
    for value in &arguments.plans {
        inputs.extend(collection::expand(value)?);
    }
    let compiled = collection::build(&inputs)?;
    write_atomically(&arguments.out, &compiled.bytes)?;
    Ok(CanonicalValue::object_declared([
        (
            "area_count",
            CanonicalValue::Uint(compiled.area_count as u64),
        ),
        ("command", CanonicalValue::text(COLLECTION_COMMAND)),
        (
            "grammar_digest",
            CanonicalValue::text(compiled.grammar_digest),
        ),
        (
            "output",
            CanonicalValue::text(arguments.out.to_string_lossy()),
        ),
        (
            "schema",
            collection::area_collection_schema().to_canonical(),
        ),
        ("start_area", CanonicalValue::text(compiled.start_area)),
        ("status", CanonicalValue::text("completed")),
    ]))
}

fn run(arguments: impl Iterator<Item = String>) -> PlanResult<CanonicalValue> {
    let arguments = Arguments::parse(arguments)?;
    let compiled = plan::compile(Inputs {
        catalog: &arguments.catalog,
        facts: &arguments.facts,
        runs: &arguments.runs,
        world: &arguments.world,
        source: &arguments.source,
    })?;
    write_atomically(&arguments.out, &compiled.bytes)?;
    Ok(CanonicalValue::object_declared([
        ("command", CanonicalValue::text(PLAN_COMMAND)),
        (
            "entity_count",
            CanonicalValue::Uint(compiled.entity_count as u64),
        ),
        (
            "interaction_count",
            CanonicalValue::Uint(compiled.interaction_count as u64),
        ),
        (
            "output",
            CanonicalValue::text(arguments.out.to_string_lossy()),
        ),
        (
            "scenario_count",
            CanonicalValue::Uint(compiled.scenario_count as u64),
        ),
        ("schema", plan::rendering_plan_schema().to_canonical()),
        ("status", CanonicalValue::text("completed")),
    ]))
}

fn rejection(error: &PlanError, command: &'static str) -> CanonicalValue {
    let mut fields = vec![
        ("code", CanonicalValue::text(error.code().as_str())),
        ("command", CanonicalValue::text(command)),
        ("message", CanonicalValue::text(error.message())),
        ("status", CanonicalValue::text("rejected")),
    ];
    if let Some(path) = error.path() {
        fields.push(("input", CanonicalValue::text(path.to_string_lossy())));
    }
    CanonicalValue::object_declared(fields)
}

fn render(document: &CanonicalValue) -> String {
    String::from_utf8(document.to_canonical_bytes())
        .expect("canonical bytes are valid UTF-8 by construction")
}

/// Writes the plan through a temporary sibling, so a failed write cannot leave
/// a truncated plan where the pipeline expects a complete one.
fn write_atomically(path: &Path, bytes: &[u8]) -> PlanResult<()> {
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(".partial");
    let temporary = PathBuf::from(temporary);
    std::fs::write(&temporary, bytes).map_err(|error| {
        PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(&temporary)
    })?;
    std::fs::rename(&temporary, path)
        .map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(path))
}

/// The collection mode's command line.
///
/// `--plans` is the one repeatable flag in this binary: an area corpus is a set,
/// and the caller may name it as one directory of area directories, as several
/// plan files, or as any mixture of the two.
#[derive(Clone, Debug)]
struct CollectionArguments {
    plans: Vec<PathBuf>,
    out: PathBuf,
}

impl CollectionArguments {
    fn parse(arguments: &[String]) -> PlanResult<Self> {
        let mut plans = Vec::new();
        let mut out = None;
        let mut arguments = arguments.iter();
        while let Some(flag) = arguments.next() {
            let value = arguments.next().ok_or_else(|| {
                PlanError::new(
                    codes::USAGE,
                    format!("`{flag}` needs a value; {COLLECTION_USAGE}"),
                )
            })?;
            match flag.as_str() {
                "--plans" => plans.push(PathBuf::from(value)),
                "--out" => {
                    if out.replace(PathBuf::from(value)).is_some() {
                        return Err(PlanError::new(
                            codes::USAGE,
                            format!("`--out` is given more than once; {COLLECTION_USAGE}"),
                        ));
                    }
                }
                other => {
                    return Err(PlanError::new(
                        codes::USAGE,
                        format!("unexpected argument `{other}`; {COLLECTION_USAGE}"),
                    ));
                }
            }
        }
        if plans.is_empty() {
            return Err(PlanError::new(
                codes::USAGE,
                format!("`--plans` is required; {COLLECTION_USAGE}"),
            ));
        }
        Ok(Self {
            plans,
            out: out.ok_or_else(|| {
                PlanError::new(
                    codes::USAGE,
                    format!("`--out` is required; {COLLECTION_USAGE}"),
                )
            })?,
        })
    }
}

#[derive(Clone, Debug)]
struct Arguments {
    catalog: PathBuf,
    facts: PathBuf,
    runs: PathBuf,
    world: PathBuf,
    source: PathBuf,
    out: PathBuf,
}

impl Arguments {
    fn parse(arguments: impl Iterator<Item = String>) -> PlanResult<Self> {
        let mut catalog = None;
        let mut facts = None;
        let mut runs = None;
        let mut world = None;
        let mut source = None;
        let mut out = None;
        let mut arguments = arguments;
        while let Some(flag) = arguments.next() {
            let slot = match flag.as_str() {
                "--catalog" => &mut catalog,
                "--facts" => &mut facts,
                "--runs" => &mut runs,
                "--world" => &mut world,
                "--source" => &mut source,
                "--out" => &mut out,
                other => {
                    return Err(PlanError::new(
                        codes::USAGE,
                        format!("unexpected argument `{other}`; {USAGE}"),
                    ));
                }
            };
            let value = arguments.next().ok_or_else(|| {
                PlanError::new(codes::USAGE, format!("`{flag}` needs a value; {USAGE}"))
            })?;
            if slot.replace(PathBuf::from(value)).is_some() {
                return Err(PlanError::new(
                    codes::USAGE,
                    format!("`{flag}` is given more than once; {USAGE}"),
                ));
            }
        }
        let required = |slot: Option<PathBuf>, flag: &str| {
            slot.ok_or_else(|| {
                PlanError::new(codes::USAGE, format!("`{flag}` is required; {USAGE}"))
            })
        };
        Ok(Self {
            catalog: required(catalog, "--catalog")?,
            facts: required(facts, "--facts")?,
            runs: required(runs, "--runs")?,
            world: required(world, "--world")?,
            source: required(source, "--source")?,
            out: required(out, "--out")?,
        })
    }
}
