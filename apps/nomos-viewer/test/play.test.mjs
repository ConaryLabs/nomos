// The play rules, ported with the study's tests.
//
// `docs/review/nomos-viewer.md` section 2 rows 15 to 28 name the lines of
// `experiments/executable-gaol/src/play-state.mjs` each rule reproduces and the
// study test each of these reproduces.

import test from "node:test";
import assert from "node:assert/strict";

import { decodePlan, scenarioOf } from "../src/plan.mjs";
import {
  advanceGaoler,
  attemptInteraction,
  attemptMove,
  completeRun,
  completionSummary,
  createPlayState,
  enterArea,
  guidanceFor,
  interactionAt,
  isHunting,
  masonryAt,
  movementKeys,
  terrainAt,
} from "../src/play.mjs";
import { FACING_CELLS, facingPlan, hallPlan, yardPlan } from "./fixtures.mjs";

const hall = decodePlan(hallPlan());
const yard = decodePlan(yardPlan());
const at = (plan, cell, overrides = {}) => ({
  ...createPlayState(plan),
  player: { ...cell },
  ...overrides,
});
const flatten = (segments) => segments.map((one) => one.text).join("");

test("movement keys map to lattice deltas", () => {
  assert.deepEqual(movementKeys.ArrowUp, { dx: 0, dy: -1 });
  assert.deepEqual(movementKeys.KeyW, movementKeys.ArrowUp);
  assert.deepEqual(movementKeys.KeyS, movementKeys.ArrowDown);
  assert.deepEqual(movementKeys.KeyA, movementKeys.ArrowLeft);
  assert.deepEqual(movementKeys.KeyD, movementKeys.ArrowRight);
  assert.equal(movementKeys.KeyQ, undefined);
});

test("a plan without an actor is refused", () => {
  const missing = structuredClone(hallPlan());
  missing.actors = missing.actors.filter((one) => one.id !== "gaoler");
  assert.throws(() => createPlayState(decodePlan(missing)), /declares no actor gaoler/);
});

test("water uses the projected traversal cost", () => {
  // Four, because this plan's projection says four. The study's areas say
  // three; a renderer that knew the number would be reading its own mind.
  const scenario = scenarioOf(hall, "01-sealed");
  assert.deepEqual(terrainAt(hall, scenario, { x: 0, y: 1 }), {
    kind: "water",
    entity: "hall_pool",
    cost: 4,
  });
  assert.deepEqual(terrainAt(hall, scenario, { x: 2, y: 2 }), {
    kind: "stone",
    entity: null,
    cost: 1,
  });
  const result = attemptMove(hall, "01-sealed", at(hall, { x: 0, y: 2 }), 0, -1);
  assert.equal(result.cost, 4);
  assert.equal(result.state.movementCost, 4);
  assert.equal(result.state.message, "Shallow water costs 4");
  assert.equal(result.state.tone, "water");
});

test("a mass blocks the cells it covers", () => {
  assert.equal(masonryAt(hall, { x: 3, y: 0 })?.id, "pillar");
  assert.equal(masonryAt(hall, { x: 3, y: 1 }), null);
  assert.equal(masonryAt(hall, { x: 2, y: 0 }), null);
  const result = attemptMove(hall, "01-sealed", at(hall, { x: 3, y: 1 }), 0, -1);
  assert.equal(result.moved, false);
  assert.equal(result.state.message, "Blocked by pillar");
});

test("the baseline gate refuses an exit", () => {
  const result = attemptMove(hall, "01-sealed", at(hall, { x: 1, y: 0 }), 0, -1);
  assert.equal(result.moved, false);
  assert.equal(result.state.tone, "blocked");
  assert.match(result.state.message, /^Blocked: /);
  // The reason comes from the projection, not from a sentence written here.
  assert.match(result.state.message, /blocks_ground/);
});

test("the breached and unsealed gate permits an exit", () => {
  const result = attemptMove(hall, "02-unsealed", at(hall, { x: 1, y: 0 }), 0, -1);
  assert.equal(result.moved, true);
  assert.equal(result.state.escaped, true);
  assert.equal(result.state.player.y, -1);
  assert.equal(result.exitGate, "hall_gate");
  assert.equal(result.cost, 1);
});

test("the unchanged second door remains blocked", () => {
  const result = attemptMove(hall, "03-dark", at(hall, { x: 2, y: 0 }), 0, -1);
  assert.equal(result.moved, false);
  assert.equal(result.state.tone, "blocked");
});

test("a move that leaves the lattice with no door finds masonry", () => {
  const result = attemptMove(hall, "02-unsealed", at(hall, { x: 0, y: 0 }), 0, -1);
  assert.equal(result.moved, false);
  assert.equal(result.state.message, "The masonry has no opening here");
});

