//! Owner of the complete `nomos.observed_scene@1` type set and strict reader.

use std::collections::BTreeSet;

use nomos_core::{CanonicalValue, RepairClass};

use crate::diagnostic::{ObservedError, ObservedResult, codes};
use crate::{json, value};

mod validate;

/// The sole R2 input schema identity.
pub const SCHEMA: &str = "nomos.observed_scene@1";

/// One bounded scene-local identity with no cross-observation meaning.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LocalId(String);

impl LocalId {
    /// Accepts `[a-z][a-z0-9_]{0,63}`.
    pub fn new(text: &str) -> ObservedResult<Self> {
        let bytes = text.as_bytes();
        let valid = (1..=64).contains(&bytes.len())
            && bytes[0].is_ascii_lowercase()
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_');
        if !valid {
            return Err(ObservedError::new(
                codes::IDENTITY_INVALID,
                format!("`{text}` is not a valid scene-local identity"),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape));
        }
        Ok(Self(text.to_owned()))
    }

    /// The exact source spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The bounded integer crop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Crop {
    pub(crate) height: u8,
    pub(crate) width: u8,
}

impl Crop {
    /// Crop width in cells.
    #[must_use]
    pub const fn width(self) -> u8 {
        self.width
    }

    /// Crop height in cells.
    #[must_use]
    pub const fn height(self) -> u8 {
        self.height
    }
}

/// The observation's scene identity wrapper.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SceneIdentity {
    pub(crate) id: LocalId,
}

impl SceneIdentity {
    /// The local scene identity.
    #[must_use]
    pub fn id(&self) -> &LocalId {
        &self.id
    }
}

/// One integer terrain cell.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TerrainCell {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

impl TerrainCell {
    /// Horizontal cell coordinate.
    #[must_use]
    pub const fn x(self) -> u8 {
        self.x
    }

    /// Vertical cell coordinate.
    #[must_use]
    pub const fn y(self) -> u8 {
        self.y
    }
}

/// The complete terrain-role vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TerrainRole {
    /// Quiet base terrain presentation.
    CalmGround,
    /// Observer-supplied route presentation.
    TraversableRoute,
    /// Observer-supplied structure footprint.
    StructureFootprint,
}

impl TerrainRole {
    /// The exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CalmGround => "calm_ground",
            Self::TraversableRoute => "traversable_route",
            Self::StructureFootprint => "structure_footprint",
        }
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        match text {
            "calm_ground" => Ok(Self::CalmGround),
            "traversable_route" => Ok(Self::TraversableRoute),
            "structure_footprint" => Ok(Self::StructureFootprint),
            _ => Err(value::enum_error(
                path,
                "`calm_ground | traversable_route | structure_footprint`",
            )),
        }
    }
}

/// One ordered terrain layer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TerrainLayer {
    pub(crate) cells: Vec<TerrainCell>,
    pub(crate) id: LocalId,
    pub(crate) role: TerrainRole,
}

impl TerrainLayer {
    /// Cells in strict row-major order.
    #[must_use]
    pub fn cells(&self) -> &[TerrainCell] {
        &self.cells
    }

    /// The local layer identity.
    #[must_use]
    pub fn id(&self) -> &LocalId {
        &self.id
    }

    /// The supplied terrain role.
    #[must_use]
    pub const fn role(&self) -> TerrainRole {
        self.role
    }
}

/// One integer actor location; elevation is fixed to zero.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ActorCell {
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) z: i8,
}

impl ActorCell {
    /// Horizontal cell coordinate.
    #[must_use]
    pub const fn x(self) -> u8 {
        self.x
    }

    /// Vertical cell coordinate.
    #[must_use]
    pub const fn y(self) -> u8 {
        self.y
    }

    /// Elevation, exactly zero in revision 1.
    #[must_use]
    pub const fn z(self) -> i8 {
        self.z
    }
}

/// The complete observed life-state vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LifeState {
    /// A living observed figure.
    Living,
    /// A dead observed figure.
    Dead,
}

