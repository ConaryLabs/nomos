// The route solver, on artifacts it has never seen.
//
// The solver is test tooling and never ships, but it is the part of the lane
// that would quietly stop proving anything if it drifted, so it is tested
// against the same hand-authored fixtures as everything else - two areas that
// are not the corpus, with different water costs and a different gate.

import test from "node:test";
import assert from "node:assert/strict";

import { decodeCollection, decodePlan } from "../src/plan.mjs";
import { solveRoute } from "../smoke/route.mjs";
import { collectionDocument, hallPlan, yardPlan } from "./fixtures.mjs";

const collection = decodeCollection(collectionDocument());
const plans = new Map([
  ["test-hall", decodePlan(hallPlan())],
  ["test-yard", decodePlan(yardPlan())],
]);

test("the solver walks the route the artifacts declare", () => {
  const route = solveRoute(collection, plans);
  assert.deepEqual(
    route.legs.map((leg) => leg.area),
    ["test-hall", "test-yard"],
  );
  assert.deepEqual(route.legs[0].keys, ["ArrowUp", "ArrowUp", "KeyE", "ArrowRight", "ArrowUp"]);
  assert.deepEqual(route.legs[1].keys, ["ArrowUp", "KeyE", "ArrowRight", "ArrowUp"]);
  assert.equal(route.areas, 2);
});

test("the walk crosses the water, so the cost is not the move count", () => {
  const route = solveRoute(collection, plans);
  // The hall's pool costs four. Four moves at a cost of seven means exactly one
  // of them was through it: without the water waypoint the cheapest walk avoids
  // the pool entirely and a regression in projected cost would pass unnoticed.
  assert.equal(route.legs[0].moves, 4);
  assert.equal(route.legs[0].cost, 7);
  assert.equal(route.moves, 7);
  assert.equal(route.cost, 10);
  assert.notEqual(route.moves, route.cost);
  assert.equal(route.summary, "2 areas · 7 moves · 10 traversal cost");
});

test("the solver opens the gate through the declared interaction chain", () => {
  const route = solveRoute(collection, plans);
  // One `E` per interaction needed to make the objective gate traversable, and
  // not one more: the hall's extinguish edge is never taken, because the gate is
  // already open by then.
  assert.equal(route.legs[0].keys.filter((one) => one === "KeyE").length, 1);
  assert.equal(route.legs[0].scenario, "02-unsealed");
  assert.equal(route.legs[1].scenario, "02-open");
});

test("the solver ends each leg on the gate cell, facing out", () => {
  const route = solveRoute(collection, plans);
  for (const leg of route.legs) {
    assert.equal(leg.keys.at(-1), "ArrowUp", "both fixture gates face north");
  }
  assert.equal(route.legs[0].gate, "hall_gate");
  assert.equal(route.legs[1].gate, "yard_gate");
  assert.equal(route.legs[1].to, null);
});

test("the solver refuses a route it cannot walk", () => {
  // A gate no interaction opens is a broken corpus, not a lane that retries.
  const stuck = structuredClone(hallPlan());
  stuck.interactions = [];
  stuck.scenarios = stuck.scenarios.slice(0, 1);
  const broken = new Map(plans);
  broken.set("test-hall", decodePlan(stuck));
  assert.throws(() => solveRoute(collection, broken), /offers no interaction/);
});
