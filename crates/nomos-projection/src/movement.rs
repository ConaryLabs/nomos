//! Shared runtime-facing ground movement resolver and effective-fact types.

use std::collections::BTreeSet;

use nomos_core::canonical::keyed_array;
use nomos_core::id::StableId;
use nomos_core::{
    CanonicalValue, ClaimRef, Diagnostic, EntityId, Ident, NamespaceId, RepairClass, SchemaId,
    SourceSpan,
};

use crate::navigation_schema;

/// One projected integer lattice cell.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LatticeCell {
    x: i32,
    y: i32,
    z: i32,
}

impl LatticeCell {
    /// Builds one projected lattice cell.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// X coordinate.
    #[must_use]
    pub const fn x(self) -> i32 {
        self.x
    }

    /// Y coordinate.
    #[must_use]
    pub const fn y(self) -> i32 {
        self.y
    }

    /// Z coordinate.
    #[must_use]
    pub const fn z(self) -> i32 {
        self.z
    }

    fn to_canonical(self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("x", CanonicalValue::Int(i64::from(self.x))),
            ("y", CanonicalValue::Int(i64::from(self.y))),
            ("z", CanonicalValue::Int(i64::from(self.z))),
        ])
    }
}

/// Compiler-derived ground connectivity for one movement subject.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum MovementConnectivity {
    /// A horizontal face joins two adjacent lattice cells.
    FaceAdjacent {
        /// Cell on the authored side.
        first: LatticeCell,
        /// Cell across the face.
        second: LatticeCell,
    },
    /// A nonempty closed lattice region.
    Region {
        /// Component-wise minimum cell.
        min: LatticeCell,
        /// Component-wise maximum cell.
        max: LatticeCell,
    },
}

impl MovementConnectivity {
    fn to_canonical(&self) -> CanonicalValue {
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

/// Runtime-facing typed activation expression.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProjectedActivation {
    /// Always active.
    Always,
    /// Active when one machine equals the named state.
    StateEquals {
        /// Machine namespace.
        namespace: NamespaceId,
        /// Required state.
        state: Ident,
    },
    /// Active when any child is active.
    Any(Vec<ProjectedActivation>),
    /// Active when every child is active.
    All(Vec<ProjectedActivation>),
    /// Logical negation.
    Not(Box<ProjectedActivation>),
}

impl ProjectedActivation {
    /// Visits every state predicate in deterministic expression order.
    pub fn visit_state_equals(&self, visitor: &mut impl FnMut(&NamespaceId, &Ident)) {
        match self {
            Self::Always => {}
            Self::StateEquals { namespace, state } => visitor(namespace, state),
            Self::Any(children) | Self::All(children) => {
                for child in children {
                    child.visit_state_equals(visitor);
                }
            }
            Self::Not(child) => child.visit_state_equals(visitor),
        }
    }

    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Always => {
                CanonicalValue::object_declared([("kind", CanonicalValue::text("always"))])
            }
            Self::StateEquals { namespace, state } => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("state_equals")),
                ("namespace", namespace.to_canonical()),
                ("state", CanonicalValue::text(state.as_str())),
            ]),
            Self::Any(children) => activation_group("any", children),
            Self::All(children) => activation_group("all", children),
            Self::Not(child) => CanonicalValue::object_declared([
                ("child", child.to_canonical()),
                ("kind", CanonicalValue::text("not")),
            ]),
        }
    }
}

fn activation_group(kind: &'static str, children: &[ProjectedActivation]) -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "children",
            CanonicalValue::Array(
                children
                    .iter()
                    .map(ProjectedActivation::to_canonical)
                    .collect(),
            ),
        ),
        ("kind", CanonicalValue::text(kind)),
    ])
}

/// One typed movement claim projected from the compiler-owned catalog.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MovementClaim {
    /// Boolean ground blocker.
    Blocker {
        /// Stable claim identity.
        id: ClaimRef,
        /// Runtime activation expression.
        activation: ProjectedActivation,
        /// Boolean value supplied while active.
        value: bool,
        /// Source declaration span for explanation lookup.
        source: SourceSpan,
    },
    /// Positive ground traversal cost.
    TraversalCost {
        /// Stable claim identity.
        id: ClaimRef,
        /// Runtime activation expression.
        activation: ProjectedActivation,
        /// Positive cost supplied while active.
        cost: u32,
        /// Source declaration span for explanation lookup.
        source: SourceSpan,
    },
}

