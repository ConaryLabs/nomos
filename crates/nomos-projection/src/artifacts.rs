//! Persistence and diagnostics projection artifacts.

use nomos_core::{CanonicalValue, Diagnostic, SchemaId};

use crate::state::sort_entities;
use crate::{LightResolverPlan, ProjectedEntity, diagnostics_schema, persistence_schema};

/// Implemented persistence projection.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PersistencePlan {
    schema: SchemaId,
    entities: Vec<ProjectedEntity>,
    light_resolver: LightResolverPlan,
}

impl PersistencePlan {
    /// Builds a persistence artifact with stable entity ordering.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic when one entity occurs twice.
    pub fn new(
        entities: Vec<ProjectedEntity>,
        light_resolver: LightResolverPlan,
    ) -> Result<Self, Diagnostic> {
        Ok(Self {
            schema: persistence_schema(),
            entities: sort_entities(entities)?,
            light_resolver,
        })
    }

    /// Projection schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Persistent entities in stable order.
    #[must_use]
    pub fn entities(&self) -> &[ProjectedEntity] {
        &self.entities
    }

    /// Shared compiler-projected light semantics.
    #[must_use]
    pub const fn light_resolver(&self) -> &LightResolverPlan {
        &self.light_resolver
    }

    /// Canonical persistence projection bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        artifact_value(&self.schema, &self.entities, &self.light_resolver).to_canonical_bytes()
    }
}

/// Implemented diagnostics projection.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DiagnosticsPlan {
    schema: SchemaId,
    entities: Vec<ProjectedEntity>,
    light_resolver: LightResolverPlan,
}

impl DiagnosticsPlan {
    /// Builds a diagnostics artifact with stable entity ordering.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic when one entity occurs twice.
    pub fn new(
        entities: Vec<ProjectedEntity>,
        light_resolver: LightResolverPlan,
    ) -> Result<Self, Diagnostic> {
        Ok(Self {
            schema: diagnostics_schema(),
            entities: sort_entities(entities)?,
            light_resolver,
        })
    }

    /// Projection schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Diagnosable entities in stable order.
    #[must_use]
    pub fn entities(&self) -> &[ProjectedEntity] {
        &self.entities
    }

    /// Shared compiler-projected light semantics.
    #[must_use]
    pub const fn light_resolver(&self) -> &LightResolverPlan {
        &self.light_resolver
    }

    /// Canonical diagnostics projection bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        artifact_value(&self.schema, &self.entities, &self.light_resolver).to_canonical_bytes()
    }
}

/// Verifies that independently versioned consumers received one light plan.
///
/// # Errors
///
/// Returns `EK0912` when persistence or diagnostics differs from simulation.
pub fn validate_light_projection_agreement(
    simulation: &LightResolverPlan,
    persistence: &PersistencePlan,
    diagnostics: &DiagnosticsPlan,
) -> Result<(), Diagnostic> {
    if simulation != persistence.light_resolver()
        || simulation != diagnostics.light_resolver()
        || persistence.light_resolver() != diagnostics.light_resolver()
    {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::LIGHT_PROJECTION_MISMATCH,
            "simulation, persistence, and diagnostics light resolver plans differ",
        ));
    }
    Ok(())
}

fn artifact_value(
    schema: &SchemaId,
    entities: &[ProjectedEntity],
    light_resolver: &LightResolverPlan,
) -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "entities",
            CanonicalValue::Array(entities.iter().map(ProjectedEntity::to_canonical).collect()),
        ),
        ("light_resolver", light_resolver.to_canonical()),
        ("schema", schema.to_canonical()),
    ])
}
