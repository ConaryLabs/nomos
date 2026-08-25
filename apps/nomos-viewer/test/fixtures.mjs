// Hand-authored artifacts for the tests.
//
// Written fresh in this file's own vocabulary. Nothing here is copied from
// `experiments/`, and no test reads the study's committed fixtures: RUNTIME.md
// section 2 makes the study a specification and a comparison target, not a
// source of truth, and a test that read its files would quietly make it one.
//
// The hall is deliberately not any of the four corpus areas. It is 4x3, its
// gate is at x = 1, it has a pillar where a walk would like to go, and its
// water sits between the player and everything else, so the movement rules are
// exercised rather than described.

import { createHash } from "node:crypto";

const hash = (seed) => seed.repeat(64).slice(0, 64);

export const HASHES = Object.freeze({
  sealed: hash("a1"),
  unsealed: hash("b2"),
  dark: hash("c3"),
  yardSealed: hash("d4"),
  yardOpen: hash("e5"),
});

const door = (id, x, direction = "north") => ({
  anchor: { cell: { x, y: 0, z: 0 }, direction, kind: "face" },
  id,
  kind: "door",
  machine_namespaces: [`${id}.access`, `${id}.integrity`, `${id}.ward`],
  material_family: "iron_oxidized",
  provenance: [],
  visual_assembly: "visual/iron_barred_door",
});

const water = (id, min, max) => ({
  anchor: { kind: "region", max: { ...max, z: 0 }, min: { ...min, z: 0 } },
  id,
  kind: "water",
  machine_namespaces: [],
  material_family: "water_cold",
  provenance: [
    {
      claim: `${id}.region#traversal_cost_ground`,
      source: {
        byte_end: 200,
        byte_start: 120,
        column: 1,
        line: 7,
        path: "experiments/executable-gaol/areas/test-hall/world.nomos",
      },
    },
  ],
  visual_assembly: "visual/shallow_water",
});

const lamp = (id, x, y) => ({
  anchor: { cell: { x, y, z: 0 }, kind: "cell" },
  id,
  kind: "light",
  machine_namespaces: [`${id}.emission`],
  material_family: "iron_brazier",
  provenance: [],
  visual_assembly: "visual/brazier",
});

const doorMachines = (id, { access = "locked", integrity = "intact", ward = "sealed" } = {}) => [
  { namespace: `${id}.access`, state: access },
  { namespace: `${id}.integrity`, state: integrity },
  { namespace: `${id}.ward`, state: ward },
];

const PROJECTIONS = [
  { name: "nomos.projection.simulation", version: 3 },
  { name: "nomos.projection.navigation", version: 1 },
  { name: "nomos.projection.persistence", version: 1 },
  { name: "nomos.projection.diagnostics", version: 1 },
];

const digests = PROJECTIONS.map((one, index) => ({
  digest: hash(String(index + 1)),
  file: `${one.name.split(".").pop()}.json`,
}));

