//! Replaying a recorded session, and comparing what it produces.
//!
//! The comparison is over canonical bytes, receipt by receipt, and it reports
//! the **first** difference with its ordinal rather than "the final hash
//! differs". A recorded session names the content digests it was played
//! against, and a replay pointed at different content is refused `PL0402`
//! before it starts: a mismatch there is a harness error, not a runtime
//! difference, and it must not be reported as one.

use nomos_core::{Sha256Digest, StateHash};

use crate::error::{PlayError, PlayResult, codes};
use crate::receipt::PlayReceipt;
use crate::session::{PlaySession, RecordedSession};

/// The first place a replay disagreed with what it was checking.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Divergence {
    /// The receipt ordinal, when the difference is inside one.
    pub ordinal: Option<u64>,
    /// Which comparison failed.
    pub field: &'static str,
    /// The area the difference is in, when it is known.
    pub area: Option<String>,
    /// What differed.
    pub detail: String,
}

/// What a replay produced.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReplayReport {
    /// Areas entered.
    pub areas: usize,
    /// Inputs replayed.
    pub commands: usize,
    /// Receipts produced.
    pub receipts: usize,
    /// The chain head the replay produced.
    pub chain_head: Sha256Digest,
    /// The live area's kernel state hash at the end.
    pub final_kernel_state_hash: StateHash,
    /// The first disagreement, if any.
    pub divergence: Option<Divergence>,
    /// The session the replay produced, for a caller that wants to emit it.
    pub session: PlaySession,
}

impl ReplayReport {
    /// Whether the replay agreed with the session it was checking.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.divergence.is_none()
    }
}

/// Re-executes a recorded session's log against content the caller supplies.
///
/// `content` is asked for one area's `(rendering plan bytes, simulation
/// projection bytes)`.
///
/// # Errors
///
/// Returns `PL0402` when the content does not match the digests the session
/// names, and any shape refusal the log produces.
pub fn replay<F>(recorded: &RecordedSession, content: F) -> PlayResult<ReplayReport>
where
    F: Fn(&str) -> PlayResult<(Vec<u8>, Vec<u8>)>,
{
    if recorded.route.is_empty() {
        return Err(PlayError::new(
            codes::CONTENT_MISMATCH,
            "a recorded session names at least one area",
        ));
    }

    let mut fetched = Vec::with_capacity(recorded.route.len());
    for row in &recorded.route {
        let (plan_bytes, semantics_bytes) = content(&row.area)?;
        let plan_digest = Sha256Digest::of_bytes(&plan_bytes);
        let semantics_digest = Sha256Digest::of_bytes(&semantics_bytes);
        if plan_digest != row.plan_digest {
            return Err(PlayError::new(
                codes::CONTENT_MISMATCH,
                format!(
                    "`{}` was recorded against plan {} but the content here is {}",
                    row.area,
                    row.plan_digest.to_hex(),
                    plan_digest.to_hex()
                ),
            ));
        }
        if semantics_digest != row.semantics_digest {
            return Err(PlayError::new(
                codes::CONTENT_MISMATCH,
                format!(
                    "`{}` was recorded against simulation.json {} but the content here is {}",
                    row.area,
                    row.semantics_digest.to_hex(),
                    semantics_digest.to_hex()
                ),
            ));
        }
        fetched.push((plan_bytes, semantics_bytes));
    }

    let mut session = PlaySession::start(&fetched[0].0, &fetched[0].1)?;
    let mut entered = 1_usize;
    for input in &recorded.log {
        session.step(input)?;
        if session.pending_area().is_some() && entered < fetched.len() {
            let (plan_bytes, semantics_bytes) = &fetched[entered];
            session.enter(plan_bytes, semantics_bytes)?;
            entered += 1;
        }
    }

    let divergence = compare(recorded, &session);
    Ok(ReplayReport {
        areas: entered,
        commands: recorded.log.len(),
        receipts: session.receipts().len(),
        chain_head: session.receipt_chain_head(),
        final_kernel_state_hash: session.live().state.kernel_state_hash(),
        divergence,
        session,
    })
}

fn compare(recorded: &RecordedSession, session: &PlaySession) -> Option<Divergence> {
    if recorded.receipts.len() != session.receipts().len() {
        return Some(Divergence {
            ordinal: None,
            field: "receipt_count",
            area: None,
            detail: format!(
                "recorded {} receipts; the replay produced {}",
                recorded.receipts.len(),
                session.receipts().len()
            ),
        });
    }
    for (index, (expected, actual)) in recorded
        .receipts
        .iter()
        .zip(session.receipts().iter())
        .enumerate()
    {
        let expected_bytes = expected.to_canonical_bytes();
        let actual_bytes = actual.to_canonical_bytes();
        if expected_bytes != actual_bytes {
            return Some(Divergence {
                ordinal: Some(index as u64),
                field: "receipt_bytes",
                area: Some(actual.area.clone()),
                detail: first_difference(&expected_bytes, &actual_bytes),
            });
        }
    }
    if recorded.receipt_chain_head != session.receipt_chain_head() {
        return Some(Divergence {
            ordinal: None,
            field: "receipt_chain_head",
            area: None,
            detail: format!(
                "recorded {}; the replay produced {}",
                recorded.receipt_chain_head.to_hex(),
                session.receipt_chain_head().to_hex()
            ),
        });
    }
    if recorded.areas_cleared != session.areas_cleared() {
        return Some(Divergence {
            ordinal: None,
            field: "areas_cleared",
            area: None,
            detail: format!(
                "recorded {}; the replay produced {}",
                recorded.areas_cleared,
                session.areas_cleared()
            ),
        });
    }
    if recorded.outcome != session.outcome().as_str() {
        return Some(Divergence {
            ordinal: None,
            field: "outcome",
            area: None,
            detail: format!(
                "recorded `{}`; the replay produced `{}`",
                recorded.outcome,
                session.outcome().as_str()
            ),
        });
    }
    let produced = session.to_canonical();
    let produced_areas = match &produced {
        nomos_core::CanonicalValue::Object(fields) => fields
            .get(&nomos_core::FieldName::declared("areas"))
            .cloned()
            .unwrap_or(nomos_core::CanonicalValue::Null),
        _ => nomos_core::CanonicalValue::Null,
    };
    let expected_areas = nomos_core::CanonicalValue::Array(recorded.areas.clone());
    if produced_areas.to_canonical_bytes() != expected_areas.to_canonical_bytes() {
        return Some(Divergence {
            ordinal: None,
            field: "areas",
            area: None,
            detail: first_difference(
                &expected_areas.to_canonical_bytes(),
                &produced_areas.to_canonical_bytes(),
            ),
        });
    }
    None
}

/// A short, deterministic description of where two byte strings first differ.
fn first_difference(expected: &[u8], actual: &[u8]) -> String {
    let at = expected
        .iter()
        .zip(actual.iter())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| expected.len().min(actual.len()));
    let window = |bytes: &[u8]| {
        let start = at.saturating_sub(24);
        let end = (at + 24).min(bytes.len());
        String::from_utf8_lossy(&bytes[start..end]).into_owned()
    };
    format!(
        "byte {at}: recorded `{}` vs replayed `{}`",
        window(expected),
        window(actual)
    )
}

/// A receipt's hash, for a caller building a chain by hand.
#[must_use]
pub fn hash_of(receipt: &PlayReceipt) -> Sha256Digest {
    receipt.hash()
}
