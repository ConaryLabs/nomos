// The scene graph the plan asks for.
//
// The study could only assert its renderer's source text
// (`experiments/executable-gaol/src/webgl-viewer.test.mjs`), because a CDN
// import cannot be loaded in node. Passing the namespace in turns those into
// assertions about what is actually built.

import test from "node:test";
import assert from "node:assert/strict";

import { LOOK_PROFILES, PALETTE, VERTICAL_SCALE, cellsOf, resolveSocket } from "../src/catalog.mjs";
import { decodePlan } from "../src/plan.mjs";
import { createGaolRenderer } from "../src/render.mjs";
import { census, makeHost, makeThree, meshesOf } from "./three-stub.mjs";
import { facingPlan, hallPlan } from "./fixtures.mjs";

const hall = decodePlan(hallPlan());

const present = (plan, scenarioId, options = {}) => {
  const three = makeThree();
  const { container, host } = makeHost();
  const renderer = createGaolRenderer(container, three, host);
  renderer.present(plan, scenarioId, options.forensic ?? false, options.presentation ?? {});
  return { three, renderer, world: renderer.worldRoot() };
};

test("the scene graph matches the plan", () => {
  const { world, three, renderer } = present(hall, "01-sealed");
  const { width, height } = hall.architecture.bounds;

  // One floor box per cell.
  const floors = meshesOf(world, "BoxGeometry").filter((mesh) => mesh.geometry.args[1] === 0.12);
  assert.ok(floors.length >= width * height);

  // The renderer took the canvas and the camera frames the room.
  assert.equal(container_of(renderer), "CANVAS");
  assert.equal(three.rendered.length > 0, true, "the first frame was drawn");
});

const container_of = (renderer) => renderer.renderer.domElement.tagName;

test("a wall opens exactly where a door declares its face", () => {
  // The hall has two north doors, at x = 1 and x = 2, and a wall four cells
  // wide, so two of the four north columns are open.
  const { world } = present(hall, "01-sealed");
  const capstones = meshesOf(world, "BoxGeometry").filter(
    (mesh) => mesh.geometry.args[0] === 1.02 && mesh.geometry.args[1] === 0.12,
  );
  const northCaps = capstones.filter((mesh) => mesh.position.z < 0);
  assert.equal(northCaps.length, 2, "two of four north columns carry a wall");
  const openColumns = northCaps.map((mesh) => mesh.position.x).sort();
  assert.deepEqual(openColumns, [-1.5, 1.5], "the walls are the columns with no door");
});

test("a destroyed door draws wreckage and a sealed ward draws its ring", () => {
  const sealed = present(hall, "01-sealed").world;
  // Intact and locked: five bars and a rail, no wreckage, ward ring present.
  const sealedBars = meshesOf(sealed, "BoxGeometry").filter(
    (mesh) => mesh.geometry.args[0] === 0.065,
  );
  assert.equal(sealedBars.length, 10, "five bars on each of the two doors");
  const rings = meshesOf(sealed, "TorusGeometry").filter((mesh) => mesh.geometry.args[0] === 0.42);
  assert.equal(rings.length, 2, "both doors are sealed at the start");
  assert.equal(census(sealed).get("LineLoop"), 2, "each sealed ward draws its diamond");

  const breached = present(hall, "02-unsealed").world;
  const wreckage = meshesOf(breached, "BoxGeometry").filter(
    (mesh) => mesh.geometry.args[0] === 0.075,
  );
  assert.equal(wreckage.length, 3, "the destroyed door is three broken bars");
  const breachedRings = meshesOf(breached, "TorusGeometry").filter(
    (mesh) => mesh.geometry.args[0] === 0.42,
  );
  assert.equal(breachedRings.length, 1, "only the untouched door keeps its ward");
});

test("an extinguished brazier has no flame and no light", () => {
  const lit = present(hall, "01-sealed").world;
  assert.equal(meshesOf(lit, "ConeGeometry").filter((mesh) => mesh.geometry.args[1] === 0.55).length, 1);
  const litLights = [];
  lit.traverse((node) => {
    if (node.type === "PointLight" && node.args[0] === PALETTE.amber) litLights.push(node);
  });
  assert.equal(litLights.length, 1);

  const dark = present(hall, "03-dark").world;
  assert.equal(meshesOf(dark, "ConeGeometry").filter((mesh) => mesh.geometry.args[1] === 0.55).length, 0);
  const darkLights = [];
  dark.traverse((node) => {
    if (node.type === "PointLight" && node.args[0] === PALETTE.amber) darkLights.push(node);
  });
  assert.equal(darkLights.length, 0);
  // The pedestal and bowl stay: the brazier is still there, it is just out.
  assert.equal(meshesOf(dark, "CylinderGeometry").length, 2);
});

test("the crescent sits at the resolved socket, and only while the ward is sealed", () => {
  const sealed = present(hall, "01-sealed").world;
  const crescents = meshesOf(sealed, "TorusGeometry").filter(
    (mesh) => mesh.geometry.args[0] === 0.38,
  );
  assert.equal(crescents.length, 1);
  const gate = hall.entities.find((one) => one.id === "hall_gate");
  const socket = resolveSocket(gate, "ward");
  const { width, height } = hall.architecture.bounds;
  assert.deepEqual(
    { x: crescents[0].position.x, y: crescents[0].position.y, z: crescents[0].position.z },
    { x: socket.x - width / 2, y: socket.z * VERTICAL_SCALE, z: socket.y - height / 2 },
  );
  // All three components are honoured: a renderer that dropped the socket's
  // elevation would be the wall-height double authority again in a new field.
  assert.ok(crescents[0].position.y > 1.2);

  const unsealed = present(hall, "02-unsealed").world;
  assert.equal(
    meshesOf(unsealed, "TorusGeometry").filter((mesh) => mesh.geometry.args[0] === 0.38).length,
    0,
  );
});

