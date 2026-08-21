//! Typed forensic provenance for Canonical World IR construction snapshots.

use std::collections::BTreeSet;
use std::fmt;

use nomos_core::diagnostic::codes;
use nomos_core::id::StableId;
use nomos_core::{
    CanonicalValue, CatalogValueId, Diagnostic, EntityId, Ident, PrimitiveKindId, RepairClass,
    SourceSpan,
};

use crate::Binding;
use crate::ir::span_to_canonical;

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

/// Closed identity of a Gate K semantic fact.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum FactIdentity {
    /// Graph-owned entity identity.
    EntityIdentity(EntityId),
    /// Lattice-owned authored anchor.
    EntitySpatialAnchor(EntityId),
    /// Linker-derived binding between entity and lattice.
    EntitySpatialBinding(EntityId),
    /// Linker-resolved catalog credential.
    EntityCredential(EntityId),
    /// Graph-owned relation edge.
    Relation {
        /// Relation subject.
        subject: EntityId,
        /// Relation vocabulary item.
        kind: Ident,
        /// Relation object.
        object: EntityId,
    },
}

impl FactIdentity {
    /// Canonical structured identity.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::EntityIdentity(entity) => fact_entity("entity_identity", entity),
            Self::EntitySpatialAnchor(entity) => fact_entity("entity_spatial_anchor", entity),
            Self::EntitySpatialBinding(entity) => fact_entity("entity_spatial_binding", entity),
            Self::EntityCredential(entity) => fact_entity("entity_credential", entity),
            Self::Relation {
                subject,
                kind,
                object,
            } => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("relation")),
                ("object", object.to_canonical()),
                ("relation", CanonicalValue::text(kind.as_str())),
                ("subject", subject.to_canonical()),
            ]),
        }
    }
}

impl fmt::Display for FactIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityIdentity(entity) => write!(formatter, "entity.{entity}.identity"),
            Self::EntitySpatialAnchor(entity) => {
                write!(formatter, "entity.{entity}.spatial_anchor")
            }
            Self::EntitySpatialBinding(entity) => {
                write!(formatter, "entity.{entity}.spatial_binding")
            }
            Self::EntityCredential(entity) => write!(formatter, "entity.{entity}.credential"),
            Self::Relation {
                subject,
                kind,
                object,
            } => write!(formatter, "relation.{subject}.{kind}.{object}"),
        }
    }
}

fn fact_entity(kind: &'static str, entity: &EntityId) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("entity", entity.to_canonical()),
        ("kind", CanonicalValue::text(kind)),
    ])
}

/// Closed resolved-value vocabulary for Gate K fact classes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ResolvedFactValue {
    /// A resolved graph entity.
    Entity(EntityId),
    /// A resolved typed lattice binding.
    Binding(Binding),
    /// A resolved catalog value reference.
    CatalogValue(CatalogValueId),
    /// A resolved graph relation.
    Relation {
        /// Relation subject.
        subject: EntityId,
        /// Relation vocabulary item.
        kind: Ident,
        /// Relation object.
        object: EntityId,
    },
}

impl ResolvedFactValue {
    /// Canonical structured value.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Entity(entity) => CanonicalValue::object_declared([
                ("entity", entity.to_canonical()),
                ("kind", CanonicalValue::text("entity")),
            ]),
            Self::Binding(binding) => CanonicalValue::object_declared([
                ("binding", binding.to_canonical()),
                ("kind", CanonicalValue::text("binding")),
            ]),
            Self::CatalogValue(value) => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("catalog_value")),
                ("value", value.to_canonical()),
            ]),
            Self::Relation {
                subject,
                kind,
                object,
            } => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("relation")),
                ("object", object.to_canonical()),
                ("relation", CanonicalValue::text(kind.as_str())),
                ("subject", subject.to_canonical()),
            ]),
        }
    }
}

impl fmt::Display for ResolvedFactValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entity(entity) => write!(formatter, "entity {entity}"),
            Self::Binding(binding) => write_binding(formatter, binding),
            Self::CatalogValue(value) => write!(formatter, "catalog value {value}"),
            Self::Relation {
                subject,
                kind,
                object,
            } => write!(formatter, "relation {subject} {kind} {object}"),
        }
    }
}

fn write_binding(formatter: &mut fmt::Formatter<'_>, binding: &Binding) -> fmt::Result {
    match binding {
        Binding::Cell(cell) => write!(formatter, "cell({},{},{})", cell.x(), cell.y(), cell.z()),
        Binding::Face { cell, direction } => write!(
            formatter,
            "face(cell({},{},{}),{})",
            cell.x(),
            cell.y(),
            cell.z(),
            direction.as_str()
        ),
        Binding::Region { min, max } => write!(
            formatter,
            "region(cell({},{},{}),cell({},{},{}))",
            min.x(),
            min.y(),
            min.z(),
            max.x(),
            max.y(),
            max.z()
        ),
    }
}

