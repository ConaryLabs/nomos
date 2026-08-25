import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
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

// The corpus is discovered, not named. Every directory under `areas/` is one
// area; adding a fifth (or a fifteenth) changes nothing below this line.
const areasDir = fileURLToPath(new URL("../areas/", import.meta.url));
const areaIds = readdirSync(areasDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort();

const readPlan = (id) => JSON.parse(readFileSync(new URL(`../areas/${id}/rendering-plan.example.json`, import.meta.url)));
const readSource = (id) => readFileSync(new URL(`../areas/${id}/presentation.json`, import.meta.url), "utf8");

const plans = areaIds.map(readPlan);
const byId = new Map(plans.map((plan) => [plan.area.id, plan]));

// The area collection itself is not read here. `nomos.area_collection@2` is
// emitted by `crates/nomos-render-plan/src/collection.rs`, its route-graph and
// visual-grammar refusals are proved by `crates/nomos-render-plan/tests/collection.rs`,
// and the committed `area-collection.example.json` is compared byte for byte by
// `verify.mjs --collection`, which also re-derives every plan digest it
// publishes. What is left in this file is what it was always about: every
// committed plan, and the route graph each plan's own `route` already carries
// — discovered from `areas/*/` rather than named here.

const grammar = (plan) => ({
  architectureStyle: plan.architecture.style,
  entities: [...new Set(plan.entities.map((entity) => entity.kind))].sort(),
  actors: [...new Set(plan.actors.map((actor) => actor.assembly))].sort(),
  effects: [...new Set(plan.effects.map((effect) => effect.assembly))].sort(),
});

test("every area retains one exact visual grammar", () => {
  const [reference, ...rest] = plans;
  for (const plan of rest) {
    assert.notEqual(plan.area.id, reference.area.id);
    assert.deepEqual(grammar(plan), grammar(reference));
  }
});

test("every area exposes one exact compiled-door objective", () => {
  for (const plan of plans) {
    // The objective is derived from the one authored `route.exit.gate`, so
    // there is no `target` to agree with a `primaryGate` any more: the triple
    // the ownership audit recorded is one field.
    assert.deepEqual(Object.keys(plan.objective).sort(), ["gate", "kind"]);
    assert.equal(plan.objective.kind, "exit_via");
    assert.equal(plan.entities.find((entity) => entity.id === plan.objective.gate)?.kind, "door");
  }
});

test("no plan or source carries a decimal, a camera, a palette, or ui anchors", () => {
  for (const plan of plans) {
    for (const gone of ["camera", "palette", "uiAnchors", "ui_anchors", "deterministic", "presentation"]) {
      assert.equal(plan[gone], undefined, `${plan.area.id} still carries ${gone}`);
    }
    const withoutStrings = JSON.stringify(plan).replaceAll(/"[^"]*"/g, '""');
    assert.doesNotMatch(withoutStrings, /[-\d]\.\d/, `${plan.area.id} carries a decimal literal`);
  }
  for (const id of areaIds) {
    const withoutStrings = readSource(id).replaceAll(/"[^"]*"/g, '""');
    assert.doesNotMatch(withoutStrings, /[-\d]\.\d/, `${id} source carries a decimal literal`);
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
});

test("architecture heights are declared in bounds, as integer steps", () => {
  // AUTHORING.md: walls are 1..=50 steps and masonry masses 1..=40.
  for (const plan of plans) {
    const wall = plan.architecture.wall_height_steps;
    assert.ok(Number.isInteger(wall) && wall >= 1 && wall <= 50, `${plan.area.id} wall height ${wall} is out of bounds`);
    for (const mass of plan.architecture.masses) {
      const height = mass.height_steps;
      assert.ok(Number.isInteger(height) && height >= 1 && height <= 40, `${plan.area.id} mass ${mass.id} height ${height} is out of bounds`);
    }
  }
});

test("content selects from the renderer catalog's closed sets", () => {
  // The catalog defines what is legal; the presentation source selects one
  // member. The Rust decoder checks the grammar, this checks the membership.
  for (const plan of plans) {
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
  for (const plan of plans) {
    assert.equal(plan.schema, "nomos.rendering_plan@3");
    const roles = plan.actors.map((actor) => actor.role);
    assert.equal(roles.filter((role) => role === "player").length, 1, `${plan.area.id} declares exactly one player`);
    assert.ok(roles.filter((role) => role === "pursuer").length <= 1, `${plan.area.id} declares at most one pursuer`);
    for (const entity of plan.entities) {
      assert.equal(entity.visual_assembly, undefined);
      assert.equal(entity.material_family, undefined);
      assert.ok(assemblyOf(entity.kind), "the catalog owns the assembly for every kind");
    }
  }
});

test("effects attach by socket and carry no coordinate", () => {
  for (const plan of plans) {
    for (const effect of plan.effects) {
      assert.deepEqual(Object.keys(effect.anchor).sort(), ["entity", "socket"]);
      assert.equal(effect.anchor.socket, "ward");
      assert.equal(effect.presentationAnchor, undefined);
    }
  }
});

test("exactly one area is the start", () => {
  assert.equal(plans.filter((plan) => plan.area.start).length, 1);
});

test("every non-start area's arrival cell lies inside its own bounds; the start declares none", () => {
  for (const plan of plans) {
    if (plan.area.start) {
      assert.equal(plan.route.entry, undefined, `${plan.area.id} is the start and declares an entry`);
      continue;
    }
    assert.ok(plan.route.entry, `${plan.area.id} is not the start and declares no entry`);
    const { x, y, z } = plan.route.entry;
    assert.ok(x >= 0 && x < plan.architecture.bounds.width, `${plan.area.id} entry x is out of bounds`);
    assert.ok(y >= 0 && y < plan.architecture.bounds.height, `${plan.area.id} entry y is out of bounds`);
    assert.equal(z, 0, `${plan.area.id} entry is not at z 0`);
  }
});

test("every declared to_area names an enumerated area", () => {
  for (const plan of plans) {
    if (plan.route.to_area === null) continue;
    assert.ok(byId.has(plan.route.to_area), `${plan.area.id} names undeclared area ${plan.route.to_area}`);
  }
});

test("the route chain from the start visits every area exactly once and ends at the one null exit", () => {
  const terminal = plans.filter((plan) => plan.route.to_area === null);
  assert.equal(terminal.length, 1, "exactly one area declares no to_area");
  const start = plans.find((plan) => plan.area.start);
  const chain = [];
  let current = start.area.id;
  while (current !== null) {
    assert.ok(!chain.includes(current), `the route cycles at ${current}`);
    const plan = byId.get(current);
    assert.ok(plan, `the route names undeclared area ${current}`);
    chain.push(current);
    current = plan.route.to_area;
  }
  assert.equal(chain.length, plans.length, `the route visits ${chain.length} of ${plans.length} areas`);
  assert.equal(chain[chain.length - 1], terminal[0].area.id);
});

test("every area's pursuit light and exit gate are compiled entities of the right kind", () => {
  for (const plan of plans) {
    const light = plan.entities.find((entity) => entity.id === plan.pursuit.light);
    assert.ok(light, `${plan.area.id} pursuit light ${plan.pursuit.light} is not a compiled entity`);
    assert.equal(light.kind, "light");
    const gate = plan.entities.find((entity) => entity.id === plan.objective.gate);
    assert.ok(gate, `${plan.area.id} exit gate ${plan.objective.gate} is not a compiled entity`);
    assert.equal(gate.kind, "door");
  }
});

test("every area is a distinct composition", () => {
  const anchorsOf = (plan) => JSON.stringify(plan.entities.map((entity) => entity.anchor).sort());
  const seen = new Map();
  for (const plan of plans) {
    const key = anchorsOf(plan);
    assert.ok(!seen.has(key), `${plan.area.id} and ${seen.get(key)} share an identical entity layout`);
    seen.set(key, plan.area.id);
  }
});

// The six tests that drove `play-state.mjs` over these areas are gone with
// it. What they asserted - that each committed area is winnable and that one
// run crosses every declared connection - is now proved end to end by
// `apps/nomos-viewer/smoke/`, which plays the same corpus in a browser and
// checks the cumulative counters against a walk solved from these artifacts.
// RUNTIME.md section 2: the study is the specification, and this is what it
// looks like when a behaviour is promoted rather than copied.
