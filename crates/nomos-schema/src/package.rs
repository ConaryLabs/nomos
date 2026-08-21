//! Canonical registry of the schema identities persisted by one world package.

use std::collections::BTreeSet;

use nomos_core::canonical::keyed_array;
use nomos_core::{CanonicalValue, Diagnostic, RepairClass, SchemaId};

use crate::schema_registry_schema;

/// Crate that owns an authoritative persisted schema type.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum SchemaOwner {
    /// Canonical bytes, hashes, and generic package manifest types.
    Core,
    /// Canonical source, World IR, and schema-registry types.
    Schema,
    /// Projection artifact types.
    Projection,
    /// Compiler build-receipt types.
    Compiler,
}

impl SchemaOwner {
    /// Stable workspace-crate spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Core => "nomos-core",
            Self::Schema => "nomos-schema",
            Self::Projection => "nomos-projection",
            Self::Compiler => "nomos-compiler",
        }
    }
}

/// One artifact-to-schema ownership row.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SchemaRegistration {
    artifact: String,
    schema: SchemaId,
    owner: SchemaOwner,
}

impl SchemaRegistration {
    /// Builds one registration. Artifact names are checked again by the
    /// package layer before filesystem publication.
    #[must_use]
    pub fn new(artifact: impl Into<String>, schema: SchemaId, owner: SchemaOwner) -> Self {
        Self {
            artifact: artifact.into(),
            schema,
            owner,
        }
    }

    /// Package member that carries the schema.
    #[must_use]
    pub fn artifact(&self) -> &str {
        &self.artifact
    }

    /// Persisted schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Authoritative crate owner.
    #[must_use]
    pub const fn owner(&self) -> SchemaOwner {
        self.owner
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("artifact", CanonicalValue::text(&self.artifact)),
            ("owner", CanonicalValue::text(self.owner.as_str())),
            ("schema", self.schema.to_canonical()),
        ])
    }
}

/// Exact schema/owner registry persisted as `schemas.json`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SchemaRegistry {
    schema: SchemaId,
    entries: Vec<SchemaRegistration>,
}

impl SchemaRegistry {
    /// Builds a registry in canonical artifact-name order.
    ///
    /// # Errors
    ///
    /// Returns `EK0304` when an artifact or schema identity occurs twice.
    pub fn new(mut entries: Vec<SchemaRegistration>) -> Result<Self, Diagnostic> {
        let mut artifacts = BTreeSet::new();
        let mut schemas = BTreeSet::new();
        for entry in &entries {
            if !artifacts.insert(entry.artifact.clone()) || !schemas.insert(entry.schema.clone()) {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::CANONICAL_DUPLICATE_IDENTITY,
                    "schema registry repeats an artifact or authoritative schema identity",
                )
                .with_repair(RepairClass::RemoveDuplicateDeclaration));
            }
        }
        entries.sort_by(|left, right| left.artifact.cmp(&right.artifact));
        Ok(Self {
            schema: schema_registry_schema(),
            entries,
        })
    }

    /// Registry schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Registrations in package-member order.
    #[must_use]
    pub fn entries(&self) -> &[SchemaRegistration] {
        &self.entries
    }

    /// Canonical registry bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            (
                "entries",
                keyed_array(
                    self.entries
                        .iter()
                        .map(|entry| (entry.artifact.clone(), entry.to_canonical())),
                )
                .expect("SchemaRegistry validates unique artifacts"),
            ),
            ("schema", self.schema.to_canonical()),
        ])
        .to_canonical_bytes()
    }
}
