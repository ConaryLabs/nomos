//! Occupancy and step cost, resolved at the batch's kernel state.
//!
//! `docs/review/nomos-play.md` section 3.2 is the rule. Four conditions, three
//! owners: the lattice bounds and the architectural masses come from the
//! rendering plan; the effective movement disposition and traversal cost come
//! from `nomos_sim::resolve_movement` at the embedded kernel state; the other
//! actors come from the play state.
//!
//! Condition 3 is where the shadow resolver dies. `play.mjs:101-108` read a
//! water region's cost off a *captured scenario*; the same number now comes
//! from the resolver evaluated against the live state, so an interaction that
//! changes a claim's activation changes the cost of the next step without
//! anything being recaptured.

use nomos_core::id::EntityId;
use nomos_projection::{
    LatticeCell, MovementDisposition, ProjectedEntity, ResolvedMovementFacts, RuntimeBinding,
    SimulationPlan,
};

use crate::error::{PlayError, PlayResult, codes};
use crate::plan::AreaPlan;
use crate::state::PlayState;

/// What the reducer learned about a target cell.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Step {
    /// The traversal cost of entering the cell.
    pub cost: u64,
}

/// Whether a compiled entity's binding occupies a lattice cell.
///
/// A `Cell` binding occupies its own cell. A `Region` binding occupies every
/// cell of its closed rectangle, inclusive on both corners, because that is how
/// `nomos_projection::RuntimeBinding::Region` is defined.
///
/// **A `Face` binding occupies nothing.** A face is the boundary between two
/// cells, not either of them: a door bound to the north face of `(5, 0)`
/// governs passage *through* that face, which is what [`crate::batch`]'s
/// crossing rule resolves, and it does not stop the player standing on `(5, 0)`.
/// The study's reducer had the same behaviour by accident — `terrainAt`
/// (`play.mjs:101-108`) looked only at water — and it is a rule here.
#[must_use]
pub fn occupies_cell(binding: &RuntimeBinding, x: i32, y: i32) -> bool {
    match binding {
        RuntimeBinding::Cell(cell) => cell.x() == x && cell.y() == y,
        RuntimeBinding::Face { .. } => false,
        RuntimeBinding::Region { min, max } => {
            x >= min.x() && x <= max.x() && y >= min.y() && y <= max.y()
        }
    }
}

/// Manhattan distance from a cell to a compiled entity's binding.
///
/// Total for all three binding kinds so that reach is a rule and not a special
/// case: to a `Cell` or a `Face` it is the distance to the owning cell; to a
/// `Region` it is the distance to the nearest cell of the region, taken by
/// clamping component-wise.
#[must_use]
pub fn binding_distance(binding: &RuntimeBinding, from: LatticeCell) -> i32 {
    match binding {
        RuntimeBinding::Cell(cell) | RuntimeBinding::Face { cell, .. } => {
            (cell.x() - from.x()).abs() + (cell.y() - from.y()).abs()
        }
        RuntimeBinding::Region { min, max } => {
            let x = from.x().clamp(min.x(), max.x());
            let y = from.y().clamp(min.y(), max.y());
            (x - from.x()).abs() + (y - from.y()).abs()
        }
    }
}

/// Decides whether the player may enter a cell, and what the step costs.
///
/// # Errors
///
/// Returns the `PL03##` code naming which of the four conditions failed:
/// `PL0306` outside the lattice, `PL0302` masonry, `PL0303` a blocked entity,
/// `PL0304` another actor.
pub fn enter(
    plan: &AreaPlan,
    semantics: &SimulationPlan,
    state: &PlayState,
    movement: &ResolvedMovementFacts,
    x: i32,
    y: i32,
) -> PlayResult<Step> {
    if !plan.in_bounds(x, y) {
        return Err(PlayError::new(
            codes::NO_OPENING,
            format!(
                "({x}, {y}) is outside the {}x{} lattice",
                plan.width, plan.height
            ),
        ));
    }
    if let Some(mass) = plan.mass_at(x, y) {
        return Err(PlayError::new(
            codes::MASONRY,
            format!("({x}, {y}) is inside masonry mass `{}`", mass.id),
        ));
    }
    if let Some(actor) = state.actor_at(x, y) {
        return Err(PlayError::new(
            codes::OCCUPIED,
            format!("({x}, {y}) is occupied by `{}`", actor.id),
        ));
    }

    // The maximum cost over every covering entity, or 1 when none covers the
    // cell. Maximum rather than first-found because maximum-of-active is the
    // kernel's own composition rule; for the committed corpus the two agree,
    // since each area declares one non-overlapping water region.
    let mut cost = 1_u64;
    for entity in covering(semantics, x, y) {
        match movement.get(entity.id()) {
            Some(MovementDisposition::Traversable {
                cost: entity_cost, ..
            }) => cost = cost.max(u64::from(*entity_cost)),
            Some(MovementDisposition::Blocked { reasons }) => {
                let reasons = reasons
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" + ");
                return Err(PlayError::new(
                    codes::BLOCKED,
                    format!("`{}` blocks ({x}, {y}): {reasons}", entity.id()),
                ));
            }
            // An entity the movement resolver has no subject for imposes
            // nothing: it is not a ground-movement subject at all.
            None => {}
        }
    }
    Ok(Step { cost })
}

/// The compiled entities whose binding occupies a cell, in stable entity order.
pub fn covering(
    semantics: &SimulationPlan,
    x: i32,
    y: i32,
) -> impl Iterator<Item = &ProjectedEntity> {
    semantics
        .entities()
        .iter()
        .filter(move |entity| occupies_cell(entity.binding(), x, y))
}

/// The compiled entity with this identity, if the projection declares one.
#[must_use]
pub fn entity<'a>(semantics: &'a SimulationPlan, id: &EntityId) -> Option<&'a ProjectedEntity> {
    semantics.entities().iter().find(|entity| entity.id() == id)
}
