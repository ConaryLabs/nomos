import assert from "node:assert/strict";
import test from "node:test";

import { CATALOG, sparseVariationSelected } from "../src/catalog.mjs";

test("the issue-frozen catalog is exact and deeply frozen", () => {
  assert.deepEqual(CATALOG.viewport, { width: 1280, height: 720, pixel_ratio: 1 });
  assert.equal(CATALOG.camera.azimuth_degrees, 45);
  assert.equal(CATALOG.camera.elevation_radians, Math.atan(1 / Math.sqrt(2)));
  assert.equal(CATALOG.terrain.stack_offset, 0.004);
  assert.equal(CATALOG.terrain.visual_epsilon, 0.006);
  assert.deepEqual(Object.keys(CATALOG.terrain.assemblies), [
    "terrain/calm_ground",
    "terrain/traversable_route",
    "terrain/structure_footprint",
  ]);
  assert.deepEqual(Object.keys(CATALOG.terrain.materials), [
    "ground_muted",
    "route_worn",
    "structure_stone",
  ]);
  assert.deepEqual(Object.keys(CATALOG.actor.poses), ["upright_living", "prone_dead"]);
  assert.deepEqual(Object.keys(CATALOG.actions.markers), ["action/enabled", "action/disabled"]);
  const visit = (value) => {
    if (!value || typeof value !== "object") return;
    assert.equal(Object.isFrozen(value), true);
    Object.values(value).forEach(visit);
  };
  visit(CATALOG);
});

test("sparse variation is only the frozen integer predicate", () => {
  for (let x = 0; x < 32; x += 1) {
    for (let y = 0; y < 32; y += 1) {
      for (const stack of [0, 10, 20]) {
        assert.equal(sparseVariationSelected(x, y, stack), (17 * x + 31 * y + stack) % 16 === 0);
      }
    }
  }
});
