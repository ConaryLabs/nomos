//! The typed presentation source, decoded strictly.
//!
//! This is the owner file for `nomos.presentation_source@2`, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`, and it replaces the unversioned
//! `area.json` reader R1-2 landed. `RUNTIME.md` section 5 R1-3 and
//! `docs/review/presentation-source.md` are the design; this module is that
//! design as code.
//!
//! What changed from `area.json`, and why each one is here rather than in a
//! consumer:
//!
//! - **Versioned.** The file declares `nomos.presentation_source@2`. A
//!   different name or version is refused with `RP0104` naming both sides, so
//!   a source written for a later schema cannot be read as if it were this one.
//! - **Integer-only.** [`crate::json`] has no decimal variant, so a raw
//!   floating-point transform cannot reach this module at all. That covers the
//!   twenty-six values `docs/review/executable-gaol-ownership-audit.md`'s
//!   floating-point section lists.
//! - **Closed.** Every object's field set is checked exactly. An unknown field
//!   is refused rather than ignored, so a typo cannot silently disable a fact.
//! - **Named identifiers.** Three grammars — [`AREA_ID`], [`ENTITY_ID`],
//!   [`ASSEMBLY_NAME`] — each checked, so an identifier's shape is a schema
//!   property instead of whatever the first consumer happens to tolerate.
//! - **Attachment by socket.** An effect names `{entity, socket}` and carries
//!   no coordinate. The socket vocabulary is closed per entity kind
//!   ([`crate::catalog::EntityKind::sockets`]); the socket's *offset* is
//!   renderer-catalog data and appears nowhere in this crate.
//! - **One owner per fact.** `primaryGate`, `objective.target`, and `exit.gate`
//!   were three fields forced equal; only `route.exit.gate` survives, and
//!   [`crate::plan`] derives the objective from it.
//! - **`route.entry` is this area's own arrival cell.** `area.json` had the
//!   exiting area author a cell inside its *destination*; that cross-area
//!   authority is gone. Every non-start area declares the cell a player arrives
//!   on, and it is validated here against *that area's* bounds and masses and
//!   against the starting cell of its sole player-role actor.
//!
//! The bounded-area invariants `experiments/executable-gaol/src/build-plan.mjs:73-84`
//! enforced as compiler magic numbers — the 9x6 lattice, the wall-height bound,
//! the mass-height bound — are now declared constants of this schema, stated in
//! `experiments/executable-gaol/AUTHORING.md` and refused with `RP0202`. That is
//! the audit's "Derived by convention" items 8 and 9.

use std::collections::BTreeSet;
use std::path::Path;

use nomos_core::id::SchemaId;

use crate::catalog::EntityKind;
use crate::error::{PlanError, PlanResult, codes};
use crate::json::{self, Json};

/// The presentation source's schema identity.
///
/// Spelled in the file as the bare `name@version` string, which is what
/// `nomos entity-catalog`, `nomos effective-facts`, and the rendering plan all
/// use. Issue #145 owns choosing one spelling across R1 documents; this slice
/// changes none of them.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn presentation_source_schema() -> SchemaId {
    SchemaId::new("nomos.presentation_source", 2)
        .expect("the presentation-source schema id is a literal")
}

/// The widest lattice an area may declare.
pub const MAX_BOUNDS_WIDTH: i64 = 9;
/// The deepest lattice an area may declare.
pub const MAX_BOUNDS_HEIGHT: i64 = 6;
/// The tallest wall, in `vertical_step` units of one tenth of a lattice cell.
pub const MAX_WALL_HEIGHT_STEPS: i64 = 50;
/// The tallest masonry mass, in the same units.
pub const MAX_MASS_HEIGHT_STEPS: i64 = 40;
/// The most masonry masses one area may declare.
pub const MAX_MASSES: usize = 8;
/// The most effects one area may declare.
pub const MAX_EFFECTS: usize = 8;
/// The longest an authored label may be, in characters.
pub const MAX_LABEL_CHARS: usize = 64;
/// The longest an identifier may be, in bytes.
pub const MAX_IDENTIFIER_BYTES: usize = 64;
/// The longest an assembly name may be, in bytes.
pub const MAX_ASSEMBLY_BYTES: usize = 96;
/// The longest a socket name may be, in bytes.
pub const MAX_SOCKET_BYTES: usize = 32;

