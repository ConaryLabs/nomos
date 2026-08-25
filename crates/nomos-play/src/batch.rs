//! The command batch reducer.
//!
//! # Ordering rule
//!
//! > One input is one batch, one batch is one tick, and a batch is a total
//! > function of the input and the state it is applied to. Within a batch the
//! > player's action resolves first and completely — occupancy, traversal cost,
//! > the kernel transaction, the crossing, the outcome — and the state it
//! > produces is what every later step in the same batch observes. Then every
//! > non-player actor is offered a step, in ascending stable actor-id order,
//! > and each one either steps exactly once or does not step at all, according
//! > to its own rule read against the state the steps before it in this same
//! > batch have already produced. When the batch ends its tick is
//! > `tick_before + 1`, whether or not the player's action was accepted and
//! > whether or not any actor stepped, and nothing outside the batch ever
//! > observes it half-applied. No wall-clock reading, elapsed time, frame
//! > count, frame rate, fractional value, or random draw appears anywhere in
//! > this order or in anything it decides.
//!
//! # Pursuit rule
//!
//! > The pursuer is the single actor whose declared role is `pursuer`. It is
//! > offered a step only at the end of a batch whose player action was an
//! > accepted `move` inside the lattice — never after a refused move, never
//! > after an `interact`, and never after a `cross`. It declines the step
//! > unless the outcome is still `playing` and it is hunting, and it is hunting
//! > exactly when the area's declared pursuit light is not emitting at the
//! > batch's post-action kernel state, as resolved by
//! > `nomos_sim::resolve_light`. When it is offered a step and does not
//! > decline, it increments `pursuit.moves_since_step`; if the result is less
//! > than 2 the batch ends there with the counter raised and the pursuer where
//! > it was. Otherwise it takes exactly one step and sets
//! > `pursuit.moves_since_step` back to 0. The step is greedy along the
//! > dominant axis: let `dx` be the player's `x` minus the pursuer's `x`, and
//! > `dy` the player's `y` minus the pursuer's `y`; if `|dx| > |dy|` the
//! > pursuer moves one cell by `signum(dx)` along `x`; otherwise if `dy != 0`
//! > it moves one cell by `signum(dy)` along `y`; otherwise it moves by
//! > `signum(dx)` along `x`, which in that branch is necessarily zero because
//! > `|dx| <= |dy| = 0`. The tie `|dx| = |dy| != 0` therefore resolves to the
//! > `y` axis, and the only branch that does not move is the one in which the
//! > pursuer already stands on the player's cell. The step consults nothing
//! > else: not the lattice bounds, not the architecture's masses, not traversal
//! > cost, not any other actor. If after the step the pursuer's cell equals the
//! > player's cell the outcome becomes `caught`, and every later command in
//! > that area is refused until a new session begins.
//!
//! Both paragraphs are `docs/review/nomos-play.md` sections 3.1 and 3.3
//! verbatim. The pursuit rule is `apps/nomos-viewer/src/play.mjs:198-220`
//! reproduced line for line, with three things said in words that the
//! JavaScript left to be read out of control flow: it fires from exactly one
//! call site (`play.mjs:195`, the accepted-in-lattice move branch); the strict
//! `>` comparison sends the `|dx| == |dy|` tie to the `y` axis; and the third
//! branch is reachable only when `dx == dy == 0`, a zero step onto the cell the
//! pursuer already shares with the player.
//!
//! The pursuer's step ignores occupancy, and that is a faithful port, not an
//! oversight: R1-5 moves the authority for this rule out of JavaScript, it does
//! not change what the rule does.

use nomos_core::Ident;
use nomos_core::id::EntityId;
use nomos_projection::{
    LatticeCell, MovementDisposition, ResolvedLightFacts, ResolvedMovementFacts, RuntimeBinding,
    SimulationPlan,
};
use nomos_sim::{CommandRequest, PersistedRuntimeState, commit_transaction, resolve_command};

use crate::command::{Direction, PlayCommand};
use crate::error::{PlayError, PlayResult, codes};
use crate::occupancy;
use crate::plan::{AreaPlan, Role};
use crate::receipt::{ActorDelta, PlayReceipt};
use crate::state::{Actor, Outcome, PlayState};

/// One area, open for play: its plan, its executable semantics, and its state.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Area {
    /// The rendering plan's facts.
    pub plan: AreaPlan,
    /// The decoded simulation projection.
    pub semantics: SimulationPlan,
    /// The authoritative state.
    pub state: PlayState,
}

/// What one batch did, beyond the receipt.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Committed {
    /// The receipt.
    pub receipt: PlayReceipt,
    /// The area this run continues into, set only by an accepted crossing.
    pub crossed_to: Option<String>,
}

