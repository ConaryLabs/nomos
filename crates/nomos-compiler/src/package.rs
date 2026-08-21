//! Stable World IR compilation and complete package-member validation.

use std::collections::{BTreeMap, BTreeSet};

use nomos_core::canonical::keyed_array;
use nomos_core::package::{
    COMPILER_RECEIPTS_FILE, MANIFEST_FILE, MemberName, WorldPackage, manifest_schema,
};
use nomos_core::{
    CanonicalValue, Diagnostic, EntityId, FieldName, RepairClass, SchemaId, Sha256Digest,
    SourcePath,
};
use nomos_projection::{DiagnosticsPlan, NavigationPlan, PersistencePlan, SimulationPlan};
use nomos_schema::{
    SchemaOwner, SchemaRegistration, SchemaRegistry, StableWorldIr, construction_world_ir_schema,
    schema_registry_schema, source_schema, stable_world_ir_schema,
};

use crate::{
    COMPILER_VERSION, PRIMITIVE_CATALOG_VERSION, compile_diagnostics_plan, compile_navigation_plan,
    compile_persistence_plan, compile_simulation_plan, compile_world,
};

/// Stable world-IR member name.
pub const WORLD_IR_FILE: &str = "world-ir.json";
/// Simulation projection member name.
pub const SIMULATION_FILE: &str = "simulation.json";
/// Navigation projection member name.
pub const NAVIGATION_FILE: &str = "navigation.json";
/// Persistence projection member name.
pub const PERSISTENCE_FILE: &str = "persistence.json";
/// Diagnostics projection member name.
pub const DIAGNOSTICS_FILE: &str = "diagnostics.json";
/// Schema registry member name.
pub const SCHEMAS_FILE: &str = "schemas.json";

const PACKAGE_MEMBERS: [&str; 7] = [
    COMPILER_RECEIPTS_FILE,
    DIAGNOSTICS_FILE,
    NAVIGATION_FILE,
    PERSISTENCE_FILE,
    SCHEMAS_FILE,
    SIMULATION_FILE,
    WORLD_IR_FILE,
];

const PASSES: [&str; 10] = [
    "parse_source",
    "link_world",
    "validate_semantics",
    "promote_stable_ir",
    "project_simulation",
    "project_navigation",
    "project_persistence",
    "project_diagnostics",
    "validate_projection_agreement",
    "assemble_package_members",
];

const INVARIANTS: [&str; 5] = [
    "canonical_members",
    "complete_section_4",
    "exact_package_member_set",
    "projection_agreement",
    "stable_v1_movement",
];

/// Schema for the canonical compiler build receipt member.
#[must_use]
pub fn compiler_receipts_schema() -> SchemaId {
    SchemaId::new("nomos.compiler_receipts", 1)
        .expect("the compiler receipt schema id is a valid literal")
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct ArtifactDigest {
    name: String,
    digest: Sha256Digest,
}

impl ArtifactDigest {
    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("name", CanonicalValue::text(&self.name)),
            ("sha256", CanonicalValue::text(self.digest.to_hex())),
        ])
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct CompilerReceipts {
    source_digest: Sha256Digest,
    artifacts: Vec<ArtifactDigest>,
}

