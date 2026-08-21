//! Parsed source-schema types. Parsing itself belongs to `nomos-compiler`.

use nomos_core::{CatalogValueId, EntityId, Ident, PrimitiveKindId, SchemaId, SourceSpan};

use crate::Binding;

/// A typed source value paired with the exact source location that declared it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Spanned<T> {
    value: T,
    span: SourceSpan,
}

impl<T> Spanned<T> {
    /// Pairs a value with its source span.
    #[must_use]
    pub fn new(value: T, span: SourceSpan) -> Self {
        Self { value, span }
    }
    /// The parsed value.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }
    /// The source span.
    #[must_use]
    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
    /// Consumes the wrapper and returns its value.
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

/// One field in an entity declaration.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SourceField {
    /// Typed lattice binding owned by source.
    Anchor(Spanned<Binding>),
    /// Typed catalog reference.
    Credential(Spanned<CatalogValueId>),
    /// A relation illegally placed in a lattice/entity field.
    LatticeRelation {
        /// Relation kind.
        relation: Spanned<Ident>,
        /// Referenced entity.
        target: Spanned<EntityId>,
        /// Span of the complete rejected field.
        span: SourceSpan,
    },
    /// A raw transform, which source schema version 1 forbids.
    RawTransform(SourceSpan),
    /// A source-authored derived fact, which the compiler owns.
    DerivedFact {
        /// Attempted fact name.
        name: String,
        /// Span of the complete rejected field.
        span: SourceSpan,
    },
}

impl SourceField {
    /// The complete field span.
    #[must_use]
    pub fn span(&self) -> &SourceSpan {
        match self {
            Self::Anchor(value) => value.span(),
            Self::Credential(value) => value.span(),
            Self::LatticeRelation { span, .. }
            | Self::RawTransform(span)
            | Self::DerivedFact { span, .. } => span,
        }
    }
}

/// One primitive instance from source.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SourceEntity {
    id: Spanned<EntityId>,
    primitive: Spanned<PrimitiveKindId>,
    fields: Vec<SourceField>,
    span: SourceSpan,
}

impl SourceEntity {
    /// Builds a parsed entity declaration.
    #[must_use]
    pub fn new(
        id: Spanned<EntityId>,
        primitive: Spanned<PrimitiveKindId>,
        fields: Vec<SourceField>,
        span: SourceSpan,
    ) -> Self {
        Self {
            id,
            primitive,
            fields,
            span,
        }
    }
    /// The stable entity ID.
    #[must_use]
    pub fn id(&self) -> &Spanned<EntityId> {
        &self.id
    }
    /// The approved primitive-kind reference.
    #[must_use]
    pub fn primitive(&self) -> &Spanned<PrimitiveKindId> {
        &self.primitive
    }
    /// Source fields in declaration order.
    #[must_use]
    pub fn fields(&self) -> &[SourceField] {
        &self.fields
    }
    /// The complete declaration span.
    #[must_use]
    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
}

/// A graph relation between two world entities.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SourceRelation {
    subject: Spanned<EntityId>,
    kind: Spanned<Ident>,
    object: Spanned<EntityId>,
    span: SourceSpan,
}

impl SourceRelation {
    /// Builds a parsed graph relation.
    #[must_use]
    pub fn new(
        subject: Spanned<EntityId>,
        kind: Spanned<Ident>,
        object: Spanned<EntityId>,
        span: SourceSpan,
    ) -> Self {
        Self {
            subject,
            kind,
            object,
            span,
        }
    }
    /// The subject entity reference.
    #[must_use]
    pub fn subject(&self) -> &Spanned<EntityId> {
        &self.subject
    }
    /// The relation kind.
    #[must_use]
    pub fn kind(&self) -> &Spanned<Ident> {
        &self.kind
    }
    /// The object entity reference.
    #[must_use]
    pub fn object(&self) -> &Spanned<EntityId> {
        &self.object
    }
    /// The complete relation span.
    #[must_use]
    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
}

/// A forbidden attempt by content to declare a canonical fact owner.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ForbiddenFactOwner {
    fact: String,
    owner: String,
    span: SourceSpan,
}

impl ForbiddenFactOwner {
    /// Records the rejected declaration for the ownership linker.
    #[must_use]
    pub fn new(fact: String, owner: String, span: SourceSpan) -> Self {
        Self { fact, owner, span }
    }
    /// The attempted fact class.
    #[must_use]
    pub fn fact(&self) -> &str {
        &self.fact
    }
    /// The attempted owner.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }
    /// The complete declaration span.
    #[must_use]
    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
}

/// One parsed `.nomos` file before name resolution or primitive expansion.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SourceDocument {
    schema: Spanned<SchemaId>,
    catalog_values: Vec<Spanned<CatalogValueId>>,
    entities: Vec<SourceEntity>,
    relations: Vec<SourceRelation>,
    forbidden_fact_owners: Vec<ForbiddenFactOwner>,
}

impl SourceDocument {
    /// Builds a parsed source document.
    #[must_use]
    pub fn new(
        schema: Spanned<SchemaId>,
        catalog_values: Vec<Spanned<CatalogValueId>>,
        entities: Vec<SourceEntity>,
        relations: Vec<SourceRelation>,
        forbidden_fact_owners: Vec<ForbiddenFactOwner>,
    ) -> Self {
        Self {
            schema,
            catalog_values,
            entities,
            relations,
            forbidden_fact_owners,
        }
    }
    /// The declared source schema.
    #[must_use]
    pub fn schema(&self) -> &Spanned<SchemaId> {
        &self.schema
    }
    /// Catalog values declared by this source file.
    #[must_use]
    pub fn catalog_values(&self) -> &[Spanned<CatalogValueId>] {
        &self.catalog_values
    }
    /// Primitive instances in declaration order.
    #[must_use]
    pub fn entities(&self) -> &[SourceEntity] {
        &self.entities
    }
    /// Graph relations in declaration order.
    #[must_use]
    pub fn relations(&self) -> &[SourceRelation] {
        &self.relations
    }
    /// Forbidden owner declarations retained for a precise linker rejection.
    #[must_use]
    pub fn forbidden_fact_owners(&self) -> &[ForbiddenFactOwner] {
        &self.forbidden_fact_owners
    }
}
