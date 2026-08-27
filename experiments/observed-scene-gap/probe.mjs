#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, rmSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const experimentDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(experimentDir, "../..");
const outputDir = join(repoRoot, "target/observed-scene-gap");
const fixturePath = join(experimentDir, "fixture.json");
const sourcePath = join(
  repoRoot,
  "experiments/executable-gaol/areas/cistern-walk/presentation.json",
);
const substrateDir = join(
  repoRoot,
  "target/executable-gaol/areas/cistern-walk",
);
const compiler = join(repoRoot, "target/debug/nomos-render-plan");

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function loadJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function exactFields(value, expected, context) {
  assert.equal(
    value !== null && typeof value === "object" && !Array.isArray(value),
    true,
    `${context} must be an object`,
  );
  assert.deepEqual(Object.keys(value).sort(), [...expected].sort(), `${context} fields`);
}

function integerCell(cell, crop, context) {
  exactFields(cell, ["x", "y", "z"], context);
  for (const key of ["x", "y", "z"]) {
    assert.equal(Number.isSafeInteger(cell[key]), true, `${context}.${key} must be an integer`);
  }
  assert.equal(cell.x >= 0 && cell.x < crop.width, true, `${context}.x must be in bounds`);
  assert.equal(cell.y >= 0 && cell.y < crop.height, true, `${context}.y must be in bounds`);
  assert.equal(cell.z, 0, `${context}.z must use the bounded logical plane`);
}

function logicalPoint(point, crop, context, inclusiveMaximum = false) {
  exactFields(point, ["x", "y"], context);
  for (const key of ["x", "y"]) {
    assert.equal(Number.isSafeInteger(point[key]), true, `${context}.${key} must be an integer`);
  }
  const xLimit = inclusiveMaximum ? point.x <= crop.width : point.x < crop.width;
  const yLimit = inclusiveMaximum ? point.y <= crop.height : point.y < crop.height;
  assert.equal(point.x >= 0 && xLimit, true, `${context}.x must be in bounds`);
  assert.equal(point.y >= 0 && yLimit, true, `${context}.y must be in bounds`);
}

function logicalRegion(region, crop, context) {
  exactFields(region, ["max", "min"], context);
  logicalPoint(region.min, crop, `${context}.min`);
  logicalPoint(region.max, crop, `${context}.max`, true);
  assert.equal(region.min.x < region.max.x, true, `${context} must have positive width`);
  assert.equal(region.min.y < region.max.y, true, `${context} must have positive height`);
}

function validateFixture(fixture) {
  exactFields(
    fixture,
    ["actors", "crop", "observed_actions", "schema", "terrain_layers"],
    "fixture",
  );
  assert.equal(fixture.schema, "nomos.experiment.observed_scene_gap@1");
  exactFields(fixture.crop, ["height", "width"], "crop");
  assert.equal(Number.isSafeInteger(fixture.crop.width), true);
  assert.equal(Number.isSafeInteger(fixture.crop.height), true);
  assert.equal(fixture.crop.width > 0, true);
  assert.equal(fixture.crop.height > 0, true);

  assert.deepEqual(
    fixture.terrain_layers.map((layer) => layer.role).sort(),
    ["calm_ground", "structure_footprint", "traversable_route"],
  );
  const groundLayer = fixture.terrain_layers.find((layer) => layer.role === "calm_ground");
  const routeLayer = fixture.terrain_layers.find(
    (layer) => layer.role === "traversable_route",
  );
  const structureLayer = fixture.terrain_layers.find(
    (layer) => layer.role === "structure_footprint",
  );
  exactFields(groundLayer, ["id", "region", "role"], "calm-ground layer");
  exactFields(routeLayer, ["cells", "id", "role"], "route layer");
  exactFields(structureLayer, ["id", "region", "role"], "structure layer");
  logicalRegion(groundLayer.region, fixture.crop, "calm-ground region");
  logicalRegion(structureLayer.region, fixture.crop, "structure-footprint region");
  assert.equal(routeLayer.cells.length > 0, true, "route layer must contain cells");
  for (const [index, cell] of routeLayer.cells.entries()) {
    logicalPoint(cell, fixture.crop, `route cells[${index}]`);
  }
  assert.equal(new Set(routeLayer.cells.map((cell) => `${cell.x},${cell.y}`)).size, routeLayer.cells.length);
  assert.deepEqual(groundLayer.region, {
    max: { x: fixture.crop.width, y: fixture.crop.height },
    min: { x: 0, y: 0 },
  });
  assert.deepEqual(
    fixture.actors.map((actor) => actor.id).sort(),
    ["controlled_actor", "hostile_actor", "protected_actor"],
  );
  for (const [index, actor] of fixture.actors.entries()) {
    exactFields(
      actor,
      ["cell", "controlled", "hostile", "id", "life_state", "protected"],
      `actors[${index}]`,
    );
    integerCell(actor.cell, fixture.crop, `actors[${index}].cell`);
    assert.equal(actor.life_state, "living");
    for (const fact of ["controlled", "hostile", "protected"]) {
      assert.equal(typeof actor[fact], "boolean", `actors[${index}].${fact}`);
    }
  }
  assert.equal(fixture.actors.filter((actor) => actor.controlled).length, 1);
  assert.equal(fixture.actors.filter((actor) => actor.hostile).length, 1);
  assert.equal(fixture.actors.filter((actor) => actor.protected).length, 1);

  assert.deepEqual(fixture.observed_actions, [
    {
      action: "converse",
      actor: "protected_actor",
      availability: "enabled",
    },
  ]);
  assert.equal(
    fixture.actors.some((actor) => actor.id === fixture.observed_actions[0].actor),
    true,
  );
  const protectedActor = fixture.actors.find((actor) => actor.id === "protected_actor");
  assert.equal(protectedActor.protected, true);

  const forbiddenKeys = new Set([
    "color",
    "image",
    "matrix",
    "palette",
    "pixels",
    "quaternion",
    "rotation",
    "scale",
    "shader",
    "transform",
    "translation",
  ]);
  function refusePresentationEscape(value) {
    if (Array.isArray(value)) {
      value.forEach(refusePresentationEscape);
      return;
    }
    if (value !== null && typeof value === "object") {
      for (const [key, child] of Object.entries(value)) {
        assert.equal(forbiddenKeys.has(key), false, `fixture carries forbidden key ${key}`);
        refusePresentationEscape(child);
      }
    }
  }
  refusePresentationEscape(fixture);
}