/// The start area: `test-hall`.
export function hallPlan() {
  const scenario = (id, label, tick, stateHash, { gateOpen, lit }) => ({
    effective_light: [{ emitting: lit, entity: "hall_lamp" }],
    id,
    label,
    machine_states: [
      ...doorMachines("hall_gate", gateOpen ? { ward: "unsealed", integrity: "destroyed" } : {}),
      { namespace: "hall_lamp.emission", state: lit ? "lit" : "extinguished" },
      ...doorMachines("side_gate"),
    ].sort((left, right) => left.namespace.localeCompare(right.namespace)),
    movement: [
      {
        cost: gateOpen ? 1 : null,
        disposition: gateOpen ? "traversable" : "blocked",
        entity: "hall_gate",
        reasons: gateOpen ? [] : ["hall_gate.ward#blocks_ground"],
      },
      { cost: 4, disposition: "traversable", entity: "hall_pool", reasons: ["hall_pool.region#traversal_cost_ground"] },
      { cost: null, disposition: "blocked", entity: "side_gate", reasons: ["side_gate.ward#blocks_ground"] },
    ],
    state_hash: stateHash,
    tick,
  });

  return {
    actors: [
      { assembly: "visual/player_silhouette", cell: { x: 0, y: 2, z: 0 }, id: "player" },
      { assembly: "visual/gaoler_silhouette", cell: { x: 3, y: 2, z: 0 }, id: "gaoler" },
    ],
    architecture: {
      bounds: { height: 3, width: 4 },
      masses: [{ height_steps: 20, id: "pillar", max: { x: 4, y: 1 }, min: { x: 3, y: 0 } }],
      style: {
        assembly: "visual/beveled_masonry",
        material_family: "stone_bounded",
        trim_family: "broad_mortar",
      },
      wall_height_steps: 30,
    },
    area: { id: "test-hall", label: "Test Hall", start: true },
    effects: [
      { anchor: { entity: "hall_gate", socket: "ward" }, assembly: "visual/cyan_crescent", id: "ward_mark" },
    ],
    entities: [door("hall_gate", 1), lamp("hall_lamp", 2, 2), water("hall_pool", { x: 0, y: 1 }, { x: 1, y: 1 }), door("side_gate", 2)],
    interactions: [
      {
        action: "unseal",
        from_scenario: "01-sealed",
        id: "01-sealed:unseal:hall_gate",
        input_state_hash: HASHES.sealed,
        resulting_state_hash: HASHES.unsealed,
        target_entity: "hall_gate",
        to_scenario: "02-unsealed",
      },
      {
        action: "extinguish",
        from_scenario: "02-unsealed",
        id: "02-unsealed:extinguish:hall_lamp",
        input_state_hash: HASHES.unsealed,
        resulting_state_hash: HASHES.dark,
        target_entity: "hall_lamp",
        to_scenario: "03-dark",
      },
    ],
    objective: { gate: "hall_gate", kind: "exit_via" },
    projection_digests: digests,
    projection_schemas: PROJECTIONS,
    pursuit: { light: "hall_lamp" },
    route: { to_area: "test-yard" },
    scenarios: [
      scenario("01-sealed", "sealed", 0, HASHES.sealed, { gateOpen: false, lit: true }),
      scenario("02-unsealed", "unsealed", 1, HASHES.unsealed, { gateOpen: true, lit: true }),
      scenario("03-dark", "dark", 2, HASHES.dark, { gateOpen: true, lit: false }),
    ],
    schema: "nomos.rendering_plan@2",
  };
}

/// The terminal area: `test-yard`, which declares its own arrival cell.
export function yardPlan() {
  const scenario = (id, label, tick, stateHash, gateOpen) => ({
    effective_light: [{ emitting: true, entity: "yard_lamp" }],
    id,
    label,
    machine_states: [
      { namespace: "yard_gate.access", state: gateOpen ? "open" : "locked" },
      { namespace: "yard_gate.integrity", state: "intact" },
      { namespace: "yard_gate.ward", state: gateOpen ? "unsealed" : "sealed" },
      { namespace: "yard_lamp.emission", state: "lit" },
    ],
    movement: [
      {
        cost: gateOpen ? 1 : null,
        disposition: gateOpen ? "traversable" : "blocked",
        entity: "yard_gate",
        reasons: gateOpen ? [] : ["yard_gate.ward#blocks_ground"],
      },
    ],
    state_hash: stateHash,
    tick,
  });

  return {
    actors: [
      { assembly: "visual/player_silhouette", cell: { x: 1, y: 1, z: 0 }, id: "player" },
      { assembly: "visual/gaoler_silhouette", cell: { x: 0, y: 0, z: 0 }, id: "gaoler" },
    ],
    architecture: {
      bounds: { height: 2, width: 3 },
      masses: [],
      style: {
        assembly: "visual/beveled_masonry",
        material_family: "stone_bounded",
        trim_family: "broad_mortar",
      },
      wall_height_steps: 25,
    },
    area: { id: "test-yard", label: "Test Yard", start: false },
    effects: [],
    entities: [door("yard_gate", 2), lamp("yard_lamp", 0, 1)],
    interactions: [
      {
        action: "unseal",
        from_scenario: "01-shut",
        id: "01-shut:unseal:yard_gate",
        input_state_hash: HASHES.yardSealed,
        resulting_state_hash: HASHES.yardOpen,
        target_entity: "yard_gate",
        to_scenario: "02-open",
      },
    ],
    objective: { gate: "yard_gate", kind: "exit_via" },
    projection_digests: digests,
    projection_schemas: PROJECTIONS,
    pursuit: { light: "yard_lamp" },
    route: { entry: { x: 1, y: 1, z: 0 }, to_area: null },
    scenarios: [
      scenario("01-shut", "shut", 0, HASHES.yardSealed, false),
      scenario("02-open", "open", 1, HASHES.yardOpen, true),
    ],
    schema: "nomos.rendering_plan@2",
  };
}

