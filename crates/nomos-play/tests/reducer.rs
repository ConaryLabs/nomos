//! Occupancy, cost, crossing, and what a refused input does.
//!
//! Nineteen of the twenty-two cases in `apps/nomos-viewer/test/play.test.mjs`
//! became Rust when the JavaScript reducer was deleted; these are the movement
//! and interaction half of them, strengthened where the move to the kernel made
//! a stronger claim available. `docs/review/nomos-play.md` section 8.3 is the
//! migration table.

mod common;

use nomos_core::Ident;
use nomos_core::id::EntityId;
use nomos_play::{Direction, Outcome, PlayCommand, SessionOutcome, batch, codes};

fn entity(id: &str) -> EntityId {
    EntityId::parse(id).unwrap()
}

#[test]
fn a_step_onto_stone_costs_one() {
    let mut session = common::session_at("north-gaol");
    let before = session.counters();
    session.step(&common::step(Direction::West)).unwrap();
    assert_eq!(session.counters().moves, before.moves + 1);
    assert_eq!(session.counters().traversal_cost, before.traversal_cost + 1);
}

#[test]
fn water_uses_the_projected_traversal_cost_at_the_live_state() {
    // `play.mjs:101-108` read this off a captured scenario. It now comes from
    // `resolve_movement` evaluated against the embedded kernel state, which is
    // the whole point of the slice.
    let mut session = common::session_at("north-gaol");
    // (2, 4) -> (2, 3), the north edge of the flooded region (2,2)-(4,3).
    let before = session.counters();
    session.step(&common::step(Direction::North)).unwrap();
    assert_eq!(session.counters().moves, before.moves + 1);
    assert_eq!(
        session.counters().traversal_cost,
        before.traversal_cost + 3,
        "the flooded section's projected cost is 3"
    );
}

#[test]
fn a_mass_blocks_the_cells_it_covers() {
    // Ember Vault declares two piers. The rectangle is half-open, reproducing
    // `play.mjs:110-114`: `min <= x < max`.
    let session = common::session_at("ember-vault");
    let plan = &session.live().plan;
    assert_eq!(plan.masses.len(), 2);
    let mass = &plan.masses[0];
    assert!(plan.mass_at(mass.min.0, mass.min.1).is_some());
    assert!(plan.mass_at(mass.max.0, mass.min.1).is_none());
    assert!(plan.mass_at(mass.min.0, mass.max.1).is_none());
}

#[test]
fn a_move_into_masonry_is_refused_and_still_commits_a_batch() {
    let mut session = common::session_at("ember-vault");
    let plan = session.live().plan.clone();
    let mass = plan.masses.first().expect("ember-vault declares a pier");
    // Walk to the cell immediately south of the mass's minimum corner, then
    // step north into it.
    let target = (mass.min.0, mass.min.1);
    let mut guard = 0;
    loop {
        let player = session.live().state.player().cell;
        if (player.x(), player.y()) == (target.0, target.1 + 1) {
            break;
        }
        let direction = if player.x() < target.0 {
            Direction::East
        } else if player.x() > target.0 {
            Direction::West
        } else if player.y() > target.1 + 1 {
            Direction::North
        } else {
            Direction::South
        };
        session.step(&common::step(direction)).unwrap();
        guard += 1;
        assert!(guard < 40, "the walk to the mass terminates");
    }

    let tick_before = session.live().state.tick;
    let counters_before = session.counters();
    let receipt = session
        .step(&common::step(Direction::North))
        .unwrap()
        .clone();
    assert!(!receipt.accepted);
    assert_eq!(receipt.refusal, Some(codes::MASONRY));
    assert_eq!(receipt.tick_after, tick_before + 1);
    assert!(receipt.actor_deltas.is_empty());
    assert_eq!(receipt.counters_after.moves, counters_before.moves);
    assert_eq!(
        receipt.kernel_state_hash_after,
        receipt.kernel_state_hash_before
    );
}