/// The closed set of declared actor roles, in sorted order.
///
/// This replaces `REQUIRED_ACTORS`, the pair of magic identities `player` and
/// `gaoler` that `@1` required. The ownership audit's "Derived by convention"
/// items 7 and 21 recorded those as a magic-id gameplay role and deferred them
/// to R1-5; a declared role belongs with the authoritative actor collection,
/// and it is here. The identities are now free: content may name its actors
/// anything, and `tests/source.rs` renames both to prove nothing depends on the
/// strings.
///
/// Exactly one actor declares `player`, and at most one declares `pursuer`.
/// `RUNTIME.md` section 5 R1-5 rules a second pursuer out as content, so the
/// bound is enforced here rather than left to the runtime to discover.
pub const ACTOR_ROLES: [&str; 2] = ["player", "pursuer"];

/// A lattice cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    /// Column.
    pub x: i64,
    /// Row.
    pub y: i64,
    /// Elevation; always zero in the bounded profile.
    pub z: i64,
}

/// A lattice corner, used by a mass rectangle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Corner {
    /// Column.
    pub x: i64,
    /// Row.
    pub y: i64,
}

/// The area's identity in the route graph.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Area {
    /// Stable area identity; also the directory name.
    pub id: String,
    /// The authored display label, the only authored prose in the model.
    pub label: String,
    /// Whether the route begins here.
    pub start: bool,
}

/// The gate this area leaves by, and where it leads.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Exit {
    /// The compiled door the objective targets.
    pub gate: String,
    /// The area the gate leads to, or `None` at the route's terminal.
    pub to_area: Option<String>,
}

/// This area's place in the route.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Route {
    /// The declared exit.
    pub exit: Exit,
    /// The cell a player arrives on; `None` exactly for the start area.
    pub entry: Option<Cell>,
}

/// The pursuit facts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pursuit {
    /// The compiled light whose extinction wakes the gaoler.
    pub light: String,
}

/// The bounded lattice extent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    /// Columns.
    pub width: i64,
    /// Rows.
    pub height: i64,
}

/// The shared architectural style.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Style {
    /// The masonry assembly name.
    pub assembly: String,
    /// The masonry material family.
    pub material_family: String,
    /// The masonry trim family.
    pub trim_family: String,
}

/// One masonry mass.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Mass {
    /// Stable mass identity, unique within the area.
    pub id: String,
    /// Inclusive lower corner.
    pub min: Corner,
    /// Exclusive upper corner.
    pub max: Corner,
    /// Height in `vertical_step` units.
    pub height_steps: i64,
}

/// The bounded architecture block.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Architecture {
    /// The lattice extent.
    pub bounds: Bounds,
    /// Wall height in `vertical_step` units.
    pub wall_height_steps: i64,
    /// The shared style.
    pub style: Style,
    /// The masonry masses, in authored order.
    pub masses: Vec<Mass>,
}

/// One presentation actor.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Actor {
    /// Stable actor identity.
    pub id: String,
    /// The renderer assembly this actor selects.
    pub assembly: String,
    /// The cell the actor starts on.
    pub cell: Cell,
    /// The role the runtime plays this actor as: `player` or `pursuer`.
    pub role: String,
}

/// Where an effect attaches.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EffectAnchor {
    /// The compiled entity the effect attaches to.
    pub entity: String,
    /// The named socket on that entity's assembly.
    pub socket: String,
}

/// One presentation effect.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Effect {
    /// Stable effect identity.
    pub id: String,
    /// The renderer assembly this effect selects.
    pub assembly: String,
    /// The socket attachment; deliberately not a coordinate.
    pub anchor: EffectAnchor,
}

/// The decoded presentation source.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PresentationSource {
    /// Area identity.
    pub area: Area,
    /// Route placement.
    pub route: Route,
    /// Pursuit facts.
    pub pursuit: Pursuit,
    /// Bounded architecture.
    pub architecture: Architecture,
    /// Presentation actors.
    pub actors: Vec<Actor>,
    /// Presentation effects.
    pub effects: Vec<Effect>,
}