impl LifeState {
    /// The exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Living => "living",
            Self::Dead => "dead",
        }
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        match text {
            "living" => Ok(Self::Living),
            "dead" => Ok(Self::Dead),
            _ => Err(value::enum_error(path, "`living | dead`")),
        }
    }
}

/// One actor with four independently supplied facts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Actor {
    pub(crate) cell: ActorCell,
    pub(crate) controlled: bool,
    pub(crate) hostile: bool,
    pub(crate) id: LocalId,
    pub(crate) life_state: LifeState,
    pub(crate) protected: bool,
}

impl Actor {
    /// The supplied actor cell.
    #[must_use]
    pub const fn cell(&self) -> ActorCell {
        self.cell
    }

    /// Whether the observer marked this actor controlled.
    #[must_use]
    pub const fn controlled(&self) -> bool {
        self.controlled
    }

    /// Whether the observer marked this actor hostile.
    #[must_use]
    pub const fn hostile(&self) -> bool {
        self.hostile
    }

    /// The local actor identity.
    #[must_use]
    pub fn id(&self) -> &LocalId {
        &self.id
    }

    /// The supplied life state.
    #[must_use]
    pub const fn life_state(&self) -> LifeState {
        self.life_state
    }

    /// Whether the observer marked this actor protected.
    #[must_use]
    pub const fn protected(&self) -> bool {
        self.protected
    }
}

/// The complete supplied action-availability vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Availability {
    /// The supplying observer marked the action enabled.
    Enabled,
    /// The supplying observer marked the action disabled.
    Disabled,
}

impl Availability {
    /// The exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        match text {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(value::enum_error(path, "`enabled | disabled`")),
        }
    }
}

/// One opaque, target-associated observed action.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Action {
    pub(crate) availability: Availability,
    pub(crate) id: LocalId,
    pub(crate) target_actor: LocalId,
}

impl Action {
    /// The supplied availability.
    #[must_use]
    pub const fn availability(&self) -> Availability {
        self.availability
    }

    /// The opaque local action identity.
    #[must_use]
    pub fn id(&self) -> &LocalId {
        &self.id
    }

    /// The supplied target actor identity.
    #[must_use]
    pub fn target_actor(&self) -> &LocalId {
        &self.target_actor
    }
}

/// The complete typed `nomos.observed_scene@1` document.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ObservedScene {
    pub(crate) actions: Vec<Action>,
    pub(crate) actors: Vec<Actor>,
    pub(crate) crop: Crop,
    pub(crate) scene: SceneIdentity,
    pub(crate) terrain_layers: Vec<TerrainLayer>,
}

impl ObservedScene {
    /// Strictly reads exact canonical input bytes.
    pub fn from_bytes(bytes: &[u8]) -> ObservedResult<Self> {
        let value = json::parse(bytes)?;
        Self::from_canonical(&value)
    }

    /// Strictly reads an already-canonical value.
    pub fn from_canonical(value: &CanonicalValue) -> ObservedResult<Self> {
        let root = value::object(value, "$")?;
        let schema = root
            .iter()
            .find_map(|(name, value)| (name.as_str() == "schema").then_some(value));
        if !matches!(schema, Some(CanonicalValue::Text(text)) if text == SCHEMA) {
            return Err(ObservedError::new(
                codes::SCHEMA_MISMATCH,
                format!("`$.schema` must be `{SCHEMA}`"),
            ));
        }
        value::exact_fields(
            root,
            &[
                "actions",
                "actors",
                "crop",
                "scene",
                "schema",
                "terrain_layers",
            ],
            "$",
        )?;
        validate::document(root)?;

        let crop = parse_crop(value::field(root, "crop", "$")?)?;
        let scene = parse_scene(value::field(root, "scene", "$")?)?;
        let terrain_layers = parse_layers(value::field(root, "terrain_layers", "$")?, crop)?;
        let actors = parse_actors(value::field(root, "actors", "$")?, crop)?;
        let actions = parse_actions(value::field(root, "actions", "$")?, &actors)?;

        Ok(Self {
            actions,
            actors,
            crop,
            scene,
            terrain_layers,
        })
    }

