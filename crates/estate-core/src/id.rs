//! Typed stable IDs.
//!
//! `KERNEL.md` section 4 requires stable symbolic IDs in the Canonical World
//! IR, section 7 orders hashed collections by them, and acceptance 2 requires
//! entity and catalog namespaces to stay distinct so `credential/gaoler_key`
//! cannot quietly become a fourth entity.
//!
//! Every ID here is a separate type with no conversion to any other. A catalog
//! value cannot satisfy an entity reference because it cannot be spelled as
//! one:
//!
//! ```compile_fail
//! use estate_core::id::{CatalogValueId, EntityId};
//! let gate = EntityId::parse("north_gate").unwrap();
//! let key = CatalogValueId::parse("credential/gaoler_key").unwrap();
//! let _ = gate == key;
//! ```
//!
//! The same comparison within one type is of course fine:
//!
//! ```
//! use estate_core::id::EntityId;
//! let gate = EntityId::parse("north_gate").unwrap();
//! let other = EntityId::parse("brazier_02").unwrap();
//! assert!(gate != other);
//! ```

use std::fmt;

use crate::canonical::CanonicalValue;
use crate::diagnostic::{Diagnostic, RepairClass, codes};
use crate::ident::{Ident, split_exact};

/// Behaviour shared by every stable ID: one canonical string spelling, and a
/// canonical value built from it.
pub trait StableId: Ord + fmt::Display {
    /// The canonical string spelling used for ordering and encoding.
    fn canonical_string(&self) -> String;

    /// The ID as a canonical value.
    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::Text(self.canonical_string())
    }
}

/// A world entity, for example `north_gate`.
///
/// One identifier segment. Entities live in their own symbol table; catalog
/// values live in another.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EntityId(Ident);

impl EntityId {
    /// Parses `name`.
    ///
    /// # Errors
    ///
    /// Returns `EK0101` when the name is not a legal identifier segment.
    pub fn parse(name: &str) -> Result<Self, Diagnostic> {
        Ident::new(name).map(Self)
    }

    /// The entity's identifier segment.
    #[must_use]
    pub fn ident(&self) -> &Ident {
        &self.0
    }
}

impl StableId for EntityId {
    fn canonical_string(&self) -> String {
        self.0.as_str().to_owned()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

/// A catalog value, for example `credential/gaoler_key`.
///
/// Shape: `<catalog>/<name>`. A catalog value is never a world entity.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CatalogValueId {
    catalog: Ident,
    name: Ident,
}

impl CatalogValueId {
    /// Parses `<catalog>/<name>`.
    ///
    /// # Errors
    ///
    /// Returns `EK0104` when the shape is wrong and `EK0101` when a segment is
    /// not a legal identifier.
    pub fn parse(text: &str) -> Result<Self, Diagnostic> {
        let [catalog, name] = split_exact::<2>(text, b'/', "<catalog>/<name>")?;
        Ok(Self { catalog, name })
    }

    /// The catalog this value belongs to, for example `credential`.
    #[must_use]
    pub fn catalog(&self) -> &Ident {
        &self.catalog
    }

    /// The value's name within its catalog.
    #[must_use]
    pub fn name(&self) -> &Ident {
        &self.name
    }
}

impl StableId for CatalogValueId {
    fn canonical_string(&self) -> String {
        format!("{}/{}", self.catalog, self.name)
    }
}

impl fmt::Display for CatalogValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.catalog, self.name)
    }
}

/// An entity-local semantic namespace, for example `north_gate.access`.
///
/// State machines live in namespaces, but not every namespace is a machine:
/// static topology claims use names such as `flooded_section.region`. Section
/// 7 orders machine collections by these IDs; the broader type also gives
/// every static claim an honest semantic home instead of inventing a fake
/// state machine for it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NamespaceId {
    entity: EntityId,
    local_name: Ident,
}

impl NamespaceId {
    /// Builds a namespace ID from its parts.
    #[must_use]
    pub fn new(entity: EntityId, local_name: Ident) -> Self {
        Self { entity, local_name }
    }

    /// Parses `<entity>.<namespace>`.
    ///
    /// # Errors
    ///
    /// Returns `EK0104` when the shape is wrong and `EK0101` when a segment is
    /// not a legal identifier.
    pub fn parse(text: &str) -> Result<Self, Diagnostic> {
        let [entity, local_name] = split_exact::<2>(text, b'.', "<entity>.<namespace>")?;
        Ok(Self {
            entity: EntityId(entity),
            local_name,
        })
    }

    /// The entity that owns this machine.
    #[must_use]
    pub fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// The namespace's entity-local name, for example `access` or `region`.
    #[must_use]
    pub fn local_name(&self) -> &Ident {
        &self.local_name
    }
}

impl StableId for NamespaceId {
    fn canonical_string(&self) -> String {
        format!("{}.{}", self.entity, self.local_name)
    }
}

impl fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.entity, self.local_name)
    }
}

/// A reference to one capability claim, for example
/// `north_gate.ward#blocks_ground`.
///
/// Section 3 requires `MovementDisposition<ground>` to carry a nonempty
/// *ordered* list of claim references as its reasons; this is the element type
/// of that list, and its ordering is the canonical-string ordering.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ClaimRef {
    namespace: NamespaceId,
    capability: Ident,
}

impl ClaimRef {
    /// Builds a claim reference from its parts.
    #[must_use]
    pub fn new(namespace: NamespaceId, capability: Ident) -> Self {
        Self {
            namespace,
            capability,
        }
    }

