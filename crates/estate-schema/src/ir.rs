//! Canonical World IR construction types produced by the ownership linker.

use std::collections::BTreeSet;

use estate_core::canonical::keyed_array;
use estate_core::id::StableId;
use estate_core::{
    CanonicalValue, CatalogValueId, ClaimRef, Diagnostic, EntityId, Ident, NamespaceId,
    PrimitiveKindId, RepairClass, SchemaId, SourceSpan,
};

use crate::{Binding, InteractionDefinition, TransitionDefinition, construction_world_ir_schema};

/// A capability in the sealed Gate K basis used by the three approved kinds.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CapabilityKind {
    /// Lattice boundary/topology.
    Boundary,
    /// Openable or breachable portal.
    Portal,
    /// Ground movement blocking.
    BlocksGround,
    /// Ground traversal cost.
    TraversalCostGround,
    /// Namespace-local state machine.
    Machine,
    /// Typed interaction surface.
    Interactable,
    /// Lattice region.
    Region,
    /// Effective light emission.
    EmitsLight,
    /// Exactly one authoritative owner.
    Authority,
    /// Persistent semantic state.
    Persisted,
}

impl CapabilityKind {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boundary => "boundary",
            Self::Portal => "portal",
            Self::BlocksGround => "blocks_ground",
            Self::TraversalCostGround => "traversal_cost_ground",
            Self::Machine => "machine",
            Self::Interactable => "interactable",
            Self::Region => "region",
            Self::EmitsLight => "emits_light",
            Self::Authority => "authority",
            Self::Persisted => "persisted",
        }
    }
}

/// The value supplied by a capability claim template.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClaimValue {
    /// Boolean claim value.
    Bool(bool),
    /// Non-negative integer claim value.
    Uint(u32),
}

impl ClaimValue {
    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Bool(value) => CanonicalValue::Bool(*value),
            Self::Uint(value) => CanonicalValue::Uint(u64::from(*value)),
        }
    }
}

/// A typed activation expression for a capability claim template.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClaimActivation {
    /// Always active.
    Always,
    /// Active when one namespace-local machine equals the given state.
    StateEquals {
        /// Machine namespace.
        namespace: NamespaceId,
        /// Required state.
        state: Ident,
    },
    /// Active when any child is active.
    Any(Vec<ClaimActivation>),
    /// Active when every child is active.
    All(Vec<ClaimActivation>),
    /// Logical negation.
    Not(Box<ClaimActivation>),
}

impl ClaimActivation {
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

fn activation_group(kind: &'static str, children: &[ClaimActivation]) -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "children",
            CanonicalValue::Array(children.iter().map(ClaimActivation::to_canonical).collect()),
        ),
        ("kind", CanonicalValue::text(kind)),
    ])
}

/// A namespace-local state-machine template prepared at compile time.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MachineTemplate {
    namespace: NamespaceId,
    states: Vec<Ident>,
    initial: Ident,
    transitions: Vec<TransitionDefinition>,
}

impl MachineTemplate {
    /// Builds a state-machine template.
    #[must_use]
    pub fn new(namespace: NamespaceId, states: Vec<Ident>, initial: Ident) -> Self {
        Self {
            namespace,
            states,
            initial,
            transitions: Vec::new(),
        }
    }

    /// Attaches transitions in stable signature order.
    ///
    /// # Errors
    ///
    /// Returns `EK0704` when a transition signature occurs more than once.
    pub fn with_transitions(
        mut self,
        mut transitions: Vec<TransitionDefinition>,
    ) -> Result<Self, Diagnostic> {
        require_unique_with_code(
            transitions.iter().map(TransitionDefinition::stable_key),
            "transition signature",
            estate_core::diagnostic::codes::TRANSITION_SIGNATURE_DUPLICATE,
        )?;
        transitions.sort_by_key(TransitionDefinition::stable_key);
        self.transitions = transitions;
        Ok(self)
    }
    /// The machine namespace.
    #[must_use]
    pub fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }
    /// States in schema-declared order.
    #[must_use]
    pub fn states(&self) -> &[Ident] {
        &self.states
    }
    /// Initial state.
    #[must_use]
    pub fn initial(&self) -> &Ident {
        &self.initial
    }
    /// Transition definitions in stable signature order.
    #[must_use]
    pub fn transitions(&self) -> &[TransitionDefinition] {
        &self.transitions
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("initial", CanonicalValue::text(self.initial.as_str())),
            ("namespace", self.namespace.to_canonical()),
            (
                "states",
                CanonicalValue::Array(
                    self.states
                        .iter()
                        .map(|state| CanonicalValue::text(state.as_str()))
                        .collect(),
                ),
            ),
            (
                "transitions",
                CanonicalValue::Array(
                    self.transitions
                        .iter()
                        .map(TransitionDefinition::to_canonical)
                        .collect(),
                ),
            ),
        ])
    }
}

