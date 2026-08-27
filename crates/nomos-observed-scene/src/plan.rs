//! Owner of `nomos.observed_scene_plan@1`, its compiler, and strict reader.

use nomos_core::{CanonicalValue, Sha256Digest};

use crate::diagnostic::{ObservedError, ObservedResult, codes};
use crate::input::{
    self, Action, Actor, Availability, Crop, LifeState, LocalId, ObservedScene, SceneIdentity,
    TerrainCell, TerrainLayer, TerrainRole,
};
use crate::{json, value};

mod validate;

/// The sole R2 plan schema identity.
pub const SCHEMA: &str = "nomos.observed_scene_plan@1";

/// Closed terrain assembly selection.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TerrainAssembly {
    /// Assembly for calm ground.
    CalmGround,
    /// Assembly for the supplied traversable route.
    TraversableRoute,
    /// Assembly for a supplied structure footprint.
    StructureFootprint,
}

impl TerrainAssembly {
    /// Exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CalmGround => "terrain/calm_ground",
            Self::TraversableRoute => "terrain/traversable_route",
            Self::StructureFootprint => "terrain/structure_footprint",
        }
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        match text {
            "terrain/calm_ground" => Ok(Self::CalmGround),
            "terrain/traversable_route" => Ok(Self::TraversableRoute),
            "terrain/structure_footprint" => Ok(Self::StructureFootprint),
            _ => Err(value::enum_error(
                path,
                "the three declared terrain assemblies",
            )),
        }
    }
}

/// Closed terrain material-family selection.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum MaterialFamily {
    /// Muted ground material family.
    GroundMuted,
    /// Worn route material family.
    RouteWorn,
    /// Stone structure material family.
    StructureStone,
}

impl MaterialFamily {
    /// Exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GroundMuted => "ground_muted",
            Self::RouteWorn => "route_worn",
            Self::StructureStone => "structure_stone",
        }
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        match text {
            "ground_muted" => Ok(Self::GroundMuted),
            "route_worn" => Ok(Self::RouteWorn),
            "structure_stone" => Ok(Self::StructureStone),
            _ => Err(value::enum_error(
                path,
                "the three declared material families",
            )),
        }
    }
}

/// One terrain row with copied facts and finite presentation selections.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TerrainPlan {
    pub(crate) assembly: TerrainAssembly,
    pub(crate) cells: Vec<TerrainCell>,
    pub(crate) id: LocalId,
    pub(crate) material_family: MaterialFamily,
    pub(crate) role: TerrainRole,
    pub(crate) stack: i64,
}

impl TerrainPlan {
    /// Selected assembly.
    #[must_use]
    pub const fn assembly(&self) -> TerrainAssembly {
        self.assembly
    }

    /// Copied cells.
    #[must_use]
    pub fn cells(&self) -> &[TerrainCell] {
        &self.cells
    }

    /// Copied local identity.
    #[must_use]
    pub fn id(&self) -> &LocalId {
        &self.id
    }

    /// Selected material family.
    #[must_use]
    pub const fn material_family(&self) -> MaterialFamily {
        self.material_family
    }

    /// Copied terrain role.
    #[must_use]
    pub const fn role(&self) -> TerrainRole {
        self.role
    }

    /// Selected draw stack.
    #[must_use]
    pub const fn stack(&self) -> i64 {
        self.stack
    }
}

/// The one actor assembly in revision 1.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ActorAssembly {
    /// The observed-figure assembly.
    ObservedFigure,
}

impl ActorAssembly {
    /// Exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        "actor/observed_figure"
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        if text == Self::ObservedFigure.as_str() {
            Ok(Self::ObservedFigure)
        } else {
            Err(value::enum_error(path, "`actor/observed_figure`"))
        }
    }
}

/// Closed actor pose selection.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ActorPose {
    /// Living upright pose.
    UprightLiving,
    /// Dead prone pose.
    ProneDead,
}

