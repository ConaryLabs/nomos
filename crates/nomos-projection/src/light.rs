//! Shared runtime-facing light resolver and effective-fact types.

use std::collections::BTreeSet;

use nomos_core::canonical::keyed_array;
use nomos_core::id::StableId;
use nomos_core::{
    CanonicalValue, ClaimRef, Diagnostic, EntityId, RepairClass, SchemaId, SourceSpan,
};

use crate::{ProjectedActivation, diagnostics_schema, persistence_schema, simulation_schema};

/// A projection that consumes effective light facts.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LightProjectionConsumer {
    /// Diagnostics and forensic projection.
    Diagnostics,
    /// Persistence projection.
    Persistence,
    /// Simulation projection.
    Simulation,
}

impl LightProjectionConsumer {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Diagnostics => "diagnostics",
            Self::Persistence => "persistence",
            Self::Simulation => "simulation",
        }
    }

    /// Independently versioned projection schema receiving the fact.
    #[must_use]
    pub fn schema(self) -> SchemaId {
        match self {
            Self::Diagnostics => diagnostics_schema(),
            Self::Persistence => persistence_schema(),
            Self::Simulation => simulation_schema(),
        }
    }
}

/// One compiler-projected positive light-emission claim.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LightClaim {
    id: ClaimRef,
    activation: ProjectedActivation,
    value: bool,
    source: SourceSpan,
}

impl LightClaim {
    /// Builds one projected claim. Runtime validation rejects `false`, which
    /// exists only so a tampered projection can be represented and refused.
    #[must_use]
    pub fn new(
        id: ClaimRef,
        activation: ProjectedActivation,
        value: bool,
        source: SourceSpan,
    ) -> Self {
        Self {
            id,
            activation,
            value,
            source,
        }
    }

    /// Stable claim identity.
    #[must_use]
    pub const fn id(&self) -> &ClaimRef {
        &self.id
    }

    /// Runtime activation expression.
    #[must_use]
    pub const fn activation(&self) -> &ProjectedActivation {
        &self.activation
    }

    /// Positive value supplied while active.
    #[must_use]
    pub const fn value(&self) -> bool {
        self.value
    }

    /// Source declaration span for forensic lookup.
    #[must_use]
    pub const fn source(&self) -> &SourceSpan {
        &self.source
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("activation", self.activation.to_canonical()),
            ("capability", CanonicalValue::text("emits_light")),
            ("id", self.id.to_canonical()),
            ("source", span_to_canonical(&self.source)),
            ("value", CanonicalValue::Bool(self.value)),
        ])
    }
}

/// Prepared runtime inputs for one entity-level light subject.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LightSubject {
    entity: EntityId,
    claims: Vec<LightClaim>,
}

impl LightSubject {
    /// Builds one subject with stable claim ordering.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for duplicate or cross-entity claims.
    pub fn new(entity: EntityId, mut claims: Vec<LightClaim>) -> Result<Self, Diagnostic> {
        let mut seen = BTreeSet::new();
        for claim in &claims {
            if claim.id().namespace().entity() != &entity {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_CLAIM_ENTITY_MISMATCH,
                    format!("light claim `{}` does not belong to `{entity}`", claim.id()),
                ));
            }
            if !seen.insert(claim.id().clone()) {
                return Err(duplicate("light claim"));
            }
        }
        claims.sort_by(|left, right| left.id().cmp(right.id()));
        Ok(Self { entity, claims })
    }

    /// Subject entity.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Typed claims in stable identity order.
    #[must_use]
    pub fn claims(&self) -> &[LightClaim] {
        &self.claims
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("entity", self.entity.to_canonical()),
            (
                "claims",
                keyed_array(
                    self.claims
                        .iter()
                        .map(|claim| (claim.id().clone(), claim.to_canonical())),
                )
                .expect("LightSubject validates unique claims"),
            ),
        ])
    }
}

/// Shared simulation/persistence/diagnostics light resolver plan.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LightResolverPlan {
    union_active: bool,
    consumers: Vec<LightProjectionConsumer>,
    subjects: Vec<LightSubject>,
}

impl LightResolverPlan {
    /// Builds one projected light resolver with stable ordering.
    ///
    /// # Errors
    ///
    /// Returns `EK0901` when a consumer or subject repeats.
    pub fn new(
        union_active: bool,
        mut consumers: Vec<LightProjectionConsumer>,
        mut subjects: Vec<LightSubject>,
    ) -> Result<Self, Diagnostic> {
        require_unique(consumers.iter().copied(), "light projection consumer")?;
        require_unique(
            subjects.iter().map(LightSubject::entity),
            "light resolver subject",
        )?;
        consumers.sort();
        subjects.sort_by(|left, right| left.entity.cmp(&right.entity));
        Ok(Self {
            union_active,
            consumers,
            subjects,
        })
    }

