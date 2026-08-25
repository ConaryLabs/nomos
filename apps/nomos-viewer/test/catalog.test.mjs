// The catalog is the one place the renderer's own facts live.

import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";

import { stripComments } from "../build.mjs";
import {
  ACTOR_ASSEMBLIES,
  ACTOR_ROLES,
  ARCHITECTURE_ASSEMBLIES,
  CAMERA,
  DISPOSITION_LABELS,
  EFFECT_ASSEMBLIES,
  ENTITY_KINDS,
  KIND_LABELS,
  LOOK_PROFILES,
  LOOK_PROFILE_IDS,
  MATERIAL_FAMILIES,
  PALETTE,
  SOCKETS,
  TRIM_FAMILIES,
  VERTICAL_SCALE,
  VERTICAL_STEPS_PER_CELL,
  cellsOf,
  directionOf,
  hex,
  resolveSocket,
} from "../src/catalog.mjs";

const sourceOf = (name) => readFileSync(new URL(`../src/${name}`, import.meta.url), "utf8");
// Rules about code are checked against code. The catalog's own prose explains
// what it replaced, and quotes the identifiers it no longer invents.
const codeOf = (name) => stripComments(sourceOf(name));
const appSources = readdirSync(new URL("../src/", import.meta.url)).filter((one) => one.endsWith(".mjs"));

test("steps convert by division", () => {
  assert.equal(VERTICAL_STEPS_PER_CELL, 10);
  // All ten values the corpus carries, pinned rather than asserted as a
  // property. `steps * (1 / 10)` passes a property test and moves three of
  // these; it moved every Ossuary Reach frame when R1-3 first wrote it.
  assert.equal(cellsOf(45), 4.5);
  assert.equal(cellsOf(50), 5);
  assert.equal(cellsOf(48), 4.8);
  assert.equal(cellsOf(26), 2.6);
  assert.equal(cellsOf(32), 3.2);
  assert.equal(cellsOf(7), 0.7);
  assert.equal(cellsOf(24), 2.4);
  assert.equal(cellsOf(20), 2);
  assert.equal(cellsOf(30), 3);
  assert.equal(cellsOf(25), 2.5);
  // And the three the other spelling gets wrong.
  assert.notEqual(48 * 0.1, cellsOf(48));
  assert.notEqual(7 * 0.1, cellsOf(7));
  assert.notEqual(24 * 0.1, cellsOf(24));
  assert.match(sourceOf("catalog.mjs"), /steps \/ VERTICAL_STEPS_PER_CELL/);
});

test("the ward socket offset is five zero seventeen", () => {
  // Keyed by the compiled kind, which is the only thing `nomos.rendering_plan@3`
  // says about what an entity is: the assembly name it used to carry alongside
  // is the catalog's own business now.
  assert.deepEqual(SOCKETS.door.ward, { x: 5, y: 0, z: 17 });
  // The WebGL ward ring sits at y = 1.22, and this is where the socket lands.
  assert.ok(Math.abs(cellsOf(17) * VERTICAL_SCALE - 1.22) < 0.005);
});

test("a socket resolves by the declared direction", () => {
  const gate = (direction) => ({
    id: "gate",
    kind: "door",
    anchor: { kind: "face", cell: { x: 3, y: 2, z: 0 }, direction },
  });
  // North is what the study computed: cell + (0.5, 0, 1.7). R1-3 deferred the
  // other three faces here because a fixed offset could not express them.
  assert.deepEqual(resolveSocket(gate("north"), "ward"), { x: 3.5, y: 2, z: 1.7 });
  assert.deepEqual(resolveSocket(gate("south"), "ward"), { x: 3.5, y: 3, z: 1.7 });
  assert.deepEqual(resolveSocket(gate("west"), "ward"), { x: 3, y: 2.5, z: 1.7 });
  assert.deepEqual(resolveSocket(gate("east"), "ward"), { x: 4, y: 2.5, z: 1.7 });
  // Fail closed, never a silently unplaced glyph.
  assert.equal(resolveSocket(gate("north"), "lintel"), null);
  assert.equal(resolveSocket({ ...gate("north"), kind: "light" }, "ward"), null);
  assert.equal(resolveSocket({ ...gate("north"), anchor: { kind: "cell" } }, "ward"), null);
});

