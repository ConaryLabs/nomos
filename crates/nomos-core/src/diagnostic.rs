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
/// | `EK00xx` | command-line usage and host input |
/// | `EK01xx` | identifiers and stable IDs |
/// | `EK02xx` | checked arithmetic |
/// | `EK03xx` | canonical encoding |
/// | `EK04xx` | world packages |
/// | `EK05xx` | source parsing |
/// | `EK06xx` | name resolution and linking |
/// | `EK07xx` | transitions and causal interactions |
/// | `EK08xx` | runtime transaction preparation |
/// | `EK09xx` | effective-fact resolution |
/// | `EK10xx` | typed forensic provenance |
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

    /// Resolves one known stable diagnostic-code spelling.
    #[must_use]
    pub fn parse(code: &str) -> Option<Self> {
        codes::ALL
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
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
    /// Correct the source to match the published `.nomos` grammar.
    FixSourceSyntax,
    /// Declare the referenced entity before linking it.
    DeclareReferencedEntity,
    /// Declare the referenced catalog value before linking it.
    DeclareReferencedCatalogValue,
    /// Use one of the approved primitive kinds.
    UseApprovedPrimitive,
    /// Move a relational fact into the graph relation syntax.
    MoveRelationToGraph,
    /// Remove a source-authored fact that belongs to a compiler projection.
    RemoveDerivedFact,
    /// Replace a raw transform with a typed lattice binding.
    ReplaceRawTransformWithBinding,
    /// Remove an attempted canonical-owner declaration from content.
    RestoreCanonicalFactOwner,
    /// Remove or rename a duplicate declaration.
    RemoveDuplicateDeclaration,
    /// Supply the field required by the selected primitive kind.
    SupplyRequiredField,
    /// Remove a field the selected primitive kind does not accept.
    RemoveUnsupportedField,
    /// Use a value from the catalog namespace required by the field.
    UseExpectedCatalogNamespace,
    /// Use a relation kind from the approved relation vocabulary.
    UseApprovedRelationKind,
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
            Self::FixSourceSyntax => "fix_source_syntax",
            Self::DeclareReferencedEntity => "declare_referenced_entity",
            Self::DeclareReferencedCatalogValue => "declare_referenced_catalog_value",
            Self::UseApprovedPrimitive => "use_approved_primitive",
            Self::MoveRelationToGraph => "move_relation_to_graph",
            Self::RemoveDerivedFact => "remove_derived_fact",
            Self::ReplaceRawTransformWithBinding => "replace_raw_transform_with_binding",
            Self::RestoreCanonicalFactOwner => "restore_canonical_fact_owner",
            Self::RemoveDuplicateDeclaration => "remove_duplicate_declaration",
            Self::SupplyRequiredField => "supply_required_field",
            Self::RemoveUnsupportedField => "remove_unsupported_field",
            Self::UseExpectedCatalogNamespace => "use_expected_catalog_namespace",
            Self::UseApprovedRelationKind => "use_approved_relation_kind",
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

    /// The span as the canonical `byte_end`/`byte_start`/`column`/`line`/`path`
    /// object every artifact and report already spells.
    ///
    /// This crate owns [`SourceSpan`], so it owns the one rendering of it.
    /// Before this accessor the same five fields were written out separately in
    /// `nomos-core`, `nomos-schema`, `nomos-projection` (twice), and
    /// `nomos-cli`; each of those now calls here and the bytes are unchanged.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("byte_end", CanonicalValue::Uint(u64::from(self.byte_end))),
            (
                "byte_start",
                CanonicalValue::Uint(u64::from(self.byte_start)),
            ),
            ("column", CanonicalValue::Uint(u64::from(self.column))),
            ("line", CanonicalValue::Uint(u64::from(self.line))),
            ("path", CanonicalValue::text(self.path.as_str())),
        ])
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
            fields.push((FieldName::declared("span"), span.to_canonical()));
        }
        CanonicalValue::object(fields)
            .expect("diagnostic construction supplies each canonical field once")
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

/// Codes owned by `nomos-core`.
pub mod codes {
    use super::DiagnosticCode;