/// A capability claim prepared for later command-time resolution.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClaimTemplate {
    id: ClaimRef,
    capability: CapabilityKind,
    activation: ClaimActivation,
    value: ClaimValue,
}

impl ClaimTemplate {
    /// Builds a typed claim template.
    #[must_use]
    pub fn new(
        id: ClaimRef,
        capability: CapabilityKind,
        activation: ClaimActivation,
        value: ClaimValue,
    ) -> Self {
        Self {
            id,
            capability,
            activation,
            value,
        }
    }
    /// Stable claim reference.
    #[must_use]
    pub fn id(&self) -> &ClaimRef {
        &self.id
    }
    /// Capability supplied by this claim.
    #[must_use]
    pub const fn capability(&self) -> CapabilityKind {
        self.capability
    }
    /// Activation expression.
    #[must_use]
    pub fn activation(&self) -> &ClaimActivation {
        &self.activation
    }
    /// Supplied value.
    #[must_use]
    pub fn value(&self) -> &ClaimValue {
        &self.value
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("activation", self.activation.to_canonical()),
            ("capability", CanonicalValue::text(self.capability.as_str())),
            ("id", self.id.to_canonical()),
            ("value", self.value.to_canonical()),
        ])
    }
}

/// The approved primitive expansion attached to one entity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PrimitiveExpansion {
    capabilities: BTreeSet<CapabilityKind>,
    machines: Vec<MachineTemplate>,
    claims: Vec<ClaimTemplate>,
    interactions: Vec<InteractionDefinition>,
}

impl PrimitiveExpansion {
    /// Builds an expansion and imposes stable namespace/claim ordering.
    ///
    /// # Errors
    ///
    /// Returns `EK0304` when a machine namespace or claim reference occurs
    /// more than once.
    pub fn new(
        capabilities: impl IntoIterator<Item = CapabilityKind>,
        mut machines: Vec<MachineTemplate>,
        mut claims: Vec<ClaimTemplate>,
    ) -> Result<Self, Diagnostic> {
        require_unique(
            machines.iter().map(|machine| machine.namespace.clone()),
            "machine namespace",
        )?;
        require_unique(
            claims.iter().map(|claim| claim.id.clone()),
            "claim reference",
        )?;
        machines.sort_by(|left, right| left.namespace.cmp(&right.namespace));
        claims.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(Self {
            capabilities: capabilities.into_iter().collect(),
            machines,
            claims,
            interactions: Vec::new(),
        })
    }
    /// Attaches causal interactions in explicit phase and semantic order.
    ///
    /// # Errors
    ///
    /// Returns `EK0705` when an interaction identity occurs more than once.
    pub fn with_interactions(
        mut self,
        mut interactions: Vec<InteractionDefinition>,
    ) -> Result<Self, Diagnostic> {
        require_unique_with_code(
            interactions.iter().map(InteractionDefinition::stable_key),
            "interaction identity",
            estate_core::diagnostic::codes::INTERACTION_IDENTITY_DUPLICATE,
        )?;
        interactions.sort_by_key(InteractionDefinition::stable_key);
        self.interactions = interactions;
        Ok(self)
    }
    /// Capability bundle in stable order.
    #[must_use]
    pub fn capabilities(&self) -> &BTreeSet<CapabilityKind> {
        &self.capabilities
    }
    /// Namespace-local machines in stable namespace order.
    #[must_use]
    pub fn machines(&self) -> &[MachineTemplate] {
        &self.machines
    }
    /// Claim templates in stable claim-reference order.
    #[must_use]
    pub fn claims(&self) -> &[ClaimTemplate] {
        &self.claims
    }
    /// Causal interactions in explicit phase and semantic order.
    #[must_use]
    pub fn interactions(&self) -> &[InteractionDefinition] {
        &self.interactions
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "capabilities",
                CanonicalValue::Array(
                    self.capabilities
                        .iter()
                        .map(|item| CanonicalValue::text(item.as_str()))
                        .collect(),
                ),
            ),
            (
                "claims",
                keyed_array(
                    self.claims
                        .iter()
                        .map(|claim| (claim.id.clone(), claim.to_canonical())),
                )
                .expect("PrimitiveExpansion validates unique claim references"),
            ),
            (
                "interactions",
                CanonicalValue::Array(
                    self.interactions
                        .iter()
                        .map(InteractionDefinition::to_canonical)
                        .collect(),
                ),
            ),
            (
                "machines",
                keyed_array(
                    self.machines
                        .iter()
                        .map(|machine| (machine.namespace.clone(), machine.to_canonical())),
                )
                .expect("PrimitiveExpansion validates unique machine namespaces"),
            ),
        ])
    }
}

