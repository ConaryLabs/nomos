//! Structured diagnostics.
//!
//! `KERNEL.md` section 9 fixes the shape: a stable code, a message, a source
//! span where source exists, and the legal repair classes. Wording may improve;
//! the code's meaning may not change.
//!
//! Diagnostics are deliberately excluded from the authoritative state hash
//! (section 7), so a message improvement can never move a hash.

use core::fmt;

use crate::canonical::{CanonicalValue, FieldName};

/// A stable diagnostic code.
///
/// The code — not the message — is the contract. Codes are `EK` followed by
/// four decimal digits and are allocated per owning area:
///
/// | Range | Area |
/// | --- | --- |
/// | `EK01xx` | identifiers and stable IDs |
/// | `EK02xx` | checked arithmetic |
/// | `EK03xx` | canonical encoding |
/// | `EK04xx` | world packages |
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    /// Declares a code. Well-formedness is asserted by test, not by the type
    /// system, because `const fn` cannot yet return a checked error here.
    #[must_use]
    pub const fn new(code: &'static str) -> Self {
        Self(code)
    }

    /// The code as text, for example `"EK0101"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// Whether the code matches the `EK` + four-digit shape.
    #[must_use]
    pub fn is_well_formed(self) -> bool {
        let bytes = self.0.as_bytes();
        bytes.len() == 6
            && bytes[0] == b'E'
            && bytes[1] == b'K'
            && bytes[2..].iter().all(u8::is_ascii_digit)
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// A class of legal repair for a rejected fact, command, or artifact.
///
/// A repair class tells an author what kind of edit is legal, without the
/// kernel pretending to know the author's intent.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum RepairClass {
    /// Use only the supported identifier characters and shape.
    UseSupportedIdentifierShape,
    /// Reduce the magnitude of an operand so the result fits its integer type.
    ReduceOperandMagnitude,
    /// Emit the value under the canonical byte profile.
    EmitCanonicalBytes,
    /// Supply a required member that is missing.
    SupplyMissingMember,
    /// Remove a member that is not declared by the manifest.
    RemoveUndeclaredMember,
    /// Write the output to a path that does not already exist.
    WriteToNewOutputPath,
    /// Rebuild the artifact from its source rather than editing it in place.
    RebuildFromSource,
}

impl RepairClass {
    /// The stable wire spelling of this repair class.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UseSupportedIdentifierShape => "use_supported_identifier_shape",
            Self::ReduceOperandMagnitude => "reduce_operand_magnitude",
            Self::EmitCanonicalBytes => "emit_canonical_bytes",
            Self::SupplyMissingMember => "supply_missing_member",
            Self::RemoveUndeclaredMember => "remove_undeclared_member",
            Self::WriteToNewOutputPath => "write_to_new_output_path",
            Self::RebuildFromSource => "rebuild_from_source",
        }
    }
}

impl fmt::Display for RepairClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A repository-relative source path.
///
/// Absolute paths are refused at construction: `KERNEL.md` section 7 excludes
/// absolute paths from hashed material, and the cheapest way to keep them out
/// is to make them unrepresentable in a diagnostic.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SourcePath(String);

impl SourcePath {
    /// Accepts a repository-relative path.
    ///
    /// # Errors
    ///
    /// Returns [`Diagnostic`] `EK0102` for an empty path, an absolute path, a
    /// Windows-style drive path, or a path containing a `..` component.
    pub fn new(path: &str) -> Result<Self, Diagnostic> {
        let rejected = path.is_empty()
            || path.starts_with('/')
            || path.starts_with('\\')
            || path.split(['/', '\\']).any(|segment| segment == "..")
            || path.as_bytes().get(1) == Some(&b':');
        if rejected {
            return Err(Diagnostic::new(
                codes::SOURCE_PATH_NOT_RELATIVE,
                format!("source path `{path}` is not repository-relative"),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape));
        }
        Ok(Self(path.to_owned()))
    }

    /// The path as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourcePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A half-open byte range within a source file, plus its display position.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SourceSpan {
    path: SourcePath,
    byte_start: u32,
    byte_end: u32,
    line: u32,
    column: u32,
}

impl SourceSpan {
    /// Builds a span. `line` and `column` are 1-based display coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`Diagnostic`] `EK0103` when the byte range is inverted or the
    /// display coordinates are zero.
    pub fn new(
        path: SourcePath,
        byte_start: u32,
        byte_end: u32,
        line: u32,
        column: u32,
    ) -> Result<Self, Diagnostic> {
        if byte_end < byte_start || line == 0 || column == 0 {
            return Err(Diagnostic::new(
                codes::SOURCE_SPAN_INVALID,
                format!(
                    "span {byte_start}..{byte_end} at {line}:{column} in `{path}` is not a valid span"
                ),
            ));
        }
        Ok(Self {
            path,
            byte_start,
            byte_end,
            line,
            column,
        })
    }

    /// The file this span points into.
    #[must_use]
    pub fn path(&self) -> &SourcePath {
        &self.path
    }

    /// The half-open byte range within the file.
    #[must_use]
    pub fn byte_range(&self) -> (u32, u32) {
        (self.byte_start, self.byte_end)
    }

    /// The 1-based display line and column.
    #[must_use]
    pub fn position(&self) -> (u32, u32) {
        (self.line, self.column)
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.path, self.line, self.column)
    }
}

/// A rejection with a stable code, a message, an optional source span, and the
/// repair classes that would make the rejected thing legal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Diagnostic {
    code: DiagnosticCode,
    message: String,
    span: Option<SourceSpan>,
    repairs: Vec<RepairClass>,
}