    /// The command-line argument vector does not name one supported operation.
    pub const CLI_USAGE: DiagnosticCode = DiagnosticCode::new("EK0001");
    /// A command-line filesystem path is not a safe relative spelling.
    pub const CLI_PATH_NOT_RELATIVE: DiagnosticCode = DiagnosticCode::new("EK0002");
    /// Source bytes are not UTF-8 text.
    pub const CLI_SOURCE_ENCODING: DiagnosticCode = DiagnosticCode::new("EK0003");
    /// A host filesystem operation failed before semantic work could complete.
    pub const CLI_IO: DiagnosticCode = DiagnosticCode::new("EK0004");

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
    /// A canonical builder received one semantic field or stable ID twice.
    pub const CANONICAL_DUPLICATE_IDENTITY: DiagnosticCode = DiagnosticCode::new("EK0304");

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
    /// A package writer received the same member name more than once.
    pub const PACKAGE_MEMBER_DUPLICATE: DiagnosticCode = DiagnosticCode::new("EK0408");
    /// A package root, manifest, or member has a forbidden filesystem entry type.
    pub const PACKAGE_ENTRY_TYPE_INVALID: DiagnosticCode = DiagnosticCode::new("EK0409");
    /// A manifest-declared member is not canonical semantic bytes.
    pub const PACKAGE_MEMBER_NON_CANONICAL: DiagnosticCode = DiagnosticCode::new("EK0410");
    /// A complete Gate K package has the wrong semantic member set.
    pub const PACKAGE_MEMBER_SET_INVALID: DiagnosticCode = DiagnosticCode::new("EK0411");
    /// A package member has the wrong schema or semantic shape.
    pub const PACKAGE_MEMBER_SCHEMA_INVALID: DiagnosticCode = DiagnosticCode::new("EK0412");
    /// Canonical package members disagree with one another.
    pub const PACKAGE_MEMBER_INCONSISTENT: DiagnosticCode = DiagnosticCode::new("EK0413");
    /// A stable-v1 compiled world requires explicit migration before active use.
    pub const WORLD_IR_MIGRATION_REQUIRED: DiagnosticCode = DiagnosticCode::new("EK0414");
    /// A migration target is absent or unsupported.
    pub const MIGRATION_TARGET_UNSUPPORTED: DiagnosticCode = DiagnosticCode::new("EK0415");
    /// A migration output would overlap its immutable input package.
    pub const MIGRATION_OUTPUT_OVERLAPS_INPUT: DiagnosticCode = DiagnosticCode::new("EK0416");

    /// A transition refers to a source machine namespace that does not exist.
    pub const TRANSITION_SOURCE_NAMESPACE_MISSING: DiagnosticCode = DiagnosticCode::new("EK0701");
    /// An interaction refers to a target machine namespace that does not exist.
    pub const INTERACTION_TARGET_NAMESPACE_MISSING: DiagnosticCode = DiagnosticCode::new("EK0702");
    /// A transition or on-enter trigger names a state absent from its machine.
    pub const TRANSITION_STATE_MISSING: DiagnosticCode = DiagnosticCode::new("EK0703");
    /// One machine declares the same transition signature more than once.
    pub const TRANSITION_SIGNATURE_DUPLICATE: DiagnosticCode = DiagnosticCode::new("EK0704");
    /// The interaction set declares the same stable edge identity more than once.
    pub const INTERACTION_IDENTITY_DUPLICATE: DiagnosticCode = DiagnosticCode::new("EK0705");
    /// An interaction target does not own a matching internal event handler.
    pub const INTERACTION_HANDLER_MISSING: DiagnosticCode = DiagnosticCode::new("EK0706");
    /// Causal interaction settlement contains a cycle.
    pub const INTERACTION_CYCLE: DiagnosticCode = DiagnosticCode::new("EK0707");
    /// A catalog transition has an input form that cannot be projected.
    pub const TRANSITION_INPUT_INVALID: DiagnosticCode = DiagnosticCode::new("EK0708");