/// Resolves the effective movement facts at an area's current kernel state.
///
/// # Errors
///
/// Returns `PL0308` when the kernel's resolver refuses.
pub fn movement_facts(area: &Area) -> PlayResult<ResolvedMovementFacts> {
    nomos_sim::resolve_movement(&area.semantics, area.state.kernel.state())
        .map_err(|error| PlayError::from_kernel(codes::KERNEL_REFUSED, &error))
}

/// Resolves the effective light facts at an area's current kernel state.
///
/// # Errors
///
/// Returns `PL0308` when the kernel's resolver refuses.
pub fn light_facts(area: &Area) -> PlayResult<ResolvedLightFacts> {
    nomos_sim::resolve_light(&area.semantics, area.state.kernel.state())
        .map_err(|error| PlayError::from_kernel(codes::KERNEL_REFUSED, &error))
}

/// Whether the pursuer is hunting at an area's current kernel state.
///
/// True exactly when the declared pursuit light is not emitting. One helper:
/// the study computed this twice, once for gameplay and once for the HUD, with
/// opposite comparison operators.
///
/// # Errors
///
/// Returns `PL0308` when the kernel's light resolver refuses.
pub fn hunting(area: &Area) -> PlayResult<bool> {
    Ok(is_hunting(&light_facts(area)?, &area.state.pursuit_light))
}

fn is_hunting(light: &ResolvedLightFacts, pursuit_light: &EntityId) -> bool {
    light
        .get(pursuit_light)
        .is_some_and(|fact| !fact.emitting())
}

/// Applies one input as exactly one committed batch.
///
/// # Errors
///
/// Returns a `PL01##`/`PL02##` shape refusal, which produces no batch and does
/// not advance the tick. A `PL03##`/`PL04##` rule refusal is not returned: it
/// is recorded in the receipt this returns, with `accepted: false`.
pub fn step(
    area: &mut Area,
    ordinal: u64,
    previous_receipt_hash: nomos_core::Sha256Digest,
    input: &PlayCommand,
) -> PlayResult<Committed> {
    let tick_before = area.state.tick;
    let kernel_before = area.state.kernel_state_hash();
    let outcome_before = area.state.outcome;
    let cells_before: Vec<(EntityId, LatticeCell)> = area
        .state
        .actors
        .iter()
        .map(|actor| (actor.id.clone(), actor.cell))
        .collect();

    let mut refusal: Option<&'static str> = None;
    let mut crossed_to = None;
    let mut offer_pursuer = false;

    if area.state.outcome == Outcome::Playing {
        match apply(area, input) {
            Ok(effect) => {
                offer_pursuer = effect.offer_pursuer;
                crossed_to = effect.crossed_to;
            }
            Err(error) if error.is_rule_refusal() => refusal = Some(error.code()),
            Err(error) => return Err(error),
        }
    } else {
        refusal = Some(codes::NOT_PLAYING);
    }

    if offer_pursuer {
        advance_pursuer(area)?;
    }

    area.state.tick = area
        .state
        .tick
        .checked_add(1)
        .ok_or_else(|| PlayError::new(codes::DOCUMENT_VALUE, "the play tick overflowed"))?;

    let mut actor_deltas = Vec::new();
    for (id, before) in cells_before {
        if let Some(actor) = area
            .state
            .actors
            .iter()
            .find(|actor| actor.id == id && actor.cell != before)
        {
            actor_deltas.push(ActorDelta {
                id,
                from: before,
                to: actor.cell,
            });
        }
    }

    let receipt = PlayReceipt {
        ordinal,
        area: area.state.area.clone(),
        input: input.clone(),
        accepted: refusal.is_none(),
        refusal,
        tick_before,
        tick_after: area.state.tick,
        kernel_state_hash_before: kernel_before,
        kernel_state_hash_after: area.state.kernel_state_hash(),
        actor_deltas,
        outcome_before,
        outcome_after: area.state.outcome,
        counters_after: area.state.counters,
        previous_receipt_hash,
        play_state_hash_after: area.state.state_hash(),
    };
    Ok(Committed {
        receipt,
        crossed_to,
    })
}

struct Effect {
    offer_pursuer: bool,
    crossed_to: Option<String>,
}

fn apply(area: &mut Area, input: &PlayCommand) -> PlayResult<Effect> {
    match input {
        PlayCommand::Move { direction } => player_move(area, *direction),
        PlayCommand::Interact { entity, action } => {
            interact(area, entity, action).map(|()| Effect {
                offer_pursuer: false,
                crossed_to: None,
            })
        }
        PlayCommand::Cross { gate } => crossing(area, gate).map(|crossed_to| Effect {
            offer_pursuer: false,
            crossed_to,
        }),
    }
}

