//! The pursuit rule, and capture on a fixed log.
//!
//! `crates/nomos-play/src/batch.rs` quotes the rule verbatim. These are the
//! five cases `apps/nomos-viewer/test/play.test.mjs` carried, plus the two the
//! move to Rust made available: that a refused move does not advance the
//! pursuit counter, and that capture ends the area for every later command.

mod common;

use nomos_play::{Direction, Outcome, batch, codes};

/// Walks the behavior fixture's player to `(3, 2)`, one cell south of its
/// brazier, and
/// extinguishes it. From here the pursuit light is out and the gaoler hunts.
fn dark_at_the_brazier() -> nomos_play::PlaySession {
    let mut session = common::behavior_session();
    common::drive(&mut session, "^^>");
    let player = session.live().state.player().cell;
    assert_eq!((player.x(), player.y()), (3, 2));
    let available = batch::available_interactions(session.live()).unwrap();
    assert_eq!(
        available.len(),
        1,
        "only the brazier is within reach of (3, 2): {available:?}"
    );
    assert_eq!(available[0].1.as_str(), "extinguish");
    common::drive(&mut session, "*");
    session
}

#[test]
fn the_gaoler_stays_dormant_while_the_light_is_lit() {
    let mut session = common::behavior_session();
    let start = session.live().state.pursuer().unwrap().cell;
    assert!(!batch::hunting(session.live()).unwrap());
    for _ in 0..6 {
        session.step(&common::step(Direction::West)).unwrap();
    }
    assert_eq!(session.live().state.pursuer().unwrap().cell, start);
    assert_eq!(
        session.live().state.moves_since_step,
        0,
        "a dormant pursuer does not even raise its counter"
    );
}

#[test]
fn the_gaoler_hunts_only_when_the_pursuit_light_is_out() {
    let mut session = common::behavior_session();
    assert!(!batch::hunting(session.live()).unwrap());
    session = dark_at_the_brazier();
    assert!(batch::hunting(session.live()).unwrap());
}

#[test]
fn the_dark_gaoler_advances_every_second_successful_move() {
    let mut session = dark_at_the_brazier();
    let start = session.live().state.pursuer().unwrap().cell;
    assert_eq!((start.x(), start.y()), (5, 3));

    // First accepted move: the counter rises, the pursuer stays.
    session.step(&common::step(Direction::East)).unwrap();
    assert_eq!(session.live().state.moves_since_step, 1);
    assert_eq!(session.live().state.pursuer().unwrap().cell, start);

    // Second: it steps once and the counter resets. The ordering rule puts the
    // player's move first, so the pursuer reads the cell the player has already
    // reached — (3, 2), not the (4, 2) it left.
    session.step(&common::step(Direction::West)).unwrap();
    assert_eq!(session.live().state.moves_since_step, 0);
    let after = session.live().state.pursuer().unwrap().cell;
    assert_eq!(
        (after.x(), after.y()),
        (4, 3),
        "from (5, 3) toward (3, 2): |dx| = 2 > |dy| = 1, so the x axis wins"
    );

    // Both actors moved on this batch, and the collection is ordered by actor
    // identity: `gaoler` before `player`.
    let delta = &session.receipts().last().unwrap().actor_deltas;
    assert_eq!(delta.len(), 2);
    assert_eq!(delta[0].id.to_string(), "gaoler");
    assert_eq!((delta[0].from.x(), delta[0].from.y()), (5, 3));
    assert_eq!((delta[0].to.x(), delta[0].to.y()), (4, 3));
    assert_eq!(delta[1].id.to_string(), "player");
}

