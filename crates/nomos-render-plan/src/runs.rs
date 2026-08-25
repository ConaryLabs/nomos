//! Run bundles: machine states, declared status, and interaction edges.
//!
//! The compiler reads exactly three members of each per-scenario run bundle and
//! opens no other file in it:
//!
//! - `final-state.json` — the committed machine states the plan exposes as
//!   `scenarios[].machineStates`;
//! - `result.json` — the declared status, so a scenario that did not reach its
//!   declared state fails the compile rather than shipping a half-run;
//! - `command-log.json` — the rows the interaction-edge derivation walks.
//!
//! The scenario's `tick` and `stateHash` are **not** read from
//! `final-state.json`. `RUNTIME.md` section 5 R1-1's evidence names
//! `build-plan.mjs:134-135` as re-sourced from the effective-fact document's
//! own `tick` and `state_hash`, which binds the plan's runtime identity to the
//! same document the dispositions came from instead of to a second file that
//! could disagree. [`crate::plan`] does that.
//!
//! # Interaction edges
//!
//! Semantics are unchanged from `experiments/executable-gaol/src/build-plan.mjs:144-164`,
//! in Rust: an edge exists from scenario A to scenario B when B's committed
//! command log is exactly A's plus one row, every shared row agrees on request
//! and resulting state hash, and the extra row's input state hash is A's final
//! state hash. The pair loop keeps A-major, B-minor order so the emitted array
//! order matches. `docs/review/executable-gaol-ownership-audit.md` section 3
//! item 15 records this derivation as convention-derived; R1-2 reproduces it
//! rather than redesigning it, and R1-5 owns the declared successor pointer
//! that would replace it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use nomos_core::CanonicalValue;

use crate::error::{PlanError, PlanResult, codes};
use crate::read;

/// One scenario's run bundle, decoded.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ScenarioRun {
    /// The scenario directory name, which is the scenario id.
    pub id: String,
    /// Machine state by namespace at the committed final state.
    pub machine_states: BTreeMap<String, String>,
    /// The committed command-log rows in ordinal order.
    pub rows: Vec<CommandRow>,
}

/// One committed command-log row, reduced to the fields the derivation uses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommandRow {
    /// The request object, compared structurally between scenarios.
    pub request: CanonicalValue,
    /// The requested action.
    pub action: String,
    /// The requested entity.
    pub entity: String,
    /// The state hash the row was applied to.
    pub input_state_hash: String,
    /// The state hash the row produced.
    pub resulting_state_hash: String,
}

/// One derived interaction edge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InteractionEdge {
    /// `<from>:<action>:<entity>`, as `build-plan.mjs:154` spelled it.
    pub id: String,
    /// The scenario the edge leaves.
    pub from_scenario: String,
    /// The scenario the edge reaches.
    pub to_scenario: String,
    /// The entity the command targets.
    pub target_entity: String,
    /// The command action.
    pub action: String,
    /// The state hash the command was applied to.
    pub input_state_hash: String,
    /// The state hash the command produced.
    pub resulting_state_hash: String,
}

/// Reads every scenario run bundle under `runs`, in sorted scenario order.
///
/// # Errors
///
/// Returns `RP0101` when the directory cannot be listed, `RP0102`/`RP0105` for
/// a bundle member that is not canonical or is mis-shaped, and `RP0203` when a
/// scenario did not reach its declared state.
pub fn read_runs(runs: &Path) -> PlanResult<Vec<ScenarioRun>> {
    let mut names = Vec::new();
    let entries = std::fs::read_dir(runs)
        .map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(runs))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()))?;
        let is_dir = entry
            .file_type()
            .map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()))?
            .is_dir();
        if !is_dir {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        names.push(name);
    }
    names.sort();

    names
        .into_iter()
        .map(|name| {
            let dir = runs.join(&name);
            read_run(&name, &dir)
        })
        .collect()
}

