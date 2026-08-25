import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { attemptInteraction, attemptMove, completeRun, createPlayState, enterArea } from "./play-state.mjs";
import {
  ACTOR_ASSEMBLIES,
  ARCHITECTURE_ASSEMBLIES,
  EFFECT_ASSEMBLIES,
  MATERIAL_FAMILIES,
  SOCKETS,
  TRIM_FAMILIES,
  VERTICAL_STEPS_PER_CELL,
  cellsOf,
} from "./renderer-catalog.mjs";

const readPlan = (id) => JSON.parse(readFileSync(new URL(`../areas/${id}/rendering-plan.example.json`, import.meta.url)));
const north = readPlan("north-gaol");
const cistern = readPlan("cistern-walk");
const ember = readPlan("ember-vault");
const ossuary = readPlan("ossuary-reach");
const collection = JSON.parse(readFileSync(new URL("../area-collection.example.json", import.meta.url)));

const grammar = (plan) => ({
  architectureStyle: plan.architecture.style,
  entities: [...new Set(plan.entities.map((entity) => `${entity.kind}:${entity.visual_assembly}:${entity.material_family}`))].sort(),
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
      assert.ok(SOCKETS[anchor.visual_assembly]?.[effect.anchor.socket], "the socket resolves in the catalog");
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
  // And the collection's route reads each hop's arrival from the destination.
  for (const edge of collection.route) {
    if (edge.to_area === null) continue;
    const target = [cistern, ember, ossuary, north].find((plan) => plan.area.id === edge.to_area);
    assert.deepEqual(edge.entry, target.route.entry);
  }
});