test("the catalog declares the vertical scale and the camera", () => {
  assert.equal(VERTICAL_SCALE, 0.72);
  assert.equal(CAMERA.orthoHalfHeight, 3.7);
  assert.deepEqual(CAMERA.offset, { x: 0.86, y: 0.92, z: 1.08 });
  assert.equal(CAMERA.targetHeight, 0.5);
  assert.equal(CAMERA.near, 0.1);
  assert.equal(CAMERA.far, 80);
  assert.equal(CAMERA.maxPixelRatio, 2);
});

test("the closed sets are the ones content selects from", () => {
  assert.deepEqual([...ARCHITECTURE_ASSEMBLIES], ["visual/beveled_masonry"]);
  assert.deepEqual([...MATERIAL_FAMILIES], ["stone_bounded"]);
  assert.deepEqual([...TRIM_FAMILIES], ["broad_mortar"]);
  assert.deepEqual([...ACTOR_ASSEMBLIES], ["visual/gaoler_silhouette", "visual/player_silhouette"]);
  assert.deepEqual([...EFFECT_ASSEMBLIES], ["visual/cyan_crescent"]);
  // The roles `nomos.rendering_plan@3` added. They are what an actor is for, not
  // what it looks like, so the set is deliberately not the assembly names: two
  // silhouettes and two roles that happen to pair off in the corpus.
  assert.deepEqual([...ACTOR_ROLES], ["player", "pursuer"]);
});

// `the_catalog_knows_every_assembly_the_compiler_can_emit` stood here. It parsed
// `crates/nomos-render-plan/src/catalog.rs` and asserted that crate's
// kind-to-assembly and kind-to-material tables agreed with `ENTITY_KINDS` row for
// row, which was worth doing only while both ends held the same mapping. Issue
// #153 deleted the Rust tables and `nomos.rendering_plan@3` stopped carrying the
// two names, so this catalog is the sole place a kind becomes an assembly and
// there is no second table left to drift from. What the compiler may still emit
// is a kind, and `an unclassified entity is refused` in `plan.test.mjs` holds
// that end: a kind this catalog does not declare fails the decode.

test("one palette serves the scene and the ui", () => {
  assert.equal(Object.keys(PALETTE).length, 36);
  // The roles the two study tables disagreed on: the WebGL value wins.
  assert.equal(PALETTE.void, 0x090e13);
  assert.equal(PALETTE.stone_1, 0x2d3a43);
  assert.equal(PALETTE.cyan, 0x83eeea);
  // The roles only the SVG renderer or the stylesheet had.
  assert.equal(PALETTE.edge, 0x536168);
  assert.equal(PALETTE.danger, 0xd47158);
  assert.equal(PALETTE.surface, 0x0a1016);
  // And the procedural stone accent, which was already the same number as
  // `edge` in the other renderer's table.
  assert.equal(LOOK_PROFILES.procedural.materials.stone.accent, PALETTE.edge);
  assert.equal(hex(PALETTE.cyan), "#83eeea");
  assert.equal(hex(PALETTE.void), "#090e13");
  for (const [role, value] of Object.entries(PALETTE)) {
    assert.ok(Number.isInteger(value) && value >= 0 && value <= 0xffffff, `${role} is not a colour`);
  }
});

