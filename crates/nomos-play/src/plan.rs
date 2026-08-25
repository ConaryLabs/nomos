//! The rendering plan, read for the facts the reducer needs and nothing else.
//!
//! This crate binds `nomos.rendering_plan@3` and reads seven of its thirteen
//! fields: `area`, `architecture`, `entities[].kind`, `actors`, `pursuit`,
//! `route`, and `projection_digests`. It does **not** read `scenarios` or
//! `interactions`. Those are the SVG capture ladder and the evidence that the
//! plan compiler consumed committed run bundles; after R1-5 they are not
//! gameplay, and nothing here walks them.
//!
//! Entity *kind* is a plan fact, because the plan copied it from
//! `nomos.entity_catalog@1`, which read it from the compiled World IR. Entity
//! *binding* is deliberately not taken from the plan even though the plan
//! carries it: the binding is authoritative runtime state and it is read from
//! the kernel's own `SimulationState`, so there is one authority for where a
//! compiled entity is and it is not a copied field.

use std::collections::BTreeMap;

use nomos_core::Sha256Digest;
use nomos_core::id::EntityId;
use nomos_projection::LatticeCell;

use crate::error::{PlayError, PlayResult, codes};
use crate::read;

/// The rendering-plan identity this runtime plays.
///
/// Read from the crate that declares it.
/// gives one identity one owner file, and a second constant here would be a
/// second place a version move has to be remembered.
pub use nomos_render_plan::plan::rendering_plan_schema;

/// The projection member whose bytes are this area's executable semantics.
pub const SEMANTICS_MEMBER: &str = "simulation.json";

/// The declared role of one actor.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Role {
    /// The actor the player's commands move.
    Player,
    /// The actor the pursuit rule moves.
    Pursuer,
}

impl Role {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Player => "player",
            Self::Pursuer => "pursuer",
        }
    }

    /// Reads the wire spelling.
    ///
    /// # Errors
    ///
    /// Returns `PL0105` for any other value.
    pub fn parse(value: &str) -> PlayResult<Self> {
        match value {
            "player" => Ok(Self::Player),
            "pursuer" => Ok(Self::Pursuer),
            _ => Err(PlayError::new(
                codes::DOCUMENT_VALUE,
                format!("`{value}` is not a declared actor role"),
            )),
        }
    }
}

/// One architectural mass, as a half-open lattice rectangle.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Mass {
    /// Authored identifier, reported in a refusal.
    pub id: String,
    /// Inclusive minimum corner.
    pub min: (i32, i32),
    /// Exclusive maximum corner.
    pub max: (i32, i32),
}

impl Mass {
    /// Whether the mass covers a cell. Half-open, reproducing
    /// `apps/nomos-viewer/src/play.mjs:110-114` exactly.
    #[must_use]
    pub const fn covers(&self, x: i32, y: i32) -> bool {
        x >= self.min.0 && x < self.max.0 && y >= self.min.1 && y < self.max.1
    }
}

/// One actor as the plan declares it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlanActor {
    /// Stable actor identity.
    pub id: EntityId,
    /// Declared role.
    pub role: Role,
    /// Declared starting cell.
    pub cell: LatticeCell,
}

/// The facts one area's plan supplies to the reducer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AreaPlan {
    /// Area identifier.
    pub area: String,
    /// Whether this is the route's first area.
    pub start: bool,
    /// Lattice width.
    pub width: i32,
    /// Lattice height.
    pub height: i32,
    /// Architectural masses, in authored order.
    pub masses: Vec<Mass>,
    /// Compiled entity kinds, by entity.
    pub kinds: BTreeMap<EntityId, String>,
    /// Declared actors, in plan order.
    pub actors: Vec<PlanActor>,
    /// The entity whose emission gates the pursuit rule.
    pub pursuit_light: EntityId,
    /// The gate this area's route leaves through.
    pub objective_gate: EntityId,
    /// The area this route continues into, or `None` at the terminal area.
    pub to_area: Option<String>,
    /// This area's own arrival cell, present exactly when `start` is false.
    pub entry: Option<LatticeCell>,
    /// SHA-256 of the simulation projection this plan was compiled against.
    pub semantics_digest: Sha256Digest,
}

