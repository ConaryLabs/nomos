//! The stable Canonical World IR lineage and its one Gate K migration boundary.

use std::collections::BTreeSet;

use nomos_core::canonical::keyed_array;
use nomos_core::id::StableId;
use nomos_core::{
    CanonicalValue, ClaimRef, Diagnostic, EntityId, FieldName, RepairClass, SchemaId,
};

use crate::{
    CapabilityKind, WorldIr, construction_world_ir_schema, legacy_stable_world_ir_schema,
    stable_world_ir_schema,
};

/// One subject encoded with the legacy stable-v1 movement representation.
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
            return Err(movement_invalid(
                "stable World IR v1 requires blocked/null or traversable/positive-cost movement",
            ));
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

/// Stable-v2 ground movement value with the claims that explain it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StableMovementDispositionGround {
    /// At least one active blocker wins before traversal cost.
    Blocked {
        /// Active blocking claims in stable identity order.
        reasons: Vec<ClaimRef>,
    },
    /// No blocker is active and a positive effective cost is available.
    Traversable {
        /// Effective positive ground traversal cost.
        cost: u32,
        /// Active cost claims in stable identity order; empty means base cost.
        reasons: Vec<ClaimRef>,
    },
}

impl StableMovementDispositionGround {
    /// Builds a blocked disposition with at least one unique reason.
    ///
    /// # Errors
    ///
    /// Returns `EK0909` for empty or duplicate reasons.
    pub fn blocked(mut reasons: Vec<ClaimRef>) -> Result<Self, Diagnostic> {
        require_reasons(&mut reasons, true)?;
        Ok(Self::Blocked { reasons })
    }

    /// Builds a traversable disposition with a positive cost.
    ///
    /// # Errors
    ///
    /// Returns `EK0909` for zero cost or duplicate reasons.
    pub fn traversable(cost: u32, mut reasons: Vec<ClaimRef>) -> Result<Self, Diagnostic> {
        if cost == 0 {
            return Err(movement_invalid(
                "stable World IR v2 traversal cost must be positive",
            ));
        }
        require_reasons(&mut reasons, false)?;
        Ok(Self::Traversable { cost, reasons })
    }

    /// Claims explaining the effective disposition.
    #[must_use]
    pub fn reasons(&self) -> &[ClaimRef] {
        match self {
            Self::Blocked { reasons } | Self::Traversable { reasons, .. } => reasons,
        }
    }

    /// Effective traversal cost, absent when blocked.
    #[must_use]
    pub const fn cost(&self) -> Option<u32> {
        match self {
            Self::Blocked { .. } => None,
            Self::Traversable { cost, .. } => Some(*cost),
        }
    }

    /// Whether this value is blocked.
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked { .. })
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

/// One subject encoded with the active stable-v2 movement representation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StableGroundMovementV2 {
    entity: EntityId,
    movement_disposition_ground: StableMovementDispositionGround,
}

impl StableGroundMovementV2 {
    /// Builds one stable-v2 movement row.
    #[must_use]
    pub fn new(
        entity: EntityId,
        movement_disposition_ground: StableMovementDispositionGround,
    ) -> Self {
        Self {
            entity,
            movement_disposition_ground,
        }
    }

    /// Stable movement subject.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Typed effective initial movement disposition.
    #[must_use]
    pub const fn movement_disposition_ground(&self) -> &StableMovementDispositionGround {
        &self.movement_disposition_ground
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("entity", self.entity.to_canonical()),
            (
                "movement_disposition_ground",
                self.movement_disposition_ground.to_canonical(),
            ),
        ])
    }
}

/// Exact legacy `nomos.world_ir@1` value accepted only by migration.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LegacyStableWorldIrV1 {
    schema: SchemaId,
    construction: WorldIr,
    compiler_version: u32,
    primitive_catalog_version: u32,
    movement_v1: Vec<StableGroundMovementV1>,
}

