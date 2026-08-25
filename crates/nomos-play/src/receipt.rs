//! `nomos.play_receipt@1`: one per batch, always.
//!
//! This is the owner file for that identity, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.
//!
//! A batch the rules refused still produces a receipt, with `accepted: false`
//! and the `PL####` code that refused it. `docs/review/nomos-play.md` section
//! 3.5 records why: the smoke lane records what the browser did and replays it
//! natively, and a divergence in the rules shows up first as one side accepting
//! what the other refuses. A log with the refusals dropped cannot prove that.
//!
//! # The chain
//!
//! A receipt's own hash is `sha256(receipt.to_canonical_bytes())` and is
//! deliberately **not** a field of the receipt: a canonical document cannot
//! carry its own digest. The kernel takes the same position —
//! `CausalReceipt::digest()` is derived, not stored. The chain link is
//! `previous_receipt_hash` on the *next* receipt, 64 zeros at ordinal 0, and
//! the chain's head is the session's `receipt_chain_head`.

use nomos_core::canonical::keyed_array;
use nomos_core::id::{EntityId, SchemaId};
use nomos_core::{CanonicalValue, Sha256Digest, StateHash};
use nomos_projection::LatticeCell;

use crate::command::PlayCommand;
use crate::error::PlayResult;
use crate::state::{Counters, Outcome, cell_value};

/// The receipt identity.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn play_receipt_schema() -> SchemaId {
    SchemaId::new("nomos.play_receipt", 1).expect("the play-receipt schema id is a literal")
}

/// The all-zero digest that opens a chain.
///
/// # Panics
///
/// Panics only if 64 ASCII zeros stop being a legal SHA-256 hex spelling.
#[must_use]
pub fn chain_origin() -> Sha256Digest {
    Sha256Digest::from_hex(&"0".repeat(64)).expect("64 zeros is a legal digest spelling")
}

/// One actor that moved during a batch.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ActorDelta {
    /// Stable actor identity; the collection is ordered by it.
    pub id: EntityId,
    /// Cell before the batch.
    pub from: LatticeCell,
    /// Cell after the batch.
    pub to: LatticeCell,
}

/// The evidence one batch produced.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlayReceipt {
    /// 0-based index in the session's receipt array.
    pub ordinal: u64,
    /// The area the batch ran in.
    pub area: String,
    /// The input, verbatim.
    pub input: PlayCommand,
    /// Whether the batch changed anything but the tick.
    pub accepted: bool,
    /// The `PL####` code when it did not.
    pub refusal: Option<&'static str>,
    /// Tick before the batch.
    pub tick_before: u64,
    /// Tick after the batch; always `tick_before + 1`.
    pub tick_after: u64,
    /// Kernel state hash before the batch.
    pub kernel_state_hash_before: StateHash,
    /// Kernel state hash after the batch.
    pub kernel_state_hash_after: StateHash,
    /// Actors whose cell changed, in ascending identity order.
    pub actor_deltas: Vec<ActorDelta>,
    /// Outcome before the batch.
    pub outcome_before: Outcome,
    /// Outcome after the batch.
    pub outcome_after: Outcome,
    /// Cumulative counters after the batch.
    pub counters_after: Counters,
    /// The previous receipt's hash, or 64 zeros at ordinal 0.
    pub previous_receipt_hash: Sha256Digest,
    /// The hash of the play state the batch produced.
    pub play_state_hash_after: StateHash,
}

impl PlayReceipt {
    /// The receipt as a canonical value.
    ///
    /// # Panics
    ///
    /// Panics if two deltas share an actor identity, which the reducer cannot
    /// produce: it emits at most one delta per actor.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("accepted", CanonicalValue::Bool(self.accepted)),
            (
                "actor_deltas",
                keyed_array(self.actor_deltas.iter().map(|delta| {
                    (
                        delta.id.clone(),
                        CanonicalValue::object_declared([
                            ("from", cell_value(delta.from)),
                            ("id", CanonicalValue::text(delta.id.to_string())),
                            ("to", cell_value(delta.to)),
                        ]),
                    )
                }))
                .expect("a receipt carries at most one delta per actor"),
            ),
            ("area", CanonicalValue::text(self.area.clone())),
            (
                "counters_after",
                CanonicalValue::object_declared([
                    ("moves", CanonicalValue::Uint(self.counters_after.moves)),
                    (
                        "traversal_cost",
                        CanonicalValue::Uint(self.counters_after.traversal_cost),
                    ),
                ]),
            ),
            ("input", self.input.to_canonical()),
            (
                "kernel_state_hash_after",
                CanonicalValue::text(self.kernel_state_hash_after.to_hex()),
            ),
            (
                "kernel_state_hash_before",
                CanonicalValue::text(self.kernel_state_hash_before.to_hex()),
            ),
            ("ordinal", CanonicalValue::Uint(self.ordinal)),
            (
                "outcome_after",
                CanonicalValue::text(self.outcome_after.as_str()),
            ),
            (
                "outcome_before",
                CanonicalValue::text(self.outcome_before.as_str()),
            ),
            (
                "play_state_hash_after",
                CanonicalValue::text(self.play_state_hash_after.to_hex()),
            ),
            (
                "previous_receipt_hash",
                CanonicalValue::text(self.previous_receipt_hash.to_hex()),
            ),
            (
                "refusal",
                self.refusal
                    .map_or(CanonicalValue::Null, CanonicalValue::text),
            ),
            (
                "schema",
                CanonicalValue::text(play_receipt_schema().to_string()),
            ),
            ("tick_after", CanonicalValue::Uint(self.tick_after)),
            ("tick_before", CanonicalValue::Uint(self.tick_before)),
        ])
    }

    /// The receipt as canonical bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    /// The chain link the next receipt carries.
    #[must_use]
    pub fn hash(&self) -> Sha256Digest {
        Sha256Digest::of_bytes(&self.to_canonical_bytes())
    }
}

/// Reads one receipt back for comparison during replay.
///
/// Replay compares receipts by canonical bytes, so this exists only to report
/// *which* field diverged; it is deliberately lenient about nothing.
///
/// # Errors
///
/// Returns `PL0101` for another identity and `PL0104` for a shape this runtime
/// cannot read.
pub fn field_names(value: &CanonicalValue) -> PlayResult<Vec<String>> {
    let fields = crate::read::object(value, "play receipt")?;
    crate::read::bind_schema(fields, &play_receipt_schema(), "play receipt")?;
    Ok(fields.keys().map(|name| name.as_str().to_owned()).collect())
}