impl MovementClaim {
    /// Builds a boolean blocker claim.
    #[must_use]
    pub fn blocker(
        id: ClaimRef,
        activation: ProjectedActivation,
        value: bool,
        source: SourceSpan,
    ) -> Self {
        Self::Blocker {
            id,
            activation,
            value,
            source,
        }
    }

    /// Builds a positive traversal-cost claim.
    ///
    /// # Errors
    ///
    /// Returns `EK0909` when `cost` is zero.
    pub fn traversal_cost(
        id: ClaimRef,
        activation: ProjectedActivation,
        cost: u32,
        source: SourceSpan,
    ) -> Result<Self, Diagnostic> {
        if cost == 0 {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::MOVEMENT_DISPOSITION_INVALID,
                format!("traversal-cost claim `{id}` must be positive"),
            ));
        }
        Ok(Self::TraversalCost {
            id,
            activation,
            cost,
            source,
        })
    }

    /// Stable claim identity.
    #[must_use]
    pub const fn id(&self) -> &ClaimRef {
        match self {
            Self::Blocker { id, .. } | Self::TraversalCost { id, .. } => id,
        }
    }

    /// Runtime activation expression.
    #[must_use]
    pub const fn activation(&self) -> &ProjectedActivation {
        match self {
            Self::Blocker { activation, .. } | Self::TraversalCost { activation, .. } => activation,
        }
    }

    /// Source declaration span.
    #[must_use]
    pub const fn source(&self) -> &SourceSpan {
        match self {
            Self::Blocker { source, .. } | Self::TraversalCost { source, .. } => source,
        }
    }

    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Blocker {
                id,
                activation,
                value,
                source,
            } => CanonicalValue::object_declared([
                ("activation", activation.to_canonical()),
                ("capability", CanonicalValue::text("blocks_ground")),
                ("id", id.to_canonical()),
                ("source", span_to_canonical(source)),
                ("value", CanonicalValue::Bool(*value)),
            ]),
            Self::TraversalCost {
                id,
                activation,
                cost,
                source,
            } => CanonicalValue::object_declared([
                ("activation", activation.to_canonical()),
                ("capability", CanonicalValue::text("traversal_cost_ground")),
                ("id", id.to_canonical()),
                ("source", span_to_canonical(source)),
                ("value", CanonicalValue::Uint(u64::from(*cost))),
            ]),
        }
    }
}

/// Prepared runtime inputs for one entity-level movement subject.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MovementSubject {
    entity: EntityId,
    connectivity: MovementConnectivity,
    claims: Vec<MovementClaim>,
}

impl MovementSubject {
    /// Builds one subject with stable claim ordering.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for duplicate or cross-entity claims.
    pub fn new(
        entity: EntityId,
        connectivity: MovementConnectivity,
        mut claims: Vec<MovementClaim>,
    ) -> Result<Self, Diagnostic> {
        let mut seen = BTreeSet::new();
        for claim in &claims {
            if claim.id().namespace().entity() != &entity {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RESOLVER_CLAIM_ENTITY_MISMATCH,
                    format!(
                        "movement claim `{}` does not belong to `{entity}`",
                        claim.id()
                    ),
                ));
            }
            if !seen.insert(claim.id().clone()) {
                return Err(duplicate("movement claim"));
            }
        }
        claims.sort_by(|left, right| left.id().cmp(right.id()));
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
    pub const fn connectivity(&self) -> &MovementConnectivity {
        &self.connectivity
    }

    /// Typed claims in stable identity order.
    #[must_use]
    pub fn claims(&self) -> &[MovementClaim] {
        &self.claims
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("connectivity", self.connectivity.to_canonical()),
            ("entity", self.entity.to_canonical()),
            (
                "claims",
                keyed_array(
                    self.claims
                        .iter()
                        .map(|claim| (claim.id().clone(), claim.to_canonical())),
                )
                .expect("MovementSubject validates unique claims"),
            ),
        ])
    }
}

/// Shared simulation/navigation movement resolver plan.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MovementResolverPlan {
    channel: Ident,
    base_cost: u32,
    blockers_any_active: bool,
    costs_maximum_active: bool,
    blockers_before_cost: bool,
    requires_connectivity: bool,
    subjects: Vec<MovementSubject>,
}