impl LegacyStableWorldIrV1 {
    /// Reconstructs one supported legacy stable value.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for invalid versions or movement coverage.
    pub fn new(
        construction: WorldIr,
        compiler_version: u32,
        primitive_catalog_version: u32,
        mut movement_v1: Vec<StableGroundMovementV1>,
    ) -> Result<Self, Diagnostic> {
        validate_common(&construction, compiler_version, primitive_catalog_version)?;
        validate_coverage(
            &construction,
            &mut movement_v1,
            StableGroundMovementV1::entity,
        )?;
        Ok(Self {
            schema: legacy_stable_world_ir_schema(),
            construction,
            compiler_version,
            primitive_catalog_version,
            movement_v1,
        })
    }

    /// Legacy schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Preserved construction meaning.
    #[must_use]
    pub const fn construction(&self) -> &WorldIr {
        &self.construction
    }

    /// Legacy compiler version.
    #[must_use]
    pub const fn compiler_version(&self) -> u32 {
        self.compiler_version
    }

    /// Legacy primitive-catalog version.
    #[must_use]
    pub const fn primitive_catalog_version(&self) -> u32 {
        self.primitive_catalog_version
    }

    /// Legacy movement rows.
    #[must_use]
    pub fn movement_v1(&self) -> &[StableGroundMovementV1] {
        &self.movement_v1
    }

    /// Exact canonical legacy bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        stable_value(
            &self.schema,
            &self.construction,
            self.compiler_version,
            self.primitive_catalog_version,
            "movement_v1",
            self.movement_v1
                .iter()
                .map(|row| (row.entity().clone(), row.to_canonical())),
        )
        .to_canonical_bytes()
    }
}

/// Active contract-complete `nomos.world_ir@2`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StableWorldIr {
    schema: SchemaId,
    construction: WorldIr,
    compiler_version: u32,
    primitive_catalog_version: u32,
    movement_v2: Vec<StableGroundMovementV2>,
}

impl StableWorldIr {
    /// Promotes complete construction evidence into stable World IR v2.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for invalid versions or movement coverage.
    pub fn new(
        construction: WorldIr,
        compiler_version: u32,
        primitive_catalog_version: u32,
        mut movement_v2: Vec<StableGroundMovementV2>,
    ) -> Result<Self, Diagnostic> {
        validate_common(&construction, compiler_version, primitive_catalog_version)?;
        validate_coverage(
            &construction,
            &mut movement_v2,
            StableGroundMovementV2::entity,
        )?;
        validate_reason_references(&construction, &movement_v2)?;
        Ok(Self {
            schema: stable_world_ir_schema(),
            construction,
            compiler_version,
            primitive_catalog_version,
            movement_v2,
        })
    }

    /// Stable schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Preserved construction evidence used for typed compiler access.
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

    /// Required v2 movement rows in stable entity order.
    #[must_use]
    pub fn movement_v2(&self) -> &[StableGroundMovementV2] {
        &self.movement_v2
    }

    /// Canonical stable World IR value.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        stable_value(
            &self.schema,
            &self.construction,
            self.compiler_version,
            self.primitive_catalog_version,
            "movement_v2",
            self.movement_v2
                .iter()
                .map(|row| (row.entity().clone(), row.to_canonical())),
        )
    }

    /// Canonical stable World IR bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }
}

