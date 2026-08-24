import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createPlayState, attemptMove, terrainAt } from "./play-state.mjs";

const plan = JSON.parse(readFileSync(new URL("../rendering-plan.example.json", import.meta.url)));

test("water uses the projected traversal cost", () => {
  const scenario = plan.scenarios[0];
  assert.deepEqual(terrainAt(plan, scenario, { x: 2, y: 2 }), {
    kind: "water", entity: "flooded_section", cost: 3,
  });
  const state = { ...createPlayState(), player: { x: 2, y: 3, z: 0 } };
  const result = attemptMove(plan, scenario.id, state, 0, -1);
  assert.equal(result.cost, 3);
  assert.equal(result.state.movementCost, 3);
});

test("the baseline gate refuses an exit", () => {
  const state = { ...createPlayState(), player: { x: 5, y: 0, z: 0 } };
  const result = attemptMove(plan, "01-baseline", state, 0, -1);
  assert.equal(result.moved, false);
  assert.equal(result.state.tone, "blocked");
  assert.match(result.state.message, /Blocked/);
});

test("the breached and unsealed gate permits an exit", () => {
  const state = { ...createPlayState(), player: { x: 5, y: 0, z: 0 } };
  const result = attemptMove(plan, "03-breached-unsealed", state, 0, -1);
  assert.equal(result.moved, true);
  assert.equal(result.state.escaped, true);
  assert.equal(result.state.player.y, -1);
});

test("the unchanged second door remains blocked", () => {
  const state = { ...createPlayState(), player: { x: 7, y: 0, z: 0 } };
  const result = attemptMove(plan, "04-open-dark", state, 0, -1);
  assert.equal(result.moved, false);
  assert.equal(result.state.tone, "blocked");
});