impl ActorPose {
    /// Exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UprightLiving => "upright_living",
            Self::ProneDead => "prone_dead",
        }
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        match text {
            "upright_living" => Ok(Self::UprightLiving),
            "prone_dead" => Ok(Self::ProneDead),
            _ => Err(value::enum_error(path, "`upright_living | prone_dead`")),
        }
    }
}

/// Closed marker presence selection.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Presence {
    /// The visual consequence is present.
    Present,
    /// The visual consequence is absent.
    Absent,
}

impl Presence {
    /// Exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Absent => "absent",
        }
    }

    fn from_bool(value: bool) -> Self {
        if value { Self::Present } else { Self::Absent }
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        match text {
            "present" => Ok(Self::Present),
            "absent" => Ok(Self::Absent),
            _ => Err(value::enum_error(path, "`present | absent`")),
        }
    }
}

/// One actor row with copied facts and independent finite selections.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ActorPlan {
    pub(crate) assembly: ActorAssembly,
    pub(crate) actor: Actor,
    pub(crate) controlled_marker: Presence,
    pub(crate) hostile_outline: Presence,
    pub(crate) pose: ActorPose,
    pub(crate) protection_ring: Presence,
}

impl ActorPlan {
    /// Selected assembly.
    #[must_use]
    pub const fn assembly(&self) -> ActorAssembly {
        self.assembly
    }

    /// Every copied actor fact.
    #[must_use]
    pub fn copied_actor(&self) -> &Actor {
        &self.actor
    }

    /// Controlled-marker selection.
    #[must_use]
    pub const fn controlled_marker(&self) -> Presence {
        self.controlled_marker
    }

    /// Hostile-outline selection.
    #[must_use]
    pub const fn hostile_outline(&self) -> Presence {
        self.hostile_outline
    }

    /// Life-state pose selection.
    #[must_use]
    pub const fn pose(&self) -> ActorPose {
        self.pose
    }

    /// Protection-ring selection.
    #[must_use]
    pub const fn protection_ring(&self) -> Presence {
        self.protection_ring
    }
}

/// Closed action-marker selection.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ActionMarker {
    /// Marker for a supplied enabled action.
    Enabled,
    /// Marker for a supplied disabled action.
    Disabled,
}

impl ActionMarker {
    /// Exact wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "action/enabled",
            Self::Disabled => "action/disabled",
        }
    }

    fn parse(text: &str, path: &str) -> ObservedResult<Self> {
        match text {
            "action/enabled" => Ok(Self::Enabled),
            "action/disabled" => Ok(Self::Disabled),
            _ => Err(value::enum_error(
                path,
                "`action/enabled | action/disabled`",
            )),
        }
    }
}

/// One action row with copied facts and its presentation marker.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ActionPlan {
    pub(crate) action: Action,
    pub(crate) marker: ActionMarker,
}

impl ActionPlan {
    /// Every copied action fact.
    #[must_use]
    pub fn copied_action(&self) -> &Action {
        &self.action
    }

    /// The compiled marker.
    #[must_use]
    pub const fn marker(&self) -> ActionMarker {
        self.marker
    }
}

/// The complete typed `nomos.observed_scene_plan@1` document.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ScenePlan {
    actions: Vec<ActionPlan>,
    actors: Vec<ActorPlan>,
    crop: Crop,
    scene: SceneIdentity,
    source_sha256: Sha256Digest,
    terrain_layers: Vec<TerrainPlan>,
}

impl ScenePlan {
    /// Strictly reads canonical plan bytes and rechecks all compiled mappings.
    pub fn from_bytes(bytes: &[u8]) -> ObservedResult<Self> {
        let value = json::parse(bytes)?;
        Self::from_canonical(&value)
    }

