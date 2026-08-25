//! Replay identity, and the proof that the comparison can fail.
//!
//! `RUNTIME.md` section 5 R1-5: replaying a committed command log yields
//! byte-identical receipts and a byte-identical final state hash.

mod common;

use nomos_play::{PlayResult, RecordedSession, codes, replay};

fn content(area: &str) -> PlayResult<(Vec<u8>, Vec<u8>)> {
    Ok((common::plan(area), common::semantics(area)))
}

fn recorded() -> (Vec<u8>, RecordedSession) {
    let session = common::play_route();
    let bytes = session.to_canonical_bytes();
    let recorded = RecordedSession::decode(&bytes).expect("a session reads back");
    (bytes, recorded)
}

#[test]
fn replaying_a_committed_log_is_byte_identical_across_ten_runs() {
    let (bytes, session) = recorded();
    assert!(!session.log.is_empty(), "the log is not empty");
    assert_ne!(
        session.receipt_chain_head,
        nomos_play::chain_origin(),
        "the chain head is not the all-zero origin"
    );

    let mut results = Vec::new();
    for _ in 0..10 {
        let report = replay(&session, content).expect("the replay runs");
        assert!(
            report.passed(),
            "the replay diverged: {:?}",
            report.divergence
        );
        results.push((
            report.chain_head,
            report.final_kernel_state_hash,
            report.session.to_canonical_bytes(),
        ));
    }
    let first = &results[0];
    for other in &results[1..] {
        assert_eq!(other, first, "ten replays produce identical bytes");
    }
    assert_eq!(first.2, bytes, "the replay reproduces the recorded session");
}

#[test]
fn the_receipt_chain_links_every_batch_to_the_one_before() {
    let session = common::play_route();
    let mut previous = nomos_play::chain_origin();
    for (index, receipt) in session.receipts().iter().enumerate() {
        assert_eq!(receipt.ordinal, index as u64);
        assert_eq!(receipt.previous_receipt_hash, previous);
        previous = receipt.hash();
    }
    assert_eq!(session.receipt_chain_head(), previous);
}

#[test]
fn a_tampered_receipt_is_caught_at_its_ordinal() {
    let (bytes, _) = recorded();
    let text = String::from_utf8(bytes).unwrap();
    // Move one accepted receipt's `moves` counter by one. Every later receipt
    // still carries the recorded bytes, so the first difference is exactly here.
    let needle = r#""counters_after":{"moves":7,"#;
    assert!(
        text.contains(needle),
        "the recorded run passes through 7 moves"
    );
    let tampered = text.replacen(needle, r#""counters_after":{"moves":8,"#, 1);
    let session = RecordedSession::decode(tampered.as_bytes()).unwrap();

    let report = replay(&session, content).expect("the replay runs");
    assert!(!report.passed());
    let divergence = report.divergence.expect("a divergence is reported");
    assert_eq!(divergence.field, "receipt_bytes");
    assert_eq!(
        divergence.ordinal,
        Some(6),
        "the seventh batch is ordinal 6"
    );
    let start = common::route_expectations().route[0].area.clone();
    assert_eq!(divergence.area.as_deref(), Some(start.as_str()));
}

#[test]
fn a_replay_against_different_content_is_refused_not_reported_as_a_divergence() {
    // A content mismatch is a harness error. Reporting it as a runtime
    // difference would be a lie about what diverged.
    let (_, session) = recorded();
    let start = common::route_expectations().route[0].area.clone();
    let other = common::different_area(&start);
    let error = replay(&session, |area| {
        let selected = if area == start { other.as_str() } else { area };
        Ok((common::plan(selected), common::semantics(selected)))
    })
    .unwrap_err();
    assert_eq!(error.code(), codes::CONTENT_MISMATCH);
    assert!(error.message().contains(&start));
}

#[test]
fn the_recorded_log_carries_the_refusals_too() {
    // The reason a refused batch commits: the recorded log is what the smoke
    // lane replays, and a log with the refusals dropped could not show that the
    // browser refused the same inputs at the same ticks.
    let area = common::behavior_area();
    let mut session = common::session_at(&area);
    let keys = common::keys_for(&area);
    let before_interaction = keys
        .chars()
        .take_while(|key| *key != '*')
        .collect::<String>();
    common::drive(&mut session, &before_interaction);
    session
        .step(&common::step(nomos_play::Direction::North))
        .unwrap();
    assert_eq!(
        session.receipts().last().unwrap().refusal,
        Some(codes::NO_OPENING)
    );

    let bytes = session.to_canonical_bytes();
    let recorded = RecordedSession::decode(&bytes).unwrap();
    assert_eq!(recorded.log.len(), 7);
    let report = replay(&recorded, content).unwrap();
    assert!(report.passed(), "{:?}", report.divergence);
    assert_eq!(report.receipts, 7);
}

#[test]
fn a_replay_reproduces_the_final_kernel_state_hash() {
    let expected = common::route_expectations();
    let live = common::play_route();
    let (_, session) = recorded();
    let report = replay(&session, content).unwrap();
    assert_eq!(
        report.final_kernel_state_hash,
        live.live().state.kernel_state_hash(),
        "replay reaches the recorded route's final kernel state"
    );
    assert_eq!(report.areas, usize::try_from(expected.areas).unwrap());
    assert_eq!(report.commands, usize::try_from(expected.commands).unwrap());
}

#[test]
fn a_session_that_names_no_area_is_refused() {
    let empty = br#"{"areas":[],"areas_cleared":0,"log":[],"outcome":"playing","position":0,"receipt_chain_head":"0000000000000000000000000000000000000000000000000000000000000000","receipts":[],"route":[],"schema":"nomos.play_session@1"}"#;
    let session = RecordedSession::decode(empty).unwrap();
    let error = replay(&session, content).unwrap_err();
    assert_eq!(error.code(), codes::CONTENT_MISMATCH);
}