test("an exit uses the door's declared direction", () => {
  // R1-3 could not express a non-north door; every corpus door faces north, so
  // this is proved on a fixture with one door per face.
  for (const [direction, cell] of Object.entries(FACING_CELLS)) {
    const plan = decodePlan(facingPlan(direction));
    const delta = movementKeys[
      { north: "ArrowUp", south: "ArrowDown", west: "ArrowLeft", east: "ArrowRight" }[direction]
    ];
    const opened = attemptMove(plan, "02-open", at(plan, cell), delta.dx, delta.dy);
    assert.equal(opened.moved, true, `${direction} should open`);
    assert.equal(opened.state.escaped, true, `${direction} should escape`);
    assert.equal(opened.exitGate, "yard_gate");
    // And the same door, still sealed, refuses.
    const shut = attemptMove(plan, "01-shut", at(plan, cell), delta.dx, delta.dy);
    assert.equal(shut.moved, false, `${direction} should refuse while shut`);
  }
});

test("nearby interactions follow verified state hashes", () => {
  const beside = at(hall, { x: 1, y: 1 });
  const unseal = interactionAt(hall, "01-sealed", beside);
  assert.equal(unseal.action, "unseal");
  assert.equal(unseal.input_state_hash, scenarioOf(hall, "01-sealed").state_hash);
  assert.equal(unseal.resulting_state_hash, scenarioOf(hall, "02-unsealed").state_hash);

  const first = attemptInteraction(hall, "01-sealed", beside);
  assert.equal(first.changed, true);
  assert.equal(first.scenarioId, "02-unsealed");
  assert.equal(first.state.message, "unseal hall_gate");
});

test("interaction range does not invent remote actions", () => {
  const away = at(hall, { x: 0, y: 2 });
  const result = attemptInteraction(hall, "01-sealed", away);
  assert.equal(result.changed, false);
  assert.equal(result.scenarioId, "01-sealed");
  assert.equal(result.state.message, "Nothing responds here");
});

test("the brazier interaction follows the verified extinguish receipt", () => {
  const beside = at(hall, { x: 1, y: 2 });
  const result = attemptInteraction(hall, "02-unsealed", beside);
  assert.equal(result.changed, true);
  assert.equal(result.interaction.action, "extinguish");
  assert.equal(result.interaction.target_entity, "hall_lamp");
  assert.equal(result.scenarioId, "03-dark");
  assert.equal(result.interaction.resulting_state_hash, scenarioOf(hall, "03-dark").state_hash);
});

test("the gaoler hunts only when the pursuit light is out", () => {
  assert.equal(isHunting(hall, scenarioOf(hall, "01-sealed")), false);
  assert.equal(isHunting(hall, scenarioOf(hall, "02-unsealed")), false);
  assert.equal(isHunting(hall, scenarioOf(hall, "03-dark")), true);
});

test("the gaoler stays dormant while the light is lit", () => {
  const state = at(hall, { x: 0, y: 2 });
  const result = attemptMove(hall, "02-unsealed", state, 1, 0);
  assert.deepEqual(result.state.gaoler, state.gaoler);
  assert.equal(result.state.pursuitClock, 0);
});

test("the dark gaoler advances every second successful move", () => {
  const state = at(hall, { x: 0, y: 2 }, { gaoler: { x: 3, y: 1, z: 0 } });
  const first = attemptMove(hall, "03-dark", state, 1, 0);
  assert.deepEqual(first.state.gaoler, state.gaoler);
  assert.equal(first.state.pursuitClock, 1);

  const second = attemptMove(hall, "03-dark", first.state, 1, 0);
  assert.deepEqual(second.state.gaoler, { x: 3, y: 2, z: 0 });
  assert.equal(second.state.pursuitClock, 0);
  assert.equal(second.state.message, "The gaoler advances in the dark");
  assert.equal(second.state.tone, "danger");
});

test("the dark gaoler can catch and stop the player", () => {
  const state = at(hall, { x: 1, y: 2 }, { gaoler: { x: 3, y: 2, z: 0 }, pursuitClock: 1 });
  const caught = attemptMove(hall, "03-dark", state, 1, 0);
  assert.equal(caught.state.caught, true);
  assert.deepEqual(caught.state.gaoler, caught.state.player);
  assert.equal(attemptMove(hall, "03-dark", caught.state, 0, -1).moved, false);
  // And an interaction says so rather than firing.
  assert.equal(attemptInteraction(hall, "03-dark", caught.state).changed, false);
});

test("pursuit advances only for a scenario that is hunting", () => {
  const state = at(hall, { x: 1, y: 2 }, { pursuitClock: 1 });
  assert.equal(advanceGaoler(hall, scenarioOf(hall, "01-sealed"), state).pursuitClock, 1);
  assert.equal(advanceGaoler(hall, scenarioOf(hall, "03-dark"), state).pursuitClock, 0);
});

