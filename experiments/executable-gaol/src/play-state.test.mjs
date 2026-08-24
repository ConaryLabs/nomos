import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { attemptInteraction, createPlayState, attemptMove, interactionAt, terrainAt } from "./play-state.mjs";

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

test("nearby interactions follow Nomos-verified state hashes", () => {
  const state = { ...createPlayState(), player: { x: 5, y: 1, z: 0 } };
  const ignite = interactionAt(plan, "01-baseline", state);
  assert.equal(ignite.action, "ignite");
  assert.equal(ignite.inputStateHash, plan.scenarios[0].stateHash);
  assert.equal(ignite.resultingStateHash, plan.scenarios[1].stateHash);

  const first = attemptInteraction(plan, "01-baseline", state);
  assert.equal(first.changed, true);
  assert.equal(first.scenarioId, "02-breached-warded");
  const second = attemptInteraction(plan, first.scenarioId, first.state);
  assert.equal(second.interaction.action, "unseal");
  assert.equal(second.scenarioId, "03-breached-unsealed");
});

test("interaction range does not invent remote actions", () => {
  const state = createPlayState();
  const result = attemptInteraction(plan, "01-baseline", state);
  assert.equal(result.changed, false);
  assert.equal(result.scenarioId, "01-baseline");
});
