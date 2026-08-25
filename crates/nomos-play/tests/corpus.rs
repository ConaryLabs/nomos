//! The four committed areas, played.
//!
//! This is where the numbers live. `docs/review/nomos-viewer.md` section 5.3
//! used to predict `moves` and `cost` in JavaScript inside the smoke harness,
//! which after R1-5 would be a third implementation of rules the runtime owns.
//! The harness stopped predicting them; they are pinned here, against the
//! committed plans, in the language that owns them.

mod common;

use nomos_play::{SessionOutcome, batch};

#[test]
fn the_four_area_route_costs_what_the_record_says() {
    let session = common::play_route();
    assert_eq!(session.outcome(), SessionOutcome::Completed);
    assert_eq!(session.areas_cleared(), 4);
    assert_eq!(session.counters().moves, 44);
    assert_eq!(session.counters().traversal_cost, 60);
    // 52 keys: 44 moves and 8 interactions. Every input is one batch and one
    // tick, so the tick is the input count and not the move count.
    assert_eq!(session.log().len(), 52);
    assert_eq!(session.receipts().len(), 52);
    assert_eq!(session.live().state.tick, 52);
}

#[test]
fn water_costs_more_than_stone_on_the_route() {
    let session = common::play_route();
    assert!(
        session.counters().traversal_cost > session.counters().moves,
        "the route is deliberately walked through water, so cost exceeds the move count"
    );
}

#[test]
fn every_input_the_route_sends_is_accepted() {
    let session = common::play_route();
    for receipt in session.receipts() {
        assert!(
            receipt.accepted,
            "ordinal {} was refused with {:?}",
            receipt.ordinal, receipt.refusal
        );
        assert_eq!(receipt.tick_after, receipt.tick_before + 1);
    }
}

/// The measurement `docs/review/nomos-play.md` section 3.6 commits to.
///
/// `plan.interactions[]` is derived from committed command logs, so it encodes
/// the order a human authored the scripts. The runtime enumerates legal
/// commands instead, which is a rule — but only if it picks the edge the route
/// needs. It does, and this is why: at the cell the route stands on, only the
/// objective gate is within reach.
#[test]
fn the_enumeration_offers_exactly_the_gate_the_route_uses() {
    for (index, area_id) in common::ROUTE.iter().enumerate() {
        let mut session = common::session_at(area_id);
        if index > 0 {
            // Every area but the first is entered at its own `route.entry`;
            // opening it directly puts the player at the declared cell instead,
            // which is the same cell for this corpus. Walk the route either way.
        }
        let keys = common::ROUTE_KEYS[index];
        let before_interaction: String = keys.chars().take_while(|key| *key != '*').collect();
        common::drive(&mut session, &before_interaction);

        let available = batch::available_interactions(session.live()).unwrap();
        assert_eq!(
            available.len(),
            2,
            "{area_id}: the gate offers ignite and unseal and nothing else is in reach; got {available:?}"
        );
        assert_eq!(
            available[0].1.as_str(),
            "ignite",
            "{area_id}: the first row by (entity, action) is the ladder's first edge"
        );
        let gate = session.live().plan.objective_gate.clone();
        assert!(
            available.iter().all(|(entity, _)| entity == &gate),
            "{area_id}: only the objective gate is within reach at the exit cell"
        );
    }
}

#[test]
fn interacting_reaches_the_state_hash_the_captured_scenario_recorded() {
    // The plan's scenario ladder is evidence, not gameplay — but it is evidence
    // of the same kernel, so playing to the same point must reach the same
    // kernel state hash. This is the strongest single check that the runtime
    // and the capture agree.
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>");
    let baseline = session.live().state.kernel_state_hash().to_hex();
    assert_eq!(
        baseline, "9d81ddaf8c3310a10399ce1c727d117f5385426f5e3fc0b1575d7ec042d3e49d",
        "walking changes no kernel state, so this is still `01-baseline`"
    );

    common::drive(&mut session, "*");
    assert_eq!(
        session.live().state.kernel_state_hash().to_hex(),
        "bc01b2f3427a95341e89c1a0cf9d4aca2190e20a90b7d38fd4a4ed1a8a0f5150",
        "`ignite` reaches the `02-breached-warded` state hash"
    );

    common::drive(&mut session, "*");
    assert_eq!(
        session.live().state.kernel_state_hash().to_hex(),
        "0c0a573503282ec7b8f10dada7da267b96f04c089054936b84bece096b0ac7f2",
        "`unseal` reaches the `03-breached-unsealed` state hash"
    );
}

#[test]
fn the_initial_kernel_state_is_the_first_captured_scenario() {
    // Resolves the ownership audit's item 23. The study read
    // `plan.scenarios[0]` as "the default" by array position; the initial state
    // is `SimulationState::initialize`, a kernel fact, and it happens to be the
    // state that convention was describing.
    let session = common::session_at("north-gaol");
    assert_eq!(
        session.live().state.kernel_state_hash().to_hex(),
        "9d81ddaf8c3310a10399ce1c727d117f5385426f5e3fc0b1575d7ec042d3e49d"
    );
    assert_eq!(session.live().state.kernel.state().tick(), 0);
}

#[test]
fn only_one_entity_covers_any_cell_in_the_committed_corpus() {
    // `docs/review/nomos-play.md` section 3.2 takes the maximum cost over the
    // entities covering a cell rather than the first found. For this corpus the
    // two agree because no cell is covered twice; that is measured here rather
    // than assumed, so a content change that overlapped two water regions would
    // fail a test instead of quietly changing a cost.
    for area_id in common::ROUTE {
        let session = common::session_at(area_id);
        let area = session.live();
        for y in 0..area.plan.height {
            for x in 0..area.plan.width {
                let covering = nomos_play::occupancy::covering(&area.semantics, x, y).count();
                assert!(
                    covering <= 1,
                    "{area_id}: ({x}, {y}) is covered by {covering} entities"
                );
            }
        }
    }
}

#[test]
fn the_route_never_walks_onto_the_pursuer() {
    // Occupancy rule 4 — another actor blocks the player — is a behaviour change
    // from `play.mjs`, which let the player walk onto the gaoler. It moves no
    // committed number, and this is why.
    let session = common::play_route();
    for receipt in session.receipts() {
        assert_ne!(
            receipt.refusal,
            Some(nomos_play::codes::OCCUPIED),
            "no step of the committed route is refused for occupancy"
        );
    }
}