    /// Strictly reads an already-canonical plan value.
    pub fn from_canonical(document: &CanonicalValue) -> ObservedResult<Self> {
        let root = value::object(document, "$")?;
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
                "source_sha256",
                "terrain_layers",
            ],
            "$",
        )?;
        let source_sha256 = validate::fields(root)?;

        let stripped = stripped_input(root)?;
        let observed = ObservedScene::from_canonical(&stripped.raw)?;
        let terrain_layers = observed
            .terrain_layers
            .iter()
            .cloned()
            .zip(stripped.terrain_selections)
            .enumerate()
            .map(|(index, (layer, selected))| validate_terrain(index, layer, selected))
            .collect::<ObservedResult<Vec<_>>>()?;
        let actors = observed
            .actors
            .iter()
            .cloned()
            .zip(stripped.actor_selections)
            .enumerate()
            .map(|(index, (actor, selected))| validate_actor(index, actor, selected))
            .collect::<ObservedResult<Vec<_>>>()?;
        let actions = observed
            .actions
            .iter()
            .cloned()
            .zip(stripped.action_selections)
            .enumerate()
            .map(|(index, (action, marker))| validate_action(index, action, marker))
            .collect::<ObservedResult<Vec<_>>>()?;

        Ok(Self {
            actions,
            actors,
            crop: observed.crop,
            scene: observed.scene,
            source_sha256,
            terrain_layers,
        })
    }

    /// The canonical plan value.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "actions",
                CanonicalValue::Array(self.actions.iter().map(action_plan_value).collect()),
            ),
            (
                "actors",
                CanonicalValue::Array(self.actors.iter().map(actor_plan_value).collect()),
            ),
            ("crop", input::crop_value(self.crop)),
            (
                "scene",
                CanonicalValue::object_declared([(
                    "id",
                    CanonicalValue::text(self.scene.id.as_str()),
                )]),
            ),
            ("schema", CanonicalValue::text(SCHEMA)),
            (
                "source_sha256",
                CanonicalValue::text(self.source_sha256.to_hex()),
            ),
            (
                "terrain_layers",
                CanonicalValue::Array(self.terrain_layers.iter().map(terrain_plan_value).collect()),
            ),
        ])
    }

    /// Exact canonical plan bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    /// Ordered action rows.
    #[must_use]
    pub fn actions(&self) -> &[ActionPlan] {
        &self.actions
    }

    /// Ordered actor rows.
    #[must_use]
    pub fn actors(&self) -> &[ActorPlan] {
        &self.actors
    }

    /// Copied crop.
    #[must_use]
    pub const fn crop(&self) -> Crop {
        self.crop
    }

    /// Copied scene wrapper.
    #[must_use]
    pub fn scene(&self) -> &SceneIdentity {
        &self.scene
    }

    /// Digest of the exact canonical input bytes.
    #[must_use]
    pub const fn source_sha256(&self) -> Sha256Digest {
        self.source_sha256
    }

    /// Ordered terrain rows.
    #[must_use]
    pub fn terrain_layers(&self) -> &[TerrainPlan] {
        &self.terrain_layers
    }
}

/// Compiles exact canonical observed-scene bytes into the finite plan.
pub fn compile(source_bytes: &[u8]) -> ObservedResult<ScenePlan> {
    let observed = ObservedScene::from_bytes(source_bytes)?;
    Ok(compile_observed(
        observed,
        Sha256Digest::of_bytes(source_bytes),
    ))
}

fn compile_observed(observed: ObservedScene, source_sha256: Sha256Digest) -> ScenePlan {
    let terrain_layers = observed
        .terrain_layers
        .iter()
        .cloned()
        .map(compile_terrain)
        .collect();
    let actors = observed.actors.iter().cloned().map(compile_actor).collect();
    let actions = observed
        .actions
        .iter()
        .cloned()
        .map(compile_action)
        .collect();
    ScenePlan {
        actions,
        actors,
        crop: observed.crop,
        scene: observed.scene,
        source_sha256,
        terrain_layers,
    }
}