/// One linked and expanded world entity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IrEntity {
    id: EntityId,
    primitive: PrimitiveKindId,
    binding: Binding,
    credential: Option<CatalogValueId>,
    expansion: PrimitiveExpansion,
    source_span: SourceSpan,
}

impl IrEntity {
    /// Builds one linked entity.
    #[must_use]
    pub fn new(
        id: EntityId,
        primitive: PrimitiveKindId,
        binding: Binding,
        credential: Option<CatalogValueId>,
        expansion: PrimitiveExpansion,
        source_span: SourceSpan,
    ) -> Self {
        Self {
            id,
            primitive,
            binding,
            credential,
            expansion,
            source_span,
        }
    }
    /// Stable entity ID.
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    /// Approved primitive kind.
    #[must_use]
    pub fn primitive(&self) -> &PrimitiveKindId {
        &self.primitive
    }
    /// Typed lattice binding.
    #[must_use]
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    /// Optional typed credential reference.
    #[must_use]
    pub fn credential(&self) -> Option<&CatalogValueId> {
        self.credential.as_ref()
    }
    /// Compiler-owned primitive expansion.
    #[must_use]
    pub fn expansion(&self) -> &PrimitiveExpansion {
        &self.expansion
    }
    /// Source declaration span.
    #[must_use]
    pub fn source_span(&self) -> &SourceSpan {
        &self.source_span
    }

    fn to_canonical(&self) -> CanonicalValue {
        let credential = self
            .credential
            .as_ref()
            .map_or(CanonicalValue::Null, StableId::to_canonical);
        CanonicalValue::object_declared([
            ("binding", self.binding.to_canonical()),
            ("credential", credential),
            ("expansion", self.expansion.to_canonical()),
            ("id", self.id.to_canonical()),
            ("primitive", self.primitive.to_canonical()),
            ("source", span_to_canonical(&self.source_span)),
        ])
    }
}

/// One linked graph relation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IrRelation {
    subject: EntityId,
    kind: Ident,
    object: EntityId,
    source_span: SourceSpan,
}

impl IrRelation {
    /// Builds a linked relation.
    #[must_use]
    pub fn new(subject: EntityId, kind: Ident, object: EntityId, source_span: SourceSpan) -> Self {
        Self {
            subject,
            kind,
            object,
            source_span,
        }
    }
    /// Subject entity.
    #[must_use]
    pub fn subject(&self) -> &EntityId {
        &self.subject
    }
    /// Relation kind.
    #[must_use]
    pub fn kind(&self) -> &Ident {
        &self.kind
    }
    /// Object entity.
    #[must_use]
    pub fn object(&self) -> &EntityId {
        &self.object
    }

    fn stable_key(&self) -> String {
        format!("{}#{}#{}", self.subject, self.kind, self.object)
    }
    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("kind", CanonicalValue::text(self.kind.as_str())),
            ("object", self.object.to_canonical()),
            ("source", span_to_canonical(&self.source_span)),
            ("subject", self.subject.to_canonical()),
        ])
    }
}

/// Canonical owner of a fact class.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum FactOwner {
    /// Lattice-owned spatial truth.
    Lattice,
    /// Graph-owned identity or relation.
    Graph,
    /// Linker-derived cross-domain binding.
    WorldLinker,
}

impl FactOwner {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lattice => "lattice",
            Self::Graph => "graph",
            Self::WorldLinker => "world_linker",
        }
    }
}

/// Machine-generated proof of where a semantic fact came from and who owns it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FactOwnershipReceipt {
    fact: String,
    owner: FactOwner,
    declared_at: SourceSpan,
    resolved_to: String,
    consumers: BTreeSet<Ident>,
    derivation: Vec<String>,
}

impl FactOwnershipReceipt {
    /// Builds an ownership receipt.
    #[must_use]
    pub fn new(
        fact: String,
        owner: FactOwner,
        declared_at: SourceSpan,
        resolved_to: String,
        consumers: impl IntoIterator<Item = Ident>,
        derivation: Vec<String>,
    ) -> Self {
        Self {
            fact,
            owner,
            declared_at,
            resolved_to,
            consumers: consumers.into_iter().collect(),
            derivation,
        }
    }
    /// Stable fact path.
    #[must_use]
    pub fn fact(&self) -> &str {
        &self.fact
    }
    /// Canonical owner.
    #[must_use]
    pub const fn owner(&self) -> FactOwner {
        self.owner
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "consumers",
                CanonicalValue::Array(
                    self.consumers
                        .iter()
                        .map(|item| CanonicalValue::text(item.as_str()))
                        .collect(),
                ),
            ),
            ("declared_at", span_to_canonical(&self.declared_at)),
            (
                "derivation",
                CanonicalValue::Array(self.derivation.iter().map(CanonicalValue::text).collect()),
            ),
            ("fact", CanonicalValue::text(&self.fact)),
            ("owner", CanonicalValue::text(self.owner.as_str())),
            ("resolved_to", CanonicalValue::text(&self.resolved_to)),
        ])
    }
}