/// Runtime-facing projection that consumes a canonical fact.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProjectionConsumer {
    /// Diagnostic and forensic projection.
    Diagnostics,
    /// Navigation projection.
    Navigation,
    /// Persistence projection.
    Persistence,
    /// Simulation projection.
    Simulation,
}

impl ProjectionConsumer {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Diagnostics => "diagnostics",
            Self::Navigation => "navigation",
            Self::Persistence => "persistence",
            Self::Simulation => "simulation",
        }
    }
}

/// Closed producer vocabulary for provenance steps.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum DerivationProducer {
    /// Accepted source declarations.
    Source,
    /// Compiler world-linking pass.
    WorldLinker,
}

impl DerivationProducer {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::WorldLinker => "world_linker",
        }
    }

    /// Parses one closed producer ID.
    ///
    /// # Errors
    ///
    /// Returns `EK1003` for an unknown producer.
    pub fn parse(value: &str) -> Result<Self, Diagnostic> {
        match value {
            "source" => Ok(Self::Source),
            "world_linker" => Ok(Self::WorldLinker),
            _ => Err(Diagnostic::new(
                codes::PROVENANCE_PRODUCER_UNKNOWN,
                format!("unknown provenance producer `{value}`"),
            )
            .with_repair(RepairClass::RebuildFromSource)),
        }
    }
}

/// Closed compiler-pass vocabulary for provenance steps.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum DerivationPass {
    /// Declare a graph entity.
    DeclareEntity,
    /// Declare one typed lattice anchor.
    DeclareSpatialAnchor,
    /// Link one graph relation.
    LinkRelation,
    /// Resolve one cross-domain spatial binding.
    ResolveSpatialBinding,
    /// Resolve one catalog value reference.
    ResolveCatalogValue,
}

impl DerivationPass {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeclareEntity => "declare_entity",
            Self::DeclareSpatialAnchor => "declare_spatial_anchor",
            Self::LinkRelation => "link_relation",
            Self::ResolveSpatialBinding => "resolve_spatial_binding",
            Self::ResolveCatalogValue => "resolve_catalog_value",
        }
    }

    /// Parses one closed pass ID.
    ///
    /// # Errors
    ///
    /// Returns `EK1004` for an unknown pass.
    pub fn parse(value: &str) -> Result<Self, Diagnostic> {
        match value {
            "declare_entity" => Ok(Self::DeclareEntity),
            "declare_spatial_anchor" => Ok(Self::DeclareSpatialAnchor),
            "link_relation" => Ok(Self::LinkRelation),
            "resolve_spatial_binding" => Ok(Self::ResolveSpatialBinding),
            "resolve_catalog_value" => Ok(Self::ResolveCatalogValue),
            _ => Err(Diagnostic::new(
                codes::PROVENANCE_PASS_UNKNOWN,
                format!("unknown provenance pass `{value}`"),
            )
            .with_repair(RepairClass::RebuildFromSource)),
        }
    }
}

/// Typed input to one derivation step.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum DerivationInput {
    /// Another canonical fact.
    Fact(FactIdentity),
    /// An approved primitive kind.
    Primitive(PrimitiveKindId),
    /// A declared catalog value.
    CatalogValue(CatalogValueId),
}

impl DerivationInput {
    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Fact(fact) => CanonicalValue::object_declared([
                ("fact", fact.to_canonical()),
                ("kind", CanonicalValue::text("fact")),
            ]),
            Self::Primitive(primitive) => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("primitive")),
                ("primitive", primitive.to_canonical()),
            ]),
            Self::CatalogValue(value) => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("catalog_value")),
                ("value", value.to_canonical()),
            ]),
        }
    }
}

/// One typed causal step in a fact derivation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DerivationStep {
    producer: DerivationProducer,
    pass: DerivationPass,
    inputs: BTreeSet<DerivationInput>,
}

impl DerivationStep {
    /// Builds one step with inputs ordered by typed identity.
    ///
    /// # Errors
    ///
    /// Returns `EK0304` rather than discarding a duplicate input.
    pub fn new(
        producer: DerivationProducer,
        pass: DerivationPass,
        inputs: impl IntoIterator<Item = DerivationInput>,
    ) -> Result<Self, Diagnostic> {
        let mut unique_inputs = BTreeSet::new();
        for input in inputs {
            if !unique_inputs.insert(input) {
                return Err(duplicate_identity("provenance derivation input"));
            }
        }
        Ok(Self {
            producer,
            pass,
            inputs: unique_inputs,
        })
    }

