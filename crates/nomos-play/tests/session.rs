//! Crossing, arrival, cumulative counters, and reset.

mod common;

use nomos_play::{Direction, PlaySession, SessionOutcome, codes};

#[test]
fn arrival_uses_the_destinations_own_entry_cell() {
    // Owner ruling 3 of `docs/review/presentation-source.md`: the exiting area
    // no longer names a cell inside its neighbour, so the arrival cell is read
    // from the plan the player is arriving *in*.
    let expected = common::route_expectations();
    let next = &expected.route[1].area;
    let mut session = common::session();
    common::drive(&mut session, &expected.route[0].keys);
    assert_eq!(session.outcome(), SessionOutcome::Escaped);
    assert_eq!(session.pending_area(), Some(next.as_str()));

    common::enter(&mut session, next);
    let player = session.live().state.player().cell;
    let entry = session
        .live()
        .plan
        .entry
        .expect("a non-start area declares an entry");
    assert_eq!((player.x(), player.y()), (entry.x(), entry.y()));
}

#[test]
fn arrival_carries_the_tick_and_both_counters() {
    let expected = common::route_expectations();
    let mut session = common::session();
    common::drive(&mut session, &expected.route[0].keys);
    let tick = session.live().state.tick;
    let counters = session.counters();
    assert!(counters.moves > 0);
    assert!(counters.traversal_cost >= counters.moves);

    common::enter(&mut session, &expected.route[1].area);
    assert_eq!(session.live().state.tick, tick, "the tick does not reset");
    assert_eq!(session.counters(), counters, "the counters are cumulative");
    assert_eq!(session.live().state.moves_since_step, 0);
    assert_eq!(session.live().state.outcome, nomos_play::Outcome::Playing);
    assert_eq!(session.areas_cleared(), 1);
}

#[test]
fn arrival_opens_a_fresh_kernel_state_for_the_destination() {
    let expected = common::route_expectations();
    let mut session = common::session();
    common::drive(&mut session, &expected.route[0].keys);
    let left_behind = session.live().state.kernel_state_hash();
    common::enter(&mut session, &expected.route[1].area);
    assert_ne!(session.live().state.kernel_state_hash(), left_behind);
    assert_eq!(session.live().state.kernel.state().tick(), 0);
}

#[test]
fn arrival_into_the_wrong_area_is_refused() {
    let expected = common::route_expectations();
    let pending = &expected.route[1].area;
    let wrong = expected
        .route
        .iter()
        .map(|leg| &leg.area)
        .find(|area| *area != pending && *area != &expected.route[0].area)
        .expect("the route has a third area");
    let mut session = common::session();
    common::drive(&mut session, &expected.route[0].keys);
    let error = session
        .enter(&common::plan(wrong), &common::semantics(wrong))
        .unwrap_err();
    assert_eq!(error.code(), codes::ENTER_REFUSED);
    assert!(error.message().contains(pending));
}

#[test]
fn arrival_before_a_crossing_is_refused() {
    let expected = common::route_expectations();
    let mut session = common::session();
    common::drive(&mut session, "^^^");
    let error = session
        .enter(
            &common::plan(&expected.route[1].area),
            &common::semantics(&expected.route[1].area),
        )
        .unwrap_err();
    assert_eq!(error.code(), codes::ENTER_REFUSED);
    assert!(error.message().contains("playing"));
}

#[test]
fn the_route_records_a_digest_for_every_area_it_enters() {
    let expected = common::route_expectations();
    let session = common::play_route();
    assert_eq!(session.route().len(), expected.route.len());
    for (row, leg) in session.route().iter().zip(expected.route) {
        assert_eq!(row.area, leg.area);
        assert_eq!(
            row.plan_digest,
            nomos_core::Sha256Digest::of_bytes(&common::plan(&leg.area))
        );
        assert_eq!(
            row.semantics_digest,
            nomos_core::Sha256Digest::of_bytes(&common::semantics(&leg.area))
        );
    }
}

#[test]
fn the_terminal_area_completes_the_run() {
    let expected = common::route_expectations();
    let session = common::play_route();
    assert_eq!(session.outcome(), SessionOutcome::Completed);
    assert_eq!(session.pending_area(), None);
    assert_eq!(session.live().plan.to_area, None);
    assert_eq!(session.areas_cleared(), expected.areas);
}

#[test]
fn reset_starts_a_new_session_and_is_not_a_command() {
    // `RUNTIME.md` section 5 R1-5's Scope: reset starts a new session. Nothing
    // in the log can express it, which is what makes a recorded log replayable
    // as one continuous run.
    let mut session = common::session();
    common::drive(&mut session, "^^^");
    assert_eq!(session.log().len(), 3);

    let start = common::route_expectations().route[0].area.clone();
    let fresh = PlaySession::start(&common::plan(&start), &common::semantics(&start)).unwrap();
    assert!(fresh.log().is_empty());
    assert!(fresh.receipts().is_empty());
    assert_eq!(fresh.live().state.tick, 0);
    assert_eq!(fresh.counters().moves, 0);
    assert_eq!(fresh.receipt_chain_head(), nomos_play::chain_origin());
}

#[test]
fn a_projection_the_plan_did_not_publish_is_refused_before_it_is_decoded() {
    // The first of the decoder's two locks: the projection's bytes must hash to
    // the digest the rendering plan published for `simulation.json`.
    let start = common::route_expectations().route[0].area.clone();
    let other = common::different_area(&start);
    let error = PlaySession::start(&common::plan(&start), &common::semantics(&other)).unwrap_err();
    assert_eq!(error.code(), codes::SEMANTICS_DIGEST);
    assert!(error.message().contains(&start));
}

#[test]
fn a_projection_that_is_not_canonical_is_refused() {
    let start = common::route_expectations().route[0].area.clone();
    let mut bytes = common::semantics(&start);
    bytes.push(b'}');
    let error = PlaySession::start(&common::plan(&start), &bytes).unwrap_err();
    // The digest check fires first, which is the cheaper and more specific of
    // the two refusals.
    assert_eq!(error.code(), codes::SEMANTICS_DIGEST);
}

#[test]
fn the_session_document_carries_one_state_per_entered_area() {
    let expected = common::route_expectations();
    let mut session = common::session();
    common::drive(&mut session, &expected.route[0].keys);
    common::enter(&mut session, &expected.route[1].area);
    let value = session.to_canonical();
    let nomos_core::CanonicalValue::Object(fields) = &value else {
        panic!("a session is an object");
    };
    let areas = fields
        .get(&nomos_core::FieldName::declared("areas"))
        .unwrap();
    let nomos_core::CanonicalValue::Array(rows) = areas else {
        panic!("areas is an array");
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(
        fields
            .get(&nomos_core::FieldName::declared("position"))
            .unwrap()
            .to_canonical_bytes(),
        b"1"
    );
}

#[test]
fn a_step_into_the_next_area_is_refused_until_it_is_entered() {
    let expected = common::route_expectations();
    let mut session = common::session();
    common::drive(&mut session, &expected.route[0].keys);
    let receipt = session
        .step(&common::step(Direction::North))
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::NOT_PLAYING));
}
