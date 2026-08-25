import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const viewer = readFileSync(new URL("../viewer.html", import.meta.url), "utf8");
const renderer = readFileSync(new URL("./webgl-renderer.mjs", import.meta.url), "utf8");

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
  for (const area of ["cistern-walk", "ember-vault", "north-gaol"]) {
    assert.equal(renderer.includes(area), false, `renderer special-cases ${area}`);
  }
});

test("the GPU grammar includes the required graphical systems", () => {
  for (const primitive of [
    "WebGLRenderer",
    "MeshStandardMaterial",
    "DirectionalLight",
    "PointLight",
    "ShaderMaterial",
    "FogExp2",
    "shadowMap.enabled",
  ]) assert.equal(renderer.includes(primitive), true, `${primitive} is absent`);
});