/// Reads and validates one presentation source against the compiled entities.
///
/// `entity_kind` resolves a compiled entity id to its catalogued kind, and is
/// the only way this module learns what a door or a light is: nothing here
/// inspects an entity id, a machine namespace, or an assembly string to decide
/// a kind.
///
/// # Errors
///
/// Returns `RP0101` when the file cannot be read, `RP0103` when it is not
/// well-formed JSON, `RP0205` when it carries a number that is not an integer,
/// `RP0104` when its schema identity is not `nomos.presentation_source@2`,
/// `RP0206` when an identifier is outside its declared grammar, and `RP0202`
/// for every other invariant it breaks.
pub fn read_source(
    path: &Path,
    entity_kind: &dyn Fn(&str) -> Option<EntityKind>,
) -> PlanResult<PresentationSource> {
    let bytes = std::fs::read(path)
        .map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(path))?;
    let document = json::parse(&bytes).map_err(|error| error.at(path))?;
    decode(&document, entity_kind).map_err(|error| error.at(path))
}

fn decode(
    document: &Json,
    entity_kind: &dyn Fn(&str) -> Option<EntityKind>,
) -> PlanResult<PresentationSource> {
    exact_fields(
        document,
        &[
            "actors",
            "architecture",
            "area",
            "effects",
            "pursuit",
            "route",
            "schema",
        ],
        "the presentation source",
    )?;
    bind_schema(document)?;

    let area = decode_area(field(document, "area")?)?;
    let architecture = decode_architecture(field(document, "architecture")?)?;
    let route = decode_route(field(document, "route")?, &area, &architecture, entity_kind)?;
    let pursuit = decode_pursuit(field(document, "pursuit")?, entity_kind)?;
    let actors = decode_actors(field(document, "actors")?, &architecture)?;
    validate_arrival(&route, &actors)?;
    let effects = decode_effects(field(document, "effects")?, entity_kind)?;

    Ok(PresentationSource {
        area,
        route,
        pursuit,
        architecture,
        actors,
        effects,
    })
}

fn validate_arrival(route: &Route, actors: &[Actor]) -> PlanResult<()> {
    let Some(entry) = route.entry else {
        return Ok(());
    };
    let player = actors
        .iter()
        .find(|actor| actor.role == "player")
        .expect("decode_actors guarantees exactly one player-role actor");
    if entry != player.cell {
        return Err(invalid(format!(
            "`route.entry` ({}, {}, {}) does not equal player-role actor `{}`'s cell ({}, {}, {})",
            entry.x, entry.y, entry.z, player.id, player.cell.x, player.cell.y, player.cell.z
        )));
    }
    Ok(())
}

/// Binds the declared identity.
///
/// Checked before any other field is read, so a source written for a different
/// version is refused by identity rather than by the first shape difference it
/// happens to have.
fn bind_schema(document: &Json) -> PlanResult<()> {
    let expected = presentation_source_schema();
    let found = document
        .get("schema")
        .and_then(Json::as_text)
        .ok_or_else(|| {
            PlanError::new(
                codes::SCHEMA_MISMATCH,
                format!("expected schema `{expected}`, found no `schema` string"),
            )
        })?;
    if found != expected.to_string() {
        return Err(PlanError::new(
            codes::SCHEMA_MISMATCH,
            format!("expected schema `{expected}`, found `{found}`"),
        ));
    }
    Ok(())
}

fn decode_area(value: &Json) -> PlanResult<Area> {
    exact_fields(value, &["id", "label", "start"], "`area`")?;
    let id = area_id(text(value, "id")?, "area.id")?;
    let label = label(text(value, "label")?, "area.label")?;
    let start = value
        .get("start")
        .and_then(Json::as_bool)
        .ok_or_else(|| invalid("`area.start` must be a declared boolean".to_owned()))?;
    Ok(Area { id, label, start })
}

