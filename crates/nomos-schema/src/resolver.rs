//! Construction-IR movement composition, coherence, and resolver preparation.

use std::collections::BTreeSet;

use nomos_core::canonical::keyed_array;
use nomos_core::id::StableId;
use nomos_core::{CanonicalValue, ClaimRef, Diagnostic, EntityId, Ident, RepairClass};

use crate::Cell;

/// Compiler-selected composition operation for one movement capability.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum MovementCompositionLaw {
    /// Ground blocking is true when any active blocker claim is true.
    AnyActiveBlocker,
    /// Ground traversal cost is the maximum active cost claim.
    MaximumActiveCost,
}

impl MovementCompositionLaw {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AnyActiveBlocker => "any_active_blocker",
            Self::MaximumActiveCost => "maximum_active_cost",
        }
    }

    fn to_canonical(self) -> CanonicalValue {
        CanonicalValue::object_declared([("operation", CanonicalValue::text(self.as_str()))])
    }
}

/// Cross-capability coherence rule that emits one ground movement fact.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GroundMovementCoherence {
    channel: Ident,
    base_cost: u32,
    requires_connectivity: bool,
}

impl GroundMovementCoherence {
    /// Builds the blocker-before-cost coherence rule.
    ///
    /// # Errors
    ///
    /// Returns `EK0909` when the base traversal cost is zero.
    pub fn new(
        channel: Ident,
        base_cost: u32,
        requires_connectivity: bool,
    ) -> Result<Self, Diagnostic> {
        if base_cost == 0 {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::MOVEMENT_DISPOSITION_INVALID,
                "ground movement base cost must be positive",
            ));
        }
        Ok(Self {
            channel,
            base_cost,
            requires_connectivity,
        })
    }

    /// Movement channel.
    #[must_use]
    pub const fn channel(&self) -> &Ident {
        &self.channel
    }

    /// Positive lattice base step cost.
    #[must_use]
    pub const fn base_cost(&self) -> u32 {
        self.base_cost
    }

    /// Whether traversability requires compiler-derived connectivity.
    #[must_use]
    pub const fn requires_connectivity(&self) -> bool {
        self.requires_connectivity
    }

    fn stable_key(&self) -> &Ident {
        &self.channel
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("base_cost", CanonicalValue::Uint(u64::from(self.base_cost))),
            ("blockers_before_cost", CanonicalValue::Bool(true)),
            ("channel", CanonicalValue::text(self.channel.as_str())),
            (
                "requires_connectivity",
                CanonicalValue::Bool(self.requires_connectivity),
            ),
        ])
    }
}

/// Compiler-derived proof shape for one ground movement subject.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum GroundConnectivity {
    /// A horizontal face connects two adjacent lattice cells.
    FaceAdjacent {
        /// Cell on the authored side of the face.
        first: Cell,
        /// Adjacent cell across the face.
        second: Cell,
    },
    /// A nonempty closed lattice region is internally connected for Gate K.
    Region {
        /// Component-wise minimum cell.
        min: Cell,
        /// Component-wise maximum cell.
        max: Cell,
    },
}

impl GroundConnectivity {
    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::FaceAdjacent { first, second } => CanonicalValue::object_declared([
                ("first", first.to_canonical()),
                ("kind", CanonicalValue::text("face_adjacent")),
                ("second", second.to_canonical()),
            ]),
            Self::Region { min, max } => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("region")),
                ("max", max.to_canonical()),
                ("min", min.to_canonical()),
            ]),
        }
    }
}

/// Prepared movement inputs for one entity-level semantic subject.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MovementResolverSubject {
    entity: EntityId,
    connectivity: GroundConnectivity,
    claims: Vec<ClaimRef>,
}