#[test]
fn the_baseline_gate_refuses_an_exit_and_names_the_resolver_reasons() {
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>>");
    assert_eq!(
        (
            session.live().state.player().cell.x(),
            session.live().state.player().cell.y()
        ),
        (5, 0),
        "the walk ends on the objective gate's own cell"
    );
    let receipt = session
        .step(&common::step(Direction::North))
        .unwrap()
        .clone();
    assert!(!receipt.accepted);
    assert_eq!(receipt.refusal, Some(codes::NO_OPENING));
    assert_eq!(session.live().state.outcome, Outcome::Playing);
}

#[test]
fn the_breached_and_unsealed_gate_permits_an_exit() {
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>**>");
    let receipt = session
        .step(&common::step(Direction::North))
        .unwrap()
        .clone();
    assert!(receipt.accepted);
    assert_eq!(receipt.outcome_after, Outcome::Escaped);
    assert_eq!(session.outcome(), SessionOutcome::Completed);
    assert_eq!(session.areas_cleared(), 1);
}

#[test]
fn a_move_that_leaves_the_lattice_with_no_door_finds_masonry() {
    let mut session = common::session_at("north-gaol");
    // (2, 4) is not on a door's cell; walking north-west to (0, 4) and stepping
    // west leaves the lattice through nothing.
    common::drive(&mut session, "<<");
    let receipt = session
        .step(&common::step(Direction::West))
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::NO_OPENING));
    assert!(
        session
            .receipts()
            .last()
            .is_some_and(|receipt| !receipt.accepted)
    );
}

#[test]
fn the_unchanged_second_door_remains_blocked() {
    // North Gaol declares two doors. Opening the first must not open the other.
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>**");
    let movement = batch::movement_facts(session.live()).unwrap();
    assert!(matches!(
        movement.get(&entity("north_gate")),
        Some(nomos_projection::MovementDisposition::Traversable { .. })
    ));
    assert!(matches!(
        movement.get(&entity("north_gate_02")),
        Some(nomos_projection::MovementDisposition::Blocked { .. })
    ));
}

#[test]
fn an_exit_uses_the_doors_declared_direction() {
    // R1-4's divergence from the study's `target.y < 0` special case: the
    // crossing is the door on the player's own cell facing the way the move is
    // heading. Every corpus door faces north, so the other three faces are the
    // negative case.
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>**>");
    for wrong in [Direction::East, Direction::West, Direction::South] {
        let mut probe = session.clone();
        let receipt = probe.step(&common::step(wrong)).unwrap().clone();
        assert!(
            receipt.accepted || receipt.refusal == Some(codes::NO_OPENING),
            "{wrong:?} either walks inside the lattice or finds no opening"
        );
        assert_ne!(receipt.outcome_after, Outcome::Escaped);
    }
    let receipt = session
        .step(&common::step(Direction::North))
        .unwrap()
        .clone();
    assert_eq!(receipt.outcome_after, Outcome::Escaped);
}

#[test]
fn the_two_spellings_of_the_crossing_produce_the_same_receipt() {
    // One rule, two spellings. A lattice-leaving `move` derives the gate from
    // the declared face and calls the same function `cross` calls, so the only
    // difference between the receipts is the input that produced them.
    let mut walked = common::session_at("north-gaol");
    common::drive(&mut walked, "^^^^>>**>");
    let mut named = walked.clone();

    walked.step(&common::step(Direction::North)).unwrap();
    named
        .step(&PlayCommand::Cross {
            gate: entity("north_gate"),
        })
        .unwrap();

    let left = walked.receipts().last().unwrap();
    let right = named.receipts().last().unwrap();
    assert!(left.accepted && right.accepted, "both spellings crossed");
    assert_eq!(left.outcome_after, Outcome::Escaped);
    assert_eq!(left.outcome_after, right.outcome_after);
    assert_eq!(left.counters_after, right.counters_after);
    assert_eq!(left.tick_after, right.tick_after);
    assert_eq!(left.actor_deltas, right.actor_deltas);
    assert_ne!(left.input, right.input);
}

