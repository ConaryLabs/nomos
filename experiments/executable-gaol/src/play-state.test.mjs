import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { attemptInteraction, createPlayState, attemptMove, interactionAt, terrainAt } from "./play-state.mjs";

const plan = JSON.parse(readFileSync(new URL("../areas/north-gaol/rendering-plan.example.json", import.meta.url)));

test("water uses the projected traversal cost", () => {
  const scenario = plan.scenarios[0];
  assert.deepEqual(terrainAt(plan, scenario, { x: 2, y: 2 }), {
    kind: "water", entity: "flooded_section", cost: 3,
  });
  const state = { ...createPlayState(plan), player: { x: 2, y: 3, z: 0 } };
  const result = attemptMove(plan, scenario.id, state, 0, -1);
  assert.equal(result.cost, 3);
  assert.equal(result.state.movementCost, 3);
});

test("the baseline gate refuses an exit", () => {
  const state = { ...createPlayState(plan), player: { x: 5, y: 0, z: 0 } };
  const result = attemptMove(plan, "01-baseline", state, 0, -1);
  assert.equal(result.moved, false);
  assert.equal(result.state.tone, "blocked");
  assert.match(result.state.message, /Blocked/);
});

test("the breached and unsealed gate permits an exit", () => {
  const state = { ...createPlayState(plan), player: { x: 5, y: 0, z: 0 } };
  const result = attemptMove(plan, "03-breached-unsealed", state, 0, -1);
  assert.equal(result.moved, true);
  assert.equal(result.state.escaped, true);
  assert.equal(result.state.player.y, -1);
});

test("the unchanged second door remains blocked", () => {
  const state = { ...createPlayState(plan), player: { x: 7, y: 0, z: 0 } };
  const result = attemptMove(plan, "05-open-dark", state, 0, -1);
  assert.equal(result.moved, false);
  assert.equal(result.state.tone, "blocked");
});

test("nearby interactions follow Nomos-verified state hashes", () => {
  const state = { ...createPlayState(plan), player: { x: 5, y: 1, z: 0 } };
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
  const state = createPlayState(plan);
  const result = attemptInteraction(plan, "01-baseline", state);
  assert.equal(result.changed, false);
  assert.equal(result.scenarioId, "01-baseline");
});

test("the brazier interaction follows the verified extinguish receipt", () => {
  const state = { ...createPlayState(plan), player: { x: 4, y: 1, z: 0 } };
  const result = attemptInteraction(plan, "03-breached-unsealed", state);
  assert.equal(result.changed, true);
  assert.equal(result.interaction.action, "extinguish");
  assert.equal(result.interaction.targetEntity, "brazier_02");
  assert.equal(result.scenarioId, "04-breached-unsealed-dark");
  assert.equal(result.interaction.resultingStateHash, plan.scenarios[3].stateHash);
});

test("the gaoler stays dormant while the brazier is lit", () => {
  const state = createPlayState(plan);
  const result = attemptMove(plan, "03-breached-unsealed", state, 1, 0);
  assert.deepEqual(result.state.gaoler, state.gaoler);
  assert.equal(result.state.pursuitClock, 0);
});

test("the dark gaoler advances every second successful move", () => {
  const state = createPlayState(plan);
  const first = attemptMove(plan, "04-breached-unsealed-dark", state, 1, 0);
  assert.deepEqual(first.state.gaoler, state.gaoler);
  assert.equal(first.state.pursuitClock, 1);

  const second = attemptMove(plan, "04-breached-unsealed-dark", first.state, 1, 0);
  assert.deepEqual(second.state.gaoler, { x: 5, y: 4, z: 0 });
  assert.equal(second.state.pursuitClock, 0);
  assert.equal(second.state.message, "The gaoler advances in the dark");
});

test("the dark gaoler can catch and stop the player", () => {
  const state = {
    ...createPlayState(plan),
    player: { x: 3, y: 3, z: 0 },
    gaoler: { x: 5, y: 3, z: 0 },
    pursuitClock: 1,
  };
  const caught = attemptMove(plan, "04-breached-unsealed-dark", state, 1, 0);
  assert.equal(caught.state.caught, true);
  assert.deepEqual(caught.state.gaoler, caught.state.player);
  assert.equal(attemptMove(plan, "04-breached-unsealed-dark", caught.state, 0, -1).moved, false);
});

test("the complete extinguish-and-escape route remains winnable", () => {
  let scenarioId = "01-baseline";
  let state = createPlayState(plan);
  for (const [dx, dy] of [[0, -1], [0, -1], [0, -1], [1, 0], [1, 0], [1, 0]]) {
    state = attemptMove(plan, scenarioId, state, dx, dy).state;
  }
  for (let count = 0; count < 2; count += 1) {
    const result = attemptInteraction(plan, scenarioId, state);
    state = result.state;
    scenarioId = result.scenarioId;
  }
  state = attemptMove(plan, scenarioId, state, -1, 0).state;
  const extinguish = attemptInteraction(plan, scenarioId, state);
  state = extinguish.state;
  scenarioId = extinguish.scenarioId;
  for (const [dx, dy] of [[1, 0], [0, -1], [0, -1]]) {
    state = attemptMove(plan, scenarioId, state, dx, dy).state;
  }
  assert.equal(scenarioId, "04-breached-unsealed-dark");
  assert.equal(state.escaped, true);
  assert.equal(state.caught, false);
  assert.deepEqual(state.gaoler, { x: 5, y: 2, z: 0 });
});
