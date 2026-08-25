// The renderer catalog: what content may name, but may not define.
//
// `docs/review/nomos-viewer.md` section 3 is the design. Everything here is
// renderer-owned data or a pure function over it. This module imports nothing,
// which is the structural half of the claim that it is the bottom of the app.
//
// Three things the ownership audit deferred to R1-4 land here:
//
//  * the kind-to-assembly and kind-to-material tables (audit section 3 items 5
//    and 6). The split is the one the owner ruled for R1-3: the catalog defines
//    what an assembly name and a material family *mean*, and the compiler
//    selects one per entity kind, exactly as an area selects one architecture
//    assembly. `plan.mjs` refuses any name this file does not declare, so a
//    name legal to the compiler but unknown here fails the decode rather than a
//    frame. Issue #153 carries moving the selection itself out of Rust.
//  * the two colour tables (audit section 2 item 9). One viewer, one palette,
//    and the page chrome reads it too, so there is no third colour source.
//  * the prose `displayName()` invented from identifiers (audit section 3 item
//    26). There is no table of authored names here, because a renderer that
//    knew an entity's name would be a renderer that needs editing to accept new
//    content. Prose exists only for closed sets the accepted schema declares.

// ---------------------------------------------------------------------------
// Units and scales
// ---------------------------------------------------------------------------

// One vertical step is a tenth of a lattice cell. `nomos.presentation_source@1`
// declares wall and mass heights as integer counts of this unit.
export const VERTICAL_STEPS_PER_CELL = 10;

// A declared step count in lattice cells.
//
// The conversion is a division, not a multiplication by 0.1, and the difference
// is load-bearing: IEEE-754 division is correctly rounded, so `n / 10` is the
// nearest double to the real n/10 — the same double the decimal literal it
// replaced denoted. `48 * 0.1 === 4.800000000000001`, and writing it that way
// moved every Ossuary Reach frame when R1-3 first did it.
export const cellsOf = (steps) => steps / VERTICAL_STEPS_PER_CELL;

// One lattice cell horizontally is one world unit; vertically it is shorter, so
// a 4.5-cell wall reads at the height the door and brazier assemblies were
// modelled against.
export const CELL_WORLD_UNITS = 1;
export const VERTICAL_SCALE = 0.72;

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

export const CAMERA = Object.freeze({
  orthoHalfHeight: 3.7,
  offset: Object.freeze({ x: 0.86, y: 0.92, z: 1.08 }),
  targetHeight: 0.5,
  near: 0.1,
  far: 80,
  maxPixelRatio: 2,
  shadowMapSize: 2048,
  shadowFrustum: 8,
  lightShadowMapSize: 512,
});

// ---------------------------------------------------------------------------
// The palette
// ---------------------------------------------------------------------------

// One table. The WebGL renderer takes the integers; the page takes the same
// integers as `#rrggbb` custom properties, so no stylesheet holds a colour of
// its own. Where the study's two renderers disagreed on a role, the WebGL value
// is the one that survives, because the WebGL renderer is the one that is
// promoted.
export const PALETTE = Object.freeze({
  void: 0x090e13,
  fog: 0x111b24,
  stone_0: 0x202b34,
  stone_1: 0x2d3a43,
  stone_2: 0x3d4b52,
  edge: 0x536168,
  mortar: 0x111920,
  iron: 0x111a20,
  rust: 0x70412f,
  water: 0x244857,
  water_deep: 0x173744,
  water_light: 0x4d8290,
  water_high: 0x70909a,
  cyan: 0x83eeea,
  cyan_dim: 0x4d9f9f,
  cyan_bright: 0xbfffff,
  amber: 0xffa544,
  amber_dim: 0x87552e,
  player: 0x347b7d,
  gaoler: 0x8c5638,
  skin: 0x96735b,
  sky: 0x8aa8b8,
  ground: 0x131c24,
  moon: 0xabc8d7,
  grid: 0x30434a,
  surface: 0x0a1016,
  surface_raised: 0x101b22,
  surface_sunk: 0x091117,
  surface_button: 0x202d35,
  border: 0x33424a,
  border_strong: 0x4c6068,
  text: 0xd7ddd9,
  text_muted: 0x8e9b9c,
  text_dim: 0x6f8288,
  prompt: 0x9baaad,
  danger: 0xd47158,
});