function clone(value) {
  return structuredClone(value);
}

function buildProbes(source, fixture) {
  const actorRole = clone(source);
  actorRole.actors.push({
    assembly: "visual/observer_silhouette",
    cell: fixture.actors[2].cell,
    id: fixture.actors[2].id,
    role: "protected_interactive",
  });

  const actorFacts = clone(source);
  actorFacts.actors[0].hostile = fixture.actors[0].hostile;
  actorFacts.actors[0].life_state = fixture.actors[0].life_state;
  actorFacts.actors[0].protected = fixture.actors[0].protected;

  const terrainLayers = clone(source);
  terrainLayers.terrain_layers = fixture.terrain_layers;

  const observedActions = clone(source);
  observedActions.observed_actions = fixture.observed_actions;

  return new Map([
    ["actor-facts", actorFacts],
    ["actor-role", actorRole],
    ["observed-actions", observedActions],
    ["terrain-layers", terrainLayers],
  ]);
}

function compilerArguments(sourcePath, planPath) {
  return [
    "--catalog",
    relative(repoRoot, join(substrateDir, "entity-catalog.json")),
    "--facts",
    relative(repoRoot, join(substrateDir, "facts")),
    "--runs",
    relative(repoRoot, join(substrateDir, "runs")),
    "--world",
    relative(repoRoot, join(substrateDir, "world")),
    "--source",
    relative(repoRoot, sourcePath),
    "--out",
    relative(repoRoot, planPath),
  ];
}

function invokeCompiler(args) {
  return spawnSync(compiler, args, {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function runPositiveControl(source) {
  const controlSource = join(outputDir, "control.presentation.json");
  const controlPlan = join(outputDir, "control.plan.json");
  writeFileSync(controlSource, `${JSON.stringify(source, null, 2)}\n`);
  const args = compilerArguments(controlSource, controlPlan);
  const result = invokeCompiler(args);
  assert.equal(result.status, 0, "the unchanged standing source must compile");
  assert.equal(result.signal, null);
  assert.equal(result.stderr, "");
  assert.deepEqual(JSON.parse(result.stdout), {
    command: "render-plan",
    entity_count: 4,
    interaction_count: 3,
    output: relative(repoRoot, controlPlan),
    scenario_count: 5,
    schema: { name: "nomos.rendering_plan", version: 3 },
    status: "completed",
  });
  const planBytes = readFileSync(controlPlan);
  assert.deepEqual(
    planBytes,
    readFileSync(join(substrateDir, "rendering-plan.json")),
    "the positive control must reproduce the verified plan bytes",
  );
  return {
    command: [relative(repoRoot, compiler), ...args],
    plan_sha256: sha256(planBytes),
    status: "completed",
  };
}

function runProbe(name, source) {
  const probePath = join(outputDir, `${name}.presentation.json`);
  const planPath = join(outputDir, `${name}.plan.json`);
  writeFileSync(probePath, `${JSON.stringify(source, null, 2)}\n`);
  const args = compilerArguments(probePath, planPath);
  const result = invokeCompiler(args);
  assert.equal(result.status, 1, `${name} must be refused`);
  assert.equal(result.signal, null, `${name} must exit rather than receive a signal`);
  assert.equal(result.stderr, "", `${name} must use the structured stdout diagnostic`);
  const actual = JSON.parse(result.stdout);
  const expectedPath = join(experimentDir, "expected", `${name}.json`);
  const expectedBytes = readFileSync(expectedPath);
  assert.deepEqual(actual, JSON.parse(expectedBytes));
  return {
    command: [relative(repoRoot, compiler), ...args],
    diagnostic: actual,
    diagnostic_sha256: sha256(expectedBytes),
  };
}

for (const path of [
  compiler,
  join(substrateDir, "entity-catalog.json"),
  join(substrateDir, "facts"),
  join(substrateDir, "runs"),
  join(substrateDir, "world"),
]) {
  try {
    readFileSync(path);
  } catch (error) {
    if (error.code === "EISDIR") continue;
    throw new Error(
      `missing executable-gaol proof substrate at ${relative(repoRoot, path)}; run experiments/executable-gaol/gaol verify first`,
      { cause: error },
    );
  }
}

const fixtureBytes = readFileSync(fixturePath);
const fixture = JSON.parse(fixtureBytes);
validateFixture(fixture);
const source = loadJson(sourcePath);

rmSync(outputDir, { recursive: true, force: true });
mkdirSync(outputDir, { recursive: true });
const positiveControl = runPositiveControl(source);
const probes = {};
for (const [name, probe] of buildProbes(source, fixture)) {
  probes[name] = runProbe(name, probe);
}

const result = {
  classification: "reusable_missing_nomos_capability_candidate",
  fixture_sha256: sha256(fixtureBytes),
  positive_control: positiveControl,
  probes,
  schema: "nomos.experiment.observed_scene_gap_result@1",
  status: "pass",
};
writeFileSync(join(outputDir, "result.json"), `${JSON.stringify(result, null, 2)}\n`);
process.stdout.write(`${JSON.stringify(result)}\n`);
