//! Filesystem orchestration for complete compiled-world packages.

use std::path::Path;

use nomos_compiler::{
    CompiledWorld, MigratedCompiledWorld, OpenedCompiledWorld, compile_world_package,
    migrate_world_package_v1, open_compiled_package, validate_compiled_package,
};
use nomos_core::package::{MemberName, WorldPackage};
use nomos_core::{Diagnostic, SourcePath};
use nomos_sim::SimulationState;

/// Compiles source and publishes one complete, semantically validated world
/// package. No destination is created unless every pre-publication stage
/// succeeds.
///
/// # Errors
///
/// Returns the first compile, assembly, semantic-validation, filesystem, or
/// package-integrity diagnostic. An existing destination is never changed.
pub fn compile_and_write_world(
    source: &str,
    source_path: SourcePath,
    root: &Path,
) -> Result<WorldPackage, Diagnostic> {
    let compiled = compile_world_package(source, source_path)?;
    write_compiled_world(&compiled, root)
}

/// Strictly migrates one stable-v1 package and publishes a new active-v2 package.
///
/// # Errors
///
/// Returns the first legacy-open, migration, active-assembly, publication, or
/// verification diagnostic. Neither the input nor an existing output changes.
pub fn migrate_and_write_world(
    input: &Path,
    output: &Path,
) -> Result<(MigratedCompiledWorld, WorldPackage), Diagnostic> {
    let migrated = migrate_world_package_v1(input)?;
    let package = write_compiled_world(migrated.compiled_world(), output)?;
    Ok((migrated, package))
}

/// Publishes a complete compiled world through the generic immutable package
/// boundary used by the filesystem `compile` command.
///
/// # Errors
///
/// Returns the first member-name, canonical-byte, filesystem, manifest, or
/// semantic-package diagnostic. An existing destination is never changed.
pub fn write_compiled_world(
    compiled: &CompiledWorld,
    root: &Path,
) -> Result<WorldPackage, Diagnostic> {
    write_compiled_world_steps(
        compiled,
        root,
        CompiledWorld::members,
        CompiledWorld::validate_artifacts,
    )
}

fn write_compiled_world_steps<Assemble, Validate>(
    compiled: &CompiledWorld,
    root: &Path,
    assemble: Assemble,
    validate: Validate,
) -> Result<WorldPackage, Diagnostic>
where
    Assemble: FnOnce(&CompiledWorld) -> Result<Vec<(MemberName, Vec<u8>)>, Diagnostic>,
    Validate: FnOnce(&CompiledWorld) -> Result<(), Diagnostic>,
{
    let members = assemble(compiled)?;
    validate(compiled)?;
    let package = WorldPackage::write(root, members)?;
    validate_compiled_package(&package)?;
    Ok(package)
}

/// Opens and semantically validates a complete compiled world package.
///
/// # Errors
///
/// Returns the first generic integrity or compiled-world semantic diagnostic.
pub fn open_compiled_world(root: &Path) -> Result<OpenedCompiledWorld, Diagnostic> {
    open_compiled_package(root)
}

/// Reconstructs the initial runtime snapshot exclusively from a verified
/// package's typed, regenerated simulation projection.
///
/// # Errors
///
/// Returns a package or runtime-state diagnostic when the member is absent or
/// does not contain valid initialization material.
pub fn initial_state_from_package(
    package: &OpenedCompiledWorld,
) -> Result<SimulationState, Diagnostic> {
    SimulationState::initialize(package.simulation())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use nomos_compiler::compile_world_package;
    use nomos_core::diagnostic::codes;

    use super::*;

    fn fresh_path(label: &str) -> PathBuf {
        let path = PathBuf::from(option_env!("CARGO_TARGET_TMPDIR").unwrap_or("target/tmp"))
            .join("sw-g-publication-faults")
            .join(std::process::id().to_string())
            .join(label);
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    fn compiled() -> CompiledWorld {
        compile_world_package(
            include_str!("../../../fixtures/gaol.nomos"),
            SourcePath::new("fixtures/gaol.nomos").unwrap(),
        )
        .unwrap()
    }

    fn injected_failure(stage: &str) -> Diagnostic {
        Diagnostic::new(
            codes::PACKAGE_MEMBER_SET_INVALID,
            format!("injected {stage} failure"),
        )
    }

    #[test]
    fn assembly_failure_never_enters_publication() {
        let root = fresh_path("assembly");
        let rejected = write_compiled_world_steps(
            &compiled(),
            &root,
            |_| Err(injected_failure("member assembly")),
            CompiledWorld::validate_artifacts,
        )
        .unwrap_err();
        assert_eq!(rejected.code(), codes::PACKAGE_MEMBER_SET_INVALID);
        assert!(!root.exists());
    }

    #[test]
    fn validation_failure_never_enters_publication() {
        let root = fresh_path("validation");
        let rejected =
            write_compiled_world_steps(&compiled(), &root, CompiledWorld::members, |_| {
                Err(injected_failure("semantic validation"))
            })
            .unwrap_err();
        assert_eq!(rejected.code(), codes::PACKAGE_MEMBER_SET_INVALID);
        assert!(!root.exists());
    }
}