test("guidance derives the objective and prompt from plan data", () => {
  const initial = createPlayState(hall);
  const start = guidanceFor(hall, "01-sealed", initial);
  assert.equal(flatten(start.objective), "Exit via hall_gate");
  assert.equal(flatten(start.prompt), "Reach hall_gate");
  assert.equal(start.tone, "neutral");

  const beside = at(hall, { x: 1, y: 1 });
  const action = guidanceFor(hall, "01-sealed", beside);
  assert.equal(flatten(action.prompt), "E · unseal hall_gate");
  assert.equal(action.tone, "action");

  const open = guidanceFor(hall, "02-unsealed", initial);
  assert.equal(flatten(open.prompt), "The way through hall_gate is open");
  assert.equal(open.tone, "success");
});

test("no identifier is re-cased into prose", () => {
  // Every identifier a guidance line carries is a separate segment, marked as
  // an identifier and spelled exactly as the plan spells it.
  const guidance = guidanceFor(hall, "01-sealed", at(hall, { x: 1, y: 1 }));
  const identifiers = guidance.prompt.filter((one) => one.kind === "identifier").map((one) => one.text);
  assert.deepEqual(identifiers, ["unseal", "hall_gate"]);
  for (const segment of [...guidance.objective, ...guidance.prompt]) {
    assert.ok(["words", "identifier"].includes(segment.kind));
    if (segment.kind === "identifier") assert.match(segment.text, /^[a-z][a-z0-9_]*$/);
  }
});

test("arrival uses the destination's own entry cell", () => {
  const before = { ...createPlayState(hall), moves: 9, movementCost: 14, areasCleared: 1 };
  const arrived = enterArea(yard, before);
  assert.deepEqual(arrived.player, { x: 1, y: 1, z: 0 });
  assert.equal(arrived.moves, 9);
  assert.equal(arrived.movementCost, 14);
  assert.equal(arrived.areasCleared, 2);
  assert.equal(arrived.message, "Entered Test Yard");
  // The start area declares no arrival cell, and arriving there is a defect.
  assert.throws(() => enterArea(hall, before), /declares no arrival cell/);
});

test("completion reports cumulative run state", () => {
  const completed = completeRun({
    ...createPlayState(hall),
    areasCleared: 1,
    moves: 41,
    movementCost: 57,
  });
  assert.equal(completed.completed, true);
  assert.equal(completed.areasCleared, 2);
  assert.equal(completed.message, "Escaped the gaol");
  assert.equal(completionSummary(completed, 2), "2 areas · 41 moves · 57 traversal cost");
  const guidance = guidanceFor(hall, "03-dark", completed);
  assert.equal(flatten(guidance.objective), "Escape complete");
  assert.equal(flatten(guidance.prompt), "R · Begin a new run");
});

test("the unseal-and-escape route remains winnable across both areas", () => {
  let scenarioId = "01-sealed";
  let state = createPlayState(hall);
  // (0,2) -> (1,2) -> (1,1) through the pool -> (1,0), the gate's own cell.
  for (const [dx, dy] of [
    [1, 0],
    [0, -1],
    [0, -1],
  ]) {
    state = attemptMove(hall, scenarioId, state, dx, dy).state;
  }
  assert.deepEqual(state.player, { x: 1, y: 0, z: 0 });
  assert.equal(state.movementCost, 6, "one stone, one water at four, one stone");

  const unsealed = attemptInteraction(hall, scenarioId, state);
  state = unsealed.state;
  scenarioId = unsealed.scenarioId;
  assert.equal(scenarioId, "02-unsealed");

  const exit = attemptMove(hall, scenarioId, state, 0, -1);
  assert.equal(exit.state.escaped, true);
  assert.equal(exit.exitGate, "hall_gate");

  state = enterArea(yard, exit.state);
  scenarioId = "01-shut";
  assert.deepEqual(state.player, { x: 1, y: 1, z: 0 });
  state = attemptMove(yard, scenarioId, state, 1, 0).state;

  const opened = attemptInteraction(yard, scenarioId, state);
  state = opened.state;
  scenarioId = opened.scenarioId;
  assert.equal(scenarioId, "02-open");

  state = attemptMove(yard, scenarioId, state, 0, -1).state;
  const final = attemptMove(yard, scenarioId, state, 0, -1);
  assert.equal(final.state.escaped, true);
  state = completeRun(final.state);
  assert.equal(state.completed, true);
  assert.equal(state.areasCleared, 2);
  assert.equal(state.caught, false);
  assert.equal(completionSummary(state, 2), `2 areas · ${state.moves} moves · ${state.movementCost} traversal cost`);
});
