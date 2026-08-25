//! The `nomos-render-plan` command.
//!
//! ```text
//! nomos-render-plan --catalog <entity-catalog.json> --facts <dir> \
//!                   --runs <dir> --world <world/> --area <area.json> \
//!                   --out <plan.json>
//! ```
//!
//! Every argument is required and every one is a document or a directory of
//! documents. On success the plan is written to `--out` as canonical bytes
//! followed by one `LF`, and a canonical status document is written to stdout.
//! On failure nothing is written, a canonical rejection document goes to
//! stdout, and the exit status is 1 — the same stdout-only, fail-closed shape
//! the kernel's own commands use, so a pipeline sees a non-zero status rather
//! than a half-written plan.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use nomos_core::CanonicalValue;
use nomos_render_plan::error::{PlanError, PlanResult, codes};
use nomos_render_plan::plan::{self, Inputs};

const USAGE: &str = "usage: nomos-render-plan --catalog <entity-catalog.json> --facts <dir> \
                     --runs <dir> --world <world/> --area <area.json> --out <plan.json>";

fn main() -> ExitCode {
    match run() {
        Ok(document) => {
            println!("{}", render(&document));
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!("{}", render(&rejection(&error)));
            ExitCode::FAILURE
        }
    }
}

fn run() -> PlanResult<CanonicalValue> {
    let arguments = Arguments::parse(std::env::args().skip(1))?;
    let compiled = plan::compile(Inputs {
        catalog: &arguments.catalog,
        facts: &arguments.facts,
        runs: &arguments.runs,
        world: &arguments.world,
        area: &arguments.area,
    })?;
    write_atomically(&arguments.out, &compiled.bytes)?;
    Ok(CanonicalValue::object_declared([
        ("command", CanonicalValue::text("render-plan")),
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

fn rejection(error: &PlanError) -> CanonicalValue {
    let mut fields = vec![
        ("code", CanonicalValue::text(error.code().as_str())),
        ("command", CanonicalValue::text("render-plan")),
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

#[derive(Clone, Debug)]
struct Arguments {
    catalog: PathBuf,
    facts: PathBuf,
    runs: PathBuf,
    world: PathBuf,
    area: PathBuf,
    out: PathBuf,
}

impl Arguments {
    fn parse(arguments: impl Iterator<Item = String>) -> PlanResult<Self> {
        let mut catalog = None;
        let mut facts = None;
        let mut runs = None;
        let mut world = None;
        let mut area = None;
        let mut out = None;
        let mut arguments = arguments;
        while let Some(flag) = arguments.next() {
            let slot = match flag.as_str() {
                "--catalog" => &mut catalog,
                "--facts" => &mut facts,
                "--runs" => &mut runs,
                "--world" => &mut world,
                "--area" => &mut area,
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
            area: required(area, "--area")?,
            out: required(out, "--out")?,
        })
    }
}
