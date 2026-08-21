//! The `estate` command-line surface.
//!
//! `KERNEL.md` section 10 assigns this crate *command-line surface and artifact
//! orchestration*, and section 8 fixes the exit codes every command must use.
//! SW-B lands the exit-code contract and the boundary; the commands themselves
//! (`validate`, `compile`, `inspect`, `run`, `command`, `explain-entity`,
//! `explain-transition`, `replay`, `migrate`) belong to later slices.
//!
//! # Boundary
//!
//! Four permitted edges resolve:
//!
//! ```
//! use estate_core::id::SchemaId;
//! let _: Vec<SchemaId> = estate_compiler::produced_schemas();
//! let _: SchemaId = estate_sim::runtime_state_schema();
//! let _: SchemaId = estate_projection::persistence_schema();
//! ```
//!
//! `estate-schema` is not among them. The CLI orchestrates compiled artifacts;
//! if it could name the authoring schema it would be one refactor away from
//! parsing source outside the compiler:
//!
//! ```compile_fail
//! let _ = estate_schema::source_schema();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use estate_core::Diagnostic;

/// The process exit codes fixed by `KERNEL.md` section 8.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ExitCode {
    /// Completed successfully.
    Completed,
    /// Rejected with structured diagnostics.
    Rejected,
    /// Invalid command-line usage.
    InvalidUsage,
    /// Could not execute because of an environment or I/O failure.
    Environment,
}

impl ExitCode {
    /// The numeric code passed to the operating system.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Completed => 0,
            Self::Rejected => 1,
            Self::InvalidUsage => 2,
            Self::Environment => 3,
        }
    }

    /// Classifies a diagnostic.
    ///
    /// A diagnostic in the package I/O range means the environment refused the
    /// work rather than the world being wrong, which section 8 separates: a
    /// full disk is not a rejected world.
    #[must_use]
    pub fn for_diagnostic(diagnostic: &Diagnostic) -> Self {
        if diagnostic.code() == estate_core::diagnostic::codes::PACKAGE_IO {
            Self::Environment
        } else {
            Self::Rejected
        }
    }
}

/// Renders a diagnostic as the structured JSON every command writes to standard
/// output, per section 8.
#[must_use]
pub fn render_rejection(diagnostic: &Diagnostic) -> String {
    let value = estate_core::CanonicalValue::object_declared([
        (
            "diagnostics",
            estate_core::CanonicalValue::Array(vec![diagnostic.to_canonical()]),
        ),
        ("status", estate_core::CanonicalValue::text("rejected")),
    ]);
    String::from_utf8(value.to_canonical_bytes()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{ExitCode, render_rejection};
    use estate_core::diagnostic::codes;
    use estate_core::{Diagnostic, EntityId, Ident, NamespaceId, SourcePath};
    use estate_projection::{Command, CommandArgument};

    #[test]
    fn exit_codes_match_kernel_section_8() {
        assert_eq!(ExitCode::Completed.code(), 0);
        assert_eq!(ExitCode::Rejected.code(), 1);
        assert_eq!(ExitCode::InvalidUsage.code(), 2);
        assert_eq!(ExitCode::Environment.code(), 3);
    }

    #[test]
    fn an_io_failure_is_an_environment_failure_not_a_rejected_world() {
        let io = Diagnostic::new(codes::PACKAGE_IO, "disk full");
        let world = Diagnostic::new(codes::PACKAGE_MEMBER_HASH_MISMATCH, "tampered");
        assert_eq!(ExitCode::for_diagnostic(&io), ExitCode::Environment);
        assert_eq!(ExitCode::for_diagnostic(&world), ExitCode::Rejected);
    }

    #[test]
    fn rejections_render_as_canonical_json() {
        let diagnostic = Diagnostic::new(codes::PACKAGE_OUTPUT_EXISTS, "already exists");
        assert_eq!(
            render_rejection(&diagnostic),
            r#"{"diagnostics":[{"code":"EK0401","message":"already exists","repairs":[]}],"status":"rejected"}"#
        );
    }

    #[test]
    fn compiled_fixture_semantics_cross_the_ir_boundary_and_execute() {
        let ir = estate_compiler::compile_source(
            include_str!("../../../fixtures/gaol.estate"),
            SourcePath::new("fixtures/gaol.estate").unwrap(),
        )
        .unwrap();
        let plan = estate_compiler::compile_simulation_plan(&ir).unwrap();
        let state = estate_sim::SimulationState::initialize(&plan).unwrap();
        let combustion = NamespaceId::new(
            EntityId::parse("north_gate").unwrap(),
            Ident::new("combustion").unwrap(),
        );
        let integrity = NamespaceId::new(
            EntityId::parse("north_gate").unwrap(),
            Ident::new("integrity").unwrap(),
        );
        let prepared = estate_sim::prepare_transaction(
            &plan,
            &state,
            &Command::new(
                combustion,
                Ident::new("ignite").unwrap(),
                CommandArgument::None,
            ),
        )
        .unwrap();
        assert_eq!(prepared.steps().len(), 2);
        assert_eq!(
            prepared.after().machine(&integrity).unwrap().as_str(),
            "destroyed"
        );
    }
}