    /// Parses `<entity>.<machine>#<capability>`.
    ///
    /// # Errors
    ///
    /// Returns `EK0104` when the shape is wrong and `EK0101` when a segment is
    /// not a legal identifier.
    pub fn parse(text: &str) -> Result<Self, Diagnostic> {
        let (namespace, capability) = text.split_once('#').ok_or_else(|| {
            Diagnostic::new(
                codes::ID_SHAPE_INVALID,
                format!(
                    "`{text}` does not match the required shape `<entity>.<machine>#<capability>`"
                ),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape)
        })?;
        Ok(Self {
            namespace: NamespaceId::parse(namespace)?,
            capability: Ident::new(capability)?,
        })
    }

    /// The semantic namespace that raises this claim.
    #[must_use]
    pub fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }

    /// The capability being claimed, for example `blocks_ground`.
    #[must_use]
    pub fn capability(&self) -> &Ident {
        &self.capability
    }
}

/// An approved primitive kind, for example `primitive/iron_barred_door`.
///
/// Primitive kinds and catalog values deliberately use different Rust types.
/// A credential cannot satisfy a primitive reference merely because both use
/// slash-separated symbolic names in source.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PrimitiveKindId(Ident);

impl PrimitiveKindId {
    /// Parses `primitive/<name>`.
    ///
    /// # Errors
    ///
    /// Returns `EK0104` when the namespace or shape is wrong and `EK0101`
    /// when the name is not a supported identifier.
    pub fn parse(text: &str) -> Result<Self, Diagnostic> {
        let [namespace, name] = split_exact::<2>(text, b'/', "primitive/<name>")?;
        if namespace.as_str() != "primitive" {
            return Err(Diagnostic::new(
                codes::ID_SHAPE_INVALID,
                format!("`{text}` is not in the `primitive` namespace"),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape));
        }
        Ok(Self(name))
    }

    /// The primitive's name within the approved catalog.
    #[must_use]
    pub fn name(&self) -> &Ident {
        &self.0
    }
}

impl StableId for PrimitiveKindId {
    fn canonical_string(&self) -> String {
        format!("primitive/{}", self.0)
    }
}

impl fmt::Display for PrimitiveKindId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "primitive/{}", self.0)
    }
}

impl StableId for ClaimRef {
    fn canonical_string(&self) -> String {
        format!("{}#{}", self.namespace, self.capability)
    }
}

impl fmt::Display for ClaimRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.namespace, self.capability)
    }
}

/// A dotted schema name, for example `estate.package.manifest`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SchemaName(Vec<Ident>);

impl SchemaName {
    /// Parses a dotted schema name of one or more segments.
    ///
    /// # Errors
    ///
    /// Returns `EK0104` for an empty name and `EK0101` for an illegal segment.
    pub fn parse(text: &str) -> Result<Self, Diagnostic> {
        if text.is_empty() {
            return Err(Diagnostic::new(
                codes::ID_SHAPE_INVALID,
                "a schema name needs at least one segment",
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape));
        }
        let mut segments = Vec::new();
        for part in text.split('.') {
            segments.push(Ident::new(part)?);
        }
        Ok(Self(segments))
    }

    /// The name's segments.
    #[must_use]
    pub fn segments(&self) -> &[Ident] {
        &self.0
    }
}

impl StableId for SchemaName {
    fn canonical_string(&self) -> String {
        self.0
            .iter()
            .map(Ident::as_str)
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl fmt::Display for SchemaName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical_string())
    }
}

/// A schema name paired with its version.
///
/// Section 6 requires every persisted artifact to name its schema and version,
/// and requires those versions to move independently. This is that pair, and
/// nothing in the kernel persists an artifact without one.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SchemaId {
    name: SchemaName,
    version: u32,
}

impl SchemaId {
    /// Parses `<dotted-name>@<positive-version>`.
    ///
    /// # Errors
    ///
    /// Returns a stable identifier diagnostic when the shape, name, or
    /// version is invalid.
    pub fn parse(text: &str) -> Result<Self, Diagnostic> {
        let (name, version) = text.split_once('@').ok_or_else(|| {
            Diagnostic::new(
                codes::ID_SHAPE_INVALID,
                format!("`{text}` does not match `<schema-name>@<version>`"),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape)
        })?;
        if version.is_empty() || !version.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(Diagnostic::new(
                codes::ID_SHAPE_INVALID,
                format!("schema version `{version}` is not an unsigned integer"),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape));
        }
        let version = version.parse::<u32>().map_err(|_| {
            Diagnostic::new(
                codes::ID_SHAPE_INVALID,
                format!("schema version `{version}` does not fit a 32-bit unsigned integer"),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape)
        })?;
        Self::new(name, version)
    }

    /// Builds a schema ID.
    ///
    /// # Errors
    ///
    /// Returns `EK0105` when `version` is zero; versions start at one so an
    /// unset field can never read as a valid version. Returns `EK0101` or
    /// `EK0104` when `name` is not a legal dotted schema name.
    pub fn new(name: &str, version: u32) -> Result<Self, Diagnostic> {
        if version == 0 {
            return Err(Diagnostic::new(
                codes::SCHEMA_VERSION_ZERO,
                format!("schema `{name}` has version 0; schema versions start at 1"),
            ));
        }
        Ok(Self {
            name: SchemaName::parse(name)?,
            version,
        })
    }

    /// The schema name.
    #[must_use]
    pub fn name(&self) -> &SchemaName {
        &self.name
    }

    /// The schema version.
    #[must_use]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The schema ID as a canonical object.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("name", CanonicalValue::Text(self.name.canonical_string())),
            ("version", CanonicalValue::Uint(u64::from(self.version))),
        ])
    }
}

impl fmt::Display for SchemaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}