fn terrain_mapping(role: TerrainRole) -> (TerrainAssembly, MaterialFamily, i64) {
    match role {
        TerrainRole::CalmGround => (TerrainAssembly::CalmGround, MaterialFamily::GroundMuted, 0),
        TerrainRole::TraversableRoute => (
            TerrainAssembly::TraversableRoute,
            MaterialFamily::RouteWorn,
            10,
        ),
        TerrainRole::StructureFootprint => (
            TerrainAssembly::StructureFootprint,
            MaterialFamily::StructureStone,
            20,
        ),
    }
}

fn compile_terrain(layer: TerrainLayer) -> TerrainPlan {
    let (assembly, material_family, stack) = terrain_mapping(layer.role);
    TerrainPlan {
        assembly,
        cells: layer.cells,
        id: layer.id,
        material_family,
        role: layer.role,
        stack,
    }
}

fn compile_actor(actor: Actor) -> ActorPlan {
    let pose = match actor.life_state {
        LifeState::Living => ActorPose::UprightLiving,
        LifeState::Dead => ActorPose::ProneDead,
    };
    ActorPlan {
        assembly: ActorAssembly::ObservedFigure,
        controlled_marker: Presence::from_bool(actor.controlled),
        hostile_outline: Presence::from_bool(actor.hostile),
        pose,
        protection_ring: Presence::from_bool(actor.protected),
        actor,
    }
}

fn compile_action(action: Action) -> ActionPlan {
    let marker = match action.availability {
        Availability::Enabled => ActionMarker::Enabled,
        Availability::Disabled => ActionMarker::Disabled,
    };
    ActionPlan { action, marker }
}

#[derive(Clone, Copy)]
struct TerrainSelection {
    assembly: TerrainAssembly,
    material_family: MaterialFamily,
    stack: i64,
}

#[derive(Clone, Copy)]
struct ActorSelection {
    assembly: ActorAssembly,
    controlled_marker: Presence,
    hostile_outline: Presence,
    pose: ActorPose,
    protection_ring: Presence,
}

struct StrippedInput {
    raw: CanonicalValue,
    terrain_selections: Vec<TerrainSelection>,
    actor_selections: Vec<ActorSelection>,
    action_selections: Vec<ActionMarker>,
}

fn stripped_input(root: &value::Object) -> ObservedResult<StrippedInput> {
    let action_rows = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    let actor_rows = value::array(value::field(root, "actors", "$")?, "$.actors")?;
    let terrain_rows = value::array(
        value::field(root, "terrain_layers", "$")?,
        "$.terrain_layers",
    )?;

    let (raw_actions, action_selections) = strip_actions(action_rows)?;
    let (raw_actors, actor_selections) = strip_actors(actor_rows)?;
    let (raw_terrain, terrain_selections) = strip_terrain(terrain_rows)?;
    let raw = CanonicalValue::object_declared([
        ("actions", CanonicalValue::Array(raw_actions)),
        ("actors", CanonicalValue::Array(raw_actors)),
        ("crop", value::field(root, "crop", "$")?.clone()),
        ("scene", value::field(root, "scene", "$")?.clone()),
        ("schema", CanonicalValue::text(input::SCHEMA)),
        ("terrain_layers", CanonicalValue::Array(raw_terrain)),
    ]);
    Ok(StrippedInput {
        raw,
        terrain_selections,
        actor_selections,
        action_selections,
    })
}