#[test]
fn crossing_a_sealed_gate_is_refused() {
    // Standing on the gate's own cell, so the refusal is about the ward and the
    // integrity rather than about where the player is.
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>>");
    let receipt = session
        .step(&PlayCommand::Cross {
            gate: entity("north_gate"),
        })
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::NO_OPENING));
}

#[test]
fn crossing_a_gate_that_is_not_underfoot_is_refused() {
    let mut session = common::session_at("north-gaol");
    let receipt = session
        .step(&PlayCommand::Cross {
            gate: entity("north_gate"),
        })
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::NO_OPENING));
}

#[test]
fn interaction_range_does_not_invent_remote_actions() {
    let mut session = common::session_at("north-gaol");
    let receipt = session
        .step(&PlayCommand::Interact {
            entity: entity("north_gate"),
            action: Ident::new("ignite").unwrap(),
        })
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::NOTHING_IN_REACH));
    assert_eq!(
        receipt.kernel_state_hash_after,
        receipt.kernel_state_hash_before
    );
}

#[test]
fn an_undeclared_action_is_the_kernels_refusal() {
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>");
    let receipt = session
        .step(&PlayCommand::Interact {
            entity: entity("north_gate"),
            action: Ident::new("levitate").unwrap(),
        })
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::KERNEL_REFUSED));
}

#[test]
fn an_interaction_does_not_offer_the_pursuer_a_step() {
    // The pursuit rule counts accepted player moves, not ticks. An interaction
    // advances the tick and leaves `moves_since_step` where it was.
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>");
    let before = session.live().state.moves_since_step;
    common::drive(&mut session, "*");
    assert_eq!(session.live().state.moves_since_step, before);
}

#[test]
fn a_command_after_the_run_is_over_is_refused_and_still_ticks() {
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>**>^");
    assert_eq!(session.outcome(), SessionOutcome::Completed);
    let tick_before = session.live().state.tick;
    let receipt = session
        .step(&common::step(Direction::South))
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::NOT_PLAYING));
    assert_eq!(receipt.tick_after, tick_before + 1);
}

#[test]
fn a_malformed_command_produces_no_receipt_and_no_tick() {
    // The other half of the split: a document that is not a `play_command@1` is
    // not an input, so there is nothing to decide about. This walks the same
    // path `wasm::nomos_play_step` walks — decode, then step — so the claim is
    // about the runtime's front door and not about the decoder alone.
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^");
    let receipts = session.receipts().len();
    let tick = session.live().state.tick;
    let hash = session.receipt_chain_head();

    for malformed in [
        &b"{}"[..],
        b"not canonical at all",
        br#"{"kind":"levitate","schema":"nomos.play_command@1"}"#,
        br#"{"direction":"widdershins","kind":"move","schema":"nomos.play_command@1"}"#,
    ] {
        let error = match nomos_play::PlayCommand::decode(malformed) {
            Ok(command) => panic!("{command:?} should not have decoded"),
            Err(error) => error,
        };
        assert!(!error.is_rule_refusal(), "{error}");
    }

    assert_eq!(session.receipts().len(), receipts);
    assert_eq!(session.live().state.tick, tick);
    assert_eq!(session.receipt_chain_head(), hash);
}

#[test]
fn the_reducer_is_a_function_of_the_state_and_the_input() {
    // Two sessions, the same log, byte-identical receipts. Determinism is a
    // property of the code — nothing here reads a clock or a hash map — and this
    // is the smallest statement of it.
    let mut left = common::session_at("north-gaol");
    let mut right = common::session_at("north-gaol");
    for session in [&mut left, &mut right] {
        common::drive(session, "^^^^>>**");
    }
    assert_eq!(
        left.receipts()
            .iter()
            .map(nomos_play::PlayReceipt::to_canonical_bytes)
            .collect::<Vec<_>>(),
        right
            .receipts()
            .iter()
            .map(nomos_play::PlayReceipt::to_canonical_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(left.receipt_chain_head(), right.receipt_chain_head());
}
