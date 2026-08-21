//! The `nomos` command-line surface.
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
//! use nomos_core::id::SchemaId;
//! let _: Vec<SchemaId> = nomos_compiler::produced_schemas();
//! let _: SchemaId = nomos_sim::runtime_state_schema();
//! let _: SchemaId = nomos_projection::persistence_schema();
//! ```
//!
//! `nomos-schema` is not among them. The CLI orchestrates compiled artifacts;
//! if it could name the authoring schema it would be one refactor away from
//! parsing source outside the compiler:
//!
//! ```compile_fail
//! let _ = nomos_schema::source_schema();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use nomos_core::Diagnostic;

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
        if diagnostic.code() == nomos_core::diagnostic::codes::PACKAGE_IO {
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
    let value = nomos_core::CanonicalValue::object_declared([
        (
            "diagnostics",
            nomos_core::CanonicalValue::Array(vec![diagnostic.to_canonical()]),
        ),
        ("status", nomos_core::CanonicalValue::text("rejected")),
    ]);
    String::from_utf8(value.to_canonical_bytes()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{ExitCode, render_rejection};
    use nomos_core::diagnostic::codes;
    use nomos_core::{
        CatalogValueId, ClaimRef, Diagnostic, EntityId, Ident, NamespaceId, SourcePath,
    };
    use nomos_projection::{Command, CommandArgument, MovementDisposition};

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
        let ir = nomos_compiler::compile_source(
            include_str!("../../../fixtures/gaol.nomos"),
            SourcePath::new("fixtures/gaol.nomos").unwrap(),
        )
        .unwrap();
        let plan = nomos_compiler::compile_simulation_plan(&ir).unwrap();
        let state = nomos_sim::SimulationState::initialize(&plan).unwrap();
        let combustion = NamespaceId::new(
            EntityId::parse("north_gate").unwrap(),
            Ident::new("combustion").unwrap(),
        );
        let integrity = NamespaceId::new(
            EntityId::parse("north_gate").unwrap(),
            Ident::new("integrity").unwrap(),
        );
        let prepared = nomos_sim::prepare_transaction(
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

    #[test]
    fn compiled_fixture_resolves_exact_ground_movement_facts() {
        let ir = nomos_compiler::compile_source(
            include_str!("../../../fixtures/gaol.nomos"),
            SourcePath::new("fixtures/gaol.nomos").unwrap(),
        )
        .unwrap();
        let plan = nomos_compiler::compile_simulation_plan(&ir).unwrap();
        let navigation = nomos_compiler::compile_navigation_plan(&ir).unwrap();
        assert_eq!(
            plan.movement_resolver().to_canonical_bytes(),
            navigation.movement_resolver().to_canonical_bytes()
        );

        let initial = nomos_sim::SimulationState::initialize(&plan).unwrap();
        let initial_facts = nomos_sim::resolve_movement(&plan, &initial).unwrap();
        assert_blocked(
            initial_facts.get(&entity("north_gate")).unwrap(),
            &[
                "north_gate.portal#blocks_ground",
                "north_gate.ward#blocks_ground",
            ],
        );
        assert_traversable(
            initial_facts.get(&entity("flooded_section")).unwrap(),
            3,
            &["flooded_section.region#traversal_cost_ground"],
        );

        let access = namespace("north_gate", "access");
        let unlocked = nomos_sim::prepare_transaction(
            &plan,
            &initial,
            &Command::new(
                access.clone(),
                ident("unlock"),
                CommandArgument::Credential(
                    CatalogValueId::parse("credential/gaoler_key").unwrap(),
                ),
            ),
        )
        .unwrap()
        .into_after();
        let opened = nomos_sim::prepare_transaction(
            &plan,
            &unlocked,
            &Command::new(access, ident("open"), CommandArgument::None),
        )
        .unwrap();
        assert_blocked(
            opened.movement_after().get(&entity("north_gate")).unwrap(),
            &["north_gate.ward#blocks_ground"],
        );

        let unsealed = nomos_sim::prepare_transaction(
            &plan,
            opened.after(),
            &Command::new(
                namespace("north_gate", "ward"),
                ident("unseal"),
                CommandArgument::None,
            ),
        )
        .unwrap();
        assert_traversable(
            unsealed
                .movement_after()
                .get(&entity("north_gate"))
                .unwrap(),
            1,
            &[],
        );

        let ignited = nomos_sim::prepare_transaction(
            &plan,
            &initial,
            &Command::new(
                namespace("north_gate", "combustion"),
                ident("ignite"),
                CommandArgument::None,
            ),
        )
        .unwrap();
        assert_blocked(
            ignited.movement_after().get(&entity("north_gate")).unwrap(),
            &["north_gate.ward#blocks_ground"],
        );
    }

    fn ident(value: &str) -> Ident {
        Ident::new(value).unwrap()
    }

    fn entity(value: &str) -> EntityId {
        EntityId::parse(value).unwrap()
    }

    fn namespace(entity_id: &str, local: &str) -> NamespaceId {
        NamespaceId::new(entity(entity_id), ident(local))
    }

    fn assert_blocked(disposition: &MovementDisposition, expected: &[&str]) {
        let MovementDisposition::Blocked { reasons } = disposition else {
            panic!("expected blocked movement, got {disposition:?}");
        };
        let expected: Vec<_> = expected
            .iter()
            .map(|reason| ClaimRef::parse(reason).unwrap())
            .collect();
        assert_eq!(reasons, &expected);
    }

    fn assert_traversable(
        disposition: &MovementDisposition,
        expected_cost: u32,
        expected: &[&str],
    ) {
        let MovementDisposition::Traversable { cost, reasons } = disposition else {
            panic!("expected traversable movement, got {disposition:?}");
        };
        let expected: Vec<_> = expected
            .iter()
            .map(|reason| ClaimRef::parse(reason).unwrap())
            .collect();
        assert_eq!(*cost, expected_cost);
        assert_eq!(reasons, &expected);
    }
}