    /// Canonical value preserving every supplied fact.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "actions",
                CanonicalValue::Array(self.actions.iter().map(action_value).collect()),
            ),
            (
                "actors",
                CanonicalValue::Array(self.actors.iter().map(actor_value).collect()),
            ),
            ("crop", crop_value(self.crop)),
            (
                "scene",
                CanonicalValue::object_declared([(
                    "id",
                    CanonicalValue::text(self.scene.id.as_str()),
                )]),
            ),
            ("schema", CanonicalValue::text(SCHEMA)),
            (
                "terrain_layers",
                CanonicalValue::Array(self.terrain_layers.iter().map(layer_value).collect()),
            ),
        ])
    }

    /// Exact canonical bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    /// Ordered actions.
    #[must_use]
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Ordered actors.
    #[must_use]
    pub fn actors(&self) -> &[Actor] {
        &self.actors
    }

    /// The crop.
    #[must_use]
    pub const fn crop(&self) -> Crop {
        self.crop
    }

    /// The scene wrapper.
    #[must_use]
    pub fn scene(&self) -> &SceneIdentity {
        &self.scene
    }

    /// Ordered terrain layers.
    #[must_use]
    pub fn terrain_layers(&self) -> &[TerrainLayer] {
        &self.terrain_layers
    }
}

fn parse_crop(value_: &CanonicalValue) -> ObservedResult<Crop> {
    let object = value::object(value_, "$.crop")?;
    value::exact_fields(object, &["height", "width"], "$.crop")?;
    let height = bounded_coordinate(
        value::field(object, "height", "$.crop")?,
        "$.crop.height",
        1,
        32,
    )?;
    let width = bounded_coordinate(
        value::field(object, "width", "$.crop")?,
        "$.crop.width",
        1,
        32,
    )?;
    Ok(Crop { height, width })
}

fn parse_scene(value_: &CanonicalValue) -> ObservedResult<SceneIdentity> {
    let object = value::object(value_, "$.scene")?;
    value::exact_fields(object, &["id"], "$.scene")?;
    let id = LocalId::new(value::text(
        value::field(object, "id", "$.scene")?,
        "$.scene.id",
    )?)?;
    Ok(SceneIdentity { id })
}

fn parse_layers(value_: &CanonicalValue, crop: Crop) -> ObservedResult<Vec<TerrainLayer>> {
    let rows = value::array(value_, "$.terrain_layers")?;
    if !(3..=8).contains(&rows.len()) {
        return Err(value::bound_error(
            "`$.terrain_layers` must contain 3..=8 rows",
        ));
    }
    let mut layers = Vec::with_capacity(rows.len());
    let mut previous: Option<LocalId> = None;
    let mut role_set = BTreeSet::new();
    let mut assignments = 0_usize;
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(object, &["cells", "id", "role"], &path)?;
        let id = LocalId::new(value::text(
            value::field(object, "id", &path)?,
            &format!("{path}.id"),
        )?)?;
        require_id_order(previous.as_ref(), &id, &path)?;
        previous = Some(id.clone());
        let role = TerrainRole::parse(
            value::text(
                value::field(object, "role", &path)?,
                &format!("{path}.role"),
            )?,
            &format!("{path}.role"),
        )?;
        role_set.insert(role);
        let cells = parse_terrain_cells(value::field(object, "cells", &path)?, crop, &path)?;
        assignments += cells.len();
        layers.push(TerrainLayer { cells, id, role });
    }
    if !(3..=4096).contains(&assignments) {
        return Err(value::bound_error(
            "total terrain cell assignments must be 3..=4096",
        ));
    }
    if role_set.len() != 3 {
        return Err(value::bound_error(
            "the scene must contain at least one layer of every terrain role",
        ));
    }
    Ok(layers)
}

