// The presentation model, without a browser.

import test from "node:test";
import assert from "node:assert/strict";

import { PALETTE, hex } from "../src/catalog.mjs";
import { decodeCollection, decodePlan } from "../src/plan.mjs";
import { completeRun, createPlayState } from "../src/play.mjs";
import { applyPalette, readout, scenarioByIndex } from "../src/ui.mjs";
import { collectionDocument, hallPlan } from "./fixtures.mjs";

const collection = decodeCollection(collectionDocument());
const hall = decodePlan(hallPlan());
const flatten = (segments) => segments.map((one) => one.text).join("");

test("the readout reports area progress and cumulative counters", () => {
  const view = readout(collection, hall, createPlayState(hall), "01-sealed");
  assert.equal(view.area, "test-hall");
  assert.equal(view.scenario, "01-sealed");
  assert.equal(view.progress, "Area 1 / 2 · Test Hall");
  assert.equal(view.meter, "areas 0/2 · moves 0 · cost 0 · gaoler dormant");
  assert.equal(flatten(view.objective), "Exit via hall_gate");
  assert.equal(view.completed, false);
  assert.equal(view.arrival, "Area 1 of 2");
  assert.equal(view.title, "Test Hall");
});

test("the readout names the pursuit state the plan implies", () => {
  const play = createPlayState(hall);
  assert.equal(readout(collection, hall, play, "01-sealed").pursuit, "dormant");
  assert.equal(readout(collection, hall, play, "03-dark").pursuit, "hunting");
  assert.equal(readout(collection, hall, { ...play, caught: true }, "03-dark").pursuit, "caught");
});

test("the readout carries the completion summary", () => {
  const done = completeRun({ ...createPlayState(hall), areasCleared: 1, moves: 44, movementCost: 60 });
  const view = readout(collection, hall, done, "03-dark");
  assert.equal(view.completed, true);
  assert.equal(view.summary, "2 areas · 44 moves · 60 traversal cost");
  assert.equal(view.meter, "areas 2/2 · moves 44 · cost 60 · gaoler hunting");
  assert.equal(view.message, "Escaped the gaol");
});

test("number keys select by scenario identity", () => {
  assert.equal(scenarioByIndex(hall, 1).id, "01-sealed");
  assert.equal(scenarioByIndex(hall, 3).id, "03-dark");
  assert.equal(scenarioByIndex(hall, 0), null);
  assert.equal(scenarioByIndex(hall, 4), null);
  assert.equal(scenarioByIndex(hall, Number.NaN), null);
  // Reversing the plan's scenarios reverses what the keys select, because the
  // selection is the plan's order and not the DOM's.
  const reversed = structuredClone(hallPlan());
  reversed.scenarios.reverse();
  assert.equal(scenarioByIndex(decodePlan(reversed), 1).id, "03-dark");
});

test("the palette reaches the page as custom properties", () => {
  const properties = new Map();
  const document = {
    documentElement: { style: { setProperty: (name, value) => properties.set(name, value) } },
  };
  applyPalette(document);
  assert.equal(properties.size, Object.keys(PALETTE).length);
  assert.equal(properties.get("--nomos-cyan"), hex(PALETTE.cyan));
  assert.equal(properties.get("--nomos-surface-sunk"), hex(PALETTE.surface_sunk));
  assert.equal(properties.get("--nomos-water-high"), hex(PALETTE.water_high));
  for (const value of properties.values()) assert.match(value, /^#[0-9a-f]{6}$/);
});

test("the readout is the page's data contract", () => {
  // The smoke lane reads `data-` attributes off the root element. They are the
  // readout, so a change here is a change there, and `ui.mjs` writes no state
  // the HUD does not also paint.
  const source = readout(collection, hall, createPlayState(hall), "01-sealed");
  for (const key of ["area", "scenario", "message", "completed", "pursuit"]) {
    assert.ok(key in source, `the readout carries ${key}`);
  }
});
