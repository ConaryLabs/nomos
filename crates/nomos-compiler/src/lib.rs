//! The compile-time half of the Gate K kernel.
//!
//! SW-C implements source schema version 1, parsing, typed name resolution,
//! approved primitive expansion, and the fact-ownership linker. Command-time
//! effective-fact resolution remains in `nomos-sim`; this crate cannot depend
//! on it by construction.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod catalog;
pub mod diagnostics;
mod linker;
mod parser;
mod projection;
mod resolver;

use nomos_core::{Diagnostic, SchemaId, SourcePath};
pub use nomos_projection::{NavigationPlan, SimulationPlan};
use nomos_schema::{SourceDocument, WorldIr};

/// Parses source schema version 1 without resolving cross-references.
///
/// # Errors
///
/// Returns the first deterministic syntax/schema diagnostic with a source span.
pub fn parse_source(source: &str, path: SourcePath) -> Result<SourceDocument, Diagnostic> {
    parser::parse(source, path)
}

/// Resolves names, enforces ownership, expands approved primitives, and emits
/// a Canonical World IR construction snapshot.
///
/// # Errors
///
/// Returns the first deterministic linker diagnostic with a source span.
pub fn link_source(document: &SourceDocument) -> Result<WorldIr, Diagnostic> {
    linker::link(document)
}

/// Compiles one `.nomos` source file through parsing and ownership linking.
///
/// # Errors
///
/// Returns a stable source or linker diagnostic. No partial IR is returned.
pub fn compile_source(source: &str, path: SourcePath) -> Result<WorldIr, Diagnostic> {
    let document = parse_source(source, path)?;
    link_source(&document)
}

/// Validates construction-IR machine semantics and emits the simulation plan.
///
/// # Errors
///
/// Returns a stable `EK07xx` diagnostic for invalid transitions, references,
/// handlers, or causal cycles. No partial projection is returned.
pub fn compile_simulation_plan(ir: &WorldIr) -> Result<SimulationPlan, Diagnostic> {
    projection::simulation_plan(ir)
}

/// Validates construction-IR movement semantics and emits the navigation plan.
///
/// # Errors
///
/// Returns a stable `EK09xx` diagnostic for invalid resolver laws, claims,
/// activations, connectivity, or subject identity.
pub fn compile_navigation_plan(ir: &WorldIr) -> Result<NavigationPlan, Diagnostic> {
    projection::navigation_plan(ir)
}

/// The schemas this compiler reads.
#[must_use]
pub fn consumed_schemas() -> Vec<SchemaId> {
    vec![
        nomos_schema::source_schema(),
        nomos_schema::construction_world_ir_schema(),
    ]
}

/// The schemas this compiler writes.
#[must_use]
pub fn produced_schemas() -> Vec<SchemaId> {
    vec![
        nomos_schema::construction_world_ir_schema(),
        nomos_projection::simulation_schema(),
        nomos_projection::navigation_schema(),
    ]
}

/// The full projection family assigned to the compile-time side of Gate K.
///
/// This is planned responsibility, not a claim that every artifact is emitted
/// by the current implementation. See [`produced_schemas`] for that evidence.
#[must_use]
pub fn planned_output_schemas() -> Vec<SchemaId> {
    let mut schemas = vec![nomos_schema::construction_world_ir_schema()];
    schemas.extend(nomos_projection::all_schemas());
    schemas
}

#[cfg(test)]
mod tests {
    use super::{consumed_schemas, planned_output_schemas, produced_schemas};

    #[test]
    fn the_compiler_is_the_only_crossing_between_ir_and_projections() {
        let consumed = consumed_schemas();
        let produced = produced_schemas();
        assert!(consumed.contains(&nomos_schema::source_schema()));
        assert!(produced.contains(&nomos_projection::simulation_schema()));
        assert!(produced.contains(&nomos_projection::navigation_schema()));
        assert!(!produced.contains(&nomos_projection::persistence_schema()));
        assert!(!produced.contains(&nomos_projection::diagnostics_schema()));
        for projection in nomos_projection::all_schemas() {
            assert!(planned_output_schemas().contains(&projection));
            assert!(!consumed.contains(&projection));
        }
    }
}