fn decode_route(
    value: &Json,
    area: &Area,
    architecture: &Architecture,
    entity_kind: &dyn Fn(&str) -> Option<EntityKind>,
) -> PlanResult<Route> {
    // The start area declares no `entry`, because nothing arrives there; every
    // other area declares exactly one. Checking the field set against
    // `area.start` is what makes that a schema property rather than a
    // convention some consumer has to remember.
    let expected: &[&str] = if area.start {
        &["exit"]
    } else {
        &["entry", "exit"]
    };
    exact_fields(value, expected, "`route`")?;

    let exit_value = field(value, "exit")?;
    exact_fields(exit_value, &["gate", "to_area"], "`route.exit`")?;
    let gate = entity_id(text(exit_value, "gate")?, "route.exit.gate")?;
    match entity_kind(&gate) {
        Some(EntityKind::Door) => {}
        Some(kind) => {
            return Err(invalid(format!(
                "`route.exit.gate` names `{gate}`, whose compiled kind is `{}`, not `door`",
                kind.as_str()
            )));
        }
        None => {
            return Err(invalid(format!(
                "`route.exit.gate` names `{gate}`, which is not a compiled entity"
            )));
        }
    }

    let to_area = match field(exit_value, "to_area")? {
        Json::Null => None,
        other => Some(area_id(
            other.as_text().ok_or_else(|| {
                invalid("`route.exit.to_area` must be a string or null".to_owned())
            })?,
            "route.exit.to_area",
        )?),
    };
    if to_area.as_deref() == Some(area.id.as_str()) {
        return Err(invalid(
            "`route.exit.to_area` may not name the area itself".to_owned(),
        ));
    }

    let entry = match value.get("entry") {
        None => None,
        Some(entry) => {
            let cell = decode_cell(entry, "route.entry")?;
            in_bounds(cell, architecture, "route.entry")?;
            if let Some(mass) = mass_at(architecture, cell) {
                return Err(invalid(format!(
                    "`route.entry` is inside masonry mass `{mass}`"
                )));
            }
            Some(cell)
        }
    };

    Ok(Route {
        exit: Exit { gate, to_area },
        entry,
    })
}

fn decode_pursuit(
    value: &Json,
    entity_kind: &dyn Fn(&str) -> Option<EntityKind>,
) -> PlanResult<Pursuit> {
    exact_fields(value, &["light"], "`pursuit`")?;
    let light = entity_id(text(value, "light")?, "pursuit.light")?;
    match entity_kind(&light) {
        Some(EntityKind::Light) => Ok(Pursuit { light }),
        Some(kind) => Err(invalid(format!(
            "`pursuit.light` names `{light}`, whose compiled kind is `{}`, not `light`",
            kind.as_str()
        ))),
        None => Err(invalid(format!(
            "`pursuit.light` names `{light}`, which is not a compiled entity"
        ))),
    }
}

fn decode_architecture(value: &Json) -> PlanResult<Architecture> {
    exact_fields(
        value,
        &["bounds", "masses", "style", "wall_height_steps"],
        "`architecture`",
    )?;

    let bounds_value = field(value, "bounds")?;
    exact_fields(bounds_value, &["height", "width"], "`architecture.bounds`")?;
    let width = integer(bounds_value, "width", "architecture.bounds.width")?;
    let height = integer(bounds_value, "height", "architecture.bounds.height")?;
    if !(1..=MAX_BOUNDS_WIDTH).contains(&width) || !(1..=MAX_BOUNDS_HEIGHT).contains(&height) {
        return Err(invalid(format!(
            "`architecture.bounds` is {width}x{height}; the bounded profile is \
             1..={MAX_BOUNDS_WIDTH} by 1..={MAX_BOUNDS_HEIGHT}"
        )));
    }
    let bounds = Bounds { width, height };

    let wall_height_steps = integer(value, "wall_height_steps", "architecture.wall_height_steps")?;
    if !(1..=MAX_WALL_HEIGHT_STEPS).contains(&wall_height_steps) {
        return Err(invalid(format!(
            "`architecture.wall_height_steps` is {wall_height_steps}; the bounded profile is \
             1..={MAX_WALL_HEIGHT_STEPS} vertical steps"
        )));
    }

    let style_value = field(value, "style")?;
    exact_fields(
        style_value,
        &["assembly", "material_family", "trim_family"],
        "`architecture.style`",
    )?;
    let style = Style {
        assembly: assembly_name(
            text(style_value, "assembly")?,
            "architecture.style.assembly",
        )?,
        material_family: family_name(
            text(style_value, "material_family")?,
            "architecture.style.material_family",
        )?,
        trim_family: family_name(
            text(style_value, "trim_family")?,
            "architecture.style.trim_family",
        )?,
    };

    let mass_values = array(value, "masses", "architecture.masses")?;
    if mass_values.len() > MAX_MASSES {
        return Err(invalid(format!(
            "`architecture.masses` declares {} masses; at most {MAX_MASSES} are allowed",
            mass_values.len()
        )));
    }
    let mut masses = Vec::with_capacity(mass_values.len());
    let mut seen = BTreeSet::new();
    for mass_value in mass_values {
        let mass = decode_mass(mass_value, bounds)?;
        if !seen.insert(mass.id.clone()) {
            return Err(invalid(format!(
                "masonry mass `{}` is declared more than once",
                mass.id
            )));
        }
        masses.push(mass);
    }

    Ok(Architecture {
        bounds,
        wall_height_steps,
        style,
        masses,
    })
}

