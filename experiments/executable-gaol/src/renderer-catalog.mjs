// The renderer catalog, and the plan accessors more than one consumer needs.
//
// `docs/review/presentation-source.md` section 3 is the design. Two kinds of
// thing live here, and nothing else:
//
// 1. Renderer-catalog data — facts about how content is drawn, which content
//    may *select* from but may not *define*. `nomos.presentation_source@2`
//    checks that an assembly, family, or socket name is well formed; this file
//    is where the legal values are, and `verify.mjs` checks each compiled plan
//    against them. That is the definition/selection split the owner ruled for
//    the audit's rows 14, 15, 16, 21, and 25, and it is the same split the
//    socket table already used.
//
// 2. Plan accessors the SVG evidence renderer and `verify.mjs` use. Each one
//    replaces a derivation the ownership audit found duplicated: the
//    machine-state lookup implemented twice with its own fallbacks, and the
//    ward test written out four times.
//
// The WebGL renderer, the play state, and the playable viewer left this tree
// with R1-4; `apps/nomos-viewer/src/catalog.mjs` is the accepted catalog, and it
// was written fresh rather than moved. What is left here is what the study's own
// SVG capture path still needs. `LOOK_PROFILE_IDS` and `isHunting` went with the
// files that used them.
//
// Per-renderer constants are deliberately NOT here. `render-core.mjs` owns the
// SVG camera and its pixels-per-cell; `webgl-renderer.mjs` owns its
// orthographic frustum and its world-units-per-cell. Two renderers projecting
// two ways is not a double authority; one content field meaning two undeclared
// things was, and that is what the shared step unit below fixes.

// ---------------------------------------------------------------------------
// Units
// ---------------------------------------------------------------------------

// One vertical step is a tenth of a lattice cell. `nomos.presentation_source@2`
// declares wall and mass heights as integer counts of this unit, and each
// renderer converts to its own space: multiply by CELL_HEIGHT_PIXELS in the SVG
// renderer, by VERTICAL_SCALE in the WebGL one.
//
// The conversion is `steps / VERTICAL_STEPS_PER_CELL` — a division, not a
// multiplication by 0.1 — and that distinction is load-bearing. IEEE-754
// division is correctly rounded, so `n / 10` is the nearest double to the real
// n/10, which is the same double the decimal literal it replaces denoted:
// `48 / 10 === 4.8`, `7 / 10 === 0.7`, `24 / 10 === 2.4`. Multiplying by the
// nearest double to 0.1 is a different operation and gives a different answer
// for exactly those three — `48 * 0.1 === 4.800000000000001` — which moved
// every Ossuary Reach frame the first time this was written that way.
// `area-collection.test.mjs` pins all ten values.
export const VERTICAL_STEPS_PER_CELL = 10;

// Converts a declared step count to lattice cells.
export const cellsOf = (steps) => steps / VERTICAL_STEPS_PER_CELL;

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

// Compiled entity kind to the assembly that draws it and the material family it
// is drawn in. `nomos.rendering_plan@2` carried these two strings on every
// entity, assigned by a table in `crates/nomos-render-plan/src/catalog.rs`
// whose own comment said the correct change was to move it out. `@3` drops both
// fields and issue #153 is that move: the mapping is renderer-catalog data and
// this is the renderer catalog.
export const ENTITY_ASSEMBLIES = Object.freeze({
  door: Object.freeze({ assembly: "visual/iron_barred_door", material_family: "iron_oxidized" }),
  light: Object.freeze({ assembly: "visual/brazier", material_family: "iron_brazier" }),
  water: Object.freeze({ assembly: "visual/shallow_water", material_family: "water_cold" }),
});

// Fail closed: a kind the catalog has no assembly for is a build failure, not a
// silently unmarked entity.
export function assemblyOf(kind) {
  const row = ENTITY_ASSEMBLIES[kind];
  if (!row) throw new Error(`the renderer catalog knows no assembly for kind ${kind}`);
  return row;
}

// ---------------------------------------------------------------------------
// Sockets
// ---------------------------------------------------------------------------

// A socket is a named attachment point on a visual assembly. Its offset is
// measured from the origin corner of the anchor entity's lattice cell, in
// vertical_step units (tenths of a cell) on all three axes, so the catalog
// holds integers only and each renderer applies its own conversion.
//
// `ward` on `visual/iron_barred_door` is {5, 0, 17}: half a cell along the
// door's own axis, on the wall plane its `anchor.direction` names, and 1.7
// cells up. Both renderers already draw the door's ward mark at that point —
// the WebGL ward ring sits at y = 1.22 and (17 / 10) * 0.72 = 1.224, and in the
// SVG the socket resolves to the door glyph's own screen column, (17 / 10) * 38 =
// 64.6 px above its anchor, which sets the crescent into the head of the gate's
// arch. Content names the socket; this table is the only place that says where
// the socket is.
//
// A non-north door would want the offset rotated by the entity's declared
// `anchor.direction`. Every door in the corpus faces north, so this table is a
// fixed offset; the promoted catalog resolves the same offset through the
// declared face, which is what RUNTIME.md section 5 R1-4 deferred here.
export const SOCKETS = Object.freeze({
  door: Object.freeze({
    ward: Object.freeze({ x: 5, y: 0, z: 17 }),
  }),
});

// Resolves a socket to lattice-space coordinates, or throws.
//
// Fail closed: a socket the renderer has no offset for is a build failure, not
// a silently unplaced glyph.
export function socketPosition(entity, socket) {
  const table = SOCKETS[entity.kind];
  const offset = table?.[socket];
  if (!offset) {
    throw new Error(`no socket ${socket} on kind ${entity.kind} for entity ${entity.id}`);
  }
  const cell = entity.anchor.cell;
  if (!cell) throw new Error(`entity ${entity.id} has no anchor cell to socket against`);
  return {
    x: cell.x + cellsOf(offset.x),
    y: cell.y + cellsOf(offset.y),
    z: cellsOf(offset.z),
  };
}

// ---------------------------------------------------------------------------
// Plan accessors
// ---------------------------------------------------------------------------

// `nomos.rendering_plan@3` spells its stable-ID collections as arrays of
// `{entity, ...}` or `{namespace, ...}` rows rather than as objects keyed by
// data, so every lookup goes through one of these.

export const machineState = (scenario, entity, machine) => {
  const namespace = `${entity}.${machine}`;
  const row = scenario.machine_states.find((entry) => entry.namespace === namespace);
  // No fallback. The audit found four independent re-derivations of this
  // lookup, each inventing its own default, so an absent machine silently drew
  // a sealed ward. It is now a build failure.
  if (!row) throw new Error(`scenario ${scenario.id} carries no machine ${namespace}`);
  return row.state;
};

export const doorState = (scenario, entity) => ({
  access: machineState(scenario, entity, "access"),
  integrity: machineState(scenario, entity, "integrity"),
  ward: machineState(scenario, entity, "ward"),
});

export const wardSealed = (scenario, entity) => machineState(scenario, entity, "ward") === "sealed";

export const movementOf = (scenario, entity) =>
  scenario.movement.find((row) => row.entity === entity) ?? null;

export const lightOf = (scenario, entity) =>
  scenario.effective_light.find((row) => row.entity === entity)?.emitting ?? null;