    /// Empty but correctly shaped Gate K plan.
    #[must_use]
    pub fn empty_gate_k() -> Self {
        Self::new(
            true,
            vec![
                LightProjectionConsumer::Diagnostics,
                LightProjectionConsumer::Persistence,
                LightProjectionConsumer::Simulation,
            ],
            Vec::new(),
        )
        .expect("built-in light resolver identities are unique")
    }

    /// Whether active claims compose by union.
    #[must_use]
    pub const fn union_active(&self) -> bool {
        self.union_active
    }

    /// Declared projection consumers in stable order.
    #[must_use]
    pub fn consumers(&self) -> &[LightProjectionConsumer] {
        &self.consumers
    }

    /// Subjects in stable entity order.
    #[must_use]
    pub fn subjects(&self) -> &[LightSubject] {
        &self.subjects
    }

    /// Canonical bytes shared verbatim by all three consumer projections.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
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
            (
                "subjects",
                keyed_array(
                    self.subjects
                        .iter()
                        .map(|subject| (subject.entity.clone(), subject.to_canonical())),
                )
                .expect("LightResolverPlan validates unique subjects"),
            ),
            ("union_active", CanonicalValue::Bool(self.union_active)),
        ])
    }
}

/// Effective light fact for one subject.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResolvedLight {
    entity: EntityId,
    emitting: bool,
    reasons: Vec<ClaimRef>,
}

impl ResolvedLight {
    /// Builds one fact with sorted, unique active claim reasons.
    ///
    /// # Errors
    ///
    /// Returns `EK0901` for duplicate reasons or `EK0910` when an emitting
    /// fact has no positive reason.
    pub fn new(
        entity: EntityId,
        emitting: bool,
        mut reasons: Vec<ClaimRef>,
    ) -> Result<Self, Diagnostic> {
        require_unique(reasons.iter().cloned(), "light reason")?;
        reasons.sort();
        if emitting != !reasons.is_empty() {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::LIGHT_CLAIM_INVALID,
                "effective light emission must equal the union of its active reasons",
            ));
        }
        Ok(Self {
            entity,
            emitting,
            reasons,
        })
    }

    /// Subject entity.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Whether the subject currently emits light.
    #[must_use]
    pub const fn emitting(&self) -> bool {
        self.emitting
    }

    /// Active claims producing the union result.
    #[must_use]
    pub fn reasons(&self) -> &[ClaimRef] {
        &self.reasons
    }

    /// Canonical form of one subject's effective light fact.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("emitting", CanonicalValue::Bool(self.emitting)),
            ("entity", self.entity.to_canonical()),
            (
                "reasons",
                CanonicalValue::Array(self.reasons.iter().map(StableId::to_canonical).collect()),
            ),
        ])
    }
}

/// Stable effective light facts for every projected subject.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResolvedLightFacts {
    facts: Vec<ResolvedLight>,
}

impl ResolvedLightFacts {
    /// Builds a stable subject-keyed fact collection.
    ///
    /// # Errors
    ///
    /// Returns `EK0901` when one subject occurs more than once.
    pub fn new(mut facts: Vec<ResolvedLight>) -> Result<Self, Diagnostic> {
        require_unique(
            facts.iter().map(ResolvedLight::entity),
            "resolved light subject",
        )?;
        facts.sort_by(|left, right| left.entity.cmp(&right.entity));
        Ok(Self { facts })
    }

    /// Facts in stable entity order.
    #[must_use]
    pub fn facts(&self) -> &[ResolvedLight] {
        &self.facts
    }

    /// Looks up one subject.
    #[must_use]
    pub fn get(&self, entity: &EntityId) -> Option<&ResolvedLight> {
        self.facts
            .binary_search_by(|fact| fact.entity.cmp(entity))
            .ok()
            .map(|index| &self.facts[index])
    }

    /// Canonical entity-ordered array of every resolved subject.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        keyed_array(
            self.facts
                .iter()
                .map(|fact| (fact.entity.clone(), fact.to_canonical())),
        )
        .expect("ResolvedLightFacts validates unique subjects")
    }

    /// Canonical bytes for deterministic evidence.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
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

fn span_to_canonical(span: &SourceSpan) -> CanonicalValue {
    let (byte_start, byte_end) = span.byte_range();
    let (line, column) = span.position();
    CanonicalValue::object_declared([
        ("byte_end", CanonicalValue::Uint(u64::from(byte_end))),
        ("byte_start", CanonicalValue::Uint(u64::from(byte_start))),
        ("column", CanonicalValue::Uint(u64::from(column))),
        ("line", CanonicalValue::Uint(u64::from(line))),
        ("path", CanonicalValue::text(span.path().as_str())),
    ])
}
