//! The compiler's stable diagnostics.
//!
//! `nomos-core` owns the Gate K `EK####` diagnostic space and
//! `DiagnosticCode::parse` resolves only codes listed in
//! `nomos_core::diagnostic::codes::ALL`. Minting `EK` codes from an R1 crate
//! would put new spellings into that frozen space, which `RUNTIME.md` section 3
//! forbids ("no Gate K … diagnostic changes"). This crate therefore carries its
//! own `RP####` space: stable, documented here, and disjoint from `EK` by its
//! prefix.
//!
//! Every rejection is a code plus a message that names what was expected and
//! what was found, so a caller can assert on either.

use std::fmt;
use std::path::{Path, PathBuf};

/// A stable rendering-plan diagnostic code.
///
/// The `RP` prefix is deliberate: `nomos-core`'s `DiagnosticCode` is `EK` plus
/// four digits, so no `RP` code can ever collide with a Gate K code or be
/// mistaken for one in a log.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PlanCode(&'static str);

impl PlanCode {
    /// The code as text, for example `"RP0101"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for PlanCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// The stable code spellings this crate emits.
pub mod codes {
    use super::PlanCode;

    /// A required input file is absent or unreadable.
    pub const INPUT_UNREADABLE: PlanCode = PlanCode("RP0101");
    /// An input document is not canonical bytes.
    pub const INPUT_NOT_CANONICAL: PlanCode = PlanCode("RP0102");
    /// An input document is not well-formed JSON.
    pub const INPUT_MALFORMED: PlanCode = PlanCode("RP0103");
    /// A document carries a schema identity or version the compiler does not
    /// accept.
    pub const SCHEMA_MISMATCH: PlanCode = PlanCode("RP0104");
    /// A document is missing a required field, or a field has the wrong shape.
    pub const DOCUMENT_SHAPE: PlanCode = PlanCode("RP0105");
    /// The command line is not the declared shape.
    pub const USAGE: PlanCode = PlanCode("RP0106");

    /// The catalog declares a primitive the compiler has no kind for, or a
    /// kind whose capability set contradicts its primitive.
    pub const CLASSIFICATION_UNSOUND: PlanCode = PlanCode("RP0201");
    /// The presentation source violates a bounded-area invariant, or its shape
    /// is not the one `nomos.presentation_source@2` declares.
    pub const AREA_INVALID: PlanCode = PlanCode("RP0202");
    /// A scenario did not reach its declared state.
    pub const SCENARIO_INCOMPLETE: PlanCode = PlanCode("RP0203");
    /// The facts directory and the runs directory disagree about the scenario
    /// set.
    pub const SCENARIO_SET_MISMATCH: PlanCode = PlanCode("RP0204");
    /// A presentation number is not a base-10 integer: it carries a fraction,
    /// an exponent, a leading `+`, a redundant leading zero, or does not fit.
    ///
    /// `RUNTIME.md` section 5 R1-3 forbids a raw floating-point transform in
    /// accepted content, so this fires on the lexeme, before any field is
    /// interpreted, and at any depth in the file.
    pub const NUMBER_UNSUPPORTED: PlanCode = PlanCode("RP0205");
    /// An identifier in the presentation source is outside the grammar its
    /// field declares.
    pub const IDENTIFIER_UNSUPPORTED: PlanCode = PlanCode("RP0206");

    /// The area collection's route graph is not one chain: it starts nowhere or
    /// twice, leads to an area that is not declared or cannot receive an
    /// arrival, cycles, leaves an area unvisited, or does not terminate at one
    /// area declaring no destination.
    pub const COLLECTION_ROUTE_INVALID: PlanCode = PlanCode("RP0301");
    /// Two areas in one collection do not share the visual grammar.
    pub const COLLECTION_GRAMMAR_DIVERGED: PlanCode = PlanCode("RP0302");
}

/// A rejection, with the file that produced it when there is one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlanError {
    code: PlanCode,
    message: String,
    path: Option<PathBuf>,
}

impl PlanError {
    /// Builds a rejection.
    #[must_use]
    pub fn new(code: PlanCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
        }
    }

    /// Attaches the input path the rejection came from.
    #[must_use]
    pub fn at(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// The stable code.
    #[must_use]
    pub const fn code(&self) -> PlanCode {
        self.code
    }

    /// The message, without the code or path prefix.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The input path, when the rejection names one.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(f, "{}: {}: {}", self.code, path.display(), self.message),
            None => write!(f, "{}: {}", self.code, self.message),
        }
    }
}

impl std::error::Error for PlanError {}

/// The crate's result alias.
pub type PlanResult<T> = Result<T, PlanError>;
