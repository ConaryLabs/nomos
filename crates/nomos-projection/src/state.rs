//! Runtime-facing entity identities and authoritative lattice bindings.

use std::collections::BTreeSet;

use nomos_core::canonical::keyed_array;
use nomos_core::id::StableId;
use nomos_core::{CanonicalValue, Diagnostic, EntityId, NamespaceId, RepairClass};

use crate::LatticeCell;

/// Direction of one projected lattice face.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProjectedDirection {
    /// Negative Y face.
    North,
    /// Positive X face.
    East,
    /// Positive Y face.
    South,
    /// Negative X face.
    West,
    /// Positive Z face.
    Up,
    /// Negative Z face.
    Down,
}

impl ProjectedDirection {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

/// Authoritative lattice binding projected for runtime snapshots.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RuntimeBinding {
    /// One lattice cell.
    Cell(LatticeCell),
    /// One directed lattice face.
    Face {
        /// Cell owning the face.
        cell: LatticeCell,
        /// Face direction.
        direction: ProjectedDirection,
    },
    /// One closed lattice region.
    Region {
        /// Component-wise minimum cell.
        min: LatticeCell,
        /// Component-wise maximum cell.
        max: LatticeCell,
    },
}

impl RuntimeBinding {
    /// Canonical semantic value used by projection and runtime-state schemas.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Cell(cell) => CanonicalValue::object_declared([
                ("cell", cell.to_canonical()),
                ("kind", CanonicalValue::text("cell")),
            ]),
            Self::Face { cell, direction } => CanonicalValue::object_declared([
                ("cell", cell.to_canonical()),
                ("direction", CanonicalValue::text(direction.as_str())),
                ("kind", CanonicalValue::text("face")),
            ]),
            Self::Region { min, max } => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("region")),
                ("max", max.to_canonical()),
                ("min", min.to_canonical()),
            ]),
        }
    }
}

/// One compiler-projected runtime entity and its machine namespaces.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProjectedEntity {
    id: EntityId,
    binding: RuntimeBinding,
    machines: Vec<NamespaceId>,
}

impl ProjectedEntity {
    /// Builds one projected entity with stable machine ordering.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for duplicate or cross-entity namespaces.
    pub fn new(
        id: EntityId,
        binding: RuntimeBinding,
        mut machines: Vec<NamespaceId>,
    ) -> Result<Self, Diagnostic> {
        let mut seen = BTreeSet::new();
        for namespace in &machines {
            if namespace.entity() != &id {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                    format!("machine `{namespace}` does not belong to projected entity `{id}`"),
                ));
            }
            if !seen.insert(namespace.clone()) {
                return Err(duplicate("projected machine namespace"));
            }
        }
        machines.sort();
        Ok(Self {
            id,
            binding,
            machines,
        })
    }

    /// Stable entity ID.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Authoritative lattice binding.
    #[must_use]
    pub const fn binding(&self) -> &RuntimeBinding {
        &self.binding
    }

    /// Machine namespaces in stable order.
    #[must_use]
    pub fn machines(&self) -> &[NamespaceId] {
        &self.machines
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("binding", self.binding.to_canonical()),
            ("id", self.id.to_canonical()),
            (
                "machines",
                keyed_array(
                    self.machines
                        .iter()
                        .map(|namespace| (namespace.clone(), namespace.to_canonical())),
                )
                .expect("ProjectedEntity validates unique machines"),
            ),
        ])
    }
}

pub(crate) fn sort_entities(
    mut entities: Vec<ProjectedEntity>,
) -> Result<Vec<ProjectedEntity>, Diagnostic> {
    let mut seen = BTreeSet::new();
    for entity in &entities {
        if !seen.insert(entity.id().clone()) {
            return Err(duplicate("projected entity"));
        }
    }
    entities.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(entities)
}

fn duplicate(identity: &str) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::CANONICAL_DUPLICATE_IDENTITY,
        format!("{identity} occurs more than once"),
    )
    .with_repair(RepairClass::RemoveDuplicateDeclaration)
}
