//! The presentation source, decoded strictly.
//!
//! `area.json` is unversioned, untyped, camelCase, and carries raw decimal
//! transforms. `RUNTIME.md` section 5 R1-3 replaces it with a typed, versioned
//! source with exactly one owner per field; this slice does not redesign it. It
//! performs the same checks `experiments/executable-gaol/src/build-plan.mjs:47-84`
//! performs today, in Rust, and copies the rest through verbatim.
//!
//! `docs/review/executable-gaol-ownership-audit.md` section 3 records three of
//! those checks as convention rather than schema — item 7 (the `{kind, target}`
//! key set, the literal `exit_via`, the literal actor ids `player` and
//! `gaoler`), item 8 (the bounded 9x6 lattice and the `0 < wallHeight <= 5`
//! bound), item 9 (the `0 < height <= 4` mass bound). All three are reproduced
//! here unchanged and none is removed by this slice; they are R1-3's to turn
//! into a schema. What does change is that they are now enforced by a compiler
//! with a stable diagnostic code rather than by a thrown `Error` string.
//!
//! Two tightenings over the JavaScript, both fail-closed and both satisfied by
//! all four committed areas: `area.start` must be a declared boolean rather
//! than any truthy value, and every presentation number must fit
//! [`crate::decimal::Decimal`]'s exact profile rather than JavaScript's binary
//! double.

use std::collections::BTreeSet;
use std::path::Path;

use crate::decimal::Decimal;
use crate::error::{PlanError, PlanResult, codes};
use crate::json::{self, Json};

/// The decoded presentation source.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AreaSource {
    /// Stable area identity.
    pub id: String,
    /// The authored display label, the only authored prose in the model.
    pub label: String,
    /// Whether this is the route's start area.
    pub start: bool,
    /// The primary gate entity id.
    pub primary_gate: String,
    /// The pursuit light entity id.
    pub pursuit_light: String,
    /// The scenario the forensic overlay renders.
    pub forensic_scenario: String,
    /// The `{kind, target}` objective, verbatim.
    pub objective: Json,
    /// The declared exit, verbatim.
    pub exit: Json,
    /// The bounded architecture block, verbatim.
    pub architecture: Json,
    /// The actor presentation anchors, verbatim.
    pub actors: Json,
    /// The effect presentation anchors, verbatim.
    pub effects: Json,
}

/// Reads and validates the presentation source against the compiled entities.
///
/// `entity_kind` resolves a compiled entity id to its classified kind, and is
/// the only way this module learns what a door is: nothing here inspects an
/// entity id, a machine namespace, or an assembly string.
///
/// # Errors
///
/// Returns `RP0101` when the file cannot be read, `RP0103`/`RP0205` when it is
/// not well-formed JSON in the accepted number profile, and `RP0202` for every
/// bounded-area invariant it breaks.
pub fn read_area(
    path: &Path,
    entity_kind: &dyn Fn(&str) -> Option<&'static str>,
) -> PlanResult<AreaSource> {
    let bytes = std::fs::read(path)
        .map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(path))?;
    let area = json::parse(&bytes).map_err(|error| error.at(path))?;
    let invalid = |message: String| PlanError::new(codes::AREA_INVALID, message).at(path);

    let id = text(&area, "id", path)?;
    let label = text(&area, "label", path)?;
    if id.is_empty() || label.is_empty() {
        return Err(invalid("area identity is required".to_owned()));
    }
    let start = area
        .get("start")
        .and_then(Json::as_bool)
        .ok_or_else(|| invalid("area must declare `start` as a boolean".to_owned()))?;

    let primary_gate = text(&area, "primaryGate", path)?;
    let compiled = |entity: &str| entity_kind(entity).is_some();
    if !compiled(&primary_gate) {
        return Err(invalid(format!(
            "primary gate {primary_gate} is not a compiled entity"
        )));
    }

    let objective = area
        .get("objective")
        .cloned()
        .unwrap_or(Json::Object(Default::default()));
    let objective_keys: BTreeSet<&str> = objective
        .as_object()
        .map(|fields| fields.keys().map(String::as_str).collect())
        .unwrap_or_default();
    if objective_keys != BTreeSet::from(["kind", "target"]) {
        return Err(invalid(
            "area objective must contain exactly kind and target".to_owned(),
        ));
    }
    if objective.get("kind").and_then(Json::as_text) != Some("exit_via") {
        return Err(invalid(
            "area objective must use the bounded exit_via kind".to_owned(),
        ));
    }
    let target = objective
        .get("target")
        .and_then(Json::as_text)
        .unwrap_or_default()
        .to_owned();
    if !compiled(&target) {
        return Err(invalid(format!(
            "objective target {target} is not a compiled entity"
        )));
    }
    if entity_kind(&target) != Some("door") {
        return Err(invalid(
            "exit_via objective must target a compiled door".to_owned(),
        ));
    }
    if target != primary_gate {
        return Err(invalid(
            "exit_via objective must target the primary gate".to_owned(),
        ));
    }

    let pursuit_light = text(&area, "pursuitLight", path)?;
    if !compiled(&pursuit_light) {
        return Err(invalid(format!(
            "pursuit light {pursuit_light} is not a compiled entity"
        )));
    }
    let forensic_scenario = text(&area, "forensicScenario", path)?;

    let exit = required(&area, "exit", path)?.clone();
    if exit.get("gate").and_then(Json::as_text) != Some(primary_gate.as_str()) {
        return Err(invalid(
            "declared exit must use the primary gate".to_owned(),
        ));
    }

    let actors = required(&area, "actors", path)?.clone();
    let actor_ids: BTreeSet<&str> = actors
        .as_array()
        .unwrap_or_default()
        .iter()
        .filter_map(|actor| actor.get("id").and_then(Json::as_text))
        .collect();
    if !actor_ids.contains("player") || !actor_ids.contains("gaoler") {
        return Err(invalid(
            "area requires player and gaoler presentation anchors".to_owned(),
        ));
    }

    let effects = required(&area, "effects", path)?.clone();
    for effect in effects.as_array().unwrap_or_default() {
        let anchor = effect
            .get("anchorEntity")
            .and_then(Json::as_text)
            .unwrap_or_default();
        if !compiled(anchor) {
            return Err(invalid(
                "effect anchor must reference a compiled entity".to_owned(),
            ));
        }
    }

    let architecture = required(&area, "architecture", path)?.clone();
    validate_architecture(&architecture, &invalid)?;

    Ok(AreaSource {
        id,
        label,
        start,
        primary_gate,
        pursuit_light,
        forensic_scenario,
        objective,
        exit,
        architecture,
        actors,
        effects,
    })
}