impl Diagnostic {
    /// Builds a diagnostic with no span and no repairs.
    #[must_use]
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            span: None,
            repairs: Vec::new(),
        }
    }

    /// Attaches a source span.
    #[must_use]
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    /// Adds a legal repair class, keeping the list sorted and deduplicated so
    /// the rendered diagnostic is stable.
    #[must_use]
    pub fn with_repair(mut self, repair: RepairClass) -> Self {
        if let Err(index) = self.repairs.binary_search(&repair) {
            self.repairs.insert(index, repair);
        }
        self
    }

    /// The stable code.
    #[must_use]
    pub fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// The human-facing message. Wording is not contractual.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The source span, when the rejected thing came from source.
    #[must_use]
    pub fn span(&self) -> Option<&SourceSpan> {
        self.span.as_ref()
    }

    /// The legal repair classes, sorted and deduplicated.
    #[must_use]
    pub fn repairs(&self) -> &[RepairClass] {
        &self.repairs
    }

    /// Renders the diagnostic as a canonical value for structured output.
    ///
    /// This is presentation, not hashed state: section 7 excludes source spans
    /// and display diagnostics from the state hash.
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
            let (byte_start, byte_end) = span.byte_range();
            let (line, column) = span.position();
            let span_value = CanonicalValue::object_declared([
                ("byte_end", CanonicalValue::Uint(u64::from(byte_end))),
                ("byte_start", CanonicalValue::Uint(u64::from(byte_start))),
                ("column", CanonicalValue::Uint(u64::from(column))),
                ("line", CanonicalValue::Uint(u64::from(line))),
                ("path", CanonicalValue::text(span.path().as_str())),
            ]);
            fields.push((FieldName::declared("span"), span_value));
        }
        CanonicalValue::object(fields)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(span) = &self.span {
            write!(f, " ({span})")?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostic {}

/// Codes owned by `estate-core`.
pub mod codes {
    use super::DiagnosticCode;

    /// An identifier segment is empty or uses unsupported characters.
    pub const IDENT_UNSUPPORTED: DiagnosticCode = DiagnosticCode::new("EK0101");
    /// A source path is absolute, empty, or escapes the repository root.
    pub const SOURCE_PATH_NOT_RELATIVE: DiagnosticCode = DiagnosticCode::new("EK0102");
    /// A source span has an inverted range or zero display coordinates.
    pub const SOURCE_SPAN_INVALID: DiagnosticCode = DiagnosticCode::new("EK0103");
    /// A stable ID does not match the shape its type requires.
    pub const ID_SHAPE_INVALID: DiagnosticCode = DiagnosticCode::new("EK0104");
    /// A schema version is zero; versions start at one.
    pub const SCHEMA_VERSION_ZERO: DiagnosticCode = DiagnosticCode::new("EK0105");

    /// Checked integer arithmetic overflowed or divided by zero.
    pub const ARITHMETIC_OVERFLOW: DiagnosticCode = DiagnosticCode::new("EK0201");

    /// An object field name is empty or uses unsupported characters.
    pub const FIELD_NAME_UNSUPPORTED: DiagnosticCode = DiagnosticCode::new("EK0301");
    /// Bytes offered as canonical JSON are malformed.
    pub const CANONICAL_MALFORMED: DiagnosticCode = DiagnosticCode::new("EK0302");
    /// Bytes parse as JSON but violate the canonical byte profile.
    pub const CANONICAL_NOT_CANONICAL: DiagnosticCode = DiagnosticCode::new("EK0303");

    /// A package output directory already exists.
    pub const PACKAGE_OUTPUT_EXISTS: DiagnosticCode = DiagnosticCode::new("EK0401");
    /// A package member named by the manifest is missing on disk.
    pub const PACKAGE_MEMBER_MISSING: DiagnosticCode = DiagnosticCode::new("EK0402");
    /// A package member's bytes do not match its recorded hash or size.
    pub const PACKAGE_MEMBER_HASH_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0403");
    /// A file in the package root is not declared by the manifest.
    pub const PACKAGE_MEMBER_UNDECLARED: DiagnosticCode = DiagnosticCode::new("EK0404");
    /// The package manifest is missing, malformed, or not canonical.
    pub const PACKAGE_MANIFEST_INVALID: DiagnosticCode = DiagnosticCode::new("EK0405");
    /// A package member name is not a legal member file name.
    pub const PACKAGE_MEMBER_NAME_INVALID: DiagnosticCode = DiagnosticCode::new("EK0406");
    /// Reading or writing the package failed for an environment reason.
    pub const PACKAGE_IO: DiagnosticCode = DiagnosticCode::new("EK0407");

    /// Every code this crate owns, for well-formedness and uniqueness tests.
    pub const ALL: &[DiagnosticCode] = &[
        IDENT_UNSUPPORTED,
        SOURCE_PATH_NOT_RELATIVE,
        SOURCE_SPAN_INVALID,
        ID_SHAPE_INVALID,
        SCHEMA_VERSION_ZERO,
        ARITHMETIC_OVERFLOW,
        FIELD_NAME_UNSUPPORTED,
        CANONICAL_MALFORMED,
        CANONICAL_NOT_CANONICAL,
        PACKAGE_OUTPUT_EXISTS,
        PACKAGE_MEMBER_MISSING,
        PACKAGE_MEMBER_HASH_MISMATCH,
        PACKAGE_MEMBER_UNDECLARED,
        PACKAGE_MANIFEST_INVALID,
        PACKAGE_MEMBER_NAME_INVALID,
        PACKAGE_IO,
    ];
}
