//! Construction-IR light composition and resolver preparation.

use std::collections::BTreeSet;

use nomos_core::canonical::keyed_array;
use nomos_core::id::StableId;
use nomos_core::{CanonicalValue, ClaimRef, Diagnostic, EntityId, RepairClass};

use crate::ProjectionConsumer;

/// Compiler-selected composition operation for effective light emission.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LightCompositionLaw {
    /// Effective emission is the union of every active positive claim.
    Union,
}

impl LightCompositionLaw {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Union => "union",
        }
    }

    fn to_canonical(self) -> CanonicalValue {
        CanonicalValue::object_declared([("operation", CanonicalValue::text(self.as_str()))])
    }
}

/// Prepared light inputs for one entity-level semantic subject.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LightResolverSubject {
    entity: EntityId,
    claims: Vec<ClaimRef>,
}

impl LightResolverSubject {
    /// Builds one subject and imposes stable claim-reference ordering.
    ///
    /// # Errors
    ///
    /// Returns `EK0901` for duplicate claims and `EK0902` when a claim belongs
    /// to another entity.
    pub fn new(entity: EntityId, mut claims: Vec<ClaimRef>) -> Result<Self, Diagnostic> {
        let mut seen = BTreeSet::new();
        for claim in &claims {
            if claim.namespace().entity() != &entity {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_CLAIM_ENTITY_MISMATCH,
                    format!("light claim `{claim}` does not belong to subject `{entity}`"),
                ));
            }
            if !seen.insert(claim.clone()) {
                return Err(duplicate("light claim"));
            }
        }
        claims.sort();
        Ok(Self { entity, claims })
    }

    /// Subject entity.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Light claim references in stable order.
    #[must_use]
    pub fn claims(&self) -> &[ClaimRef] {
        &self.claims
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("entity", self.entity.to_canonical()),
            (
                "claims",
                CanonicalValue::Array(self.claims.iter().map(StableId::to_canonical).collect()),
            ),
        ])
    }
}

/// Construction-IR plan for the Gate K light resolver.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LightResolverPlan {
    law: LightCompositionLaw,
    consumers: Vec<ProjectionConsumer>,
    subjects: Vec<LightResolverSubject>,
}

impl LightResolverPlan {
    /// Builds a light resolver plan with stable consumer and subject order.
    ///
    /// # Errors
    ///
    /// Returns `EK0901` when a consumer or subject repeats.
    pub fn new(
        law: LightCompositionLaw,
        mut consumers: Vec<ProjectionConsumer>,
        mut subjects: Vec<LightResolverSubject>,
    ) -> Result<Self, Diagnostic> {
        require_unique(consumers.iter().copied(), "light projection consumer")?;
        require_unique(
            subjects.iter().map(LightResolverSubject::entity),
            "light resolver subject",
        )?;
        consumers.sort();
        subjects.sort_by(|left, right| left.entity.cmp(&right.entity));
        Ok(Self {
            law,
            consumers,
            subjects,
        })
    }

    /// Gate K's union law and implemented light consumers, with no subjects.
    #[must_use]
    pub fn empty_gate_k() -> Self {
        Self::new(
            LightCompositionLaw::Union,
            vec![
                ProjectionConsumer::Diagnostics,
                ProjectionConsumer::Persistence,
                ProjectionConsumer::Simulation,
            ],
            Vec::new(),
        )
        .expect("built-in light resolver identities are unique")
    }

    /// Effective light composition law.
    #[must_use]
    pub const fn law(&self) -> LightCompositionLaw {
        self.law
    }

    /// Projection consumers in stable order.
    #[must_use]
    pub fn consumers(&self) -> &[ProjectionConsumer] {
        &self.consumers
    }

    /// Light subjects in stable entity order.
    #[must_use]
    pub fn subjects(&self) -> &[LightResolverSubject] {
        &self.subjects
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "consumers",
                CanonicalValue::Array(
                    self.consumers
                        .iter()
                        .map(|consumer| CanonicalValue::text(consumer.as_str()))
                        .collect(),
                ),
            ),
            ("law", self.law.to_canonical()),
            (
                "subjects",
                keyed_array(
                    self.subjects
                        .iter()
                        .map(|subject| (subject.entity.clone(), subject.to_canonical())),
                )
                .expect("LightResolverPlan validates unique subjects"),
            ),
        ])
    }
}

fn require_unique<T: Ord>(
    values: impl IntoIterator<Item = T>,
    identity: &str,
) -> Result<(), Diagnostic> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(duplicate(identity));
        }
    }
    Ok(())
}

fn duplicate(identity: &str) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RESOLVER_DUPLICATE_IDENTITY,
        format!("{identity} occurs more than once"),
    )
    .with_repair(RepairClass::RemoveDuplicateDeclaration)
}
