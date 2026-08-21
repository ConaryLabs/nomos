//! Typed lattice bindings exposed by source schema version 1.

use estate_core::CanonicalValue;

/// One integer lattice cell.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Cell {
    x: i32,
    y: i32,
    z: i32,
}

impl Cell {
    /// Builds a cell from integer lattice coordinates.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// The x coordinate.
    #[must_use]
    pub const fn x(self) -> i32 {
        self.x
    }
    /// The y coordinate.
    #[must_use]
    pub const fn y(self) -> i32 {
        self.y
    }
    /// The z coordinate.
    #[must_use]
    pub const fn z(self) -> i32 {
        self.z
    }

    pub(crate) fn to_canonical(self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("x", CanonicalValue::Int(i64::from(self.x))),
            ("y", CanonicalValue::Int(i64::from(self.y))),
            ("z", CanonicalValue::Int(i64::from(self.z))),
        ])
    }
}

/// A face direction in the typed lattice.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Direction {
    /// Negative y.
    North,
    /// Positive x.
    East,
    /// Positive y.
    South,
    /// Negative x.
    West,
    /// Positive z.
    Up,
    /// Negative z.
    Down,
}

impl Direction {
    /// Parses the source spelling of a direction.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "north" => Some(Self::North),
            "east" => Some(Self::East),
            "south" => Some(Self::South),
            "west" => Some(Self::West),
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            _ => None,
        }
    }

    /// The stable source and wire spelling.
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

/// A source-authored typed lattice binding.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Binding {
    /// An entity anchored to a cell.
    Cell(Cell),
    /// An entity anchored to one face of a cell.
    Face {
        /// The owning cell.
        cell: Cell,
        /// The selected face.
        direction: Direction,
    },
    /// A closed axis-aligned cell region.
    Region {
        /// Component-wise minimum cell.
        min: Cell,
        /// Component-wise maximum cell.
        max: Cell,
    },
}

impl Binding {
    /// The source spelling of the binding kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Cell(_) => "cell",
            Self::Face { .. } => "face",
            Self::Region { .. } => "region",
        }
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
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