fn strip_terrain(
    rows: &[CanonicalValue],
) -> ObservedResult<(Vec<CanonicalValue>, Vec<TerrainSelection>)> {
    let mut raw = Vec::with_capacity(rows.len());
    let mut selections = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(
            object,
            &[
                "assembly",
                "cells",
                "id",
                "material_family",
                "role",
                "stack",
            ],
            &path,
        )?;
        let assembly = TerrainAssembly::parse(
            value::text(
                value::field(object, "assembly", &path)?,
                &format!("{path}.assembly"),
            )?,
            &format!("{path}.assembly"),
        )?;
        let material_family = MaterialFamily::parse(
            value::text(
                value::field(object, "material_family", &path)?,
                &format!("{path}.material_family"),
            )?,
            &format!("{path}.material_family"),
        )?;
        let stack = value::integer(
            value::field(object, "stack", &path)?,
            &format!("{path}.stack"),
        )?;
        raw.push(CanonicalValue::object_declared([
            ("cells", value::field(object, "cells", &path)?.clone()),
            ("id", value::field(object, "id", &path)?.clone()),
            ("role", value::field(object, "role", &path)?.clone()),
        ]));
        selections.push(TerrainSelection {
            assembly,
            material_family,
            stack,
        });
    }
    Ok((raw, selections))
}

fn strip_actors(
    rows: &[CanonicalValue],
) -> ObservedResult<(Vec<CanonicalValue>, Vec<ActorSelection>)> {
    let mut raw = Vec::with_capacity(rows.len());
    let mut selections = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.actors[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(
            object,
            &[
                "assembly",
                "cell",
                "controlled",
                "controlled_marker",
                "hostile",
                "hostile_outline",
                "id",
                "life_state",
                "pose",
                "protected",
                "protection_ring",
            ],
            &path,
        )?;
        let selected = ActorSelection {
            assembly: ActorAssembly::parse(
                value::text(
                    value::field(object, "assembly", &path)?,
                    &format!("{path}.assembly"),
                )?,
                &format!("{path}.assembly"),
            )?,
            controlled_marker: Presence::parse(
                value::text(
                    value::field(object, "controlled_marker", &path)?,
                    &format!("{path}.controlled_marker"),
                )?,
                &format!("{path}.controlled_marker"),
            )?,
            hostile_outline: Presence::parse(
                value::text(
                    value::field(object, "hostile_outline", &path)?,
                    &format!("{path}.hostile_outline"),
                )?,
                &format!("{path}.hostile_outline"),
            )?,
            pose: ActorPose::parse(
                value::text(
                    value::field(object, "pose", &path)?,
                    &format!("{path}.pose"),
                )?,
                &format!("{path}.pose"),
            )?,
            protection_ring: Presence::parse(
                value::text(
                    value::field(object, "protection_ring", &path)?,
                    &format!("{path}.protection_ring"),
                )?,
                &format!("{path}.protection_ring"),
            )?,
        };
        raw.push(CanonicalValue::object_declared([
            ("cell", value::field(object, "cell", &path)?.clone()),
            (
                "controlled",
                value::field(object, "controlled", &path)?.clone(),
            ),
            ("hostile", value::field(object, "hostile", &path)?.clone()),
            ("id", value::field(object, "id", &path)?.clone()),
            (
                "life_state",
                value::field(object, "life_state", &path)?.clone(),
            ),
            (
                "protected",
                value::field(object, "protected", &path)?.clone(),
            ),
        ]));
        selections.push(selected);
    }
    Ok((raw, selections))
}

fn strip_actions(
    rows: &[CanonicalValue],
) -> ObservedResult<(Vec<CanonicalValue>, Vec<ActionMarker>)> {
    let mut raw = Vec::with_capacity(rows.len());
    let mut selections = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.actions[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(
            object,
            &["availability", "id", "marker", "target_actor"],
            &path,
        )?;
        let marker = ActionMarker::parse(
            value::text(
                value::field(object, "marker", &path)?,
                &format!("{path}.marker"),
            )?,
            &format!("{path}.marker"),
        )?;
        raw.push(CanonicalValue::object_declared([
            (
                "availability",
                value::field(object, "availability", &path)?.clone(),
            ),
            ("id", value::field(object, "id", &path)?.clone()),
            (
                "target_actor",
                value::field(object, "target_actor", &path)?.clone(),
            ),
        ]));
        selections.push(marker);
    }
    Ok((raw, selections))
}