fn decode_mass(value: &Json, bounds: Bounds) -> PlanResult<Mass> {
    exact_fields(
        value,
        &["height_steps", "id", "max", "min"],
        "a masonry mass",
    )?;
    let id = entity_id(text(value, "id")?, "architecture.masses[].id")?;
    let min = decode_corner(field(value, "min")?, "architecture.masses[].min")?;
    let max = decode_corner(field(value, "max")?, "architecture.masses[].max")?;
    let height_steps = integer(value, "height_steps", "architecture.masses[].height_steps")?;
    if !(1..=MAX_MASS_HEIGHT_STEPS).contains(&height_steps) {
        return Err(invalid(format!(
            "masonry mass `{id}` is {height_steps} vertical steps tall; the bounded profile is \
             1..={MAX_MASS_HEIGHT_STEPS}"
        )));
    }
    if min.x < 0 || min.y < 0 || min.x >= max.x || min.y >= max.y {
        return Err(invalid(format!(
            "masonry mass `{id}` is not a positive rectangle"
        )));
    }
    if max.x > bounds.width || max.y > bounds.height {
        return Err(invalid(format!(
            "masonry mass `{id}` leaves the {}x{} lattice",
            bounds.width, bounds.height
        )));
    }
    Ok(Mass {
        id,
        min,
        max,
        height_steps,
    })
}

fn decode_actors(value: &Json, architecture: &Architecture) -> PlanResult<Vec<Actor>> {
    let entries = value
        .as_array()
        .ok_or_else(|| invalid("`actors` must be an array".to_owned()))?;
    let mut actors = Vec::with_capacity(entries.len());
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for entry in entries {
        exact_fields(entry, &["assembly", "cell", "id", "role"], "an actor")?;
        let id = entity_id(text(entry, "id")?, "actors[].id")?;
        let assembly = assembly_name(text(entry, "assembly")?, "actors[].assembly")?;
        let cell = decode_cell(field(entry, "cell")?, "actors[].cell")?;
        let role = text(entry, "role")?.to_owned();
        if !ACTOR_ROLES.contains(&role.as_str()) {
            return Err(invalid(format!(
                "`actors[].role` must be `{}` or `{}`; `{role}` is neither",
                ACTOR_ROLES[0], ACTOR_ROLES[1]
            )));
        }
        in_bounds(cell, architecture, "actors[].cell")?;
        if let Some(mass) = mass_at(architecture, cell) {
            return Err(invalid(format!(
                "actor `{id}` starts inside masonry mass `{mass}`"
            )));
        }
        if !seen.insert(id.clone()) {
            return Err(invalid(format!("actor `{id}` is declared more than once")));
        }
        actors.push(Actor {
            id,
            assembly,
            cell,
            role,
        });
    }
    let players = actors.iter().filter(|actor| actor.role == "player").count();
    if players != 1 {
        return Err(invalid(format!(
            "`actors` must declare exactly one `player`; this area declares {players}"
        )));
    }
    let pursuers = actors
        .iter()
        .filter(|actor| actor.role == "pursuer")
        .count();
    if pursuers > 1 {
        return Err(invalid(format!(
            "`actors` must declare at most one `pursuer`; this area declares {pursuers}"
        )));
    }
    Ok(actors)
}