impl AreaPlan {
    /// Reads one `nomos.rendering_plan@3` document.
    ///
    /// # Errors
    ///
    /// Returns `PL0101` for another identity, `PL0104`/`PL0105` for a shape or
    /// value this runtime cannot read, and `PL0103` when the actor collection
    /// is not exactly one player and at most one pursuer.
    pub fn decode(bytes: &[u8]) -> PlayResult<Self> {
        let value = read::parse(bytes, "rendering plan")?;
        let fields = read::object(&value, "rendering plan")?;
        read::bind_schema(fields, &rendering_plan_schema(), "rendering plan")?;

        let area = read::object(read::field(fields, "area", "rendering plan")?, "plan area")?;
        let area_id = read::text(read::field(area, "id", "plan area")?, "area id")?.to_owned();
        let start = read::boolean(read::field(area, "start", "plan area")?, "area start")?;

        let architecture = read::object(
            read::field(fields, "architecture", "rendering plan")?,
            "plan architecture",
        )?;
        let bounds = read::object(
            read::field(architecture, "bounds", "plan architecture")?,
            "architecture bounds",
        )?;
        let width = read::int32(read::field(bounds, "width", "bounds")?, "bounds width")?;
        let height = read::int32(read::field(bounds, "height", "bounds")?, "bounds height")?;

        let mut masses = Vec::new();
        for row in read::array(
            read::field(architecture, "masses", "plan architecture")?,
            "architecture masses",
        )? {
            let mass = read::object(row, "mass")?;
            masses.push(Mass {
                id: read::text(read::field(mass, "id", "mass")?, "mass id")?.to_owned(),
                min: pair(read::field(mass, "min", "mass")?, "mass min")?,
                max: pair(read::field(mass, "max", "mass")?, "mass max")?,
            });
        }

        let mut kinds = BTreeMap::new();
        for row in read::array(
            read::field(fields, "entities", "rendering plan")?,
            "plan entities",
        )? {
            let entity = read::object(row, "plan entity")?;
            kinds.insert(
                entity_id(read::field(entity, "id", "plan entity")?, "entity id")?,
                read::text(read::field(entity, "kind", "plan entity")?, "entity kind")?.to_owned(),
            );
        }

        let mut actors = Vec::new();
        for row in read::array(
            read::field(fields, "actors", "rendering plan")?,
            "plan actors",
        )? {
            let actor = read::object(row, "plan actor")?;
            actors.push(PlanActor {
                id: entity_id(read::field(actor, "id", "plan actor")?, "actor id")?,
                role: Role::parse(read::text(
                    read::field(actor, "role", "plan actor")?,
                    "actor role",
                )?)?,
                cell: cell(read::field(actor, "cell", "plan actor")?, "actor cell")?,
            });
        }
        require_one_player(&actors)?;

        let pursuit = read::object(
            read::field(fields, "pursuit", "rendering plan")?,
            "plan pursuit",
        )?;
        let pursuit_light = entity_id(
            read::field(pursuit, "light", "plan pursuit")?,
            "pursuit light",
        )?;

        let objective = read::object(
            read::field(fields, "objective", "rendering plan")?,
            "plan objective",
        )?;
        let objective_gate = entity_id(
            read::field(objective, "gate", "plan objective")?,
            "objective gate",
        )?;

        let route = read::object(
            read::field(fields, "route", "rendering plan")?,
            "plan route",
        )?;
        let to_area = match read::field(route, "to_area", "plan route")? {
            nomos_core::CanonicalValue::Null => None,
            other => Some(read::text(other, "route to_area")?.to_owned()),
        };
        let entry = match route.get(&nomos_core::FieldName::declared("entry")) {
            Some(value) => Some(cell(value, "route entry")?),
            None => None,
        };
        if start == entry.is_some() {
            return Err(PlayError::new(
                codes::DOCUMENT_VALUE,
                "a plan declares `route.entry` exactly when it is not the start area",
            ));
        }

        let semantics_digest = semantics_digest(fields)?;

        Ok(Self {
            area: area_id,
            start,
            width,
            height,
            masses,
            kinds,
            actors,
            pursuit_light,
            objective_gate,
            to_area,
            entry,
            semantics_digest,
        })
    }