/// Versioned Canonical World IR construction snapshot from one linked source.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WorldIr {
    schema: SchemaId,
    source_schema: SchemaId,
    catalog_values: Vec<CatalogValueId>,
    entities: Vec<IrEntity>,
    relations: Vec<IrRelation>,
    ownership_receipts: Vec<FactOwnershipReceipt>,
}

impl WorldIr {
    /// Builds the IR and imposes every stable collection order.
    ///
    /// # Errors
    ///
    /// Returns `EK0304` when a keyed semantic identity occurs more than once.
    pub fn new(
        source_schema: SchemaId,
        mut catalog_values: Vec<CatalogValueId>,
        mut entities: Vec<IrEntity>,
        mut relations: Vec<IrRelation>,
        mut ownership_receipts: Vec<FactOwnershipReceipt>,
    ) -> Result<Self, Diagnostic> {
        require_unique(catalog_values.iter().cloned(), "catalog value")?;
        require_unique(entities.iter().map(|entity| entity.id.clone()), "entity")?;
        require_unique(
            relations.iter().map(IrRelation::stable_key),
            "relation identity",
        )?;
        require_unique(
            ownership_receipts
                .iter()
                .map(|receipt| receipt.fact.clone()),
            "ownership fact",
        )?;
        catalog_values.sort();
        entities.sort_by(|left, right| left.id.cmp(&right.id));
        relations.sort_by_key(IrRelation::stable_key);
        ownership_receipts.sort_by(|left, right| left.fact.cmp(&right.fact));
        Ok(Self {
            schema: construction_world_ir_schema(),
            source_schema,
            catalog_values,
            entities,
            relations,
            ownership_receipts,
        })
    }
    /// IR schema identity.
    #[must_use]
    pub fn schema(&self) -> &SchemaId {
        &self.schema
    }
    /// Source schema identity consumed to produce this IR.
    #[must_use]
    pub fn source_schema(&self) -> &SchemaId {
        &self.source_schema
    }
    /// Declared catalog values in stable ID order.
    #[must_use]
    pub fn catalog_values(&self) -> &[CatalogValueId] {
        &self.catalog_values
    }
    /// Expanded entities in stable ID order.
    #[must_use]
    pub fn entities(&self) -> &[IrEntity] {
        &self.entities
    }
    /// Linked relations in stable order.
    #[must_use]
    pub fn relations(&self) -> &[IrRelation] {
        &self.relations
    }
    /// Fact-ownership receipts in stable fact-path order.
    #[must_use]
    pub fn ownership_receipts(&self) -> &[FactOwnershipReceipt] {
        &self.ownership_receipts
    }

    /// Canonical semantic value for this construction snapshot.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "catalog_values",
                keyed_array(
                    self.catalog_values
                        .iter()
                        .map(|id| (id.clone(), id.to_canonical())),
                )
                .expect("WorldIr validates unique catalog values"),
            ),
            (
                "entities",
                keyed_array(
                    self.entities
                        .iter()
                        .map(|entity| (entity.id.clone(), entity.to_canonical())),
                )
                .expect("WorldIr validates unique entity IDs"),
            ),
            (
                "ownership_receipts",
                keyed_array(
                    self.ownership_receipts
                        .iter()
                        .map(|receipt| (receipt.fact.clone(), receipt.to_canonical())),
                )
                .expect("WorldIr validates unique ownership facts"),
            ),
            (
                "relations",
                keyed_array(
                    self.relations
                        .iter()
                        .map(|relation| (relation.stable_key(), relation.to_canonical())),
                )
                .expect("WorldIr validates unique relation identities"),
            ),
            ("schema", self.schema.to_canonical()),
            ("source_schema", self.source_schema.to_canonical()),
        ])
    }
    /// Canonical bytes for this construction snapshot.
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
            return Err(Diagnostic::new(
                estate_core::diagnostic::codes::CANONICAL_DUPLICATE_IDENTITY,
                format!("{identity} occurs more than once"),
            )
            .with_repair(RepairClass::RemoveDuplicateDeclaration));
        }
    }
    Ok(())
}

fn require_unique_with_code<T: Ord>(
    values: impl IntoIterator<Item = T>,
    identity: &str,
    code: estate_core::DiagnosticCode,
) -> Result<(), Diagnostic> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(
                Diagnostic::new(code, format!("{identity} occurs more than once"))
                    .with_repair(RepairClass::RemoveDuplicateDeclaration),
            );
        }
    }
    Ok(())
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