fn decode_effects(
    value: &Json,
    entity_kind: &dyn Fn(&str) -> Option<EntityKind>,
) -> PlanResult<Vec<Effect>> {
    let entries = value
        .as_array()
        .ok_or_else(|| invalid("`effects` must be an array".to_owned()))?;
    if entries.len() > MAX_EFFECTS {
        return Err(invalid(format!(
            "`effects` declares {} effects; at most {MAX_EFFECTS} are allowed",
            entries.len()
        )));
    }
    let mut effects = Vec::with_capacity(entries.len());
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for entry in entries {
        exact_fields(entry, &["anchor", "assembly", "id"], "an effect")?;
        let id = entity_id(text(entry, "id")?, "effects[].id")?;
        let assembly = assembly_name(text(entry, "assembly")?, "effects[].assembly")?;

        let anchor_value = field(entry, "anchor")?;
        exact_fields(anchor_value, &["entity", "socket"], "`effects[].anchor`")?;
        let entity = entity_id(text(anchor_value, "entity")?, "effects[].anchor.entity")?;
        let socket = socket_name(text(anchor_value, "socket")?, "effects[].anchor.socket")?;

        // The socket vocabulary is closed per entity kind. The offset it
        // resolves to is renderer-catalog data and is deliberately absent from
        // this crate: content selects a socket, the renderer decides where the
        // socket is.
        let Some(kind) = entity_kind(&entity) else {
            return Err(invalid(format!(
                "effect `{id}` anchors to `{entity}`, which is not a compiled entity"
            )));
        };
        if !kind.sockets().contains(&socket.as_str()) {
            let declared = if kind.sockets().is_empty() {
                "none".to_owned()
            } else {
                kind.sockets().join(", ")
            };
            return Err(invalid(format!(
                "effect `{id}` anchors to socket `{socket}` on `{entity}`, whose kind `{}` \
                 declares: {declared}",
                kind.as_str()
            )));
        }
        if !seen.insert(id.clone()) {
            return Err(invalid(format!("effect `{id}` is declared more than once")));
        }
        effects.push(Effect {
            id,
            assembly,
            anchor: EffectAnchor { entity, socket },
        });
    }
    Ok(effects)
}

fn decode_cell(value: &Json, context: &str) -> PlanResult<Cell> {
    exact_fields(value, &["x", "y", "z"], context)?;
    let cell = Cell {
        x: integer(value, "x", context)?,
        y: integer(value, "y", context)?,
        z: integer(value, "z", context)?,
    };
    if cell.z != 0 {
        return Err(invalid(format!(
            "`{context}` declares elevation {}; the bounded profile is z = 0",
            cell.z
        )));
    }
    Ok(cell)
}

fn decode_corner(value: &Json, context: &str) -> PlanResult<Corner> {
    exact_fields(value, &["x", "y"], context)?;
    Ok(Corner {
        x: integer(value, "x", context)?,
        y: integer(value, "y", context)?,
    })
}

fn in_bounds(cell: Cell, architecture: &Architecture, context: &str) -> PlanResult<()> {
    if cell.x < 0
        || cell.y < 0
        || cell.x >= architecture.bounds.width
        || cell.y >= architecture.bounds.height
    {
        return Err(invalid(format!(
            "`{context}` is ({}, {}), outside this area's {}x{} lattice",
            cell.x, cell.y, architecture.bounds.width, architecture.bounds.height
        )));
    }
    Ok(())
}

fn mass_at(architecture: &Architecture, cell: Cell) -> Option<&str> {
    architecture
        .masses
        .iter()
        .find(|mass| {
            cell.x >= mass.min.x
                && cell.x < mass.max.x
                && cell.y >= mass.min.y
                && cell.y < mass.max.y
        })
        .map(|mass| mass.id.as_str())
}

// ---------------------------------------------------------------------------
// Identifier grammars
// ---------------------------------------------------------------------------

/// `[a-z][a-z0-9]*(-[a-z0-9]+)*` — an area id, which is also a directory name
/// and a collection key. The only grammar admitting `-`.
pub const AREA_ID: &str = "[a-z][a-z0-9]*(-[a-z0-9]+)*";
/// `[a-z][a-z0-9_]*` — deliberately identical to `nomos_core::FieldName`, so an
/// entity id can key a canonical collection without widening anything. Family
/// and socket names use it too.
pub const ENTITY_ID: &str = "[a-z][a-z0-9_]*";
/// `[a-z][a-z0-9_]*(/[a-z][a-z0-9_]*)+` — a namespaced renderer assembly name.
pub const ASSEMBLY_NAME: &str = "[a-z][a-z0-9_]*(/[a-z][a-z0-9_]*)+";