impl CompilerReceipts {
    fn new(source: &[u8], artifacts: &BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            source_digest: Sha256Digest::of_bytes(source),
            artifacts: artifacts
                .iter()
                .map(|(name, bytes)| ArtifactDigest {
                    name: name.clone(),
                    digest: Sha256Digest::of_bytes(bytes),
                })
                .collect(),
        }
    }

    fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            (
                "artifacts",
                keyed_array(
                    self.artifacts
                        .iter()
                        .map(|artifact| (artifact.name.clone(), artifact.to_canonical())),
                )
                .expect("artifact digest names come from a map"),
            ),
            (
                "compiler_version",
                CanonicalValue::Uint(u64::from(COMPILER_VERSION)),
            ),
            (
                "construction_schema",
                construction_world_ir_schema().to_canonical(),
            ),
            (
                "invariants",
                CanonicalValue::Array(INVARIANTS.into_iter().map(CanonicalValue::text).collect()),
            ),
            (
                "passes",
                CanonicalValue::Array(PASSES.into_iter().map(CanonicalValue::text).collect()),
            ),
            (
                "primitive_catalog_version",
                CanonicalValue::Uint(u64::from(PRIMITIVE_CATALOG_VERSION)),
            ),
            (
                "produced_schemas",
                CanonicalValue::Array(
                    package_schema_ids()
                        .into_iter()
                        .map(|schema| schema.to_canonical())
                        .collect(),
                ),
            ),
            ("schema", compiler_receipts_schema().to_canonical()),
            ("source_schema", source_schema().to_canonical()),
            (
                "source_sha256",
                CanonicalValue::text(self.source_digest.to_hex()),
            ),
        ])
        .to_canonical_bytes()
    }
}

/// A complete, validated set of typed Gate K package artifacts before
/// filesystem publication.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompiledWorld {
    stable_ir: StableWorldIr,
    simulation: SimulationPlan,
    navigation: NavigationPlan,
    persistence: PersistencePlan,
    diagnostics: DiagnosticsPlan,
    registry: SchemaRegistry,
    receipts: Vec<u8>,
}

impl CompiledWorld {
    /// Stable World IR artifact.
    #[must_use]
    pub const fn stable_ir(&self) -> &StableWorldIr {
        &self.stable_ir
    }

    /// Simulation projection used to initialize runtime state.
    #[must_use]
    pub const fn simulation(&self) -> &SimulationPlan {
        &self.simulation
    }

    /// Navigation projection.
    #[must_use]
    pub const fn navigation(&self) -> &NavigationPlan {
        &self.navigation
    }

    /// Persistence projection.
    #[must_use]
    pub const fn persistence(&self) -> &PersistencePlan {
        &self.persistence
    }

    /// Diagnostics projection.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticsPlan {
        &self.diagnostics
    }

    /// Exact schema/owner registry.
    #[must_use]
    pub const fn registry(&self) -> &SchemaRegistry {
        &self.registry
    }

    /// Exact canonical member set for [`WorldPackage::write`].
    ///
    /// # Errors
    ///
    /// Returns `EK0406` only if a compiler-owned constant ceases to be a legal
    /// package member name.
    pub fn members(&self) -> Result<Vec<(MemberName, Vec<u8>)>, Diagnostic> {
        let members = self.member_map();
        members
            .into_iter()
            .map(|(name, bytes)| Ok((MemberName::new(&name)?, bytes)))
            .collect()
    }

    /// Revalidates the complete in-memory artifact set before publication.
    ///
    /// # Errors
    ///
    /// Returns the first member-set, schema, receipt, or cross-projection
    /// diagnostic.
    pub fn validate_artifacts(&self) -> Result<(), Diagnostic> {
        validate_member_values(&self.member_map())
    }

    fn member_map(&self) -> BTreeMap<String, Vec<u8>> {
        BTreeMap::from([
            (COMPILER_RECEIPTS_FILE.to_owned(), self.receipts.clone()),
            (
                DIAGNOSTICS_FILE.to_owned(),
                self.diagnostics.to_canonical_bytes(),
            ),
            (
                NAVIGATION_FILE.to_owned(),
                self.navigation.to_canonical_bytes(),
            ),
            (
                PERSISTENCE_FILE.to_owned(),
                self.persistence.to_canonical_bytes(),
            ),
            (SCHEMAS_FILE.to_owned(), self.registry.to_canonical_bytes()),
            (
                SIMULATION_FILE.to_owned(),
                self.simulation.to_canonical_bytes(),
            ),
            (
                WORLD_IR_FILE.to_owned(),
                self.stable_ir.to_canonical_bytes(),
            ),
        ])
    }
}