    /// Step producer.
    #[must_use]
    pub const fn producer(&self) -> DerivationProducer {
        self.producer
    }

    /// Compiler pass.
    #[must_use]
    pub const fn pass(&self) -> DerivationPass {
        self.pass
    }

    /// Typed step inputs in canonical order.
    #[must_use]
    pub fn inputs(&self) -> &BTreeSet<DerivationInput> {
        &self.inputs
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "inputs",
                CanonicalValue::Array(
                    self.inputs
                        .iter()
                        .map(DerivationInput::to_canonical)
                        .collect(),
                ),
            ),
            ("pass", CanonicalValue::text(self.pass.as_str())),
            ("producer", CanonicalValue::text(self.producer.as_str())),
        ])
    }
}

/// Machine-generated proof of where a semantic fact came from and who owns it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FactOwnershipReceipt {
    fact: FactIdentity,
    owner: FactOwner,
    declared_at: SourceSpan,
    resolved_to: ResolvedFactValue,
    consumers: BTreeSet<ProjectionConsumer>,
    derivation: Vec<DerivationStep>,
}

impl FactOwnershipReceipt {
    /// Builds an ownership receipt. [`crate::WorldIr::new`] validates all typed edges.
    ///
    /// # Errors
    ///
    /// Returns `EK0304` rather than discarding a duplicate consumer or step.
    pub fn new(
        fact: FactIdentity,
        owner: FactOwner,
        declared_at: SourceSpan,
        resolved_to: ResolvedFactValue,
        consumers: impl IntoIterator<Item = ProjectionConsumer>,
        derivation: Vec<DerivationStep>,
    ) -> Result<Self, Diagnostic> {
        let mut unique_consumers = BTreeSet::new();
        for consumer in consumers {
            if !unique_consumers.insert(consumer) {
                return Err(duplicate_identity("provenance projection consumer"));
            }
        }
        if derivation.is_empty() {
            return Err(Diagnostic::new(
                codes::PROVENANCE_DERIVATION_INVALID,
                "a provenance receipt requires at least one derivation step",
            )
            .with_repair(RepairClass::RebuildFromSource));
        }
        let mut unique_steps = BTreeSet::new();
        for step in derivation {
            if !unique_steps.insert(step) {
                return Err(duplicate_identity("provenance derivation step"));
            }
        }
        Ok(Self {
            fact,
            owner,
            declared_at,
            resolved_to,
            consumers: unique_consumers,
            derivation: unique_steps.into_iter().collect(),
        })
    }

    /// Typed fact identity.
    #[must_use]
    pub fn fact(&self) -> &FactIdentity {
        &self.fact
    }

    /// Canonical owner.
    #[must_use]
    pub const fn owner(&self) -> FactOwner {
        self.owner
    }

    /// Typed resolved value.
    #[must_use]
    pub fn resolved_to(&self) -> &ResolvedFactValue {
        &self.resolved_to
    }

    /// Typed consumers in canonical order.
    #[must_use]
    pub fn consumers(&self) -> &BTreeSet<ProjectionConsumer> {
        &self.consumers
    }

    /// Typed causal derivation.
    #[must_use]
    pub fn derivation(&self) -> &[DerivationStep] {
        &self.derivation
    }

