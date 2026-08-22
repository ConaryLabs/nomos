//! The compile-time half of the Gate K kernel.
//!
//! Source parsing and linking preserve versioned construction evidence. SW-G
//! promotes the contract-complete graph into stable World IR, compiles all four
//! projections from that stable type, and assembles the semantic package
//! members. Command-time effective-fact resolution remains in `nomos-sim`;
//! this crate cannot depend on it by construction.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod catalog;
pub mod diagnostics;
mod linker;
mod opened;
mod package;
mod parser;
mod projection;
mod resolver;
mod semantic;

use nomos_core::{Diagnostic, SchemaId, SourcePath};
pub use nomos_projection::{DiagnosticsPlan, NavigationPlan, PersistencePlan, SimulationPlan};
use nomos_schema::{SourceDocument, StableWorldIr, WorldIr};

pub use opened::{OpenedCompiledWorld, open_compiled_package, validate_compiled_package};
pub use package::{CompiledWorld, compile_world_package, compiler_receipts_schema};

/// Semantic compiler version embedded in stable World IR and build receipts.
pub const COMPILER_VERSION: u32 = 1;

/// Semantic version of the sealed Gate K primitive catalog.
pub const PRIMITIVE_CATALOG_VERSION: u32 = 1;

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

/// Validates and promotes one construction snapshot into stable World IR v1.
///
/// # Errors
///
/// Returns the first projection, resolver, provenance, or stable-schema
/// diagnostic. The construction value is never relabelled or mutated.
pub fn promote_world_ir(ir: &WorldIr) -> Result<StableWorldIr, Diagnostic> {
    let simulation = projection::simulation_plan(ir)?;
    let _navigation = projection::navigation_plan(ir)?;
    let persistence = projection::persistence_plan(ir)?;
    let diagnostics = projection::diagnostics_plan(ir)?;
    nomos_projection::validate_light_projection_agreement(
        simulation.light_resolver(),
        &persistence,
        &diagnostics,
    )?;
    StableWorldIr::new(
        ir.clone(),
        COMPILER_VERSION,
        PRIMITIVE_CATALOG_VERSION,
        projection::initial_movement_v1(ir)?,
    )
}

/// Compiles source through the construction lineage and stable promotion.
///
/// # Errors
///
/// Returns the first deterministic source, linker, projection, or promotion
/// diagnostic. No stable artifact is returned on failure.
pub fn compile_world(source: &str, path: SourcePath) -> Result<StableWorldIr, Diagnostic> {
    let construction = compile_source(source, path)?;
    promote_world_ir(&construction)
}

/// Validates construction-IR machine semantics and emits the simulation plan.
///
/// # Errors
///
/// Returns a stable `EK07xx` diagnostic for invalid transitions, references,
/// handlers, or causal cycles. No partial projection is returned.
///
/// Construction snapshots cannot cross the public projection boundary:
///
/// ```compile_fail
/// use nomos_core::SourcePath;
/// let construction = nomos_compiler::compile_source(
///     "schema nomos.source@1\n",
///     SourcePath::new("fixture.nomos").unwrap(),
/// ).unwrap();
/// let _ = nomos_compiler::compile_simulation_plan(&construction);
/// ```
pub fn compile_simulation_plan(ir: &StableWorldIr) -> Result<SimulationPlan, Diagnostic> {
    projection::simulation_plan(ir.construction())
}

/// Validates construction-IR movement semantics and emits the navigation plan.
///
/// # Errors
///
/// Returns a stable `EK09xx` diagnostic for invalid resolver laws, claims,
/// activations, connectivity, or subject identity.
pub fn compile_navigation_plan(ir: &StableWorldIr) -> Result<NavigationPlan, Diagnostic> {
    projection::navigation_plan(ir.construction())
}

/// Emits the implemented persistence projection from construction IR.
///
/// # Errors
///
/// Returns a stable resolver or projection diagnostic. No partial artifact is returned.
pub fn compile_persistence_plan(ir: &StableWorldIr) -> Result<PersistencePlan, Diagnostic> {
    projection::persistence_plan(ir.construction())
}

/// Emits the implemented diagnostics projection from construction IR.
///
/// # Errors
///
/// Returns a stable resolver or projection diagnostic. No partial artifact is returned.
pub fn compile_diagnostics_plan(ir: &StableWorldIr) -> Result<DiagnosticsPlan, Diagnostic> {
    projection::diagnostics_plan(ir.construction())
}

/// Verifies that every implemented light consumer received one typed plan.
///
/// # Errors
///
/// Returns `EK0912` when independently emitted projections disagree.
pub fn validate_light_projections(ir: &StableWorldIr) -> Result<(), Diagnostic> {
    projection::validate_all_light_projections(ir.construction())
}

/// The schemas this compiler reads.
#[must_use]
pub fn consumed_schemas() -> Vec<SchemaId> {
    vec![
        nomos_schema::source_schema(),
        nomos_schema::construction_world_ir_schema(),
        nomos_schema::stable_world_ir_schema(),
    ]
}

/// The schemas this compiler writes.
#[must_use]
pub fn produced_schemas() -> Vec<SchemaId> {
    vec![
        nomos_schema::construction_world_ir_schema(),
        nomos_schema::stable_world_ir_schema(),
        nomos_projection::simulation_schema(),
        nomos_projection::navigation_schema(),
        nomos_projection::persistence_schema(),
        nomos_projection::diagnostics_schema(),
        nomos_schema::schema_registry_schema(),
        compiler_receipts_schema(),
    ]
}

/// The full schema family assigned to the compile-time side of Gate K.
///
/// This is planned responsibility, not a claim that every artifact is emitted
/// by the current implementation. See [`produced_schemas`] for that evidence.
#[must_use]
pub fn planned_output_schemas() -> Vec<SchemaId> {
    let mut schemas = vec![
        nomos_schema::construction_world_ir_schema(),
        nomos_schema::stable_world_ir_schema(),
    ];
    schemas.extend(nomos_projection::all_schemas());
    schemas.push(nomos_schema::schema_registry_schema());
    schemas.push(compiler_receipts_schema());
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
        assert!(produced.contains(&nomos_projection::persistence_schema()));
        assert!(produced.contains(&nomos_projection::diagnostics_schema()));
        assert!(produced.contains(&nomos_schema::stable_world_ir_schema()));
        assert!(produced.contains(&nomos_schema::schema_registry_schema()));
        assert!(produced.contains(&super::compiler_receipts_schema()));
        for projection in nomos_projection::all_schemas() {
            assert!(planned_output_schemas().contains(&projection));
            assert!(!consumed.contains(&projection));
        }
    }
}