/// The player's step. A target outside the lattice is resolved as the crossing
/// through the door bound to the face of the player's own cell in the direction
/// of travel — R1-4's deliberate divergence from the study's `target.y < 0`
/// special case (`docs/review/nomos-viewer.md` section 2 row 22) — and calls the
/// same [`crossing`] the `cross` spelling calls, so there is one rule.
fn player_move(area: &mut Area, direction: Direction) -> PlayResult<Effect> {
    let (dx, dy) = direction.delta();
    let from = area.state.player().cell;
    let (x, y) = (from.x() + dx, from.y() + dy);

    if !area.plan.in_bounds(x, y) {
        let gate = exit_door(area, from, direction)
            .ok_or_else(|| PlayError::new(codes::NO_OPENING, "the masonry has no opening here"))?;
        return crossing(area, &gate).map(|crossed_to| Effect {
            offer_pursuer: false,
            crossed_to,
        });
    }

    let movement = movement_facts(area)?;
    let step = occupancy::enter(&area.plan, &area.semantics, &area.state, &movement, x, y)?;

    move_actor(&mut area.state, Role::Player, LatticeCell::new(x, y, 0));
    area.state.counters.moves = add(area.state.counters.moves, 1)?;
    area.state.counters.traversal_cost = add(area.state.counters.traversal_cost, step.cost)?;
    Ok(Effect {
        offer_pursuer: true,
        crossed_to: None,
    })
}

/// The door bound to the face of `from` in the direction of travel, if the plan
/// and the projection declare one.
fn exit_door(area: &Area, from: LatticeCell, direction: Direction) -> Option<EntityId> {
    area.semantics
        .entities()
        .iter()
        .find(|entity| {
            matches!(area.plan.kind_of(entity.id()), Some("door"))
                && matches!(
                    entity.binding(),
                    RuntimeBinding::Face { cell, direction: face }
                        if cell.x() == from.x()
                            && cell.y() == from.y()
                            && face.as_str() == direction.as_str()
                )
        })
        .map(|entity| entity.id().clone())
}

/// The crossing rule, in one place. Both spellings of the exit reach it.
fn crossing(area: &mut Area, gate: &EntityId) -> PlayResult<Option<String>> {
    let Some(entity) = occupancy::entity(&area.semantics, gate) else {
        return Err(PlayError::new(
            codes::COMMAND_TARGET,
            format!("`{gate}` is not a compiled entity of this area"),
        ));
    };
    if !matches!(area.plan.kind_of(gate), Some("door")) {
        return Err(PlayError::new(
            codes::COMMAND_TARGET,
            format!("`{gate}` is not a door"),
        ));
    }
    let player = area.state.player().cell;
    let on_this_cell = matches!(
        entity.binding(),
        RuntimeBinding::Face { cell, .. } if cell.x() == player.x() && cell.y() == player.y()
    );
    if !on_this_cell {
        return Err(PlayError::new(
            codes::NO_OPENING,
            format!("`{gate}` is not on the player's cell"),
        ));
    }

    let movement = movement_facts(area)?;
    match movement.get(gate) {
        Some(MovementDisposition::Traversable { .. }) => {}
        Some(MovementDisposition::Blocked { reasons }) => {
            let reasons = reasons
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" + ");
            return Err(PlayError::new(
                codes::NO_OPENING,
                format!("`{gate}` is blocked: {reasons}"),
            ));
        }
        None => {
            return Err(PlayError::new(
                codes::NO_OPENING,
                format!("`{gate}` is not a ground-movement subject"),
            ));
        }
    }

    // The exit step costs one move at cost 1, as `play.mjs:159` fixes it.
    area.state.counters.moves = add(area.state.counters.moves, 1)?;
    area.state.counters.traversal_cost = add(area.state.counters.traversal_cost, 1)?;
    area.state.outcome = Outcome::Escaped;
    Ok(area.plan.to_area.clone())
}

/// The only branch that touches the kernel's transaction machinery.
fn interact(area: &mut Area, entity: &EntityId, action: &Ident) -> PlayResult<()> {
    let Some(projected) = occupancy::entity(&area.semantics, entity) else {
        return Err(PlayError::new(
            codes::COMMAND_TARGET,
            format!("`{entity}` is not a compiled entity of this area"),
        ));
    };
    let player = area.state.player().cell;
    if occupancy::binding_distance(projected.binding(), player) > 1 {
        return Err(PlayError::new(
            codes::NOTHING_IN_REACH,
            format!("`{entity}` is not within reach"),
        ));
    }

    let request = CommandRequest::new(action.clone(), entity.clone(), None);
    let command = resolve_command(&area.semantics, &request).map_err(|error| {
        if error.code().as_str() == "EK0805" {
            PlayError::from_kernel(codes::CREDENTIAL_UNSUPPORTED, &error)
        } else {
            PlayError::from_kernel(codes::KERNEL_REFUSED, &error)
        }
    })?;
    let committed = commit_transaction(&area.semantics, area.state.kernel.state(), &command)
        .map_err(|error| PlayError::from_kernel(codes::KERNEL_REFUSED, &error))?;
    area.state.kernel = PersistedRuntimeState::new(&area.semantics, committed.into_snapshot())
        .map_err(|error| PlayError::from_kernel(codes::KERNEL_REFUSED, &error))?;
    Ok(())
}