fn parse_terrain_cells(
    value_: &CanonicalValue,
    crop: Crop,
    parent: &str,
) -> ObservedResult<Vec<TerrainCell>> {
    let rows = value::array(value_, &format!("{parent}.cells"))?;
    if !(1..=1024).contains(&rows.len()) {
        return Err(value::bound_error(format!(
            "`{parent}.cells` must contain 1..=1024 rows"
        )));
    }
    let mut cells = Vec::with_capacity(rows.len());
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for (index, row) in rows.iter().enumerate() {
        let path = format!("{parent}.cells[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(object, &["x", "y"], &path)?;
        let x = bounded_coordinate(
            value::field(object, "x", &path)?,
            &format!("{path}.x"),
            0,
            i64::from(crop.width) - 1,
        )?;
        let y = bounded_coordinate(
            value::field(object, "y", &path)?,
            &format!("{path}.y"),
            0,
            i64::from(crop.height) - 1,
        )?;
        let cell = TerrainCell { x, y };
        if !seen.insert(cell) {
            return Err(value::bound_error(format!(
                "duplicate terrain cell at `{path}`"
            )));
        }
        if previous.is_some_and(|prior: TerrainCell| (prior.y, prior.x) >= (cell.y, cell.x)) {
            return Err(ObservedError::new(
                codes::INPUT_NOT_CANONICAL,
                format!("`{parent}.cells` is not strict row-major order"),
            )
            .with_repair(RepairClass::EmitCanonicalBytes));
        }
        previous = Some(cell);
        cells.push(cell);
    }
    Ok(cells)
}

fn parse_actors(value_: &CanonicalValue, crop: Crop) -> ObservedResult<Vec<Actor>> {
    let rows = value::array(value_, "$.actors")?;
    if !(1..=64).contains(&rows.len()) {
        return Err(value::bound_error("`$.actors` must contain 1..=64 rows"));
    }
    let mut actors = Vec::with_capacity(rows.len());
    let mut previous: Option<LocalId> = None;
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.actors[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(
            object,
            &[
                "cell",
                "controlled",
                "hostile",
                "id",
                "life_state",
                "protected",
            ],
            &path,
        )?;
        let id = LocalId::new(value::text(
            value::field(object, "id", &path)?,
            &format!("{path}.id"),
        )?)?;
        require_id_order(previous.as_ref(), &id, &path)?;
        previous = Some(id.clone());
        let cell = parse_actor_cell(value::field(object, "cell", &path)?, crop, &path)?;
        let controlled = value::boolean(
            value::field(object, "controlled", &path)?,
            &format!("{path}.controlled"),
        )?;
        let hostile = value::boolean(
            value::field(object, "hostile", &path)?,
            &format!("{path}.hostile"),
        )?;
        let life_state = LifeState::parse(
            value::text(
                value::field(object, "life_state", &path)?,
                &format!("{path}.life_state"),
            )?,
            &format!("{path}.life_state"),
        )?;
        let protected = value::boolean(
            value::field(object, "protected", &path)?,
            &format!("{path}.protected"),
        )?;
        actors.push(Actor {
            cell,
            controlled,
            hostile,
            id,
            life_state,
            protected,
        });
    }
    Ok(actors)
}

fn parse_actor_cell(
    value_: &CanonicalValue,
    crop: Crop,
    parent: &str,
) -> ObservedResult<ActorCell> {
    let path = format!("{parent}.cell");
    let object = value::object(value_, &path)?;
    value::exact_fields(object, &["x", "y", "z"], &path)?;
    let x = bounded_coordinate(
        value::field(object, "x", &path)?,
        &format!("{path}.x"),
        0,
        i64::from(crop.width) - 1,
    )?;
    let y = bounded_coordinate(
        value::field(object, "y", &path)?,
        &format!("{path}.y"),
        0,
        i64::from(crop.height) - 1,
    )?;
    let z = value::integer(value::field(object, "z", &path)?, &format!("{path}.z"))?;
    if z != 0 {
        return Err(value::bound_error(format!(
            "`{path}.z` must be exactly zero"
        )));
    }
    Ok(ActorCell { x, y, z: 0 })
}

fn parse_actions(value_: &CanonicalValue, actors: &[Actor]) -> ObservedResult<Vec<Action>> {
    let rows = value::array(value_, "$.actions")?;
    if rows.len() > 128 {
        return Err(value::bound_error("`$.actions` must contain 0..=128 rows"));
    }
    let actor_ids: BTreeSet<&LocalId> = actors.iter().map(|actor| &actor.id).collect();
    let mut actions = Vec::with_capacity(rows.len());
    let mut previous: Option<LocalId> = None;
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.actions[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(object, &["availability", "id", "target_actor"], &path)?;
        let id = LocalId::new(value::text(
            value::field(object, "id", &path)?,
            &format!("{path}.id"),
        )?)?;
        require_id_order(previous.as_ref(), &id, &path)?;
        previous = Some(id.clone());
        let availability = Availability::parse(
            value::text(
                value::field(object, "availability", &path)?,
                &format!("{path}.availability"),
            )?,
            &format!("{path}.availability"),
        )?;
        let target_actor = LocalId::new(value::text(
            value::field(object, "target_actor", &path)?,
            &format!("{path}.target_actor"),
        )?)?;
        if !actor_ids.contains(&target_actor) {
            return Err(ObservedError::new(
                codes::TARGET_DANGLING,
                format!("`{path}.target_actor` names no actor"),
            )
            .with_repair(RepairClass::DeclareReferencedEntity));
        }
        actions.push(Action {
            availability,
            id,
            target_actor,
        });
    }
    Ok(actions)
}

fn bounded_coordinate(
    value_: &CanonicalValue,
    path: &str,
    minimum: i64,
    maximum: i64,
) -> ObservedResult<u8> {
    let number = value::integer(value_, path)?;
    if !(minimum..=maximum).contains(&number) {
        return Err(value::bound_error(format!(
            "`{path}` must be in {minimum}..={maximum}"
        )));
    }
    u8::try_from(number).map_err(|_| value::bound_error(format!("`{path}` is out of range")))
}

fn require_id_order(
    previous: Option<&LocalId>,
    current: &LocalId,
    path: &str,
) -> ObservedResult<()> {
    let Some(previous) = previous else {
        return Ok(());
    };
    if previous == current {
        return Err(ObservedError::new(
            codes::IDENTITY_INVALID,
            format!("duplicate identity `{}` at `{path}`", current.as_str()),
        )
        .with_repair(RepairClass::RemoveDuplicateDeclaration));
    }
    if previous > current {
        return Err(ObservedError::new(
            codes::INPUT_NOT_CANONICAL,
            format!("collection at `{path}` is not strictly ordered by identity"),
        )
        .with_repair(RepairClass::EmitCanonicalBytes));
    }
    Ok(())
}

pub(crate) fn crop_value(crop: Crop) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("height", CanonicalValue::Int(i64::from(crop.height))),
        ("width", CanonicalValue::Int(i64::from(crop.width))),
    ])
}