    /// A command targets a machine absent from the projection or current state.
    pub const RUNTIME_TARGET_MISSING: DiagnosticCode = DiagnosticCode::new("EK0801");
    /// An external command action is not declared by its target machine.
    pub const RUNTIME_ACTION_UNDECLARED: DiagnosticCode = DiagnosticCode::new("EK0802");
    /// An external caller attempted to invoke an internal event handler.
    pub const RUNTIME_INTERNAL_HANDLER_EXTERNAL: DiagnosticCode = DiagnosticCode::new("EK0803");
    /// The selected command or handler is illegal in the current machine state.
    pub const RUNTIME_SOURCE_STATE_ILLEGAL: DiagnosticCode = DiagnosticCode::new("EK0804");
    /// A command argument does not match its compiled input requirement.
    pub const RUNTIME_ARGUMENT_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0805");
    /// A causal event targets a machine absent from the projection or state.
    pub const RUNTIME_EVENT_TARGET_MISSING: DiagnosticCode = DiagnosticCode::new("EK0806");
    /// A causal event has no matching target-owned handler.
    pub const RUNTIME_EVENT_HANDLER_MISSING: DiagnosticCode = DiagnosticCode::new("EK0807");
    /// Causal settlement exceeded the deterministic transition budget.
    pub const RUNTIME_TRANSITION_BUDGET: DiagnosticCode = DiagnosticCode::new("EK0808");
    /// Initial or current runtime state does not conform to the projection.
    pub const RUNTIME_STATE_INVALID: DiagnosticCode = DiagnosticCode::new("EK0809");
    /// A recorded state hash does not match the canonical runtime snapshot.
    pub const RUNTIME_STATE_HASH_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0810");
    /// Runtime commit evidence disagrees with compiler-projected consumers.
    pub const RUNTIME_PROJECTION_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0811");
    /// Persisted runtime evidence has an invalid schema or semantic shape.
    pub const RUNTIME_PERSISTED_INVALID: DiagnosticCode = DiagnosticCode::new("EK0812");
    /// A persisted state belongs to different compiled runtime semantics.
    pub const RUNTIME_SEMANTICS_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0813");
    /// A command script or request violates the accepted language.
    pub const RUNTIME_COMMAND_SCRIPT_INVALID: DiagnosticCode = DiagnosticCode::new("EK0814");
    /// A user command resolves to more than one owned machine namespace.
    pub const RUNTIME_COMMAND_AMBIGUOUS: DiagnosticCode = DiagnosticCode::new("EK0815");
    /// Persisted logs, hashes, receipts, or result evidence disagree.
    pub const RUNTIME_EVIDENCE_INCONSISTENT: DiagnosticCode = DiagnosticCode::new("EK0816");
    /// A requested run-bundle destination already exists.
    pub const RUN_BUNDLE_OUTPUT_EXISTS: DiagnosticCode = DiagnosticCode::new("EK0817");
    /// A run bundle does not contain exactly the six required root files.
    pub const RUN_BUNDLE_ENTRY_SET_INVALID: DiagnosticCode = DiagnosticCode::new("EK0818");
    /// A run-bundle root or entry has a forbidden filesystem type.
    pub const RUN_BUNDLE_ENTRY_TYPE_INVALID: DiagnosticCode = DiagnosticCode::new("EK0819");
    /// Reading, staging, publishing, or cleaning a run bundle failed.
    pub const RUN_BUNDLE_IO: DiagnosticCode = DiagnosticCode::new("EK0820");
    /// A run-bundle output would overlap immutable input evidence.
    pub const RUN_BUNDLE_OUTPUT_OVERLAPS_INPUT: DiagnosticCode = DiagnosticCode::new("EK0821");
    /// A replay log is malformed, noncanonical, or internally inconsistent.
    pub const REPLAY_LOG_INVALID: DiagnosticCode = DiagnosticCode::new("EK0822");
    /// A replay log names different package, semantics, or initial-state evidence.
    pub const REPLAY_INPUT_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0823");
    /// Re-execution does not reproduce the replay log's expected committed evidence.
    pub const REPLAY_EVIDENCE_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0824");
    /// An explanation names a well-formed entity absent from the verified world.
    pub const EXPLANATION_ENTITY_MISSING: DiagnosticCode = DiagnosticCode::new("EK0825");
    /// An explanation names a tick absent from the verified committed run prefix.
    pub const EXPLANATION_TICK_MISSING: DiagnosticCode = DiagnosticCode::new("EK0826");
    /// An explanation entity is unrelated to the selected committed transition.
    pub const EXPLANATION_ENTITY_UNRELATED: DiagnosticCode = DiagnosticCode::new("EK0827");