fn mismatch(path: &str) -> ObservedError {
    ObservedError::new(
        codes::FIELD_INVALID,
        format!("compiled selection at `{path}` disagrees with its copied fact"),
    )
}

fn validate_terrain(
    index: usize,
    layer: TerrainLayer,
    selected: TerrainSelection,
) -> ObservedResult<TerrainPlan> {
    let expected = terrain_mapping(layer.role);
    if (selected.assembly, selected.material_family, selected.stack) != expected {
        return Err(mismatch(&format!("$.terrain_layers[{index}]")));
    }
    Ok(TerrainPlan {
        assembly: selected.assembly,
        cells: layer.cells,
        id: layer.id,
        material_family: selected.material_family,
        role: layer.role,
        stack: selected.stack,
    })
}

fn validate_actor(
    index: usize,
    actor: Actor,
    selected: ActorSelection,
) -> ObservedResult<ActorPlan> {
    let expected = compile_actor(actor.clone());
    if selected.assembly != expected.assembly
        || selected.controlled_marker != expected.controlled_marker
        || selected.hostile_outline != expected.hostile_outline
        || selected.pose != expected.pose
        || selected.protection_ring != expected.protection_ring
    {
        return Err(mismatch(&format!("$.actors[{index}]")));
    }
    Ok(ActorPlan {
        assembly: selected.assembly,
        actor,
        controlled_marker: selected.controlled_marker,
        hostile_outline: selected.hostile_outline,
        pose: selected.pose,
        protection_ring: selected.protection_ring,
    })
}

fn validate_action(
    index: usize,
    action: Action,
    marker: ActionMarker,
) -> ObservedResult<ActionPlan> {
    let expected = compile_action(action.clone());
    if marker != expected.marker {
        return Err(mismatch(&format!("$.actions[{index}].marker")));
    }
    Ok(ActionPlan { action, marker })
}

fn terrain_plan_value(layer: &TerrainPlan) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("assembly", CanonicalValue::text(layer.assembly.as_str())),
        (
            "cells",
            CanonicalValue::Array(
                layer
                    .cells
                    .iter()
                    .copied()
                    .map(input::terrain_cell_value)
                    .collect(),
            ),
        ),
        ("id", CanonicalValue::text(layer.id.as_str())),
        (
            "material_family",
            CanonicalValue::text(layer.material_family.as_str()),
        ),
        ("role", CanonicalValue::text(layer.role.as_str())),
        ("stack", CanonicalValue::Int(layer.stack)),
    ])
}

fn actor_plan_value(plan: &ActorPlan) -> CanonicalValue {
    let actor = &plan.actor;
    CanonicalValue::object_declared([
        ("assembly", CanonicalValue::text(plan.assembly.as_str())),
        ("cell", input::actor_cell_value(actor.cell)),
        ("controlled", CanonicalValue::Bool(actor.controlled)),
        (
            "controlled_marker",
            CanonicalValue::text(plan.controlled_marker.as_str()),
        ),
        ("hostile", CanonicalValue::Bool(actor.hostile)),
        (
            "hostile_outline",
            CanonicalValue::text(plan.hostile_outline.as_str()),
        ),
        ("id", CanonicalValue::text(actor.id.as_str())),
        (
            "life_state",
            CanonicalValue::text(actor.life_state.as_str()),
        ),
        ("pose", CanonicalValue::text(plan.pose.as_str())),
        ("protected", CanonicalValue::Bool(actor.protected)),
        (
            "protection_ring",
            CanonicalValue::text(plan.protection_ring.as_str()),
        ),
    ])
}

fn action_plan_value(plan: &ActionPlan) -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "availability",
            CanonicalValue::text(plan.action.availability.as_str()),
        ),
        ("id", CanonicalValue::text(plan.action.id.as_str())),
        ("marker", CanonicalValue::text(plan.marker.as_str())),
        (
            "target_actor",
            CanonicalValue::text(plan.action.target_actor.as_str()),
        ),
    ])
}