fn read_run(id: &str, dir: &Path) -> PlanResult<ScenarioRun> {
    let result_path: PathBuf = dir.join("result.json");
    let result = read::read_document(&result_path)?;
    let status = read::required_text(&result, "status", &result_path)?;
    let committed = read::required_uint(&result, "committed_command_count", &result_path)?;
    // build-plan.mjs:106 accepted a non-completed scenario only when its
    // directory was literally named `01-baseline`. The condition that carries
    // the meaning is the one beside it: a declared rejection commits nothing.
    // The literal name is dropped — `docs/review/executable-gaol-ownership-audit.md`
    // section 3 item 11 — and the corpus behaves identically, because
    // `01-baseline` is the only rejected scenario in it and it commits zero
    // commands.
    let declared_rejection = status == "rejected" && committed == 0;
    if status != "completed" && !declared_rejection {
        return Err(PlanError::new(
            codes::SCENARIO_INCOMPLETE,
            format!(
                "scenario `{id}` did not reach its declared state: status `{status}` with \
                 {committed} committed commands"
            ),
        )
        .at(&result_path));
    }

    let state_path = dir.join("final-state.json");
    let final_state = read::read_document(&state_path)?;
    let state = read::required(&final_state, "state", &state_path)?;
    let mut machine_states = BTreeMap::new();
    for machine in read::required_array(state, "machines", &state_path)? {
        let namespace = read::required_text(machine, "namespace", &state_path)?.to_owned();
        let value = read::required_text(machine, "state", &state_path)?.to_owned();
        if machine_states.insert(namespace.clone(), value).is_some() {
            return Err(PlanError::new(
                codes::DOCUMENT_SHAPE,
                format!("machine `{namespace}` occurs more than once"),
            )
            .at(&state_path));
        }
    }

    let log_path = dir.join("command-log.json");
    let log = read::read_document(&log_path)?;
    let mut rows = Vec::new();
    for row in read::required_array(&log, "rows", &log_path)? {
        let request = read::required(row, "request", &log_path)?.clone();
        rows.push(CommandRow {
            action: read::required_text(&request, "action", &log_path)?.to_owned(),
            entity: read::required_text(&request, "entity", &log_path)?.to_owned(),
            input_state_hash: read::required_text(row, "input_state_hash", &log_path)?.to_owned(),
            resulting_state_hash: read::required_text(row, "resulting_state_hash", &log_path)?
                .to_owned(),
            request,
        });
    }

    Ok(ScenarioRun {
        id: id.to_owned(),
        machine_states,
        rows,
    })
}

/// Derives the interaction edges between consecutive scenarios.
///
/// `state_hash_of` supplies each scenario's final state hash, which comes from
/// its effective-fact document rather than from the run bundle.
#[must_use]
pub fn interaction_edges(
    runs: &[ScenarioRun],
    state_hash_of: &BTreeMap<String, String>,
) -> Vec<InteractionEdge> {
    let mut edges = Vec::new();
    for from in runs {
        for to in runs {
            if to.rows.len() != from.rows.len() + 1 {
                continue;
            }
            let prefix_matches = from.rows.iter().zip(&to.rows).all(|(left, right)| {
                left.request == right.request
                    && left.resulting_state_hash == right.resulting_state_hash
            });
            if !prefix_matches {
                continue;
            }
            let Some(next) = to.rows.last() else {
                continue;
            };
            if state_hash_of.get(&from.id) != Some(&next.input_state_hash) {
                continue;
            }
            edges.push(InteractionEdge {
                id: format!("{}:{}:{}", from.id, next.action, next.entity),
                from_scenario: from.id.clone(),
                to_scenario: to.id.clone(),
                target_entity: next.entity.clone(),
                action: next.action.clone(),
                input_state_hash: next.input_state_hash.clone(),
                resulting_state_hash: next.resulting_state_hash.clone(),
            });
        }
    }
    edges
}
