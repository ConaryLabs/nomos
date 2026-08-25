//! The `PL####` code table, and the one error type this crate returns.
//!
//! Two ranges carry meaning, and the split is the one `batch.rs` depends on:
//!
//! * **`PL01##` and `PL02##` are shape refusals.** A document that is not a
//!   `nomos.play_command@1`, a plan this runtime cannot read, a session whose
//!   digests do not match the content it is replayed against. These produce no
//!   receipt and do not advance the tick, because a document that is not an
//!   input is not an input.
//! * **`PL03##` and `PL04##` are rule refusals.** A well-formed command the
//!   rules decline: a move into masonry, an interaction where nothing responds,
//!   a crossing at a sealed gate. These commit a batch, advance the tick, and
//!   are recorded in the receipt with `accepted: false`. `docs/review/nomos-play.md`
//!   section 3.5 records why.

use std::fmt;

use nomos_core::Diagnostic;

/// Stable refusal codes.
pub mod codes {
    /// A document names a schema this runtime does not accept.
    pub const SCHEMA_MISMATCH: &str = "PL0101";
    /// A play state and a rendering plan describe different areas.
    pub const AREA_MISMATCH: &str = "PL0102";
    /// An actor collection is not exactly one player and at most one pursuer.
    pub const ACTORS_INVALID: &str = "PL0103";
    /// A document's field set or value shape is wrong.
    pub const DOCUMENT_SHAPE: &str = "PL0104";
    /// A document's value is out of range or not an accepted spelling.
    pub const DOCUMENT_VALUE: &str = "PL0105";

    /// A command's field set does not match its declared kind.
    pub const COMMAND_SHAPE: &str = "PL0201";
    /// A command names an entity, gate, or direction this area does not declare.
    pub const COMMAND_TARGET: &str = "PL0202";

    /// The run is over: the outcome is not `playing`.
    pub const NOT_PLAYING: &str = "PL0301";
    /// The target cell is inside an architectural mass.
    pub const MASONRY: &str = "PL0302";
    /// An entity covering the target cell is not traversable at this state.
    pub const BLOCKED: &str = "PL0303";
    /// Another actor stands on the target cell.
    pub const OCCUPIED: &str = "PL0304";
    /// The resolved transition requires a credential this runtime cannot supply.
    pub const CREDENTIAL_UNSUPPORTED: &str = "PL0305";
    /// The move leaves the lattice where no traversable door is declared.
    pub const NO_OPENING: &str = "PL0306";
    /// Nothing within reach of the player responds to this command.
    pub const NOTHING_IN_REACH: &str = "PL0307";
    /// The kernel refused the command.
    pub const KERNEL_REFUSED: &str = "PL0308";

    /// Arrival was offered where the session cannot accept it.
    pub const ENTER_REFUSED: &str = "PL0401";
    /// A replay was pointed at content whose digests the session does not name.
    pub const CONTENT_MISMATCH: &str = "PL0402";
    /// A replay diverged from the session it was checking.
    pub const REPLAY_DIVERGED: &str = "PL0403";

    /// The simulation projection could not be reconstructed.
    pub const SEMANTICS_INVALID: &str = "PL0501";
    /// The simulation projection is not the one the rendering plan published.
    pub const SEMANTICS_DIGEST: &str = "PL0502";
}

/// One refusal, with the stable code a receipt records.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlayError {
    code: &'static str,
    message: String,
}

impl PlayError {
    /// Builds one refusal.
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Wraps a kernel diagnostic, keeping its code inside the message so the
    /// receipt records which kernel rule refused.
    #[must_use]
    pub fn from_kernel(code: &'static str, error: &Diagnostic) -> Self {
        Self::new(
            code,
            format!("{} {}", error.code().as_str(), error.message()),
        )
    }

    /// The stable `PL####` code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// The human-readable refusal.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Whether this refusal commits a batch and advances the tick.
    ///
    /// `PL03##` and `PL04##` are decisions the authority made about a
    /// well-formed input; `PL01##`, `PL02##`, and `PL05##` mean there was no
    /// input to decide about.
    #[must_use]
    pub fn is_rule_refusal(&self) -> bool {
        matches!(&self.code[..4], "PL03" | "PL04")
    }
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code, self.message)
    }
}

impl std::error::Error for PlayError {}

/// The crate's result alias.
pub type PlayResult<T> = Result<T, PlayError>;
