import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { attemptInteraction, attemptMove, completeRun, createPlayState, enterArea } from "./play-state.mjs";

const readPlan = (id) => JSON.parse(readFileSync(new URL(`../areas/${id}/rendering-plan.example.json`, import.meta.url)));
const north = readPlan("north-gaol");
const cistern = readPlan("cistern-walk");
const ember = readPlan("ember-vault");
const collection = JSON.parse(readFileSync(new URL("../area-collection.example.json", import.meta.url)));

const grammar = (plan) => ({
  camera: plan.camera,
  palette: plan.palette,
  architectureStyle: plan.architecture.style,
  entities: [...new Set(plan.entities.map((entity) => `${entity.kind}:${entity.visualAssembly}:${entity.materialFamily}`))].sort(),
  actors: [...new Set(plan.actors.map((actor) => actor.assembly))].sort(),
  effects: [...new Set(plan.effects.map((effect) => effect.assembly))].sort(),
  uiAnchors: plan.uiAnchors,
});

test("independent areas retain one exact visual grammar", () => {
  assert.notEqual(north.area.id, cistern.area.id);
  assert.deepEqual(grammar(cistern), grammar(north));
  assert.deepEqual(grammar(ember), grammar(north));
});

test("every area exposes one exact compiled-door objective", () => {
  for (const plan of [cistern, ember, north]) {
    assert.deepEqual(Object.keys(plan.presentation.objective).sort(), ["kind", "target"]);
    assert.equal(plan.presentation.objective.kind, "exit_via");
    assert.equal(plan.presentation.objective.target, plan.presentation.primaryGate);
    assert.equal(plan.entities.find((entity) => entity.id === plan.presentation.objective.target)?.kind, "door");
  }
});

test("the second area is a distinct composition", () => {
  const anchors = (plan) => plan.entities.map((entity) => entity.anchor);
  assert.notDeepEqual(anchors(cistern), anchors(north));
  assert.deepEqual(cistern.actors.find((actor) => actor.id === "player").anchor.cell, { x: 7, y: 4, z: 0 });
  assert.equal(cistern.entities.find((entity) => entity.kind === "water").id, "runoff_channel");
  assert.notDeepEqual(anchors(ember), anchors(north));
  assert.equal(ember.architecture.masses.length, 2);
  assert.equal(ember.architecture.wallHeight, 5);
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
  assert.equal(ignite.interaction.targetEntity, "sluice_gate");
  assert.equal(ignite.interaction.inputStateHash, cistern.scenarios[0].stateHash);
  const unseal = attemptInteraction(cistern, ignite.scenarioId, ignite.state);
  assert.equal(unseal.scenarioId, "03-breached-unsealed");
  assert.equal(unseal.interaction.resultingStateHash, cistern.scenarios[2].stateHash);
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
  assert.equal(collection.startArea, "cistern-walk");
  let solved = solveArea(
    cistern,
    createPlayState(cistern),
    [[0, -1], [0, -1], [0, -1], [-1, 0], [-1, 0], [-1, 0], [-1, 0], [-1, 0]],
    [[1, 0], [1, 0], [1, 0], [0, 1]],
    [[0, -1], [-1, 0], [-1, 0], [-1, 0], [0, -1], [0, -1]],
  );
  let edge = collection.route.find((candidate) => candidate.fromArea === "cistern-walk" && candidate.gate === solved.exitGate);
  let state = enterArea(ember, solved.state, edge.entry);
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
  edge = collection.route.find((candidate) => candidate.fromArea === "ember-vault" && candidate.gate === solved.exitGate);
  state = enterArea(north, solved.state, edge.entry);
  assert.equal(state.areasCleared, 2);
  assert.ok(state.moves > cisternMoves);
  assert.ok(state.movementCost > cisternCost);

  solved = solveArea(
    north,
    state,
    [[0, -1], [0, -1], [0, -1], [1, 0], [1, 0], [1, 0]],
    [[-1, 0]],
    [[1, 0], [0, -1], [0, -1]],
  );
  edge = collection.route.find((candidate) => candidate.fromArea === "north-gaol" && candidate.gate === solved.exitGate);
  assert.equal(edge.toArea, null);
  state = completeRun(solved.state);
  assert.equal(state.completed, true);
  assert.equal(state.areasCleared, 3);
  assert.equal(state.message, "Escaped the gaol");
});
