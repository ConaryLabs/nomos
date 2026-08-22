//! Stable-v1 package validation and stable-v2 migration.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use nomos_core::package::{COMPILER_RECEIPTS_FILE, MemberName, WorldPackage};
use nomos_core::{Diagnostic, RepairClass, Sha256Digest};
use nomos_projection::SimulationPlan;
use nomos_schema::{LegacyStableWorldIrV1, SchemaRegistry, legacy_stable_world_ir_schema};

use super::{
    CompiledWorld, DIAGNOSTICS_FILE, MIGRATION_PASSES, NAVIGATION_FILE, PACKAGE_MEMBERS, PASSES,
    PERSISTENCE_FILE, SCHEMAS_FILE, SIMULATION_FILE, WORLD_IR_FILE, assemble_world, invalid_member,
    package_schema_ids_for, registry_for, validate_receipts_profile,
};

const LEGACY_INVARIANTS: [&str; 5] = [
    "canonical_members",
    "complete_section_4",
    "exact_package_member_set",
    "projection_agreement",
    "stable_v1_movement",
];

/// One completely validated stable-v1 package converted to active v2 meaning.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MigratedCompiledWorld {
    source_package_digest: Sha256Digest,
    source_world_ir_digest: Sha256Digest,
    normalized_legacy_simulation: SimulationPlan,
    compiled: CompiledWorld,
}

impl MigratedCompiledWorld {
    /// Digest of the exact immutable stable-v1 input package.
    #[must_use]
    pub const fn source_package_digest(&self) -> Sha256Digest {
        self.source_package_digest
    }

    /// Digest of the exact legacy `world-ir.json` bytes.
    #[must_use]
    pub const fn source_world_ir_digest(&self) -> Sha256Digest {
        self.source_world_ir_digest
    }

    /// Legacy package semantics projected for v2 runtime-state normalization.
    #[must_use]
    pub const fn normalized_legacy_simulation(&self) -> &SimulationPlan {
        &self.normalized_legacy_simulation
    }

    /// Complete active-v2 artifact set ready for ordinary package publication.
    #[must_use]
    pub const fn compiled_world(&self) -> &CompiledWorld {
        &self.compiled
    }
}

/// Strictly opens and converts one supported stable-v1 compiled package.
///
/// # Errors
///
/// Returns the first package-integrity, legacy-schema, semantic, projection,
/// receipt, or active-v2 assembly diagnostic. The input is never modified.
pub fn migrate_world_package_v1(root: &Path) -> Result<MigratedCompiledWorld, Diagnostic> {
    let package = WorldPackage::open(root)?;
    let members = exact_package_members(&package)?;
    let world_ir_bytes = members
        .get(WORLD_IR_FILE)
        .expect("the exact legacy package member set contains world-ir.json");
    let legacy = LegacyStableWorldIrV1::from_canonical_bytes(world_ir_bytes)?;
    if legacy.compiler_version() != 1 || legacy.primitive_catalog_version() != 1 {
        return Err(invalid_member(
            WORLD_IR_FILE,
            "only compiler/catalog version 1 stable World IR can migrate to version 2",
        ));
    }
    crate::semantic::validate_legacy_rehydrated_ir(&legacy)?;

    let simulation = crate::projection::simulation_plan(legacy.construction())?;
    let navigation = crate::projection::navigation_plan(legacy.construction())?;
    let persistence = crate::projection::persistence_plan(legacy.construction())?;
    let diagnostics = crate::projection::diagnostics_plan(legacy.construction())?;
    require_member_bytes(&members, SIMULATION_FILE, &simulation.to_canonical_bytes())?;
    require_member_bytes(&members, NAVIGATION_FILE, &navigation.to_canonical_bytes())?;
    require_member_bytes(
        &members,
        PERSISTENCE_FILE,
        &persistence.to_canonical_bytes(),
    )?;
    require_member_bytes(
        &members,
        DIAGNOSTICS_FILE,
        &diagnostics.to_canonical_bytes(),
    )?;
    require_member_bytes(
        &members,
        SCHEMAS_FILE,
        &legacy_expected_registry()?.to_canonical_bytes(),
    )?;

    let receipt_value = nomos_core::canonical::read::parse_canonical(
        members
            .get(COMPILER_RECEIPTS_FILE)
            .expect("the exact legacy package member set contains compiler receipts"),
    )
    .map_err(|error| invalid_member(COMPILER_RECEIPTS_FILE, error.message()))?;
    let source_digest = validate_receipts_profile(
        &receipt_value,
        &members,
        1,
        &[&PASSES],
        &LEGACY_INVARIANTS,
        legacy_package_schema_ids(),
    )?;
    let stable_ir = crate::migrate_world_ir_v1_to_v2(&legacy)?;
    let compiled = assemble_world(stable_ir, source_digest, &MIGRATION_PASSES)?;
    Ok(MigratedCompiledWorld {
        source_package_digest: package.manifest().digest(),
        source_world_ir_digest: Sha256Digest::of_bytes(world_ir_bytes),
        normalized_legacy_simulation: simulation,
        compiled,
    })
}

fn exact_package_members(package: &WorldPackage) -> Result<BTreeMap<String, Vec<u8>>, Diagnostic> {
    let actual = package
        .manifest()
        .members()
        .iter()
        .map(|row| row.name().as_str())
        .collect::<BTreeSet<_>>();
    let expected = PACKAGE_MEMBERS.into_iter().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::PACKAGE_MEMBER_SET_INVALID,
            "legacy package does not contain the exact compiled-world member set",
        )
        .with_repair(RepairClass::RebuildFromSource));
    }
    PACKAGE_MEMBERS
        .into_iter()
        .map(|name| {
            let member_name = MemberName::new(name)?;
            let bytes = package.member_bytes(&member_name).ok_or_else(|| {
                invalid_member(
                    name,
                    "legacy package member is absent after manifest verification",
                )
            })?;
            Ok((name.to_owned(), bytes.to_vec()))
        })
        .collect()
}

fn require_member_bytes(
    members: &BTreeMap<String, Vec<u8>>,
    name: &str,
    expected: &[u8],
) -> Result<(), Diagnostic> {
    if members.get(name).is_some_and(|bytes| bytes == expected) {
        Ok(())
    } else {
        Err(super::inconsistent(format!(
            "legacy `{name}` is not the exact projection of its stable-v1 meaning"
        )))
    }
}

fn legacy_expected_registry() -> Result<SchemaRegistry, Diagnostic> {
    registry_for(legacy_stable_world_ir_schema())
}

fn legacy_package_schema_ids() -> Vec<nomos_core::SchemaId> {
    package_schema_ids_for(legacy_stable_world_ir_schema())
}