/// Compiles source into the exact complete package artifact set.
///
/// # Errors
///
/// Returns the first source, linker, stable-promotion, projection, registry, or
/// member-validation diagnostic. No filesystem path is written.
pub fn compile_world_package(source: &str, path: SourcePath) -> Result<CompiledWorld, Diagnostic> {
    let stable_ir = compile_world(source, path)?;
    let simulation = compile_simulation_plan(&stable_ir)?;
    let navigation = compile_navigation_plan(&stable_ir)?;
    let persistence = compile_persistence_plan(&stable_ir)?;
    let diagnostics = compile_diagnostics_plan(&stable_ir)?;
    nomos_projection::validate_light_projection_agreement(
        simulation.light_resolver(),
        &persistence,
        &diagnostics,
    )?;
    if simulation.movement_resolver().to_canonical_bytes()
        != navigation.movement_resolver().to_canonical_bytes()
    {
        return Err(inconsistent(
            "simulation and navigation movement resolver plans differ",
        ));
    }
    let registry = expected_registry()?;
    let artifacts = BTreeMap::from([
        (
            DIAGNOSTICS_FILE.to_owned(),
            diagnostics.to_canonical_bytes(),
        ),
        (NAVIGATION_FILE.to_owned(), navigation.to_canonical_bytes()),
        (
            PERSISTENCE_FILE.to_owned(),
            persistence.to_canonical_bytes(),
        ),
        (SCHEMAS_FILE.to_owned(), registry.to_canonical_bytes()),
        (SIMULATION_FILE.to_owned(), simulation.to_canonical_bytes()),
        (WORLD_IR_FILE.to_owned(), stable_ir.to_canonical_bytes()),
    ]);
    let receipts = CompilerReceipts::new(source.as_bytes(), &artifacts).to_canonical_bytes();
    let compiled = CompiledWorld {
        stable_ir,
        simulation,
        navigation,
        persistence,
        diagnostics,
        registry,
        receipts,
    };
    compiled.validate_artifacts()?;
    Ok(compiled)
}

/// Semantically validates a generic hash-verified package as one complete Gate
/// K compiled world.
///
/// # Errors
///
/// Returns `EK0411` for the wrong member set, `EK0412` for an invalid member
/// schema/shape, or `EK0413` when individually canonical members disagree.
pub fn validate_compiled_package(package: &WorldPackage) -> Result<(), Diagnostic> {
    let actual = package
        .manifest()
        .members()
        .iter()
        .map(|record| record.name().as_str().to_owned())
        .collect::<BTreeSet<_>>();
    let expected = PACKAGE_MEMBERS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::PACKAGE_MEMBER_SET_INVALID,
            format!("compiled package member set is {actual:?}; expected {expected:?}"),
        )
        .with_repair(RepairClass::RebuildFromSource));
    }
    let mut members = BTreeMap::new();
    for name in PACKAGE_MEMBERS {
        let member = MemberName::new(name)?;
        let bytes = package.member_bytes(&member).ok_or_else(|| {
            Diagnostic::new(
                nomos_core::diagnostic::codes::PACKAGE_MEMBER_MISSING,
                format!("compiled package has no `{name}` member"),
            )
            .with_repair(RepairClass::SupplyMissingMember)
        })?;
        members.insert(name.to_owned(), bytes.to_vec());
    }
    validate_member_values(&members)
}