fn validate_architecture(
    architecture: &Json,
    invalid: &dyn Fn(String) -> PlanError,
) -> PlanResult<()> {
    let bounds = architecture
        .get("bounds")
        .ok_or_else(|| invalid("architecture must declare bounds".to_owned()))?;
    let width = lattice(bounds, "width").ok_or_else(|| {
        invalid("architecture bounds must fit the bounded 9x6 lattice".to_owned())
    })?;
    let height = lattice(bounds, "height").ok_or_else(|| {
        invalid("architecture bounds must fit the bounded 9x6 lattice".to_owned())
    })?;
    if !(1..=9).contains(&width) || !(1..=6).contains(&height) {
        return Err(invalid(
            "architecture bounds must fit the bounded 9x6 lattice".to_owned(),
        ));
    }

    let wall_height = number(architecture, "wallHeight")
        .ok_or_else(|| invalid("architecture must declare wallHeight".to_owned()))?;
    if !wall_height.greater_than(0) || !wall_height.at_most(5) {
        return Err(invalid(
            "architecture wall height must be in (0, 5]".to_owned(),
        ));
    }

    let masses = architecture
        .get("masses")
        .and_then(Json::as_array)
        .ok_or_else(|| invalid("architecture must declare masses".to_owned()))?;
    for mass in masses {
        let id = mass
            .get("id")
            .and_then(Json::as_text)
            .unwrap_or("<unnamed>");
        let outside = || {
            invalid(format!(
                "masonry mass {id} exceeds the bounded architecture profile"
            ))
        };
        let min = mass.get("min").ok_or_else(outside)?;
        let max = mass.get("max").ok_or_else(outside)?;
        let min_x = lattice(min, "x").ok_or_else(outside)?;
        let min_y = lattice(min, "y").ok_or_else(outside)?;
        let max_x = lattice(max, "x").ok_or_else(outside)?;
        let max_y = lattice(max, "y").ok_or_else(outside)?;
        let mass_height = number(mass, "height").ok_or_else(outside)?;
        if min_x < 0
            || min_y < 0
            || max_x > width
            || max_y > height
            || min_x >= max_x
            || min_y >= max_y
            || !mass_height.greater_than(0)
            || !mass_height.at_most(4)
        {
            return Err(outside());
        }
    }
    Ok(())
}

fn lattice(value: &Json, name: &str) -> Option<i64> {
    value.get(name)?.as_number()?.as_i64()
}

fn number<'a>(value: &'a Json, name: &str) -> Option<&'a Decimal> {
    value.get(name)?.as_number()
}

fn required<'a>(value: &'a Json, name: &str, path: &Path) -> PlanResult<&'a Json> {
    value.get(name).ok_or_else(|| {
        PlanError::new(codes::AREA_INVALID, format!("field `{name}` is absent")).at(path)
    })
}

fn text(value: &Json, name: &str, path: &Path) -> PlanResult<String> {
    required(value, name, path)?
        .as_text()
        .map(str::to_owned)
        .ok_or_else(|| {
            PlanError::new(
                codes::AREA_INVALID,
                format!("field `{name}` is not a string"),
            )
            .at(path)
        })
}