impl MovementResolverPlan {
    /// Builds the shared Gate K movement resolver plan.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for zero base cost or duplicate subjects.
    pub fn new(
        channel: Ident,
        base_cost: u32,
        blockers_any_active: bool,
        costs_maximum_active: bool,
        blockers_before_cost: bool,
        requires_connectivity: bool,
        mut subjects: Vec<MovementSubject>,
    ) -> Result<Self, Diagnostic> {
        if base_cost == 0 {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::MOVEMENT_DISPOSITION_INVALID,
                "movement resolver base cost must be positive",
            ));
        }
        let mut seen = BTreeSet::new();
        for subject in &subjects {
            if !seen.insert(subject.entity().clone()) {
                return Err(duplicate("movement resolver subject"));
            }
        }
        subjects.sort_by(|left, right| left.entity.cmp(&right.entity));
        Ok(Self {
            channel,
            base_cost,
            blockers_any_active,
            costs_maximum_active,
            blockers_before_cost,
            requires_connectivity,
            subjects,
        })
    }

    /// Empty plan used by projections that predate SW-E data attachment.
    #[must_use]
    pub fn empty_gate_k() -> Self {
        Self::new(
            Ident::new("ground").expect("built-in channel is legal"),
            1,
            true,
            true,
            true,
            true,
            Vec::new(),
        )
        .expect("built-in resolver plan is valid")
    }

    /// Movement channel.
    #[must_use]
    pub const fn channel(&self) -> &Ident {
        &self.channel
    }

    /// Positive base step cost.
    #[must_use]
    pub const fn base_cost(&self) -> u32 {
        self.base_cost
    }

    /// Whether blockers compose by any active claim.
    #[must_use]
    pub const fn blockers_any_active(&self) -> bool {
        self.blockers_any_active
    }

    /// Whether costs compose by maximum active claim.
    #[must_use]
    pub const fn costs_maximum_active(&self) -> bool {
        self.costs_maximum_active
    }

    /// Whether blockers take precedence over traversal costs.
    #[must_use]
    pub const fn blockers_before_cost(&self) -> bool {
        self.blockers_before_cost
    }

    /// Whether traversability requires compiled connectivity.
    #[must_use]
    pub const fn requires_connectivity(&self) -> bool {
        self.requires_connectivity
    }

    /// Subjects in stable entity order.
    #[must_use]
    pub fn subjects(&self) -> &[MovementSubject] {
        &self.subjects
    }

    /// Canonical bytes shared verbatim by simulation and navigation plans.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("base_cost", CanonicalValue::Uint(u64::from(self.base_cost))),
            (
                "blockers_any_active",
                CanonicalValue::Bool(self.blockers_any_active),
            ),
            (
                "blockers_before_cost",
                CanonicalValue::Bool(self.blockers_before_cost),
            ),
            ("channel", CanonicalValue::text(self.channel.as_str())),
            (
                "costs_maximum_active",
                CanonicalValue::Bool(self.costs_maximum_active),
            ),
            (
                "requires_connectivity",
                CanonicalValue::Bool(self.requires_connectivity),
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

/// One effective ground movement answer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MovementDisposition {
    /// At least one blocker survives composition.
    Blocked {
        /// Nonempty stable claim references explaining the block.
        reasons: Vec<ClaimRef>,
    },
    /// No blocker survives and compiler-derived connectivity is valid.
    Traversable {
        /// Positive effective traversal cost.
        cost: u32,
        /// Stable cost claims that produced the maximum.
        reasons: Vec<ClaimRef>,
    },
}

