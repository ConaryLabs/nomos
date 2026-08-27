//! Stable R2 compiler diagnostics and their canonical command envelope.

use std::fmt;

use nomos_core::{CanonicalValue, FieldName, RepairClass, SourceSpan};

/// One stable `OS####` diagnostic code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ObservedCode(&'static str);

impl ObservedCode {
    /// The stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// Whether the code has the required `OS` plus four-digit shape.
    #[must_use]
    pub fn is_well_formed(self) -> bool {
        let bytes = self.0.as_bytes();
        bytes.len() == 6
            && bytes[0] == b'O'
            && bytes[1] == b'S'
            && bytes[2..].iter().all(u8::is_ascii_digit)
    }
}

impl fmt::Display for ObservedCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// The complete R2-1 diagnostic vocabulary.
pub mod codes {
    use super::ObservedCode;

    /// Invalid command or argument grammar.
    pub const USAGE: ObservedCode = ObservedCode("OS0001");
    /// Input is unreadable, symlinked, or not one regular file.
    pub const INPUT_UNREADABLE: ObservedCode = ObservedCode("OS0101");
    /// Input is malformed UTF-8 or JSON.
    pub const INPUT_MALFORMED: ObservedCode = ObservedCode("OS0102");
    /// Input bytes or a required array are not canonical.
    pub const INPUT_NOT_CANONICAL: ObservedCode = ObservedCode("OS0103");
    /// Schema identity or version is absent or mismatched.
    pub const SCHEMA_MISMATCH: ObservedCode = ObservedCode("OS0104");
    /// A field is missing, unknown, repeated, or wrong-typed.
    pub const FIELD_INVALID: ObservedCode = ObservedCode("OS0201");
    /// A crop, count, integer, cell, or collection bound is violated.
    pub const BOUND_INVALID: ObservedCode = ObservedCode("OS0202");
    /// A scene-local identity is malformed or duplicated.
    pub const IDENTITY_INVALID: ObservedCode = ObservedCode("OS0203");
    /// An action target does not name an actor.
    pub const TARGET_DANGLING: ObservedCode = ObservedCode("OS0204");
    /// Output exists, aliases input, or traverses a symlink.
    pub const OUTPUT_UNAVAILABLE: ObservedCode = ObservedCode("OS0301");
    /// Staging, writing, syncing, or publication failed.
    pub const OUTPUT_IO: ObservedCode = ObservedCode("OS0302");

    /// Every stable code, in lexical order.
    pub const ALL: [ObservedCode; 11] = [
        USAGE,
        INPUT_UNREADABLE,
        INPUT_MALFORMED,
        INPUT_NOT_CANONICAL,
        SCHEMA_MISMATCH,
        FIELD_INVALID,
        BOUND_INVALID,
        IDENTITY_INVALID,
        TARGET_DANGLING,
        OUTPUT_UNAVAILABLE,
        OUTPUT_IO,
    ];
}

/// One strict compiler rejection.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ObservedError {
    code: ObservedCode,
    message: String,
    span: Option<SourceSpan>,
    repairs: Vec<RepairClass>,
}

impl ObservedError {
    /// Builds a rejection without a span or repair suggestion.
    #[must_use]
    pub fn new(code: ObservedCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            span: None,
            repairs: Vec::new(),
        }
    }

    /// Adds source location evidence.
    #[must_use]
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    pub(crate) fn with_default_span(mut self, span: SourceSpan) -> Self {
        if self.span.is_none() {
            self.span = Some(span);
        }
        self
    }

    /// Adds one existing Nomos repair spelling, sorted and deduplicated.
    #[must_use]
    pub fn with_repair(mut self, repair: RepairClass) -> Self {
        if let Err(index) = self.repairs.binary_search(&repair) {
            self.repairs.insert(index, repair);
        }
        self
    }

    /// The stable code.
    #[must_use]
    pub const fn code(&self) -> ObservedCode {
        self.code
    }

    /// The non-contractual human-facing message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The optional source span.
    #[must_use]
    pub fn span(&self) -> Option<&SourceSpan> {
        self.span.as_ref()
    }

    /// Sorted, duplicate-free repair classes.
    #[must_use]
    pub fn repairs(&self) -> &[RepairClass] {
        &self.repairs
    }

    /// The canonical diagnostic object.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        let mut fields = vec![
            (
                FieldName::declared("code"),
                CanonicalValue::text(self.code.as_str()),
            ),
            (
                FieldName::declared("message"),
                CanonicalValue::text(&self.message),
            ),
            (
                FieldName::declared("repairs"),
                CanonicalValue::Array(
                    self.repairs
                        .iter()
                        .map(|repair| CanonicalValue::text(repair.as_str()))
                        .collect(),
                ),
            ),
        ];
        if let Some(span) = &self.span {
            fields.push((FieldName::declared("span"), span.to_canonical()));
        }
        CanonicalValue::object(fields).expect("diagnostic fields are unique")
    }
}

impl fmt::Display for ObservedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ObservedError {}

/// The crate result type.
pub type ObservedResult<T> = Result<T, ObservedError>;

/// Renders one failure in the exact unpersisted rejection envelope.
#[must_use]
pub fn render_rejection(error: &ObservedError) -> Vec<u8> {
    let value = CanonicalValue::object_declared([
        (
            "diagnostics",
            CanonicalValue::Array(vec![error.to_canonical()]),
        ),
        ("status", CanonicalValue::text("rejected")),
    ]);
    let mut bytes = value.to_canonical_bytes();
    bytes.push(b'\n');
    bytes
}
