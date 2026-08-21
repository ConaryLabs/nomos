//! The first stable Canonical World IR artifact.

use std::collections::BTreeSet;

use nomos_core::canonical::keyed_array;
use nomos_core::id::StableId;
use nomos_core::{CanonicalValue, Diagnostic, EntityId, FieldName, RepairClass, SchemaId};

use crate::{WorldIr, construction_world_ir_schema, stable_world_ir_schema};

/// One subject encoded with the required stable-v1 movement representation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StableGroundMovementV1 {
    entity: EntityId,
    blocked_ground: bool,
    traversal_cost_ground: Option<u32>,
}

impl StableGroundMovementV1 {
    /// Builds one stable-v1 movement row.
    ///
    /// # Errors
    ///
    /// Returns `EK0909` when blocked movement also carries a cost, or when
    /// traversable movement lacks a positive cost.
    pub fn new(
        entity: EntityId,
        blocked_ground: bool,
        traversal_cost_ground: Option<u32>,
    ) -> Result<Self, Diagnostic> {
        let valid = if blocked_ground {
            traversal_cost_ground.is_none()
        } else {
            traversal_cost_ground.is_some_and(|cost| cost > 0)
        };
        if !valid {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::MOVEMENT_DISPOSITION_INVALID,
                "stable World IR v1 requires blocked/null or traversable/positive-cost movement",
            )
            .with_repair(RepairClass::RebuildFromSource));
        }
        Ok(Self {
            entity,
            blocked_ground,
            traversal_cost_ground,
        })
    }

    /// Stable movement subject.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Whether ground movement is initially blocked.
    #[must_use]
    pub const fn blocked_ground(&self) -> bool {
        self.blocked_ground
    }

    /// Initial traversal cost, or null when blocked.
    #[must_use]
    pub const fn traversal_cost_ground(&self) -> Option<u32> {
        self.traversal_cost_ground
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("blocked_ground", CanonicalValue::Bool(self.blocked_ground)),
            ("entity", self.entity.to_canonical()),
            (
                "traversal_cost_ground",
                self.traversal_cost_ground
                    .map_or(CanonicalValue::Null, |cost| {
                        CanonicalValue::Uint(u64::from(cost))
                    }),
            ),
        ])
    }
}

/// Contract-complete `nomos.world_ir@1` promoted from validated construction
/// evidence without changing or relabelling that evidence.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StableWorldIr {
    schema: SchemaId,
    construction: WorldIr,
    compiler_version: u32,
    primitive_catalog_version: u32,
    movement_v1: Vec<StableGroundMovementV1>,
}

impl StableWorldIr {
    /// Promotes a complete construction snapshot into the stable lineage.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for the wrong construction identity, zero
    /// compiler/catalog versions, duplicate movement rows, or movement rows
    /// that do not exactly cover the compiler resolver subjects.
    pub fn new(
        construction: WorldIr,
        compiler_version: u32,
        primitive_catalog_version: u32,
        mut movement_v1: Vec<StableGroundMovementV1>,
    ) -> Result<Self, Diagnostic> {
        if construction.schema() != &construction_world_ir_schema() {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::PROVENANCE_VALUE_INVALID,
                "stable World IR must be promoted from the active construction schema",
            )
            .with_repair(RepairClass::RebuildFromSource));
        }
        if compiler_version == 0 || primitive_catalog_version == 0 {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::SCHEMA_VERSION_ZERO,
                "stable World IR compiler and primitive-catalog versions start at one",
            )
            .with_repair(RepairClass::RebuildFromSource));
        }
        let mut seen = BTreeSet::new();
        for row in &movement_v1 {
            if !seen.insert(row.entity().clone()) {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::CANONICAL_DUPLICATE_IDENTITY,
                    format!(
                        "stable movement subject `{}` occurs more than once",
                        row.entity()
                    ),
                )
                .with_repair(RepairClass::RemoveDuplicateDeclaration));
            }
        }
        let expected: BTreeSet<EntityId> = construction
            .movement_resolver()
            .subjects()
            .iter()
            .map(|subject| subject.entity().clone())
            .collect();
        if seen != expected {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
                "stable-v1 movement rows do not exactly cover resolver subjects",
            )
            .with_repair(RepairClass::RebuildFromSource));
        }
        movement_v1.sort_by(|left, right| left.entity.cmp(&right.entity));
        Ok(Self {
            schema: stable_world_ir_schema(),
            construction,
            compiler_version,
            primitive_catalog_version,
            movement_v1,
        })
    }

    /// Stable schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Preserved construction evidence used for typed compiler access.
    ///
    /// The stable canonical bytes are not the construction bytes. Runtime and
    /// projection crates cannot depend on this crate, and therefore cannot use
    /// this carrier to cross the compiler boundary.
    #[must_use]
    pub const fn construction(&self) -> &WorldIr {
        &self.construction
    }

    /// Compiler semantic version recorded in the artifact.
    #[must_use]
    pub const fn compiler_version(&self) -> u32 {
        self.compiler_version
    }

    /// Primitive-catalog semantic version recorded in the artifact.
    #[must_use]
    pub const fn primitive_catalog_version(&self) -> u32 {
        self.primitive_catalog_version
    }

    /// Required v1 movement rows in stable entity order.
    #[must_use]
    pub fn movement_v1(&self) -> &[StableGroundMovementV1] {
        &self.movement_v1
    }

    /// Canonical stable World IR value.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        let CanonicalValue::Object(mut fields) = self.construction.to_canonical() else {
            unreachable!("construction World IR is an object")
        };
        fields.insert(
            FieldName::declared("compiler_version"),
            CanonicalValue::Uint(u64::from(self.compiler_version)),
        );
        fields.insert(
            FieldName::declared("construction_schema"),
            self.construction.schema().to_canonical(),
        );
        fields.insert(
            FieldName::declared("movement_v1"),
            keyed_array(
                self.movement_v1
                    .iter()
                    .map(|row| (row.entity.clone(), row.to_canonical())),
            )
            .expect("StableWorldIr validates unique movement subjects"),
        );
        fields.insert(
            FieldName::declared("primitive_catalog_version"),
            CanonicalValue::Uint(u64::from(self.primitive_catalog_version)),
        );
        fields.insert(FieldName::declared("schema"), self.schema.to_canonical());
        CanonicalValue::Object(fields)
    }

    /// Canonical stable World IR bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }
}