fn validate_member_values(members: &BTreeMap<String, Vec<u8>>) -> Result<(), Diagnostic> {
    let values = members
        .iter()
        .map(|(name, bytes)| {
            nomos_core::canonical::read::parse_canonical(bytes)
                .map(|value| (name.clone(), value))
                .map_err(|diagnostic| invalid_member(name, diagnostic.message()))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    validate_world_ir(member(&values, WORLD_IR_FILE)?)?;
    validate_projection(
        member(&values, SIMULATION_FILE)?,
        &nomos_projection::simulation_schema(),
        &[
            "causal_edges",
            "entities",
            "light_resolver",
            "machines",
            "movement_resolver",
            "schema",
        ],
        SIMULATION_FILE,
    )?;
    nomos_projection::SimulationInitialization::from_canonical_bytes(
        members
            .get(SIMULATION_FILE)
            .expect("the exact member set includes simulation.json"),
    )?;
    validate_projection(
        member(&values, NAVIGATION_FILE)?,
        &nomos_projection::navigation_schema(),
        &["movement_resolver", "schema"],
        NAVIGATION_FILE,
    )?;
    validate_projection(
        member(&values, PERSISTENCE_FILE)?,
        &nomos_projection::persistence_schema(),
        &["entities", "light_resolver", "schema"],
        PERSISTENCE_FILE,
    )?;
    validate_projection(
        member(&values, DIAGNOSTICS_FILE)?,
        &nomos_projection::diagnostics_schema(),
        &["entities", "light_resolver", "schema"],
        DIAGNOSTICS_FILE,
    )?;

    if members.get(SCHEMAS_FILE) != Some(&expected_registry()?.to_canonical_bytes()) {
        return Err(invalid_member(
            SCHEMAS_FILE,
            "schema identities or authoritative owners differ from the exact package registry",
        ));
    }
    validate_receipts(member(&values, COMPILER_RECEIPTS_FILE)?, members)?;

    let simulation = object(member(&values, SIMULATION_FILE)?, SIMULATION_FILE)?;
    let navigation = object(member(&values, NAVIGATION_FILE)?, NAVIGATION_FILE)?;
    let persistence = object(member(&values, PERSISTENCE_FILE)?, PERSISTENCE_FILE)?;
    let diagnostics = object(member(&values, DIAGNOSTICS_FILE)?, DIAGNOSTICS_FILE)?;
    require_same(
        field(simulation, "movement_resolver", SIMULATION_FILE)?,
        field(navigation, "movement_resolver", NAVIGATION_FILE)?,
        "simulation and navigation movement resolver plans differ",
    )?;
    let simulation_light = field(simulation, "light_resolver", SIMULATION_FILE)?;
    require_same(
        simulation_light,
        field(persistence, "light_resolver", PERSISTENCE_FILE)?,
        "simulation and persistence light resolver plans differ",
    )?;
    require_same(
        simulation_light,
        field(diagnostics, "light_resolver", DIAGNOSTICS_FILE)?,
        "simulation and diagnostics light resolver plans differ",
    )?;
    Ok(())
}

fn validate_world_ir(value: &CanonicalValue) -> Result<(), Diagnostic> {
    let fields = object(value, WORLD_IR_FILE)?;
    require_exact_fields(
        fields,
        &[
            "catalog_values",
            "compiler_version",
            "construction_schema",
            "entities",
            "light_resolver",
            "movement_resolver",
            "movement_v1",
            "ownership_receipts",
            "primitive_catalog_version",
            "relations",
            "schema",
            "source_schema",
        ],
        WORLD_IR_FILE,
    )?;
    require_schema(fields, "schema", &stable_world_ir_schema(), WORLD_IR_FILE)?;
    require_schema(
        fields,
        "construction_schema",
        &construction_world_ir_schema(),
        WORLD_IR_FILE,
    )?;
    require_schema(fields, "source_schema", &source_schema(), WORLD_IR_FILE)?;
    require_version(fields, "compiler_version", COMPILER_VERSION, WORLD_IR_FILE)?;
    require_version(
        fields,
        "primitive_catalog_version",
        PRIMITIVE_CATALOG_VERSION,
        WORLD_IR_FILE,
    )?;
    let CanonicalValue::Array(rows) = field(fields, "movement_v1", WORLD_IR_FILE)? else {
        return Err(invalid_member(
            WORLD_IR_FILE,
            "`movement_v1` is not an array",
        ));
    };
    let mut entities = BTreeSet::new();
    for row in rows {
        let row_fields = object(row, WORLD_IR_FILE)?;
        require_exact_fields(
            row_fields,
            &["blocked_ground", "entity", "traversal_cost_ground"],
            "world-ir movement_v1 row",
        )?;
        let CanonicalValue::Text(entity) = field(row_fields, "entity", WORLD_IR_FILE)? else {
            return Err(invalid_member(WORLD_IR_FILE, "movement entity is not text"));
        };
        EntityId::parse(entity).map_err(|error| invalid_member(WORLD_IR_FILE, error.message()))?;
        if !entities.insert(entity.clone()) {
            return Err(invalid_member(
                WORLD_IR_FILE,
                "`movement_v1` repeats an entity",
            ));
        }
        let blocked = match field(row_fields, "blocked_ground", WORLD_IR_FILE)? {
            CanonicalValue::Bool(value) => *value,
            _ => {
                return Err(invalid_member(
                    WORLD_IR_FILE,
                    "`blocked_ground` is not boolean",
                ));
            }
        };
        let cost = field(row_fields, "traversal_cost_ground", WORLD_IR_FILE)?;
        let valid = if blocked {
            matches!(cost, CanonicalValue::Null)
        } else {
            unsigned(cost).is_some_and(|value| value > 0)
        };
        if !valid {
            return Err(invalid_member(
                WORLD_IR_FILE,
                "stable v1 movement is not blocked/null or traversable/positive-cost",
            ));
        }
    }
    if rows.is_empty() {
        return Err(invalid_member(
            WORLD_IR_FILE,
            "stable World IR has no v1 movement subjects",
        ));
    }
    let resolver = object(
        field(fields, "movement_resolver", WORLD_IR_FILE)?,
        "world-ir movement resolver",
    )?;
    require_exact_fields(
        resolver,
        &["coherence", "laws", "subjects"],
        "world-ir movement resolver",
    )?;
    let CanonicalValue::Array(subjects) = field(resolver, "subjects", WORLD_IR_FILE)? else {
        return Err(invalid_member(
            WORLD_IR_FILE,
            "movement resolver subjects are not an array",
        ));
    };
    let mut resolver_entities = BTreeSet::new();
    for subject in subjects {
        let subject = object(subject, "world-ir movement subject")?;
        require_exact_fields(
            subject,
            &["claims", "connectivity", "entity"],
            "world-ir movement subject",
        )?;
        let CanonicalValue::Text(entity) = field(subject, "entity", WORLD_IR_FILE)? else {
            return Err(invalid_member(
                WORLD_IR_FILE,
                "movement resolver entity is not text",
            ));
        };
        if !resolver_entities.insert(entity.clone()) {
            return Err(invalid_member(
                WORLD_IR_FILE,
                "movement resolver repeats an entity",
            ));
        }
    }
    if entities != resolver_entities {
        return Err(inconsistent(
            "stable-v1 movement rows do not match movement resolver subjects",
        ));
    }
    Ok(())
}

fn validate_projection(
    value: &CanonicalValue,
    schema: &SchemaId,
    expected_fields: &[&str],
    member_name: &str,
) -> Result<(), Diagnostic> {
    let fields = object(value, member_name)?;
    require_exact_fields(fields, expected_fields, member_name)?;
    require_schema(fields, "schema", schema, member_name)
}

fn validate_receipts(
    value: &CanonicalValue,
    members: &BTreeMap<String, Vec<u8>>,
) -> Result<(), Diagnostic> {
    let fields = object(value, COMPILER_RECEIPTS_FILE)?;
    require_exact_fields(
        fields,
        &[
            "artifacts",
            "compiler_version",
            "construction_schema",
            "invariants",
            "passes",
            "primitive_catalog_version",
            "produced_schemas",
            "schema",
            "source_schema",
            "source_sha256",
        ],
        COMPILER_RECEIPTS_FILE,
    )?;
    require_schema(
        fields,
        "schema",
        &compiler_receipts_schema(),
        COMPILER_RECEIPTS_FILE,
    )?;
    require_schema(
        fields,
        "construction_schema",
        &construction_world_ir_schema(),
        COMPILER_RECEIPTS_FILE,
    )?;
    require_schema(
        fields,
        "source_schema",
        &source_schema(),
        COMPILER_RECEIPTS_FILE,
    )?;
    require_version(
        fields,
        "compiler_version",
        COMPILER_VERSION,
        COMPILER_RECEIPTS_FILE,
    )?;
    require_version(
        fields,
        "primitive_catalog_version",
        PRIMITIVE_CATALOG_VERSION,
        COMPILER_RECEIPTS_FILE,
    )?;
    require_text_array(fields, "passes", &PASSES, COMPILER_RECEIPTS_FILE)?;
    require_text_array(fields, "invariants", &INVARIANTS, COMPILER_RECEIPTS_FILE)?;
    let expected_schemas = CanonicalValue::Array(
        package_schema_ids()
            .into_iter()
            .map(|schema| schema.to_canonical())
            .collect(),
    );
    require_same(
        field(fields, "produced_schemas", COMPILER_RECEIPTS_FILE)?,
        &expected_schemas,
        "compiler receipt produced schemas do not match package ownership",
    )?;
    let Some(CanonicalValue::Text(source_digest)) =
        fields.get(&FieldName::declared("source_sha256"))
    else {
        return Err(invalid_member(
            COMPILER_RECEIPTS_FILE,
            "`source_sha256` is not text",
        ));
    };
    if Sha256Digest::from_hex(source_digest).is_none() {
        return Err(invalid_member(
            COMPILER_RECEIPTS_FILE,
            "`source_sha256` is not a lowercase SHA-256 digest",
        ));
    }
    let CanonicalValue::Array(artifacts) = field(fields, "artifacts", COMPILER_RECEIPTS_FILE)?
    else {
        return Err(invalid_member(
            COMPILER_RECEIPTS_FILE,
            "`artifacts` is not an array",
        ));
    };
    let expected_artifacts = members
        .iter()
        .filter(|(name, _)| name.as_str() != COMPILER_RECEIPTS_FILE)
        .map(|(name, bytes)| (name.clone(), Sha256Digest::of_bytes(bytes)))
        .collect::<BTreeMap<_, _>>();
    if artifacts.len() != expected_artifacts.len() {
        return Err(inconsistent(
            "compiler receipt artifact set does not match package members",
        ));
    }
    let mut seen = BTreeSet::new();
    for artifact in artifacts {
        let artifact_fields = object(artifact, COMPILER_RECEIPTS_FILE)?;
        require_exact_fields(
            artifact_fields,
            &["name", "sha256"],
            "compiler receipt artifact row",
        )?;
        let (CanonicalValue::Text(name), CanonicalValue::Text(digest)) = (
            field(artifact_fields, "name", COMPILER_RECEIPTS_FILE)?,
            field(artifact_fields, "sha256", COMPILER_RECEIPTS_FILE)?,
        ) else {
            return Err(invalid_member(
                COMPILER_RECEIPTS_FILE,
                "artifact row name and digest must be text",
            ));
        };
        if !seen.insert(name.clone())
            || expected_artifacts.get(name).map(Sha256Digest::to_hex) != Some(digest.clone())
        {
            return Err(inconsistent(
                "compiler receipt artifact hashes do not match canonical package members",
            ));
        }
    }
    Ok(())
}

fn expected_registry() -> Result<SchemaRegistry, Diagnostic> {
    SchemaRegistry::new(vec![
        SchemaRegistration::new(
            COMPILER_RECEIPTS_FILE,
            compiler_receipts_schema(),
            SchemaOwner::Compiler,
        ),
        SchemaRegistration::new(
            DIAGNOSTICS_FILE,
            nomos_projection::diagnostics_schema(),
            SchemaOwner::Projection,
        ),
        SchemaRegistration::new(MANIFEST_FILE, manifest_schema(), SchemaOwner::Core),
        SchemaRegistration::new(
            NAVIGATION_FILE,
            nomos_projection::navigation_schema(),
            SchemaOwner::Projection,
        ),
        SchemaRegistration::new(
            PERSISTENCE_FILE,
            nomos_projection::persistence_schema(),
            SchemaOwner::Projection,
        ),
        SchemaRegistration::new(SCHEMAS_FILE, schema_registry_schema(), SchemaOwner::Schema),
        SchemaRegistration::new(
            SIMULATION_FILE,
            nomos_projection::simulation_schema(),
            SchemaOwner::Projection,
        ),
        SchemaRegistration::new(WORLD_IR_FILE, stable_world_ir_schema(), SchemaOwner::Schema),
    ])
}

fn package_schema_ids() -> Vec<SchemaId> {
    vec![
        compiler_receipts_schema(),
        nomos_projection::diagnostics_schema(),
        manifest_schema(),
        nomos_projection::navigation_schema(),
        nomos_projection::persistence_schema(),
        schema_registry_schema(),
        nomos_projection::simulation_schema(),
        stable_world_ir_schema(),
    ]
}

fn member<'a>(
    values: &'a BTreeMap<String, CanonicalValue>,
    name: &str,
) -> Result<&'a CanonicalValue, Diagnostic> {
    values
        .get(name)
        .ok_or_else(|| invalid_member(name, "member value is absent"))
}