impl MovementDisposition {
    /// Builds a blocked disposition.
    ///
    /// # Errors
    ///
    /// Returns `EK0909` for an empty reason list and `EK0901` for duplicates.
    pub fn blocked(mut reasons: Vec<ClaimRef>) -> Result<Self, Diagnostic> {
        if reasons.is_empty() {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::MOVEMENT_DISPOSITION_INVALID,
                "a blocked movement disposition requires at least one reason",
            ));
        }
        sort_unique_reasons(&mut reasons)?;
        Ok(Self::Blocked { reasons })
    }

    /// Builds a traversable disposition.
    ///
    /// # Errors
    ///
    /// Returns `EK0909` for zero cost and `EK0901` for duplicate reasons.
    pub fn traversable(cost: u32, mut reasons: Vec<ClaimRef>) -> Result<Self, Diagnostic> {
        if cost == 0 {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::MOVEMENT_DISPOSITION_INVALID,
                "a traversable movement disposition requires a positive cost",
            ));
        }
        sort_unique_reasons(&mut reasons)?;
        Ok(Self::Traversable { cost, reasons })
    }

    /// Stable reason list.
    #[must_use]
    pub fn reasons(&self) -> &[ClaimRef] {
        match self {
            Self::Blocked { reasons } | Self::Traversable { reasons, .. } => reasons,
        }
    }

    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Blocked { reasons } => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("blocked")),
                (
                    "reasons",
                    CanonicalValue::Array(reasons.iter().map(StableId::to_canonical).collect()),
                ),
            ]),
            Self::Traversable { cost, reasons } => CanonicalValue::object_declared([
                ("cost", CanonicalValue::Uint(u64::from(*cost))),
                ("kind", CanonicalValue::text("traversable")),
                (
                    "reasons",
                    CanonicalValue::Array(reasons.iter().map(StableId::to_canonical).collect()),
                ),
            ]),
        }
    }
}

/// Effective movement fact for one subject.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResolvedMovement {
    entity: EntityId,
    disposition: MovementDisposition,
}

impl ResolvedMovement {
    /// Builds one resolved subject fact.
    #[must_use]
    pub fn new(entity: EntityId, disposition: MovementDisposition) -> Self {
        Self {
            entity,
            disposition,
        }
    }

    /// Subject entity.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Effective ground movement disposition.
    #[must_use]
    pub const fn disposition(&self) -> &MovementDisposition {
        &self.disposition
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("disposition", self.disposition.to_canonical()),
            ("entity", self.entity.to_canonical()),
        ])
    }
}

/// Stable effective movement facts for every projected subject.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResolvedMovementFacts {
    facts: Vec<ResolvedMovement>,
}

impl ResolvedMovementFacts {
    /// Builds a stable subject-keyed fact collection.
    ///
    /// # Errors
    ///
    /// Returns `EK0901` when one subject occurs more than once.
    pub fn new(mut facts: Vec<ResolvedMovement>) -> Result<Self, Diagnostic> {
        let mut seen = BTreeSet::new();
        for fact in &facts {
            if !seen.insert(fact.entity().clone()) {
                return Err(duplicate("resolved movement subject"));
            }
        }
        facts.sort_by(|left, right| left.entity.cmp(&right.entity));
        Ok(Self { facts })
    }

    /// Facts in stable entity order.
    #[must_use]
    pub fn facts(&self) -> &[ResolvedMovement] {
        &self.facts
    }

    /// Looks up one subject.
    #[must_use]
    pub fn get(&self, entity: &EntityId) -> Option<&MovementDisposition> {
        self.facts
            .binary_search_by(|fact| fact.entity.cmp(entity))
            .ok()
            .map(|index| self.facts[index].disposition())
    }

    /// Canonical bytes for deterministic evidence and atomicity checks.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        keyed_array(
            self.facts
                .iter()
                .map(|fact| (fact.entity.clone(), fact.to_canonical())),
        )
        .expect("ResolvedMovementFacts validates unique subjects")
        .to_canonical_bytes()
    }
}

/// Navigation projection containing the shared movement resolver plan.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NavigationPlan {
    schema: SchemaId,
    movement_resolver: MovementResolverPlan,
}

impl NavigationPlan {
    /// Builds a navigation plan from the compiler-projected shared resolver.
    #[must_use]
    pub fn new(movement_resolver: MovementResolverPlan) -> Self {
        Self {
            schema: navigation_schema(),
            movement_resolver,
        }
    }

    /// Navigation schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Shared movement resolver plan.
    #[must_use]
    pub const fn movement_resolver(&self) -> &MovementResolverPlan {
        &self.movement_resolver
    }

    /// Canonical navigation projection bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            ("movement_resolver", self.movement_resolver.to_canonical()),
            ("schema", self.schema.to_canonical()),
        ])
        .to_canonical_bytes()
    }
}

fn sort_unique_reasons(reasons: &mut [ClaimRef]) -> Result<(), Diagnostic> {
    let mut seen = BTreeSet::new();
    for reason in reasons.iter() {
        if !seen.insert(reason.clone()) {
            return Err(duplicate("movement reason"));
        }
    }
    reasons.sort();
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
