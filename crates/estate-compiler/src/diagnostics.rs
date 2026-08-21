//! Stable diagnostics owned by source parsing and linking.

use estate_core::DiagnosticCode;

/// Source text does not match the published grammar.
pub const SOURCE_SYNTAX: DiagnosticCode = DiagnosticCode::new("EK0501");
/// The mandatory source-schema declaration is absent or misplaced.
pub const SOURCE_SCHEMA_REQUIRED: DiagnosticCode = DiagnosticCode::new("EK0502");
/// The source declares a schema version this compiler does not consume.
pub const SOURCE_SCHEMA_UNSUPPORTED: DiagnosticCode = DiagnosticCode::new("EK0503");
/// A lattice coordinate is not a signed 32-bit decimal integer.
pub const SOURCE_INTEGER_INVALID: DiagnosticCode = DiagnosticCode::new("EK0504");
/// A statement or entity field is not part of source schema version 1.
pub const SOURCE_UNKNOWN_STATEMENT: DiagnosticCode = DiagnosticCode::new("EK0505");
/// An entity declaration reaches end of file without `end`.
pub const SOURCE_UNCLOSED_ENTITY: DiagnosticCode = DiagnosticCode::new("EK0506");
/// The source is too large for the contract's 32-bit byte spans.
pub const SOURCE_TOO_LARGE: DiagnosticCode = DiagnosticCode::new("EK0507");

/// An entity ID is declared more than once.
pub const DUPLICATE_ENTITY: DiagnosticCode = DiagnosticCode::new("EK0601");
/// A catalog value is declared more than once.
pub const DUPLICATE_CATALOG_VALUE: DiagnosticCode = DiagnosticCode::new("EK0602");
/// A graph relation references an undeclared entity.
pub const DANGLING_ENTITY: DiagnosticCode = DiagnosticCode::new("EK0603");
/// A primitive field references an undeclared catalog value.
pub const DANGLING_CATALOG_VALUE: DiagnosticCode = DiagnosticCode::new("EK0604");
/// A primitive kind is not in the sealed Gate K catalog.
pub const UNAPPROVED_PRIMITIVE: DiagnosticCode = DiagnosticCode::new("EK0605");
/// An approved primitive is missing one of its required fields.
pub const REQUIRED_FIELD_MISSING: DiagnosticCode = DiagnosticCode::new("EK0606");
/// A field or binding kind is not accepted by the selected primitive.
pub const FIELD_NOT_ALLOWED: DiagnosticCode = DiagnosticCode::new("EK0607");
/// The same source-owned field is supplied more than once.
pub const DUPLICATE_FIELD: DiagnosticCode = DiagnosticCode::new("EK0608");
/// A region's component-wise minimum is greater than its maximum.
pub const REGION_BOUNDS_INVALID: DiagnosticCode = DiagnosticCode::new("EK0609");
/// A graph relation was smuggled into lattice/entity properties.
pub const RELATION_IN_LATTICE: DiagnosticCode = DiagnosticCode::new("EK0610");
/// Content attempted to author a raw transform.
pub const RAW_TRANSFORM_AUTHORED: DiagnosticCode = DiagnosticCode::new("EK0611");
/// Content attempted to supply a compiler-derived fact.
pub const DERIVED_FACT_AUTHORED: DiagnosticCode = DiagnosticCode::new("EK0612");
/// Content attempted to add a second canonical owner for a fact class.
pub const DUPLICATE_FACT_OWNER: DiagnosticCode = DiagnosticCode::new("EK0613");
/// The same graph relation is declared more than once.
pub const DUPLICATE_RELATION: DiagnosticCode = DiagnosticCode::new("EK0614");
/// A typed catalog reference points into the wrong catalog namespace.
pub const CATALOG_NAMESPACE_MISMATCH: DiagnosticCode = DiagnosticCode::new("EK0615");
/// A graph relation kind is outside the approved Gate K vocabulary.
pub const UNAPPROVED_RELATION_KIND: DiagnosticCode = DiagnosticCode::new("EK0616");

/// Every stable code owned by this crate.
pub const ALL: &[DiagnosticCode] = &[
    SOURCE_SYNTAX,
    SOURCE_SCHEMA_REQUIRED,
    SOURCE_SCHEMA_UNSUPPORTED,
    SOURCE_INTEGER_INVALID,
    SOURCE_UNKNOWN_STATEMENT,
    SOURCE_UNCLOSED_ENTITY,
    SOURCE_TOO_LARGE,
    DUPLICATE_ENTITY,
    DUPLICATE_CATALOG_VALUE,
    DANGLING_ENTITY,
    DANGLING_CATALOG_VALUE,
    UNAPPROVED_PRIMITIVE,
    REQUIRED_FIELD_MISSING,
    FIELD_NOT_ALLOWED,
    DUPLICATE_FIELD,
    REGION_BOUNDS_INVALID,
    RELATION_IN_LATTICE,
    RAW_TRANSFORM_AUTHORED,
    DERIVED_FACT_AUTHORED,
    DUPLICATE_FACT_OWNER,
    DUPLICATE_RELATION,
    CATALOG_NAMESPACE_MISMATCH,
    UNAPPROVED_RELATION_KIND,
];