    /// Whether a cell is inside the declared lattice.
    #[must_use]
    pub const fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    /// The mass covering a cell, if any, in authored order.
    #[must_use]
    pub fn mass_at(&self, x: i32, y: i32) -> Option<&Mass> {
        self.masses.iter().find(|mass| mass.covers(x, y))
    }

    /// The entity's compiled kind, if the plan declares it.
    #[must_use]
    pub fn kind_of(&self, entity: &EntityId) -> Option<&str> {
        self.kinds.get(entity).map(String::as_str)
    }
}

fn require_one_player(actors: &[PlanActor]) -> PlayResult<()> {
    let players = actors
        .iter()
        .filter(|actor| actor.role == Role::Player)
        .count();
    let pursuers = actors
        .iter()
        .filter(|actor| actor.role == Role::Pursuer)
        .count();
    if players != 1 {
        return Err(PlayError::new(
            codes::ACTORS_INVALID,
            format!("a plan declares exactly one `player` actor; this one declares {players}"),
        ));
    }
    if pursuers > 1 {
        return Err(PlayError::new(
            codes::ACTORS_INVALID,
            format!("a plan declares at most one `pursuer` actor; this one declares {pursuers}"),
        ));
    }
    let mut ids: Vec<&EntityId> = actors.iter().map(|actor| &actor.id).collect();
    ids.sort();
    let unique = ids.len();
    ids.dedup();
    if ids.len() != unique {
        return Err(PlayError::new(
            codes::ACTORS_INVALID,
            "an actor identity occurs more than once",
        ));
    }
    Ok(())
}

fn semantics_digest(
    fields: &BTreeMap<nomos_core::FieldName, nomos_core::CanonicalValue>,
) -> PlayResult<Sha256Digest> {
    for row in read::array(
        read::field(fields, "projection_digests", "rendering plan")?,
        "projection digests",
    )? {
        let entry = read::object(row, "projection digest")?;
        if read::text(
            read::field(entry, "file", "projection digest")?,
            "digest file",
        )? == SEMANTICS_MEMBER
        {
            let hex = read::text(
                read::field(entry, "digest", "projection digest")?,
                "digest value",
            )?;
            return Sha256Digest::from_hex(hex).ok_or_else(|| {
                PlayError::new(
                    codes::DOCUMENT_VALUE,
                    format!("`{SEMANTICS_MEMBER}` digest `{hex}` is not a SHA-256 hex digest"),
                )
            });
        }
    }
    Err(PlayError::new(
        codes::DOCUMENT_SHAPE,
        format!("a rendering plan publishes a `{SEMANTICS_MEMBER}` projection digest"),
    ))
}

fn entity_id(value: &nomos_core::CanonicalValue, label: &str) -> PlayResult<EntityId> {
    EntityId::parse(read::text(value, label)?).map_err(|error| {
        PlayError::new(
            codes::DOCUMENT_VALUE,
            format!("{label}: {}", error.message()),
        )
    })
}

/// Reads a `{x, y, z}` cell, requiring `z == 0`: R1's lattice is one storey.
pub(crate) fn cell(value: &nomos_core::CanonicalValue, label: &str) -> PlayResult<LatticeCell> {
    let fields = read::object(value, label)?;
    read::require_fields(fields, &["x", "y", "z"], label)?;
    let z = read::int32(read::field(fields, "z", label)?, "cell z")?;
    if z != 0 {
        return Err(PlayError::new(
            codes::DOCUMENT_VALUE,
            format!("{label} declares z {z}; this runtime plays one storey"),
        ));
    }
    Ok(LatticeCell::new(
        read::int32(read::field(fields, "x", label)?, "cell x")?,
        read::int32(read::field(fields, "y", label)?, "cell y")?,
        0,
    ))
}

fn pair(value: &nomos_core::CanonicalValue, label: &str) -> PlayResult<(i32, i32)> {
    let fields = read::object(value, label)?;
    read::require_fields(fields, &["x", "y"], label)?;
    Ok((
        read::int32(read::field(fields, "x", label)?, "corner x")?,
        read::int32(read::field(fields, "y", label)?, "corner y")?,
    ))
}