// The one place a colour becomes a string.
export const hex = (value) => `#${value.toString(16).padStart(6, "0")}`;

// ---------------------------------------------------------------------------
// Look profiles
// ---------------------------------------------------------------------------

export const LOOK_PROFILE_IDS = Object.freeze(["baseline", "procedural"]);
export const DEFAULT_LOOK_PROFILE = "procedural";

export const LOOK_PROFILES = Object.freeze({
  baseline: Object.freeze({
    id: "gaol_baseline_01",
    fogDensity: 0.045,
    exposure: 1.28,
    bevel: 0,
    actorOutline: 0,
    materials: Object.freeze({}),
  }),
  procedural: Object.freeze({
    id: "gaol_procedural_01",
    fogDensity: 0.041,
    exposure: 1.34,
    bevel: 0.055,
    actorOutline: 1.065,
    materials: Object.freeze({
      stone: Object.freeze({ scale: 1.35, variation: 0.13, accent: PALETTE.edge, accentMix: 0.16 }),
      iron: Object.freeze({ scale: 2.1, variation: 0.08, accent: PALETTE.rust, accentMix: 0.22 }),
      cloth: Object.freeze({ scale: 2.8, variation: 0.09, accent: PALETTE.mortar, accentMix: 0.08 }),
    }),
  }),
});

// ---------------------------------------------------------------------------
// Closed sets: the catalog defines, the presentation source selects
// ---------------------------------------------------------------------------

export const ARCHITECTURE_ASSEMBLIES = Object.freeze(["visual/beveled_masonry"]);
export const MATERIAL_FAMILIES = Object.freeze(["stone_bounded"]);
export const TRIM_FAMILIES = Object.freeze(["broad_mortar"]);
export const ACTOR_ASSEMBLIES = Object.freeze([
  "visual/gaoler_silhouette",
  "visual/player_silhouette",
]);
export const EFFECT_ASSEMBLIES = Object.freeze(["visual/cyan_crescent"]);

// The closed set of declared actor roles. `nomos.rendering_plan@3` carries
// `actors[].role`, which retires the ownership audit's items 7 and 21: an
// actor's identity string was the only role signal, and `player` and `gaoler`
// were magic names. The runtime reads the role to decide which actor a command
// moves and which one the pursuit rule steps; this list is what the decoder
// checks a plan's value against.
export const ACTOR_ROLES = Object.freeze(["player", "pursuer"]);

// What an actor assembly is shaped like. The study told the two silhouettes
// apart with `actor.id === "player"`, which audit section 3 item 21 recorded as
// the only role signal in the content model; the assembly is a declared field,
// so the renderer dispatches on it and holds no actor identifier at all. R1-5
// added the declared role beside it, so the *runtime* has a typed answer too
// and neither side reads an identity string.
export const ACTOR_SHAPES = Object.freeze({
  "visual/player_silhouette": Object.freeze({
    body: "player",
    cloakRadius: 0.25,
    cloakHeight: 0.78,
    cloakY: 0.43,
    headRadius: 0.15,
    headY: 0.92,
    hand: "blade",
  }),
  "visual/gaoler_silhouette": Object.freeze({
    body: "gaoler",
    cloakRadius: 0.32,
    cloakHeight: 0.9,
    cloakY: 0.48,
    headRadius: 0.17,
    headY: 1.04,
    hand: "shoulder",
  }),
});

// The entity vocabulary, and the only place in the tree that says what a
// compiled kind is drawn as and what it is made of.
//
// `nomos.rendering_plan@2` carried `visual_assembly` and `material_family` on
// every entity, assigned per kind by a table in
// `crates/nomos-render-plan/src/catalog.rs` whose own comment said the correct
// change was to move it out. `@3` drops both fields and issue #153 is that
// move, so there is no second table to drift against and the test that parsed
// the Rust one to compare them is gone with it.
//
// `anchorKind` is the binding shape a kind must declare and is checked: a door
// is bound to a face, water to a region, a light to a cell. An entity whose
// kind is not here is refused rather than drawn as a marker, which is the
// ownership audit's item 4.
export const ENTITY_KINDS = Object.freeze({
  door: Object.freeze({
    visualAssembly: "visual/iron_barred_door",
    materialFamily: "iron_oxidized",
    anchorKind: "face",
    sockets: Object.freeze(["ward"]),
    machines: Object.freeze(["access", "integrity", "ward"]),
  }),
  water: Object.freeze({
    visualAssembly: "visual/shallow_water",
    materialFamily: "water_cold",
    anchorKind: "region",
    sockets: Object.freeze([]),
    machines: Object.freeze([]),
  }),
  light: Object.freeze({
    visualAssembly: "visual/brazier",
    materialFamily: "iron_brazier",
    anchorKind: "cell",
    sockets: Object.freeze([]),
    machines: Object.freeze(["emission"]),
  }),
});