test("a socket follows its door around the four faces", () => {
  for (const direction of ["north", "south", "east", "west"]) {
    const plan = decodePlan(facingPlan(direction));
    const world = present(plan, "01-shut").world;
    const crescent = meshesOf(world, "TorusGeometry").find((mesh) => mesh.geometry.args[0] === 0.38);
    const gate = plan.entities.find((one) => one.id === "yard_gate");
    const socket = resolveSocket(gate, "ward");
    const { width, height } = plan.architecture.bounds;
    assert.equal(crescent.position.x, socket.x - width / 2, `${direction} x`);
    assert.equal(crescent.position.z, socket.y - height / 2, `${direction} z`);
  }
});

test("actors are placed at their cells", () => {
  const { world } = present(hall, "01-sealed");
  // Two actors, and the procedural profile gives each mesh a silhouette that
  // shares its geometry, so each head appears twice.
  const heads = meshesOf(world, "IcosahedronGeometry");
  assert.equal(heads.length, 4);
  const bodies = heads.filter((mesh) => mesh.material.side !== "BackSide");
  assert.equal(bodies.length, 2);
  // Each actor is a group placed on its cell, with its parts in local space.
  const actorGroups = [];
  world.traverse((node) => {
    if (node.type === "Group" && node.children.some((one) => one.geometry?.type === "IcosahedronGeometry")) {
      actorGroups.push(node);
    }
  });
  assert.equal(actorGroups.length, 2);
  assert.deepEqual(
    actorGroups.map((group) => [group.position.x, group.position.z]).sort(),
    // The player at (0,2) and the gaoler at (3,2), centred on their cells in a
    // four-by-three room.
    [
      [-1.5, 1],
      [1.5, 1],
    ].sort(),
  );
  const cloaks = meshesOf(world, "ConeGeometry").filter((mesh) => mesh.geometry.args[1] !== 0.55);
  assert.equal(cloaks.length, 4);
  // The player carries a blade and the gaoler a shoulder guard, chosen by the
  // assembly the plan declares.
  const blades = meshesOf(world, "BoxGeometry").filter((mesh) => mesh.geometry.args[0] === 0.055);
  const shoulders = meshesOf(world, "BoxGeometry").filter((mesh) => mesh.geometry.args[0] === 0.72);
  assert.equal(blades.length, 1);
  assert.equal(shoulders.length, 1);
});

test("presentation positions move the actors without rebuilding", () => {
  const three = makeThree();
  const { container, host } = makeHost();
  const renderer = createGaolRenderer(container, three, host);
  renderer.present(hall, "01-sealed", false, {});
  const before = renderer.worldRoot();
  renderer.present(hall, "01-sealed", false, {
    actorPositions: { player: { x: 2, y: 2, z: 0.08 }, gaoler: { x: 3, y: 2, z: 0 } },
  });
  assert.equal(renderer.worldRoot(), before, "the same scenario does not rebuild the world");
});

test("switching look profiles rebuilds the world", () => {
  const three = makeThree();
  const { container, host } = makeHost();
  const renderer = createGaolRenderer(container, three, host);
  renderer.present(hall, "01-sealed", false, {});
  const before = renderer.worldRoot();
  assert.equal(renderer.lookProfile(), LOOK_PROFILES.procedural);

  renderer.setLookProfile("baseline");
  assert.equal(renderer.lookProfile(), LOOK_PROFILES.baseline);
  renderer.present(hall, "01-sealed", false, {});
  assert.notEqual(renderer.worldRoot(), before, "a new look rebuilds");
  // The baseline profile has no bevel, so the masses are plain boxes.
  const beveled = meshesOf(renderer.worldRoot(), "ExtrudeGeometry");
  assert.equal(beveled.length, 0);
  assert.equal(renderer.renderer.toneMappingExposure, LOOK_PROFILES.baseline.exposure);

  renderer.setLookProfile("procedural");
  renderer.present(hall, "01-sealed", false, {});
  assert.ok(meshesOf(renderer.worldRoot(), "ExtrudeGeometry").length > 0);
  assert.throws(() => renderer.setLookProfile("cinematic"), /unknown look profile/);
});

test("the forensic overlay is a grid and nothing else", () => {
  const three = makeThree();
  const { container, host } = makeHost();
  const renderer = createGaolRenderer(container, three, host);
  renderer.present(hall, "01-sealed", false, {});
  assert.equal(census(renderer.worldRoot()).get("GridHelper"), undefined);
  renderer.present(hall, "01-sealed", true, {});
  assert.equal(census(renderer.worldRoot()).get("GridHelper"), 1);
  renderer.present(hall, "01-sealed", false, {});
  assert.equal(census(renderer.worldRoot()).get("GridHelper"), undefined);
});

test("wall and mass heights come through the step conversion", () => {
  const { world } = present(hall, "01-sealed");
  // The pillar is 20 steps: two cells, scaled into world units.
  const massHeight = cellsOf(20) * VERTICAL_SCALE;
  const pillars = meshesOf(world, "ExtrudeGeometry").filter(
    (mesh) => Math.abs(mesh.position.y - massHeight / 2) < 1e-9,
  );
  assert.equal(pillars.length, 1);
});
