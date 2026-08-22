//! One strict semantic boundary for persisted compiled worlds.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use nomos_core::package::{MemberName, PackageManifest, WorldPackage};
use nomos_core::{Diagnostic, RepairClass, Sha256Digest};
use nomos_projection::{DiagnosticsPlan, NavigationPlan, PersistencePlan, SimulationPlan};
use nomos_schema::{LegacyStableWorldIrV1, SchemaRegistry, StableWorldIr};

use crate::package::{
    DIAGNOSTICS_FILE, NAVIGATION_FILE, PACKAGE_MEMBERS, PERSISTENCE_FILE, SCHEMAS_FILE,
    SIMULATION_FILE, WORLD_IR_FILE, expected_registry, validate_member_integrity,
};
use crate::{
    COMPILER_VERSION, PRIMITIVE_CATALOG_VERSION, compile_diagnostics_plan, compile_navigation_plan,
    compile_persistence_plan, compile_simulation_plan,
};

/// A package whose complete stable IR has been reconstructed and whose four
/// persisted projections are exact consequences of that typed meaning.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OpenedCompiledWorld {
    package: WorldPackage,
    stable_ir: StableWorldIr,
    simulation: SimulationPlan,
    navigation: NavigationPlan,
    persistence: PersistencePlan,
    diagnostics: DiagnosticsPlan,
    registry: SchemaRegistry,
    receipts: Vec<u8>,
}

impl OpenedCompiledWorld {
    /// The generic hash-verified package carrying this compiled world.
    #[must_use]
    pub const fn package(&self) -> &WorldPackage {
        &self.package
    }

    /// The verified package manifest.
    #[must_use]
    pub fn manifest(&self) -> &PackageManifest {
        self.package.manifest()
    }

    /// The package digest binding the exact member bytes.
    #[must_use]
    pub fn package_digest(&self) -> Sha256Digest {
        self.package.manifest().digest()
    }

    /// Completely reconstructed stable World IR.
    #[must_use]
    pub const fn stable_ir(&self) -> &StableWorldIr {
        &self.stable_ir
    }

    /// Regenerated simulation projection.
    #[must_use]
    pub const fn simulation(&self) -> &SimulationPlan {
        &self.simulation
    }

    /// Regenerated navigation projection.
    #[must_use]
    pub const fn navigation(&self) -> &NavigationPlan {
        &self.navigation
    }

    /// Regenerated persistence projection.
    #[must_use]
    pub const fn persistence(&self) -> &PersistencePlan {
        &self.persistence
    }

    /// Regenerated diagnostics projection.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticsPlan {
        &self.diagnostics
    }

    /// Exact schema and authoritative-owner registry.
    #[must_use]
    pub const fn registry(&self) -> &SchemaRegistry {
        &self.registry
    }

    /// Verified canonical compiler receipt bytes.
    #[must_use]
    pub fn compiler_receipts(&self) -> &[u8] {
        &self.receipts
    }

    fn from_package(package: WorldPackage) -> Result<Self, Diagnostic> {
        reject_legacy_runtime_input(&package)?;
        let members = package_members(&package)?;
        let rehydrated = rehydrate_members(&members)?;
        Ok(Self {
            package,
            stable_ir: rehydrated.stable_ir,
            simulation: rehydrated.simulation,
            navigation: rehydrated.navigation,
            persistence: rehydrated.persistence,
            diagnostics: rehydrated.diagnostics,
            registry: rehydrated.registry,
            receipts: members
                .get(nomos_core::package::COMPILER_RECEIPTS_FILE)
                .expect("the exact compiled member set contains compiler receipts")
                .clone(),
        })
    }
}

fn reject_legacy_runtime_input(package: &WorldPackage) -> Result<(), Diagnostic> {
    let name = MemberName::new(WORLD_IR_FILE)?;
    let Some(bytes) = package.member_bytes(&name) else {
        return Ok(());
    };
    if LegacyStableWorldIrV1::from_canonical_bytes(bytes).is_ok() {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::WORLD_IR_MIGRATION_REQUIRED,
            "stable World IR v1 must be migrated explicitly before active v2 loading",
        )
        .with_repair(RepairClass::WriteToNewOutputPath));
    }
    Ok(())
}

/// Opens one package and proves its complete persisted semantics.
///
/// # Errors
///
/// Returns the first generic package-integrity, member-schema, or semantic
/// disagreement diagnostic. No partially decoded world is returned.
pub fn open_compiled_package(root: &Path) -> Result<OpenedCompiledWorld, Diagnostic> {
    OpenedCompiledWorld::from_package(WorldPackage::open(root)?)
}

/// Semantically validates an already hash-verified package.
///
/// # Errors
///
/// Returns `EK0411` for the wrong member set, `EK0412` for bytes that do not
/// reconstruct exact typed meaning, or `EK0413` when a projection is not the
/// exact result of compiling the packaged stable IR.
pub fn validate_compiled_package(package: &WorldPackage) -> Result<(), Diagnostic> {
    rehydrate_members(&package_members(package)?).map(|_| ())
}

pub(crate) fn validate_compiled_members(
    members: &BTreeMap<String, Vec<u8>>,
) -> Result<(), Diagnostic> {
    require_member_set(members.keys().cloned().collect())?;
    rehydrate_members(members).map(|_| ())
}