pub(crate) fn terrain_cell_value(cell: TerrainCell) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("x", CanonicalValue::Int(i64::from(cell.x))),
        ("y", CanonicalValue::Int(i64::from(cell.y))),
    ])
}

fn layer_value(layer: &TerrainLayer) -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "cells",
            CanonicalValue::Array(
                layer
                    .cells
                    .iter()
                    .copied()
                    .map(terrain_cell_value)
                    .collect(),
            ),
        ),
        ("id", CanonicalValue::text(layer.id.as_str())),
        ("role", CanonicalValue::text(layer.role.as_str())),
    ])
}

pub(crate) fn actor_cell_value(cell: ActorCell) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("x", CanonicalValue::Int(i64::from(cell.x))),
        ("y", CanonicalValue::Int(i64::from(cell.y))),
        ("z", CanonicalValue::Int(i64::from(cell.z))),
    ])
}

fn actor_value(actor: &Actor) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("cell", actor_cell_value(actor.cell)),
        ("controlled", CanonicalValue::Bool(actor.controlled)),
        ("hostile", CanonicalValue::Bool(actor.hostile)),
        ("id", CanonicalValue::text(actor.id.as_str())),
        (
            "life_state",
            CanonicalValue::text(actor.life_state.as_str()),
        ),
        ("protected", CanonicalValue::Bool(actor.protected)),
    ])
}

fn action_value(action: &Action) -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "availability",
            CanonicalValue::text(action.availability.as_str()),
        ),
        ("id", CanonicalValue::text(action.id.as_str())),
        (
            "target_actor",
            CanonicalValue::text(action.target_actor.as_str()),
        ),
    ])
}