fn area_id(value: &str, context: &str) -> PlanResult<String> {
    let legal = !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        });
    if legal {
        return Ok(value.to_owned());
    }
    Err(grammar(context, value, AREA_ID))
}

fn entity_id(value: &str, context: &str) -> PlanResult<String> {
    if is_identifier(value) && value.len() <= MAX_IDENTIFIER_BYTES {
        return Ok(value.to_owned());
    }
    Err(grammar(context, value, ENTITY_ID))
}

fn family_name(value: &str, context: &str) -> PlanResult<String> {
    if is_identifier(value) && value.len() <= MAX_IDENTIFIER_BYTES {
        return Ok(value.to_owned());
    }
    Err(grammar(context, value, ENTITY_ID))
}

fn socket_name(value: &str, context: &str) -> PlanResult<String> {
    if is_identifier(value) && value.len() <= MAX_SOCKET_BYTES {
        return Ok(value.to_owned());
    }
    Err(grammar(context, value, ENTITY_ID))
}

fn assembly_name(value: &str, context: &str) -> PlanResult<String> {
    let segments: Vec<&str> = value.split('/').collect();
    if value.len() <= MAX_ASSEMBLY_BYTES
        && segments.len() >= 2
        && segments.iter().all(|segment| is_identifier(segment))
    {
        return Ok(value.to_owned());
    }
    Err(grammar(context, value, ASSEMBLY_NAME))
}

fn label(value: &str, context: &str) -> PlanResult<String> {
    let characters = value.chars().count();
    if characters == 0 || characters > MAX_LABEL_CHARS {
        return Err(invalid(format!(
            "`{context}` is {characters} characters; the bounded profile is 1..={MAX_LABEL_CHARS}"
        )));
    }
    if value.chars().any(|character| (character as u32) < 0x20) {
        return Err(invalid(format!("`{context}` carries a control character")));
    }
    Ok(value.to_owned())
}

fn is_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

// ---------------------------------------------------------------------------
// Shape helpers
// ---------------------------------------------------------------------------

fn invalid(message: String) -> PlanError {
    PlanError::new(codes::AREA_INVALID, message)
}

fn grammar(context: &str, value: &str, expected: &str) -> PlanError {
    PlanError::new(
        codes::IDENTIFIER_UNSUPPORTED,
        format!("`{context}` is `{value}`, which is not `{expected}`"),
    )
}

/// Checks an object's field set exactly.
///
/// An absent field and an unknown field are both refusals. Ignoring an unknown
/// field would let a typo silently disable a fact, which is the class of defect
/// the audit found throughout `area.json`.
fn exact_fields(value: &Json, expected: &[&str], context: &str) -> PlanResult<()> {
    let fields = value
        .as_object()
        .ok_or_else(|| invalid(format!("{context} must be an object")))?;
    let found: BTreeSet<&str> = fields.keys().map(String::as_str).collect();
    let wanted: BTreeSet<&str> = expected.iter().copied().collect();
    if found == wanted {
        return Ok(());
    }
    let missing: Vec<&str> = wanted.difference(&found).copied().collect();
    let unknown: Vec<&str> = found.difference(&wanted).copied().collect();
    let mut reasons = Vec::new();
    if !missing.is_empty() {
        reasons.push(format!("is missing {}", missing.join(", ")));
    }
    if !unknown.is_empty() {
        reasons.push(format!("carries unknown {}", unknown.join(", ")));
    }
    Err(invalid(format!("{context} {}", reasons.join(" and "))))
}

fn field<'a>(value: &'a Json, name: &str) -> PlanResult<&'a Json> {
    value
        .get(name)
        .ok_or_else(|| invalid(format!("field `{name}` is absent")))
}

fn text<'a>(value: &'a Json, name: &str) -> PlanResult<&'a str> {
    field(value, name)?
        .as_text()
        .ok_or_else(|| invalid(format!("field `{name}` is not a string")))
}

fn integer(value: &Json, name: &str, context: &str) -> PlanResult<i64> {
    field(value, name)?
        .as_integer()
        .ok_or_else(|| invalid(format!("`{context}` is not an integer")))
}

fn array<'a>(value: &'a Json, name: &str, context: &str) -> PlanResult<&'a [Json]> {
    field(value, name)?
        .as_array()
        .ok_or_else(|| invalid(format!("`{context}` must be an array")))
}