export const ENTITY_KIND_IDS = Object.freeze(Object.keys(ENTITY_KINDS));
export const DISPOSITIONS = Object.freeze(["blocked", "traversable"]);
export const DIRECTIONS = Object.freeze(["east", "north", "south", "west"]);
export const ANCHOR_KINDS = Object.freeze(["cell", "face", "region"]);
export const OBJECTIVE_KINDS = Object.freeze(["exit_via"]);

// ---------------------------------------------------------------------------
// Sockets
// ---------------------------------------------------------------------------

// A socket is a named attachment point on a visual assembly, in the entity's own
// face frame: x runs along the face, y runs inward from it, z is up, all in
// vertical steps, so the table holds integers only.
//
// `ward` on a `door` is {5, 0, 17}: half a cell along the
// door's axis, on the face plane, 1.7 cells up — where both study renderers
// already drew the ward mark, the WebGL ring sitting at y = 1.22 against
// (17 / 10) * 0.72 = 1.224.
export const SOCKETS = Object.freeze({
  door: Object.freeze({
    ward: Object.freeze({ x: 5, y: 0, z: 17 }),
  }),
});

// Resolves a socket to lattice cells, honouring the entity's declared face.
//
// R1-3 left the offset anchored to north because every corpus door faces north
// and a fixed offset could not say otherwise; `docs/review/presentation-source.md`
// section 3.3 deferred direction-aware resolution here. The north row is
// arithmetically identical to what the study computed.
export function resolveSocket(entity, socket) {
  // Keyed by compiled kind. `nomos.rendering_plan@3` no longer carries an
  // assembly name per entity — this catalog owns the kind-to-assembly mapping
  // now (issue #153) — so the socket table is keyed by the thing the plan does
  // carry.
  const table = SOCKETS[entity.kind];
  const offset = table?.[socket];
  if (!offset) return null;
  const cell = entity.anchor.cell;
  if (!cell) return null;
  const along = cellsOf(offset.x);
  const inward = cellsOf(offset.y);
  const up = cellsOf(offset.z);
  switch (entity.anchor.direction) {
    case "north":
      return { x: cell.x + along, y: cell.y + inward, z: up };
    case "south":
      return { x: cell.x + along, y: cell.y + 1 - inward, z: up };
    case "west":
      return { x: cell.x + inward, y: cell.y + along, z: up };
    case "east":
      return { x: cell.x + 1 - inward, y: cell.y + along, z: up };
    default:
      return null;
  }
}

// The unit lattice step a move in this direction takes, which is also the way
// out through a door bound to that face.
export const DIRECTION_DELTAS = Object.freeze({
  north: Object.freeze({ dx: 0, dy: -1 }),
  south: Object.freeze({ dx: 0, dy: 1 }),
  west: Object.freeze({ dx: -1, dy: 0 }),
  east: Object.freeze({ dx: 1, dy: 0 }),
});

export const directionOf = (dx, dy) =>
  Object.keys(DIRECTION_DELTAS).find(
    (name) => DIRECTION_DELTAS[name].dx === dx && DIRECTION_DELTAS[name].dy === dy,
  ) ?? null;

// ---------------------------------------------------------------------------
// Prose
// ---------------------------------------------------------------------------

// Authored prose for closed sets the accepted schema declares, and nothing else.
// An identifier is never re-cased into a name: `north_gate` is shown as
// `north_gate`, in the identifier style, because the only authored prose in the
// content model is `area.label`.
export const KIND_LABELS = Object.freeze({
  door: "door",
  water: "water",
  light: "light",
});

export const DISPOSITION_LABELS = Object.freeze({
  traversable: "open",
  blocked: "blocked",
});