impl MovementResolverSubject {
    /// Builds one subject and imposes stable claim-reference ordering.
    ///
    /// # Errors
    ///
    /// Returns `EK0901` for duplicate claims and `EK0902` when a claim belongs
    /// to another entity.
    pub fn new(
        entity: EntityId,
        connectivity: GroundConnectivity,
        mut claims: Vec<ClaimRef>,
    ) -> Result<Self, Diagnostic> {
        let mut seen = BTreeSet::new();
        for claim in &claims {
            if claim.namespace().entity() != &entity {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_CLAIM_ENTITY_MISMATCH,
                    format!("movement claim `{claim}` does not belong to subject `{entity}`"),
                ));
            }
            if !seen.insert(claim.clone()) {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_DUPLICATE_IDENTITY,
                    format!("movement claim `{claim}` occurs more than once"),
                )
                .with_repair(RepairClass::RemoveDuplicateDeclaration));
            }
        }
        claims.sort();
        Ok(Self {
            entity,
            connectivity,
            claims,
        })
    }

    /// Subject entity.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Compiler-derived lattice connectivity.
    #[must_use]
    pub const fn connectivity(&self) -> &GroundConnectivity {
        &self.connectivity
    }

    /// Movement claim references in stable order.
    #[must_use]
    pub fn claims(&self) -> &[ClaimRef] {
        &self.claims
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("connectivity", self.connectivity.to_canonical()),
            ("entity", self.entity.to_canonical()),
            (
                "claims",
                CanonicalValue::Array(self.claims.iter().map(StableId::to_canonical).collect()),
            ),
        ])
    }
}

/// Construction-IR plan for the Gate K ground movement resolver.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MovementResolverPlan {
    laws: Vec<MovementCompositionLaw>,
    coherence: Vec<GroundMovementCoherence>,
    subjects: Vec<MovementResolverSubject>,
}

impl MovementResolverPlan {
    /// Builds a resolver plan and imposes stable semantic ordering.
    ///
    /// # Errors
    ///
    /// Returns `EK0901` when a law, coherence channel, or subject repeats.
    pub fn new(
        mut laws: Vec<MovementCompositionLaw>,
        mut coherence: Vec<GroundMovementCoherence>,
        mut subjects: Vec<MovementResolverSubject>,
    ) -> Result<Self, Diagnostic> {
        require_unique(laws.iter().copied(), "movement composition law")?;
        require_unique(
            coherence.iter().map(GroundMovementCoherence::stable_key),
            "movement coherence channel",
        )?;
        require_unique(
            subjects.iter().map(MovementResolverSubject::entity),
            "movement resolver subject",
        )?;
        laws.sort();
        coherence.sort();
        subjects.sort_by(|left, right| left.entity.cmp(&right.entity));
        Ok(Self {
            laws,
            coherence,
            subjects,
        })
    }

    /// Gate K's two movement composition laws, one ground coherence rule, and
    /// no subjects. Manual construction tests may attach subjects later.
    #[must_use]
    pub fn empty_gate_k() -> Self {
        Self::new(
            vec![
                MovementCompositionLaw::AnyActiveBlocker,
                MovementCompositionLaw::MaximumActiveCost,
            ],
            vec![
                GroundMovementCoherence::new(
                    Ident::new("ground").expect("built-in channel is legal"),
                    1,
                    true,
                )
                .expect("built-in cost is positive"),
            ],
            Vec::new(),
        )
        .expect("built-in resolver identities are unique")
    }

    /// Composition laws in stable operation order.
    #[must_use]
    pub fn laws(&self) -> &[MovementCompositionLaw] {
        &self.laws
    }

    /// Coherence rules in stable channel order.
    #[must_use]
    pub fn coherence(&self) -> &[GroundMovementCoherence] {
        &self.coherence
    }

    /// Movement subjects in stable entity order.
    #[must_use]
    pub fn subjects(&self) -> &[MovementResolverSubject] {
        &self.subjects
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "coherence",
                CanonicalValue::Array(
                    self.coherence
                        .iter()
                        .map(GroundMovementCoherence::to_canonical)
                        .collect(),
                ),
            ),
            (
                "laws",
                CanonicalValue::Array(self.laws.iter().map(|law| law.to_canonical()).collect()),
            ),
            (
                "subjects",
                keyed_array(
                    self.subjects
                        .iter()
                        .map(|subject| (subject.entity.clone(), subject.to_canonical())),
                )
                .expect("MovementResolverPlan validates unique subjects"),
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
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RESOLVER_DUPLICATE_IDENTITY,
                format!("{identity} occurs more than once"),
            )
            .with_repair(RepairClass::RemoveDuplicateDeclaration));
        }
    }
    Ok(())
}