/// The bytes a plan is published as: canonical bytes plus one `LF`, which is
/// what `nomos-render-plan` writes and what `publish()` in `scan.test.mjs`
/// stages. The collection's digest is over exactly these, so the staged build
/// checks a digest that is true rather than one invented for the fixture.
export const publishedPlanBytes = (plan) => `${JSON.stringify(plan)}\n`;

const planDigest = (plan) => createHash("sha256").update(publishedPlanBytes(plan)).digest("hex");

/// The two-area collection over the two plans above.
export function collectionDocument() {
  return {
    areas: [
      {
        entry: null,
        exit: { gate: "hall_gate", to_area: "test-yard" },
        id: "test-hall",
        label: "Test Hall",
        plan: { file: "test-hall.json", sha256: planDigest(hallPlan()) },
        start: true,
      },
      {
        entry: { x: 1, y: 1, z: 0 },
        exit: { gate: "yard_gate", to_area: null },
        id: "test-yard",
        label: "Test Yard",
        plan: { file: "test-yard.json", sha256: planDigest(yardPlan()) },
        start: false,
      },
    ],
    route: [
      { entry: { x: 1, y: 1, z: 0 }, from_area: "test-hall", gate: "hall_gate", to_area: "test-yard" },
      { entry: null, from_area: "test-yard", gate: "yard_gate", to_area: null },
    ],
    schema: "nomos.area_collection@1",
    start_area: "test-hall",
    visual_grammar: {
      actor_assemblies: ["visual/gaoler_silhouette", "visual/player_silhouette"],
      architecture_style: {
        assembly: "visual/beveled_masonry",
        material_family: "stone_bounded",
        trim_family: "broad_mortar",
      },
      digest: hash("f6"),
      effect_assemblies: ["visual/cyan_crescent"],
      entity_assemblies: [
        { kind: "door", material_family: "iron_oxidized", visual_assembly: "visual/iron_barred_door" },
        { kind: "light", material_family: "iron_brazier", visual_assembly: "visual/brazier" },
        { kind: "water", material_family: "water_cold", visual_assembly: "visual/shallow_water" },
      ],
      projection_schemas: PROJECTIONS,
      rendering_plan_schema: "nomos.rendering_plan@2",
    },
  };
}

/// A plan whose one door faces `direction`, on the boundary cell that face
/// belongs to, for the socket-resolution and exit-direction rows. The yard is
/// three cells wide and two deep, so each face has exactly one such cell here.
export const FACING_CELLS = Object.freeze({
  north: Object.freeze({ x: 1, y: 0, z: 0 }),
  south: Object.freeze({ x: 1, y: 1, z: 0 }),
  west: Object.freeze({ x: 0, y: 0, z: 0 }),
  east: Object.freeze({ x: 2, y: 0, z: 0 }),
});

export function facingPlan(direction) {
  const plan = yardPlan();
  const gate = plan.entities.find((one) => one.id === "yard_gate");
  gate.anchor = { cell: { ...FACING_CELLS[direction] }, direction, kind: "face" };
  plan.effects = [
    { anchor: { entity: "yard_gate", socket: "ward" }, assembly: "visual/cyan_crescent", id: "yard_mark" },
  ];
  return plan;
}

/// `structuredClone` with a path edit, for the refusal tests.
export function edited(document, path, value) {
  const copy = structuredClone(document);
  const keys = path.split(".");
  const last = keys.pop();
  let cursor = copy;
  for (const key of keys) cursor = cursor[/^\d+$/.test(key) ? Number(key) : key];
  if (value === undefined) delete cursor[last];
  else cursor[/^\d+$/.test(last) ? Number(last) : last] = value;
  return copy;
}

/// A `fetch` over an in-memory tree, for `loadArtifacts`.
export function fetchFrom(files) {
  return async (url) => {
    const path = new URL(url).pathname.replace(/^\/+/, "");
    if (!(path in files)) return { ok: false, status: 404, text: async () => "" };
    return { ok: true, status: 200, text: async () => JSON.stringify(files[path]) };
  };
}

export const stagedFiles = () => ({
  "areas.json": collectionDocument(),
  "areas/test-hall.json": hallPlan(),
  "areas/test-yard.json": yardPlan(),
});