struct RehydratedWorld {
    stable_ir: StableWorldIr,
    simulation: SimulationPlan,
    navigation: NavigationPlan,
    persistence: PersistencePlan,
    diagnostics: DiagnosticsPlan,
    registry: SchemaRegistry,
}

fn rehydrate_members(members: &BTreeMap<String, Vec<u8>>) -> Result<RehydratedWorld, Diagnostic> {
    validate_member_integrity(members)?;

    let stable_ir = StableWorldIr::from_canonical_bytes(member(members, WORLD_IR_FILE)?)
        .map_err(|error| invalid_member(WORLD_IR_FILE, error.message()))?;
    if stable_ir.compiler_version() != COMPILER_VERSION {
        return Err(invalid_member(
            WORLD_IR_FILE,
            format!(
                "compiler version {} is not supported version {COMPILER_VERSION}",
                stable_ir.compiler_version()
            ),
        ));
    }
    if stable_ir.primitive_catalog_version() != PRIMITIVE_CATALOG_VERSION {
        return Err(invalid_member(
            WORLD_IR_FILE,
            format!(
                "primitive catalog version {} is not supported version {PRIMITIVE_CATALOG_VERSION}",
                stable_ir.primitive_catalog_version()
            ),
        ));
    }
    crate::semantic::validate_rehydrated_ir(&stable_ir)?;

    let simulation = compile_simulation_plan(&stable_ir)
        .map_err(|error| invalid_member(WORLD_IR_FILE, error.message()))?;
    let navigation = compile_navigation_plan(&stable_ir)
        .map_err(|error| invalid_member(WORLD_IR_FILE, error.message()))?;
    let persistence = compile_persistence_plan(&stable_ir)
        .map_err(|error| invalid_member(WORLD_IR_FILE, error.message()))?;
    let diagnostics = compile_diagnostics_plan(&stable_ir)
        .map_err(|error| invalid_member(WORLD_IR_FILE, error.message()))?;
    nomos_projection::validate_light_projection_agreement(
        simulation.light_resolver(),
        &persistence,
        &diagnostics,
    )
    .map_err(|error| inconsistent(error.message()))?;

    require_projection(members, SIMULATION_FILE, &simulation.to_canonical_bytes())?;
    require_projection(members, NAVIGATION_FILE, &navigation.to_canonical_bytes())?;
    require_projection(members, PERSISTENCE_FILE, &persistence.to_canonical_bytes())?;
    require_projection(members, DIAGNOSTICS_FILE, &diagnostics.to_canonical_bytes())?;

    let registry = expected_registry()?;
    if member(members, SCHEMAS_FILE)? != registry.to_canonical_bytes() {
        return Err(invalid_member(
            SCHEMAS_FILE,
            "schema identities or authoritative owners differ from the exact package registry",
        ));
    }

    Ok(RehydratedWorld {
        stable_ir,
        simulation,
        navigation,
        persistence,
        diagnostics,
        registry,
    })
}

fn package_members(package: &WorldPackage) -> Result<BTreeMap<String, Vec<u8>>, Diagnostic> {
    let actual = package
        .manifest()
        .members()
        .iter()
        .map(|record| record.name().as_str().to_owned())
        .collect::<BTreeSet<_>>();
    require_member_set(actual)?;

    PACKAGE_MEMBERS
        .into_iter()
        .map(|name| {
            let member_name = MemberName::new(name)?;
            let bytes = package.member_bytes(&member_name).ok_or_else(|| {
                Diagnostic::new(
                    nomos_core::diagnostic::codes::PACKAGE_MEMBER_MISSING,
                    format!("compiled package has no `{name}` member"),
                )
                .with_repair(RepairClass::SupplyMissingMember)
            })?;
            Ok((name.to_owned(), bytes.to_vec()))
        })
        .collect()
}

fn require_member_set(actual: BTreeSet<String>) -> Result<(), Diagnostic> {
    let expected = PACKAGE_MEMBERS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if actual == expected {
        Ok(())
    } else {
        Err(Diagnostic::new(
            nomos_core::diagnostic::codes::PACKAGE_MEMBER_SET_INVALID,
            format!("compiled package member set is {actual:?}; expected {expected:?}"),
        )
        .with_repair(RepairClass::RebuildFromSource))
    }
}

fn member<'a>(members: &'a BTreeMap<String, Vec<u8>>, name: &str) -> Result<&'a [u8], Diagnostic> {
    members
        .get(name)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid_member(name, "member value is absent"))
}

fn require_projection(
    members: &BTreeMap<String, Vec<u8>>,
    name: &str,
    expected: &[u8],
) -> Result<(), Diagnostic> {
    if member(members, name)? == expected {
        Ok(())
    } else {
        Err(inconsistent(format!(
            "`{name}` is not the exact projection compiled from `{WORLD_IR_FILE}`"
        )))
    }
}

fn invalid_member(member_name: &str, reason: impl AsRef<str>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::PACKAGE_MEMBER_SCHEMA_INVALID,
        format!(
            "`{member_name}` is not a valid compiled-world member: {}",
            reason.as_ref()
        ),
    )
    .with_repair(RepairClass::RebuildFromSource)
}

fn inconsistent(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::PACKAGE_MEMBER_INCONSISTENT,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}