fn validate_common(
    construction: &WorldIr,
    compiler_version: u32,
    primitive_catalog_version: u32,
) -> Result<(), Diagnostic> {
    if construction.schema() != &construction_world_ir_schema() {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::PROVENANCE_VALUE_INVALID,
            "stable World IR must preserve the active construction schema",
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
    Ok(())
}

fn validate_coverage<T>(
    construction: &WorldIr,
    rows: &mut [T],
    entity: impl Fn(&T) -> &EntityId,
) -> Result<(), Diagnostic> {
    let mut seen = BTreeSet::new();
    for row in rows.iter() {
        if !seen.insert(entity(row).clone()) {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::CANONICAL_DUPLICATE_IDENTITY,
                format!(
                    "stable movement subject `{}` occurs more than once",
                    entity(row)
                ),
            )
            .with_repair(RepairClass::RemoveDuplicateDeclaration));
        }
    }
    let expected = construction
        .movement_resolver()
        .subjects()
        .iter()
        .map(|subject| subject.entity().clone())
        .collect::<BTreeSet<_>>();
    if seen != expected {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RESOLVER_PLAN_INVALID,
            "stable movement rows do not exactly cover resolver subjects",
        )
        .with_repair(RepairClass::RebuildFromSource));
    }
    rows.sort_by(|left, right| entity(left).cmp(entity(right)));
    Ok(())
}

fn require_reasons(reasons: &mut Vec<ClaimRef>, require_nonempty: bool) -> Result<(), Diagnostic> {
    let before = reasons.len();
    reasons.sort();
    reasons.dedup();
    if reasons.len() != before || (require_nonempty && reasons.is_empty()) {
        return Err(movement_invalid(
            "stable World IR v2 reasons must be unique and blocked reasons cannot be empty",
        ));
    }
    Ok(())
}

fn validate_reason_references(
    construction: &WorldIr,
    rows: &[StableGroundMovementV2],
) -> Result<(), Diagnostic> {
    for row in rows {
        let subject = construction
            .movement_resolver()
            .subjects()
            .iter()
            .find(|subject| subject.entity() == row.entity())
            .expect("stable movement coverage matches resolver subjects");
        let entity = construction
            .entities()
            .iter()
            .find(|entity| entity.id() == row.entity())
            .expect("resolver subjects belong to construction entities");
        let required_capability = match row.movement_disposition_ground() {
            StableMovementDispositionGround::Blocked { .. } => CapabilityKind::BlocksGround,
            StableMovementDispositionGround::Traversable { .. } => {
                CapabilityKind::TraversalCostGround
            }
        };
        for reason in row.movement_disposition_ground().reasons() {
            if subject.claims().binary_search(reason).is_err() {
                return Err(movement_invalid(format!(
                    "stable World IR v2 reason `{reason}` is not a movement claim for subject `{}`",
                    row.entity()
                )));
            }
            let claim = entity
                .expansion()
                .claims()
                .iter()
                .find(|claim| claim.id() == reason)
                .ok_or_else(|| {
                    movement_invalid(format!(
                        "stable World IR v2 reason `{reason}` has no claim definition for subject `{}`",
                        row.entity()
                    ))
                })?;
            if claim.capability() != required_capability {
                return Err(movement_invalid(format!(
                    "stable World IR v2 reason `{reason}` does not supply the disposition for subject `{}`",
                    row.entity()
                )));
            }
        }
    }
    Ok(())
}

fn stable_value(
    schema: &SchemaId,
    construction: &WorldIr,
    compiler_version: u32,
    primitive_catalog_version: u32,
    movement_field: &'static str,
    movement: impl IntoIterator<Item = (EntityId, CanonicalValue)>,
) -> CanonicalValue {
    let CanonicalValue::Object(mut fields) = construction.to_canonical() else {
        unreachable!("construction World IR is an object")
    };
    fields.insert(
        FieldName::declared("compiler_version"),
        CanonicalValue::Uint(u64::from(compiler_version)),
    );
    fields.insert(
        FieldName::declared("construction_schema"),
        construction.schema().to_canonical(),
    );
    fields.insert(
        FieldName::declared(movement_field),
        keyed_array(movement).expect("stable World IR validates unique movement subjects"),
    );
    fields.insert(
        FieldName::declared("primitive_catalog_version"),
        CanonicalValue::Uint(u64::from(primitive_catalog_version)),
    );
    fields.insert(FieldName::declared("schema"), schema.to_canonical());
    CanonicalValue::Object(fields)
}

fn movement_invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::MOVEMENT_DISPOSITION_INVALID,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}