test("no colour literal lives outside the catalog", () => {
  const files = [
    ...appSources.filter((one) => one !== "catalog.mjs").map((one) => [one, codeOf(one)]),
    ["index.html", stripComments(readFileSync(new URL("../index.html", import.meta.url), "utf8"))],
  ];
  for (const [name, text] of files) {
    for (const pattern of [/#[0-9a-fA-F]{3,8}\b/, /\b0x[0-9a-fA-F]{6}\b/, /\brgba?\(/, /\bhsla?\(/]) {
      const found = text.match(pattern);
      assert.equal(found, null, `${name} carries a colour literal: ${found?.[0]}`);
    }
  }
});

test("two look profiles and no bare look literal", () => {
  assert.deepEqual([...LOOK_PROFILE_IDS], ["baseline", "procedural"]);
  assert.equal(LOOK_PROFILES.baseline.id, "gaol_baseline_01");
  assert.equal(LOOK_PROFILES.procedural.id, "gaol_procedural_01");
  assert.equal(LOOK_PROFILES.baseline.fogDensity, 0.045);
  assert.equal(LOOK_PROFILES.procedural.fogDensity, 0.041);
  assert.equal(LOOK_PROFILES.baseline.exposure, 1.28);
  assert.equal(LOOK_PROFILES.procedural.exposure, 1.34);
  assert.equal(LOOK_PROFILES.baseline.bevel, 0);
  assert.equal(LOOK_PROFILES.procedural.bevel, 0.055);
  assert.equal(LOOK_PROFILES.baseline.actorOutline, 0);
  assert.equal(LOOK_PROFILES.procedural.actorOutline, 1.065);
  assert.deepEqual(Object.keys(LOOK_PROFILES.procedural.materials), ["stone", "iron", "cloth"]);
  assert.deepEqual(Object.keys(LOOK_PROFILES), [...LOOK_PROFILE_IDS]);
  // The ids exist once. A consumer that writes "procedural" as a literal is
  // the drift the audit found four times over.
  for (const name of appSources.filter((one) => one !== "catalog.mjs")) {
    for (const id of LOOK_PROFILE_IDS) {
      assert.equal(
        new RegExp(`["'\`]${id}["'\`]`).test(codeOf(name)),
        false,
        `${name} carries the bare look id ${id}`,
      );
    }
  }
});

test("no identifier is re-cased into prose", () => {
  // Audit section 3 item 26. There is no `displayName` here and no table of
  // authored entity names, because a renderer that knew an entity's name would
  // need editing to accept new content.
  for (const name of appSources) {
    const text = codeOf(name);
    for (const pattern of [/displayName/, /toUpperCase/, /toLowerCase\(\)/, /\btitleCase\b/, /\\b\\w/]) {
      assert.equal(pattern.test(text), false, `${name} matches ${pattern}`);
    }
  }
  // Prose exists only for closed sets the accepted schema declares.
  assert.deepEqual(Object.keys(KIND_LABELS).sort(), Object.keys(ENTITY_KINDS).sort());
  assert.deepEqual(Object.keys(DISPOSITION_LABELS).sort(), ["blocked", "traversable"]);
});

test("no area, entity, or actor identifier appears in the app", () => {
  // The renderer must not special-case content. These are every area id in the
  // corpus, the two actor ids, and one entity id from each area.
  const forbidden = [
    "cistern-walk",
    "ember-vault",
    "ossuary-reach",
    "north-gaol",
    "north_gate",
    "sluice_gate",
    "vault_gate",
    "bone_gate",
    "brazier_02",
  ];
  for (const name of appSources) {
    for (const one of forbidden) {
      assert.equal(codeOf(name).includes(one), false, `${name} names ${one}`);
    }
  }
});

test("a direction is the lattice step it takes", () => {
  assert.equal(directionOf(0, -1), "north");
  assert.equal(directionOf(0, 1), "south");
  assert.equal(directionOf(-1, 0), "west");
  assert.equal(directionOf(1, 0), "east");
  assert.equal(directionOf(1, 1), null);
});

test("the entity vocabulary is closed and typed", () => {
  assert.deepEqual(Object.keys(ENTITY_KINDS).sort(), ["door", "light", "water"]);
  assert.equal(ENTITY_KINDS.door.anchorKind, "face");
  assert.equal(ENTITY_KINDS.water.anchorKind, "region");
  assert.equal(ENTITY_KINDS.light.anchorKind, "cell");
  assert.deepEqual([...ENTITY_KINDS.door.sockets], ["ward"]);
  assert.deepEqual([...ENTITY_KINDS.water.sockets], []);
});