fn object<'a>(
    value: &'a CanonicalValue,
    member_name: &str,
) -> Result<&'a BTreeMap<FieldName, CanonicalValue>, Diagnostic> {
    let CanonicalValue::Object(fields) = value else {
        return Err(invalid_member(
            member_name,
            "top-level value is not an object",
        ));
    };
    Ok(fields)
}

fn require_exact_fields(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &[&str],
    context: &str,
) -> Result<(), Diagnostic> {
    let actual = fields
        .keys()
        .map(FieldName::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(invalid_member(
            context,
            format!("fields are {actual:?}; expected {expected:?}"),
        ));
    }
    Ok(())
}

fn field<'a>(
    fields: &'a BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
    context: &str,
) -> Result<&'a CanonicalValue, Diagnostic> {
    fields
        .get(&FieldName::declared(name))
        .ok_or_else(|| invalid_member(context, format!("missing `{name}`")))
}

fn require_schema(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
    expected: &SchemaId,
    context: &str,
) -> Result<(), Diagnostic> {
    if field(fields, name, context)?.to_canonical_bytes()
        == expected.to_canonical().to_canonical_bytes()
    {
        Ok(())
    } else {
        Err(invalid_member(
            context,
            format!("`{name}` is not `{expected}`"),
        ))
    }
}

fn require_version(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
    expected: u32,
    context: &str,
) -> Result<(), Diagnostic> {
    if unsigned(field(fields, name, context)?) == Some(u64::from(expected)) {
        Ok(())
    } else {
        Err(invalid_member(
            context,
            format!("`{name}` is not supported version {expected}"),
        ))
    }
}

fn require_text_array<const N: usize>(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
    expected: &[&str; N],
    context: &str,
) -> Result<(), Diagnostic> {
    let value = CanonicalValue::Array(
        expected
            .iter()
            .map(|item| CanonicalValue::text(*item))
            .collect(),
    );
    require_same(
        field(fields, name, context)?,
        &value,
        &format!("{context} `{name}` is not the closed compiler vocabulary"),
    )
}

fn require_same(
    left: &CanonicalValue,
    right: &CanonicalValue,
    message: &str,
) -> Result<(), Diagnostic> {
    if left.to_canonical_bytes() == right.to_canonical_bytes() {
        Ok(())
    } else {
        Err(inconsistent(message))
    }
}

fn unsigned(value: &CanonicalValue) -> Option<u64> {
    match value {
        CanonicalValue::Int(value) => u64::try_from(*value).ok(),
        CanonicalValue::Uint(value) => Some(*value),
        _ => None,
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
