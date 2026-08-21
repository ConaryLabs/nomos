//! The compile-time half of the Gate K kernel.
//!
//! SW-C implements source schema version 1, parsing, typed name resolution,
//! approved primitive expansion, and the fact-ownership linker. Command-time
//! effective-fact resolution remains in `estate-sim`; this crate cannot depend
//! on it by construction.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod catalog;
pub mod diagnostics;
mod linker;
mod parser;

use estate_core::{Diagnostic, SchemaId, SourcePath};
use estate_schema::{SourceDocument, WorldIr};

/// Parses source schema version 1 without resolving cross-references.
///
/// # Errors
///
/// Returns the first deterministic syntax/schema diagnostic with a source span.
pub fn parse_source(source: &str, path: SourcePath) -> Result<SourceDocument, Diagnostic> {
    parser::parse(source, path)
}

/// Resolves names, enforces ownership, expands approved primitives, and emits
/// Canonical World IR.
///
/// # Errors
///
/// Returns the first deterministic linker diagnostic with a source span.
pub fn link_source(document: &SourceDocument) -> Result<WorldIr, Diagnostic> {
    linker::link(document)
}

/// Compiles one `.estate` source file through parsing and ownership linking.
///
/// # Errors
///
/// Returns a stable source or linker diagnostic. No partial IR is returned.
pub fn compile_source(source: &str, path: SourcePath) -> Result<WorldIr, Diagnostic> {
    let document = parse_source(source, path)?;
    link_source(&document)
}

/// The schemas this compiler reads.
#[must_use]
pub fn consumed_schemas() -> Vec<SchemaId> {
    vec![
        estate_schema::source_schema(),
        estate_schema::world_ir_schema(),
    ]
}

/// The schemas this compiler writes.
#[must_use]
pub fn produced_schemas() -> Vec<SchemaId> {
    let mut schemas = vec![estate_schema::world_ir_schema()];
    schemas.extend(estate_projection::all_schemas());
    schemas
}

#[cfg(test)]
mod tests {
    use super::{consumed_schemas, produced_schemas};

    #[test]
    fn the_compiler_is_the_only_crossing_between_ir_and_projections() {
        let consumed = consumed_schemas();
        let produced = produced_schemas();
        assert!(consumed.contains(&estate_schema::source_schema()));
        for projection in estate_projection::all_schemas() {
            assert!(produced.contains(&projection));
            assert!(!consumed.contains(&projection));
        }
    }
}