test("every added area is a distinct composition", () => {
  const anchors = (plan) => plan.entities.map((entity) => entity.anchor);
  assert.notDeepEqual(anchors(cistern), anchors(north));
  assert.deepEqual(cistern.actors.find((actor) => actor.id === "player").cell, { x: 7, y: 4, z: 0 });
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

test("declared masonry masses block presentation movement", () => {
  const state = { ...createPlayState(cistern), player: { x: 3, y: 0, z: 0 } };
  const result = attemptMove(cistern, "01-baseline", state, 1, 0);
  assert.equal(result.moved, false);
  assert.equal(result.state.message, "Blocked by channel_buttress");
});

test("the cistern interactions remain projection-bound", () => {
  const state = { ...createPlayState(cistern), player: { x: 2, y: 1, z: 0 } };
  const ignite = attemptInteraction(cistern, "01-baseline", state);
  assert.equal(ignite.interaction.target_entity, "sluice_gate");
  assert.equal(ignite.interaction.input_state_hash, cistern.scenarios[0].state_hash);
  const unseal = attemptInteraction(cistern, ignite.scenarioId, ignite.state);
  assert.equal(unseal.scenarioId, "03-breached-unsealed");
  assert.equal(unseal.interaction.resulting_state_hash, cistern.scenarios[2].state_hash);
});

test("the cistern dark route is winnable", () => {
  let scenarioId = "01-baseline";
  let state = createPlayState(cistern);
  for (const [dx, dy] of [[0, -1], [0, -1], [0, -1], [-1, 0], [-1, 0], [-1, 0], [-1, 0], [-1, 0]]) {
    state = attemptMove(cistern, scenarioId, state, dx, dy).state;
  }
  for (let count = 0; count < 2; count += 1) {
    const result = attemptInteraction(cistern, scenarioId, state);
    state = result.state;
    scenarioId = result.scenarioId;
  }
  for (const [dx, dy] of [[1, 0], [1, 0], [1, 0], [0, 1]]) {
    state = attemptMove(cistern, scenarioId, state, dx, dy).state;
  }
  const extinguish = attemptInteraction(cistern, scenarioId, state);
  state = extinguish.state;
  scenarioId = extinguish.scenarioId;
  for (const [dx, dy] of [[0, -1], [-1, 0], [-1, 0], [-1, 0], [0, -1], [0, -1]]) {
    state = attemptMove(cistern, scenarioId, state, dx, dy).state;
  }
  assert.equal(state.escaped, true);
  assert.equal(state.caught, false);
});

test("the ember vault dark route is winnable", () => {
  let scenarioId = "01-baseline";
  let state = createPlayState(ember);
  for (const [dx, dy] of [[0, -1], [0, -1], [0, -1], [-1, 0], [-1, 0], [-1, 0], [0, -1]]) {
    state = attemptMove(ember, scenarioId, state, dx, dy).state;
  }
  for (let count = 0; count < 2; count += 1) {
    const result = attemptInteraction(ember, scenarioId, state);
    state = result.state;
    scenarioId = result.scenarioId;
  }
  for (const [dx, dy] of [[-1, 0], [0, 1], [-1, 0], [-1, 0]]) {
    state = attemptMove(ember, scenarioId, state, dx, dy).state;
  }
  const extinguish = attemptInteraction(ember, scenarioId, state);
  state = extinguish.state;
  scenarioId = extinguish.scenarioId;
  for (const [dx, dy] of [[1, 0], [1, 0], [0, -1], [1, 0], [0, -1], [0, -1]]) {
    state = attemptMove(ember, scenarioId, state, dx, dy).state;
  }
  assert.equal(state.escaped, true);
  assert.equal(state.caught, false);
});

const solveArea = (plan, initialState, toGate, toLight, toExit) => {
  let scenarioId = "01-baseline";
  let state = initialState;
  for (const [dx, dy] of toGate) state = attemptMove(plan, scenarioId, state, dx, dy).state;
  for (let count = 0; count < 2; count += 1) {
    const interaction = attemptInteraction(plan, scenarioId, state);
    state = interaction.state;
    scenarioId = interaction.scenarioId;
  }
  for (const [dx, dy] of toLight) state = attemptMove(plan, scenarioId, state, dx, dy).state;
  const extinguish = attemptInteraction(plan, scenarioId, state);
  state = extinguish.state;
  scenarioId = extinguish.scenarioId;
  let result;
  for (const [dx, dy] of toExit) {
    result = attemptMove(plan, scenarioId, state, dx, dy);
    state = result.state;
  }
  assert.equal(state.caught, false);
  assert.equal(state.escaped, true);
  return { state, exitGate: result.exitGate };
};

test("one run traverses all declared area connections", () => {
  assert.equal(collection.start_area, "cistern-walk");
  let solved = solveArea(
    cistern,
    createPlayState(cistern),
    [[0, -1], [0, -1], [0, -1], [-1, 0], [-1, 0], [-1, 0], [-1, 0], [-1, 0]],
    [[1, 0], [1, 0], [1, 0], [0, 1]],
    [[0, -1], [-1, 0], [-1, 0], [-1, 0], [0, -1], [0, -1]],
  );
  let edge = collection.route.find((candidate) => candidate.from_area === "cistern-walk" && candidate.gate === solved.exitGate);
  assert.equal(edge.to_area, "ember-vault");
  let state = enterArea(ember, solved.state);
  const cisternMoves = state.moves;
  const cisternCost = state.movementCost;
  assert.equal(state.areasCleared, 1);

  solved = solveArea(
    ember,
    state,
    [[0, -1], [0, -1], [0, -1], [-1, 0], [-1, 0], [-1, 0], [0, -1]],
    [[-1, 0], [0, 1], [-1, 0], [-1, 0]],
    [[1, 0], [1, 0], [0, -1], [1, 0], [0, -1], [0, -1]],
  );
  edge = collection.route.find((candidate) => candidate.from_area === "ember-vault" && candidate.gate === solved.exitGate);
  assert.equal(edge.to_area, "ossuary-reach");
  state = enterArea(ossuary, solved.state);
  assert.equal(state.areasCleared, 2);
  assert.ok(state.moves > cisternMoves);
  assert.ok(state.movementCost > cisternCost);

  solved = solveArea(
    ossuary,
    state,
    [[1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [0, -1], [0, -1], [0, -1], [0, -1]],
    [[0, 1], [0, 1], [0, 1]],
    [[0, -1], [0, -1], [0, -1], [0, -1], [0, -1]],
  );
  edge = collection.route.find((candidate) => candidate.from_area === "ossuary-reach" && candidate.gate === solved.exitGate);
  assert.equal(edge.to_area, "north-gaol");
  state = enterArea(north, solved.state);
  assert.equal(state.areasCleared, 3);

  solved = solveArea(
    north,
    state,
    [[0, -1], [0, -1], [0, -1], [1, 0], [1, 0], [1, 0]],
    [[-1, 0]],
    [[1, 0], [0, -1], [0, -1]],
  );
  edge = collection.route.find((candidate) => candidate.from_area === "north-gaol" && candidate.gate === solved.exitGate);
  assert.equal(edge.to_area, null);
  state = completeRun(solved.state);
  assert.equal(state.completed, true);
  assert.equal(state.areasCleared, 4);
  assert.equal(state.message, "Escaped the gaol");
});
