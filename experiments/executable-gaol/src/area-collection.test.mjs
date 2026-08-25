import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  ACTOR_ASSEMBLIES,
  ARCHITECTURE_ASSEMBLIES,
  EFFECT_ASSEMBLIES,
  MATERIAL_FAMILIES,
  SOCKETS,
  TRIM_FAMILIES,
  VERTICAL_STEPS_PER_CELL,
  assemblyOf,
  cellsOf,
} from "./renderer-catalog.mjs";

const readPlan = (id) => JSON.parse(readFileSync(new URL(`../areas/${id}/rendering-plan.example.json`, import.meta.url)));
const north = readPlan("north-gaol");
const cistern = readPlan("cistern-walk");
const ember = readPlan("ember-vault");
const ossuary = readPlan("ossuary-reach");

// The area collection itself is not read here any more. `nomos.area_collection@2`
// is emitted by `crates/nomos-render-plan/src/collection.rs`, its route-graph and
// visual-grammar refusals are proved by `crates/nomos-render-plan/tests/collection.rs`,
// and the committed `area-collection.example.json` is compared byte for byte by
// `verify.mjs --collection`, which also re-derives every plan digest it publishes.
// What is left in this file is what it was always about: the four committed plans.

const grammar = (plan) => ({
  architectureStyle: plan.architecture.style,
  entities: [...new Set(plan.entities.map((entity) => entity.kind))].sort(),
  actors: [...new Set(plan.actors.map((actor) => actor.assembly))].sort(),
  effects: [...new Set(plan.effects.map((effect) => effect.assembly))].sort(),
});

test("independent areas retain one exact visual grammar", () => {
  assert.notEqual(north.area.id, cistern.area.id);
  assert.deepEqual(grammar(cistern), grammar(north));
  assert.deepEqual(grammar(ember), grammar(north));
  assert.deepEqual(grammar(ossuary), grammar(north));
});

test("every area exposes one exact compiled-door objective", () => {
  for (const plan of [cistern, ember, ossuary, north]) {
    // The objective is derived from the one authored `route.exit.gate`, so
    // there is no `target` to agree with a `primaryGate` any more: the triple
    // the ownership audit recorded is one field.
    assert.deepEqual(Object.keys(plan.objective).sort(), ["gate", "kind"]);
    assert.equal(plan.objective.kind, "exit_via");
    assert.equal(plan.entities.find((entity) => entity.id === plan.objective.gate)?.kind, "door");
  }
});

test("no plan or source carries a decimal, a camera, a palette, or ui anchors", () => {
  for (const plan of [cistern, ember, ossuary, north]) {
    for (const gone of ["camera", "palette", "uiAnchors", "ui_anchors", "deterministic", "presentation"]) {
      assert.equal(plan[gone], undefined, `${plan.area.id} still carries ${gone}`);
    }
    const withoutStrings = JSON.stringify(plan).replaceAll(/"[^"]*"/g, '""');
    assert.doesNotMatch(withoutStrings, /[-\d]\.\d/, `${plan.area.id} carries a decimal literal`);
  }
  for (const area of ["cistern-walk", "ember-vault", "north-gaol", "ossuary-reach"]) {
    const text = readFileSync(new URL(`../areas/${area}/presentation.json`, import.meta.url), "utf8");
    const withoutStrings = text.replaceAll(/"[^"]*"/g, '""');
    assert.doesNotMatch(withoutStrings, /[-\d]\.\d/, `${area} source carries a decimal literal`);
  }
});

test("vertical steps convert by division, reproducing every replaced decimal", () => {
  // The conversion is `steps / 10`. IEEE-754 division is correctly rounded, so
  // each result is the same double the decimal literal it replaced denoted.
  // Multiplying by the nearest double to 0.1 is a different operation and is
  // wrong for three of these ten, so the test pins the values rather than the
  // property.
  assert.equal(VERTICAL_STEPS_PER_CELL, 10);
  const expected = [[45, 4.5], [50, 5], [48, 4.8], [26, 2.6], [32, 3.2], [7, 0.7], [24, 2.4]];
  for (const [steps, cells] of expected) {
    assert.equal(cellsOf(steps), cells, `${steps} steps is not ${cells} cells`);
  }
  assert.equal(cellsOf(north.architecture.wall_height_steps), 4.5);
  assert.equal(cellsOf(ember.architecture.wall_height_steps), 5);
  assert.equal(cellsOf(ossuary.architecture.wall_height_steps), 4.8);
  assert.deepEqual(ossuary.architecture.masses.map((mass) => cellsOf(mass.height_steps)), [0.7, 0.7, 2.4]);
});