    /// Canonical structured forensic record.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
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
                CanonicalValue::Array(
                    self.derivation
                        .iter()
                        .map(DerivationStep::to_canonical)
                        .collect(),
                ),
            ),
            ("fact", self.fact.to_canonical()),
            ("owner", CanonicalValue::text(self.owner.as_str())),
            ("resolved_to", self.resolved_to.to_canonical()),
        ])
    }

    /// Human-readable explanation derived from typed semantics.
    #[must_use]
    pub fn render_text(&self) -> String {
        let steps = self
            .derivation
            .iter()
            .map(|step| format!("{}/{}", step.producer.as_str(), step.pass.as_str()))
            .collect::<Vec<_>>()
            .join(" -> ");
        format!(
            "{} is owned by {} and resolves to {}; derivation: {}",
            self.fact,
            self.owner.as_str(),
            self.resolved_to,
            steps
        )
    }

    pub(crate) fn validate(
        &self,
        facts: &BTreeSet<FactIdentity>,
        entities: &BTreeSet<EntityId>,
        relations: &BTreeSet<FactIdentity>,
        catalog_values: &BTreeSet<CatalogValueId>,
        primitives: &BTreeSet<PrimitiveKindId>,
    ) -> Result<(), Diagnostic> {
        if !self.value_matches_fact() {
            return Err(Diagnostic::new(
                codes::PROVENANCE_VALUE_INVALID,
                format!(
                    "fact `{}` carries an incompatible resolved value",
                    self.fact
                ),
            )
            .with_span(self.declared_at.clone())
            .with_repair(RepairClass::RebuildFromSource));
        }
        for step in &self.derivation {
            if !supported_step(step.producer, step.pass) {
                return Err(Diagnostic::new(
                    codes::PROVENANCE_DERIVATION_INVALID,
                    format!(
                        "producer `{}` cannot execute provenance pass `{}`",
                        step.producer.as_str(),
                        step.pass.as_str()
                    ),
                )
                .with_span(self.declared_at.clone())
                .with_repair(RepairClass::RebuildFromSource));
            }
            for input in &step.inputs {
                match input {
                    DerivationInput::Fact(fact) if !facts.contains(fact) => {
                        return Err(Diagnostic::new(
                            codes::PROVENANCE_FACT_REFERENCE_MISSING,
                            format!("fact `{}` derives from missing fact `{fact}`", self.fact),
                        )
                        .with_span(self.declared_at.clone())
                        .with_repair(RepairClass::RebuildFromSource));
                    }
                    DerivationInput::CatalogValue(value) if !catalog_values.contains(value) => {
                        return Err(missing_input(&self.fact, value, &self.declared_at));
                    }
                    DerivationInput::Primitive(primitive) if !primitives.contains(primitive) => {
                        return Err(missing_input(&self.fact, primitive, &self.declared_at));
                    }
                    _ => {}
                }
            }
        }
        if !self.fact_exists_in_world(entities, relations) {
            return Err(Diagnostic::new(
                codes::PROVENANCE_FACT_REFERENCE_MISSING,
                format!(
                    "receipt names fact `{}` absent from the compiled world",
                    self.fact
                ),
            )
            .with_span(self.declared_at.clone())
            .with_repair(RepairClass::RebuildFromSource));
        }
        if let ResolvedFactValue::CatalogValue(value) = &self.resolved_to
            && !catalog_values.contains(value)
        {
            return Err(missing_input(&self.fact, value, &self.declared_at));
        }
        Ok(())
    }

    fn value_matches_fact(&self) -> bool {
        match (&self.fact, &self.resolved_to) {
            (FactIdentity::EntityIdentity(fact), ResolvedFactValue::Entity(value)) => fact == value,
            (FactIdentity::EntitySpatialAnchor(_), ResolvedFactValue::Binding(_))
            | (FactIdentity::EntitySpatialBinding(_), ResolvedFactValue::Binding(_)) => true,
            (FactIdentity::EntityCredential(_), ResolvedFactValue::CatalogValue(_)) => true,
            (
                FactIdentity::Relation {
                    subject: fact_subject,
                    kind: fact_kind,
                    object: fact_object,
                },
                ResolvedFactValue::Relation {
                    subject,
                    kind,
                    object,
                },
            ) => fact_subject == subject && fact_kind == kind && fact_object == object,
            _ => false,
        }
    }

    fn fact_exists_in_world(
        &self,
        entities: &BTreeSet<EntityId>,
        relations: &BTreeSet<FactIdentity>,
    ) -> bool {
        match &self.fact {
            FactIdentity::EntityIdentity(entity)
            | FactIdentity::EntitySpatialAnchor(entity)
            | FactIdentity::EntitySpatialBinding(entity)
            | FactIdentity::EntityCredential(entity) => entities.contains(entity),
            FactIdentity::Relation { .. } => relations.contains(&self.fact),
        }
    }
}

fn supported_step(producer: DerivationProducer, pass: DerivationPass) -> bool {
    matches!(
        (producer, pass),
        (
            DerivationProducer::Source,
            DerivationPass::DeclareEntity
                | DerivationPass::DeclareSpatialAnchor
                | DerivationPass::LinkRelation
        ) | (
            DerivationProducer::WorldLinker,
            DerivationPass::ResolveSpatialBinding | DerivationPass::ResolveCatalogValue
        )
    )
}

fn missing_input(fact: &FactIdentity, input: &impl fmt::Display, span: &SourceSpan) -> Diagnostic {
    Diagnostic::new(
        codes::PROVENANCE_INPUT_REFERENCE_MISSING,
        format!("fact `{fact}` derives from missing typed input `{input}`"),
    )
    .with_span(span.clone())
    .with_repair(RepairClass::RebuildFromSource)
}

fn duplicate_identity(identity: &str) -> Diagnostic {
    Diagnostic::new(
        codes::CANONICAL_DUPLICATE_IDENTITY,
        format!("{identity} occurs more than once"),
    )
    .with_repair(RepairClass::RemoveDuplicateDeclaration)
}