#[test]
fn the_player_resolves_before_the_pursuer_reads_the_state() {
    // The ordering rule's load-bearing clause: the pursuer steps against the
    // state the player's action already produced, not the one it started from.
    let mut session = dark_at_the_brazier();
    session.step(&common::step(Direction::East)).unwrap();
    let player_before = session.live().state.player().cell;
    session.step(&common::step(Direction::North)).unwrap();
    let player_after = session.live().state.player().cell;
    let pursuer = session.live().state.pursuer().unwrap().cell;
    assert_ne!(
        (player_before.x(), player_before.y()),
        (player_after.x(), player_after.y())
    );
    // From (5, 3) the pursuer closed on (4, 1), the cell the player reached in
    // this same batch: dx = -1, dy = -2, so |dy| wins and it moves along y.
    assert_eq!((player_after.x(), player_after.y()), (4, 1));
    assert_eq!((pursuer.x(), pursuer.y()), (5, 2));
}

#[test]
fn a_refused_move_does_not_advance_the_pursuit_counter() {
    let mut session = dark_at_the_brazier();
    // West from (3, 2) to (2, 2) is water and legal; the counter rises to 1.
    session.step(&common::step(Direction::West)).unwrap();
    assert_eq!(session.live().state.moves_since_step, 1);
    // West again to (1, 2), then west to (0, 2), then west leaves the lattice
    // where no door is declared: refused.
    session.step(&common::step(Direction::West)).unwrap();
    session.step(&common::step(Direction::West)).unwrap();
    let raised = session.live().state.moves_since_step;
    let where_it_was = session.live().state.pursuer().unwrap().cell;
    let receipt = session
        .step(&common::step(Direction::West))
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::NO_OPENING));
    assert_eq!(session.live().state.moves_since_step, raised);
    assert_eq!(session.live().state.pursuer().unwrap().cell, where_it_was);
}

#[test]
fn the_dark_gaoler_can_catch_and_stop_the_player() {
    let mut session = dark_at_the_brazier();
    let mut guard = 0;
    // Pace east and west on the same two cells. The pursuer closes on every
    // second accepted move and the walk is bounded, so this terminates.
    while session.live().state.outcome == Outcome::Playing {
        let direction = if guard % 2 == 0 {
            Direction::East
        } else {
            Direction::West
        };
        session.step(&common::step(direction)).unwrap();
        guard += 1;
        assert!(
            guard < 40,
            "the pursuer closes in a bounded number of moves"
        );
    }
    assert_eq!(session.live().state.outcome, Outcome::Caught);
    let capture = session.receipts().last().unwrap();
    assert_eq!(capture.outcome_after, Outcome::Caught);
    assert_eq!(
        session.live().state.pursuer().unwrap().cell,
        session.live().state.player().cell
    );

    // Capture ends the area: every later command is refused and still ticks.
    let tick = session.live().state.tick;
    let receipt = session
        .step(&common::step(Direction::North))
        .unwrap()
        .clone();
    assert_eq!(receipt.refusal, Some(codes::NOT_PLAYING));
    assert_eq!(receipt.tick_after, tick + 1);
    assert_eq!(session.outcome(), nomos_play::SessionOutcome::Caught);
}

#[test]
fn the_same_log_produces_the_same_capture() {
    // `RUNTIME.md` section 5 R1-5: the pursuit rule is authoritative and
    // deterministic, so the same command log produces the same capture outcome.
    let capture = || {
        let mut session = dark_at_the_brazier();
        for index in 0..12 {
            let direction = if index % 2 == 0 {
                Direction::East
            } else {
                Direction::West
            };
            session.step(&common::step(direction)).unwrap();
        }
        (
            session.live().state.outcome,
            session.live().state.tick,
            session.receipt_chain_head(),
        )
    };
    assert_eq!(capture(), capture());
    assert_eq!(capture().0, Outcome::Caught);
}

#[test]
fn a_crossing_does_not_offer_the_pursuer_a_step() {
    let mut session = common::behavior_session();
    common::drive(&mut session, "^^^^>>**>");
    let before = session.live().state.pursuer().unwrap().cell;
    let counter = session.live().state.moves_since_step;
    session.step(&common::step(Direction::North)).unwrap();
    assert_eq!(session.live().state.outcome, Outcome::Escaped);
    assert_eq!(session.live().state.pursuer().unwrap().cell, before);
    assert_eq!(session.live().state.moves_since_step, counter);
}
