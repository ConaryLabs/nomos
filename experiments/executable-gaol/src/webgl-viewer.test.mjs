import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const viewer = readFileSync(new URL("../viewer.html", import.meta.url), "utf8");
const renderer = readFileSync(new URL("./webgl-renderer.mjs", import.meta.url), "utf8");
const catalog = readFileSync(new URL("./renderer-catalog.mjs", import.meta.url), "utf8");

test("the playable viewer presents WebGL rather than SVG", () => {
  assert.match(viewer, /createGaolRenderer/);
  assert.match(viewer, /gpu\.present/);
  assert.doesNotMatch(viewer, /renderSvg|innerHTML\s*=\s*render/);
});

test("the WebGL backend is exact-version pinned and projection-only", () => {
  assert.match(renderer, /three@0\.185\.1\/build\/three\.module\.min\.js/);
  for (const forbidden of [".nomos", "world-ir", "world_ir", "simulation.json", "navigation.json"]) {
    assert.equal(renderer.includes(forbidden), false, `renderer contains forbidden input ${forbidden}`);
  }
});

test("no declared area receives renderer special treatment", () => {
  for (const area of ["cistern-walk", "ember-vault", "ossuary-reach", "north-gaol"]) {
    assert.equal(renderer.includes(area), false, `renderer special-cases ${area}`);
  }
});

test("one bounded procedural grammar has an explicit baseline comparison", () => {
  assert.match(renderer, /export const lookProfiles/);
  for (const control of ["palette", "fogDensity", "exposure", "bevel", "actorOutline", "materials"]) {
    assert.equal(renderer.includes(control), true, `${control} is not a named look control`);
  }
  assert.match(renderer, /lookProfiles\.procedural/);
  assert.match(viewer, /Look: procedural/);
  assert.match(viewer, /setLookProfile/);
  assert.doesNotMatch(renderer, /TextureLoader|\.png|\.jpg|\.webp/);
});

test("the renderer owns its own scale, camera, and socket offsets", () => {
  // The audit's second and third double authorities were an undeclared `* 0.72`
  // applied to two content fields. The scale is now a named renderer constant,
  // content declares integer vertical steps, and the offsets an effect resolves
  // to live in the catalog rather than in any content file.
  assert.match(renderer, /const VERTICAL_SCALE = 0\.72;/);
  assert.match(renderer, /const ORTHO_HALF_HEIGHT = 3\.7;/);
  assert.match(catalog, /export const SOCKETS/);
  assert.match(catalog, /ward: Object\.freeze\(\{ x: 5, y: 0, z: 17 \}\)/);
  // And no consumer keeps a private fallback for a machine state, or a bare
  // look-profile literal.
  for (const forbidden of [", \"sealed\")", ", \"locked\")", ", \"intact\")", "machineStates", "effectiveLight"]) {
    assert.equal(renderer.includes(forbidden), false, `renderer still carries ${forbidden}`);
  }
  assert.match(viewer, /LOOK_PROFILE_IDS/);
});

test("the GPU grammar includes the required graphical systems", () => {
  for (const primitive of [
    "WebGLRenderer",
    "MeshStandardMaterial",
    "ExtrudeGeometry",
    "DirectionalLight",
    "PointLight",
    "ShaderMaterial",
    "FogExp2",
    "shadowMap.enabled",
  ]) assert.equal(renderer.includes(primitive), true, `${primitive} is absent`);
});