/// The pursuit rule, quoted verbatim in this module's doc-comment.
fn advance_pursuer(area: &mut Area) -> PlayResult<()> {
    if area.state.outcome != Outcome::Playing {
        return Ok(());
    }
    let Some(pursuer) = area.state.pursuer() else {
        return Ok(());
    };
    let pursuer_cell = pursuer.cell;
    let light = light_facts(area)?;
    if !is_hunting(&light, &area.state.pursuit_light) {
        return Ok(());
    }

    let raised = add(area.state.moves_since_step, 1)?;
    if raised < 2 {
        area.state.moves_since_step = raised;
        return Ok(());
    }

    let player = area.state.player().cell;
    let dx = player.x() - pursuer_cell.x();
    let dy = player.y() - pursuer_cell.y();
    let stepped = if dx.abs() > dy.abs() {
        LatticeCell::new(pursuer_cell.x() + dx.signum(), pursuer_cell.y(), 0)
    } else if dy != 0 {
        LatticeCell::new(pursuer_cell.x(), pursuer_cell.y() + dy.signum(), 0)
    } else {
        LatticeCell::new(pursuer_cell.x() + dx.signum(), pursuer_cell.y(), 0)
    };

    move_actor(&mut area.state, Role::Pursuer, stepped);
    area.state.moves_since_step = 0;
    if stepped.x() == player.x() && stepped.y() == player.y() {
        area.state.outcome = Outcome::Caught;
    }
    Ok(())
}

fn move_actor(state: &mut PlayState, role: Role, cell: LatticeCell) {
    if let Some(actor) = state.actors.iter_mut().find(|actor| actor.role == role) {
        actor.cell = cell;
    }
}

fn add(left: u64, right: u64) -> PlayResult<u64> {
    left.checked_add(right)
        .ok_or_else(|| PlayError::new(codes::DOCUMENT_VALUE, "a play counter overflowed"))
}

/// Every command legal at this state on an entity within reach of the player,
/// ordered by `(entity, action)`.
///
/// This is the authoritative replacement for `plan.interactions[]` as
/// gameplay. The plan's interaction edges are derived from committed command
/// *logs* (`crates/nomos-render-plan/src/runs.rs`), so they encode the order a
/// human authored the scripts rather than a rule; this is a rule. Legal means
/// the entity owns exactly one machine declaring the action — the same
/// uniqueness `nomos_sim::resolve_command` requires — that machine's current
/// state is the `source` of a transition for it, and the transition's
/// requirement is `none`.
///
/// `docs/review/nomos-play.md` section 3.6 records the measurement that made
/// this safe: with the reach filter applied at the cell the four-area route
/// actually stands on, the first row by `(entity, action)` is the edge the
/// authored ladder used, at every step the route takes.
///
/// # Errors
///
/// Returns `PL0308` when the kernel refuses to resolve a candidate.
pub fn available_interactions(area: &Area) -> PlayResult<Vec<(EntityId, Ident)>> {
    let player = area.state.player().cell;
    let mut available = Vec::new();
    for entity in area.semantics.entities() {
        if occupancy::binding_distance(entity.binding(), player) > 1 {
            continue;
        }
        for machine in area.semantics.machines() {
            if !entity.machines().contains(machine.namespace()) {
                continue;
            }
            let Some(current) = area.state.kernel.state().machine(machine.namespace()) else {
                continue;
            };
            for transition in machine.commands() {
                if transition.source() != current {
                    continue;
                }
                if !matches!(
                    transition.requirement(),
                    nomos_projection::CommandRequirement::None
                ) {
                    continue;
                }
                let request =
                    CommandRequest::new(transition.action().clone(), entity.id().clone(), None);
                if resolve_command(&area.semantics, &request).is_ok() {
                    available.push((entity.id().clone(), transition.action().clone()));
                }
            }
        }
    }
    available.sort();
    available.dedup();
    Ok(available)
}

/// The actors, for a presentation reader that does not want the whole state.
#[must_use]
pub fn actors(area: &Area) -> &[Actor] {
    &area.state.actors
}