    /// A resolver-plan collection repeats one stable semantic identity.
    pub const RESOLVER_DUPLICATE_IDENTITY: DiagnosticCode = DiagnosticCode::new("EK0901");
    /// A movement claim belongs to an entity other than its resolver subject.
    pub const RESOLVER_CLAIM_ENTITY_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0902");
    /// A claim activation refers to a machine namespace that does not exist.
    pub const RESOLVER_ACTIVATION_NAMESPACE_MISSING: DiagnosticCode = DiagnosticCode::new("EK0903");
    /// A claim activation refers to a state absent from its machine.
    pub const RESOLVER_ACTIVATION_STATE_MISSING: DiagnosticCode = DiagnosticCode::new("EK0904");
    /// A movement claim value does not match its capability's required type.
    pub const RESOLVER_CLAIM_VALUE_INVALID: DiagnosticCode = DiagnosticCode::new("EK0905");
    /// A movement subject has no valid compiler-derived ground connectivity.
    pub const RESOLVER_CONNECTIVITY_INVALID: DiagnosticCode = DiagnosticCode::new("EK0906");
    /// A movement resolver plan omits or contradicts required Gate K semantics.
    pub const RESOLVER_PLAN_INVALID: DiagnosticCode = DiagnosticCode::new("EK0907");
    /// Runtime claim evaluation cannot resolve a projected namespace or state.
    pub const RESOLVER_RUNTIME_REFERENCE_MISSING: DiagnosticCode = DiagnosticCode::new("EK0908");
    /// A resolved movement disposition violates its nonempty/positive invariant.
    pub const MOVEMENT_DISPOSITION_INVALID: DiagnosticCode = DiagnosticCode::new("EK0909");
    /// A light claim contradicts the positive-only Gate K union semantics.
    pub const LIGHT_CLAIM_INVALID: DiagnosticCode = DiagnosticCode::new("EK0910");
    /// A light resolver plan omits or contradicts required Gate K semantics.
    pub const LIGHT_RESOLVER_PLAN_INVALID: DiagnosticCode = DiagnosticCode::new("EK0911");
    /// Persistence and diagnostics do not carry the same projected light facts.
    pub const LIGHT_PROJECTION_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0912");

    /// A provenance record names a fact absent from the receipt graph or world.
    pub const PROVENANCE_FACT_REFERENCE_MISSING: DiagnosticCode = DiagnosticCode::new("EK1001");
    /// A provenance fact carries a resolved value incompatible with its class.
    pub const PROVENANCE_VALUE_INVALID: DiagnosticCode = DiagnosticCode::new("EK1002");
    /// A provenance record names a producer outside the closed Gate K vocabulary.
    pub const PROVENANCE_PRODUCER_UNKNOWN: DiagnosticCode = DiagnosticCode::new("EK1003");
    /// A provenance record names a pass outside the closed Gate K vocabulary.
    pub const PROVENANCE_PASS_UNKNOWN: DiagnosticCode = DiagnosticCode::new("EK1004");
    /// A provenance step uses an unsupported producer/pass combination.
    pub const PROVENANCE_DERIVATION_INVALID: DiagnosticCode = DiagnosticCode::new("EK1005");
    /// A typed non-fact provenance input is absent from the compiled world.
    pub const PROVENANCE_INPUT_REFERENCE_MISSING: DiagnosticCode = DiagnosticCode::new("EK1006");
    /// A provenance fact is assigned to a non-canonical owner.
    pub const PROVENANCE_OWNER_INVALID: DiagnosticCode = DiagnosticCode::new("EK1007");

