import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { sceneSignature } from "./r2-scene-signature.mjs";

const fixture = () => JSON.parse(readFileSync("fixtures/r2/scenes/scene_one.json", "utf8"));

test("identity rename and array permutation do not change a scene signature", () => {
  const original = fixture();
  const changed = structuredClone(original);
  changed.scene.id = "another_scene";
  const actors = new Map();
  changed.actors.forEach((actor, index) => {
    actors.set(actor.id, `renamed_${index}`);
    actor.id = `renamed_${index}`;
  });
  changed.actions.forEach((action, index) => {
    action.id = `different_${index}`;
    action.target_actor = actors.get(action.target_actor);
  });
  changed.terrain_layers.forEach((row, index) => { row.id = `terrain_${index}`; });
  changed.actions.reverse();
  changed.actors.reverse();
  changed.terrain_layers.reverse();
  assert.equal(sceneSignature(changed).sha256, sceneSignature(original).sha256);
});

test("each required semantic axis has its own digest", () => {
  const baseline = sceneSignature(fixture());
  const cases = {
    crop: (scene) => { scene.crop.width += 1; },
    terrain: (scene) => { scene.terrain_layers[0].cells.push({ x: 5, y: 5 }); },
    actors: (scene) => { scene.actors[0].cell.x += 1; },
    actions: (scene) => { scene.actions[0].availability = "enabled"; },
  };
  for (const [axis, mutate] of Object.entries(cases)) {
    const changed = fixture();
    mutate(changed);
    const signature = sceneSignature(changed);
    assert.notEqual(signature.axis_sha256[axis], baseline.axis_sha256[axis]);
  }
});

test("duplicate proof actor tuples are refused", () => {
  const changed = fixture();
  changed.actors[1] = { ...structuredClone(changed.actors[0]), id: changed.actors[1].id };
  assert.throws(() => sceneSignature(changed), /not unique/);
});
