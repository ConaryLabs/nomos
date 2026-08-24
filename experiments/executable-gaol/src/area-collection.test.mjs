import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { attemptInteraction, attemptMove, createPlayState } from "./play-state.mjs";

const readPlan = (id) => JSON.parse(readFileSync(new URL(`../areas/${id}/rendering-plan.example.json`, import.meta.url)));
const north = readPlan("north-gaol");
const cistern = readPlan("cistern-walk");

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
});

test("the second area is a distinct composition", () => {
  const anchors = (plan) => plan.entities.map((entity) => entity.anchor);
  assert.notDeepEqual(anchors(cistern), anchors(north));
  assert.deepEqual(cistern.actors.find((actor) => actor.id === "player").anchor.cell, { x: 7, y: 4, z: 0 });
  assert.equal(cistern.entities.find((entity) => entity.kind === "water").id, "runoff_channel");
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