test("content selects from the renderer catalog's closed sets", () => {
  // The catalog defines what is legal; the presentation source selects one
  // member. The Rust decoder checks the grammar, this checks the membership.
  for (const plan of [cistern, ember, ossuary, north]) {
    assert.ok(ARCHITECTURE_ASSEMBLIES.includes(plan.architecture.style.assembly));
    assert.ok(MATERIAL_FAMILIES.includes(plan.architecture.style.material_family));
    assert.ok(TRIM_FAMILIES.includes(plan.architecture.style.trim_family));
    for (const actor of plan.actors) assert.ok(ACTOR_ASSEMBLIES.includes(actor.assembly));
    for (const effect of plan.effects) {
      assert.ok(EFFECT_ASSEMBLIES.includes(effect.assembly));
      const anchor = plan.entities.find((entity) => entity.id === effect.anchor.entity);
      assert.ok(SOCKETS[anchor.kind]?.[effect.anchor.socket], "the socket resolves in the catalog");
    }
  }
});

test("every actor declares a role and no plan entity names an assembly", () => {
  for (const plan of [cistern, ember, ossuary, north]) {
    assert.equal(plan.schema, "nomos.rendering_plan@3");
    const roles = plan.actors.map((actor) => actor.role).sort();
    assert.deepEqual(roles, ["player", "pursuer"]);
    for (const entity of plan.entities) {
      assert.equal(entity.visual_assembly, undefined);
      assert.equal(entity.material_family, undefined);
      assert.ok(assemblyOf(entity.kind), "the catalog owns the assembly for every kind");
    }
  }
});

test("effects attach by socket and carry no coordinate", () => {
  for (const plan of [cistern, ember, ossuary, north]) {
    for (const effect of plan.effects) {
      assert.deepEqual(Object.keys(effect.anchor).sort(), ["entity", "socket"]);
      assert.equal(effect.anchor.socket, "ward");
      assert.equal(effect.presentationAnchor, undefined);
    }
  }
});

test("each area declares its own arrival cell, and only the start area declares none", () => {
  assert.equal(cistern.area.start, true);
  assert.equal(cistern.route.entry, undefined);
  assert.deepEqual(ember.route.entry, { x: 7, y: 5, z: 0 });
  assert.deepEqual(ossuary.route.entry, { x: 1, y: 5, z: 0 });
  assert.deepEqual(north.route.entry, { x: 2, y: 4, z: 0 });
  assert.equal(north.route.to_area, null);
});

test("every added area is a distinct composition", () => {
  const anchors = (plan) => plan.entities.map((entity) => entity.anchor);
  assert.notDeepEqual(anchors(cistern), anchors(north));
  assert.deepEqual(cistern.actors.find((actor) => actor.role === "player").cell, { x: 7, y: 4, z: 0 });
  assert.equal(cistern.entities.find((entity) => entity.kind === "water").id, "runoff_channel");
  assert.notDeepEqual(anchors(ember), anchors(north));
  assert.equal(ember.architecture.masses.length, 2);
  assert.equal(ember.architecture.wall_height_steps, 50);
  assert.notDeepEqual(anchors(ossuary), anchors(north));
  assert.equal(ossuary.entities.find((entity) => entity.kind === "water").id, "burial_channel");
  assert.deepEqual(ossuary.entities.find((entity) => entity.kind === "water").anchor, {
    kind: "region",
    min: { x: 3, y: 1, z: 0 },
    max: { x: 5, y: 4, z: 0 },
  });
  assert.deepEqual(ossuary.architecture.masses.map((mass) => mass.id), [
    "west_tomb_bank", "east_tomb_bank", "reliquary_pier",
  ]);
});


// The six tests that drove `play-state.mjs` over these four areas are gone with
// it. What they asserted - that each committed area is winnable and that one run
// crosses every declared connection - is now proved end to end by
// `apps/nomos-viewer/smoke/`, which plays the same corpus in a browser and
// checks the cumulative counters against a walk solved from these artifacts.
// RUNTIME.md section 2: the study is the specification, and this is what it
// looks like when a behaviour is promoted rather than copied.