    /// Every code this crate owns, for well-formedness and uniqueness tests.
    pub const ALL: &[DiagnosticCode] = &[
        CLI_USAGE,
        CLI_PATH_NOT_RELATIVE,
        CLI_SOURCE_ENCODING,
        CLI_IO,
        IDENT_UNSUPPORTED,
        SOURCE_PATH_NOT_RELATIVE,
        SOURCE_SPAN_INVALID,
        ID_SHAPE_INVALID,
        SCHEMA_VERSION_ZERO,
        ARITHMETIC_OVERFLOW,
        FIELD_NAME_UNSUPPORTED,
        CANONICAL_MALFORMED,
        CANONICAL_NOT_CANONICAL,
        CANONICAL_DUPLICATE_IDENTITY,
        PACKAGE_OUTPUT_EXISTS,
        PACKAGE_MEMBER_MISSING,
        PACKAGE_MEMBER_HASH_MISMATCH,
        PACKAGE_MEMBER_UNDECLARED,
        PACKAGE_MANIFEST_INVALID,
        PACKAGE_MEMBER_NAME_INVALID,
        PACKAGE_IO,
        PACKAGE_MEMBER_DUPLICATE,
        PACKAGE_ENTRY_TYPE_INVALID,
        PACKAGE_MEMBER_NON_CANONICAL,
        PACKAGE_MEMBER_SET_INVALID,
        PACKAGE_MEMBER_SCHEMA_INVALID,
        PACKAGE_MEMBER_INCONSISTENT,
        WORLD_IR_MIGRATION_REQUIRED,
        MIGRATION_TARGET_UNSUPPORTED,
        MIGRATION_OUTPUT_OVERLAPS_INPUT,
        TRANSITION_SOURCE_NAMESPACE_MISSING,
        INTERACTION_TARGET_NAMESPACE_MISSING,
        TRANSITION_STATE_MISSING,
        TRANSITION_SIGNATURE_DUPLICATE,
        INTERACTION_IDENTITY_DUPLICATE,
        INTERACTION_HANDLER_MISSING,
        INTERACTION_CYCLE,
        TRANSITION_INPUT_INVALID,
        RUNTIME_TARGET_MISSING,
        RUNTIME_ACTION_UNDECLARED,
        RUNTIME_INTERNAL_HANDLER_EXTERNAL,
        RUNTIME_SOURCE_STATE_ILLEGAL,
        RUNTIME_ARGUMENT_MISMATCH,
        RUNTIME_EVENT_TARGET_MISSING,
        RUNTIME_EVENT_HANDLER_MISSING,
        RUNTIME_TRANSITION_BUDGET,
        RUNTIME_STATE_INVALID,
        RUNTIME_STATE_HASH_MISMATCH,
        RUNTIME_PROJECTION_MISMATCH,
        RUNTIME_PERSISTED_INVALID,
        RUNTIME_SEMANTICS_MISMATCH,
        RUNTIME_COMMAND_SCRIPT_INVALID,
        RUNTIME_COMMAND_AMBIGUOUS,
        RUNTIME_EVIDENCE_INCONSISTENT,
        RUN_BUNDLE_OUTPUT_EXISTS,
        RUN_BUNDLE_ENTRY_SET_INVALID,
        RUN_BUNDLE_ENTRY_TYPE_INVALID,
        RUN_BUNDLE_IO,
        RUN_BUNDLE_OUTPUT_OVERLAPS_INPUT,
        REPLAY_LOG_INVALID,
        REPLAY_INPUT_MISMATCH,
        REPLAY_EVIDENCE_MISMATCH,
        EXPLANATION_ENTITY_MISSING,
        EXPLANATION_TICK_MISSING,
        EXPLANATION_ENTITY_UNRELATED,
        RESOLVER_DUPLICATE_IDENTITY,
        RESOLVER_CLAIM_ENTITY_MISMATCH,
        RESOLVER_ACTIVATION_NAMESPACE_MISSING,
        RESOLVER_ACTIVATION_STATE_MISSING,
        RESOLVER_CLAIM_VALUE_INVALID,
        RESOLVER_CONNECTIVITY_INVALID,
        RESOLVER_PLAN_INVALID,
        RESOLVER_RUNTIME_REFERENCE_MISSING,
        MOVEMENT_DISPOSITION_INVALID,
        LIGHT_CLAIM_INVALID,
        LIGHT_RESOLVER_PLAN_INVALID,
        LIGHT_PROJECTION_MISMATCH,
        PROVENANCE_FACT_REFERENCE_MISSING,
        PROVENANCE_VALUE_INVALID,
        PROVENANCE_PRODUCER_UNKNOWN,
        PROVENANCE_PASS_UNKNOWN,
        PROVENANCE_DERIVATION_INVALID,
        PROVENANCE_INPUT_REFERENCE_MISSING,
        PROVENANCE_OWNER_INVALID,
    ];
}
