//! `nomos.play_command@1`: exactly one input per batch.
//!
//! This is the owner file for that identity, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.
//!
//! There is no `argument` field. `nomos_sim::CommandRequest::new` takes an
//! optional `CatalogValueId`, and the committed corpus declares no command
//! transition whose requirement is anything but `none`. An `interact` that
//! resolves to a transition requiring a credential is refused `PL0305` rather
//! than guessed at, and the field arrives when content needs it.

use nomos_core::id::{EntityId, SchemaId};
use nomos_core::{CanonicalValue, Ident};

use crate::error::{PlayError, PlayResult, codes};
use crate::read;

/// The command identity.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn play_command_schema() -> SchemaId {
    SchemaId::new("nomos.play_command", 1).expect("the play-command schema id is a literal")
}

/// One of the four lattice directions.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Direction {
    /// `+x`.
    East,
    /// `-y`.
    North,
    /// `+y`.
    South,
    /// `-x`.
    West,
}

impl Direction {
    /// The lattice delta, which is the table
    /// `apps/nomos-viewer/src/catalog.mjs:266-271` declares for the renderer.
    #[must_use]
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::East => (1, 0),
            Self::North => (0, -1),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        }
    }

    /// Stable wire spelling, which is also the spelling a door's declared
    /// `anchor.direction` uses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::East => "east",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
        }
    }

    /// Reads the wire spelling.
    ///
    /// # Errors
    ///
    /// Returns `PL0105` for any other value.
    pub fn parse(value: &str) -> PlayResult<Self> {
        match value {
            "east" => Ok(Self::East),
            "north" => Ok(Self::North),
            "south" => Ok(Self::South),
            "west" => Ok(Self::West),
            _ => Err(PlayError::new(
                codes::DOCUMENT_VALUE,
                format!("`{value}` is not a lattice direction"),
            )),
        }
    }
}

/// One authoritative input.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PlayCommand {
    /// Move the player one cell.
    Move {
        /// Direction of travel.
        direction: Direction,
    },
    /// Send one kernel command to an entity within reach.
    Interact {
        /// Target entity.
        entity: EntityId,
        /// Namespace-local action.
        action: Ident,
    },
    /// Leave the area through a declared traversable gate.
    Cross {
        /// The door entity to cross.
        gate: EntityId,
    },
}

impl PlayCommand {
    /// The command as a canonical value.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Move { direction } => CanonicalValue::object_declared([
                ("direction", CanonicalValue::text(direction.as_str())),
                ("kind", CanonicalValue::text("move")),
                (
                    "schema",
                    CanonicalValue::text(play_command_schema().to_string()),
                ),
            ]),
            Self::Interact { entity, action } => CanonicalValue::object_declared([
                ("action", CanonicalValue::text(action.as_str())),
                ("entity", CanonicalValue::text(entity.to_string())),
                ("kind", CanonicalValue::text("interact")),
                (
                    "schema",
                    CanonicalValue::text(play_command_schema().to_string()),
                ),
            ]),
            Self::Cross { gate } => CanonicalValue::object_declared([
                ("gate", CanonicalValue::text(gate.to_string())),
                ("kind", CanonicalValue::text("cross")),
                (
                    "schema",
                    CanonicalValue::text(play_command_schema().to_string()),
                ),
            ]),
        }
    }

    /// The command as canonical bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    /// Reads one command from canonical bytes.
    ///
    /// # Errors
    ///
    /// Returns `PL0101` for another identity and `PL0201` when the field set
    /// does not match the declared kind exactly.
    pub fn decode(bytes: &[u8]) -> PlayResult<Self> {
        Self::from_canonical(&read::parse(bytes, "play command")?)
    }

    /// Reads one command from an already-parsed value.
    ///
    /// # Errors
    ///
    /// As [`PlayCommand::decode`].
    pub fn from_canonical(value: &CanonicalValue) -> PlayResult<Self> {
        let fields = read::object(value, "play command")?;
        read::bind_schema(fields, &play_command_schema(), "play command")?;
        let kind = read::text(read::field(fields, "kind", "play command")?, "command kind")?;
        match kind {
            "move" => {
                shape(fields, &["direction", "kind", "schema"])?;
                Ok(Self::Move {
                    direction: Direction::parse(read::text(
                        read::field(fields, "direction", "move command")?,
                        "move direction",
                    )?)?,
                })
            }
            "interact" => {
                shape(fields, &["action", "entity", "kind", "schema"])?;
                Ok(Self::Interact {
                    entity: EntityId::parse(read::text(
                        read::field(fields, "entity", "interact command")?,
                        "interact entity",
                    )?)
                    .map_err(|error| PlayError::new(codes::COMMAND_SHAPE, error.message()))?,
                    action: Ident::new(read::text(
                        read::field(fields, "action", "interact command")?,
                        "interact action",
                    )?)
                    .map_err(|error| PlayError::new(codes::COMMAND_SHAPE, error.message()))?,
                })
            }
            "cross" => {
                shape(fields, &["gate", "kind", "schema"])?;
                Ok(Self::Cross {
                    gate: EntityId::parse(read::text(
                        read::field(fields, "gate", "cross command")?,
                        "cross gate",
                    )?)
                    .map_err(|error| PlayError::new(codes::COMMAND_SHAPE, error.message()))?,
                })
            }
            other => Err(PlayError::new(
                codes::COMMAND_SHAPE,
                format!("`{other}` is not a play command kind"),
            )),
        }
    }
}

fn shape(
    fields: &std::collections::BTreeMap<nomos_core::FieldName, CanonicalValue>,
    expected: &[&str],
) -> PlayResult<()> {
    read::require_fields(fields, expected, "play command")
        .map_err(|error| PlayError::new(codes::COMMAND_SHAPE, error.message().to_owned()))
}
