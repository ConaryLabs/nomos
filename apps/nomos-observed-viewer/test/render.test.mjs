import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { decodePlanBytes } from "../src/plan.mjs";
import { buildScene, cameraFrame, publicSceneGraph, renderView } from "../src/render.mjs";
import { app, bytesOf, planBytes, planObject } from "./helpers.mjs";
import { makeThree } from "./three-stub.mjs";

test("three-role overlap and every consequence survive in the scene graph", () => {
  const view = decodePlanBytes(planBytes());
  const built = buildScene(makeThree(), view);
  const graph = publicSceneGraph(built.scene);
  const kinds = graph.map((row) => row.kind);
  assert.equal(kinds.filter((kind) => kind === "terrain/calm_ground").length, 3);
  assert.equal(kinds.filter((kind) => kind === "terrain/traversable_route").length, 3);
  assert.equal(kinds.filter((kind) => kind === "terrain/structure_footprint").length, 3);
  assert.equal(kinds.filter((kind) => kind === "controlled-marker").length, 2);
  assert.equal(kinds.filter((kind) => kind === "hostile-outline").length, 2);
  assert.equal(kinds.filter((kind) => kind === "protection-ring").length, 2);
  assert.equal(kinds.filter((kind) => kind === "action/enabled").length, 1);
  assert.equal(kinds.filter((kind) => kind === "action/disabled").length, 1);
  assert.deepEqual(built.counts, {
    actions: 2,
    actors: 5,
    controlled_markers: 2,
    hostile_outlines: 2,
    protection_rings: 2,
    terrain_cells: 9,
    terrain_layers: 3,
  });

  const terrainGroup = built.scene.children[0];
  const overlap = terrainGroup.children
    .filter((group) => group.children[0].position.x === 2 && group.children[0].position.z === 2)
    .map((group) => ({
      kind: group.children[0].userData.kind,
      height: group.children[0].position.y,
    }));
  assert.deepEqual(overlap, [
    { kind: "terrain/calm_ground", height: 0.04 },
    { kind: "terrain/traversable_route", height: 0.07 },
    { kind: "terrain/structure_footprint", height: 0.22999999999999998 },
  ]);

  const accent = terrainGroup.children[0].children[1];
  assert.equal(accent.userData.kind, "terrain-accent");
  assert.deepEqual([accent.position.x, accent.position.y, accent.position.z], [0, 0.08600000000000001, 0]);

  const actorGroup = built.scene.children[1];
  const kindsAt = (x, z) => {
    const actor = actorGroup.children.find((node) => node.position.x === x && node.position.z === z);
    const selected = [];
    actor.traverse((node) => selected.push(node.userData.kind));
    return selected;
  };
  assert.deepEqual(kindsAt(1, 1).filter((kind) => kind?.includes("marker") || kind?.includes("outline") || kind?.includes("ring")), ["controlled-marker"]);
  assert.deepEqual(kindsAt(3, 1).filter((kind) => kind?.includes("marker") || kind?.includes("outline") || kind?.includes("ring")), ["hostile-outline"]);
  assert.deepEqual(kindsAt(1, 4).filter((kind) => kind?.includes("marker") || kind?.includes("outline") || kind?.includes("ring")), ["protection-ring"]);
  assert.deepEqual(kindsAt(2, 2).filter((kind) => kind?.includes("marker") || kind?.includes("outline") || kind?.includes("ring")), [
    "controlled-marker", "hostile-outline", "protection-ring",
  ]);
  assert.ok(kindsAt(4, 3).includes("prone_dead"));
  assert.ok(kindsAt(2, 2).includes("upright_living"));

  const actionGroup = built.scene.children[2];
  assert.deepEqual(actionGroup.children.map((node) => [node.userData.kind, node.position.x, node.position.y, node.position.z]), [
    ["action/enabled", 2, 1.12, 2],
    ["action/disabled", 4, 1.12, 3],
  ]);

  const directional = built.scene.children.find((node) => node.userData.kind === "directional-light");
  assert.deepEqual([directional.position.x, directional.position.y, directional.position.z], [8.5, 10, -5.5]);
  assert.deepEqual(
    [directional.target.position.x, directional.target.position.y, directional.target.position.z],
    [2.5, 0, 2.5],
  );
});

test("camera is orthographic isometric with one-cell margin and aspect expansion", () => {
  const frame = cameraFrame({ width: 6, height: 6 });
  assert.equal((frame.right - frame.left) / (frame.top - frame.bottom), 1280 / 720);
  for (const [x, y, z] of frame.corners) {
    const dx = x - frame.target[0];
    const dy = y - frame.target[1];
    const dz = z - frame.target[2];
    const horizontal = (dx - dz) / Math.sqrt(2);
    const vertical = (-dx + 2 * dy - dz) / Math.sqrt(6);
    assert.ok(horizontal >= frame.left + 1 - 1e-12 && horizontal <= frame.right - 1 + 1e-12);
    assert.ok(vertical >= frame.bottom + 1 - 1e-12 && vertical <= frame.top - 1 + 1e-12);
  }
  const built = buildScene(makeThree(), decodePlanBytes(planBytes()));
  assert.equal(built.camera.type, "OrthographicCamera");
});

test("renderer can read only the render view and exposes no private handles", () => {
  const view = decodePlanBytes(planBytes());
  const excluded = new Set(["id", "role", "life_state", "controlled", "hostile", "protected", "availability", "schema", "source_sha256"]);
  const wrap = (value) => {
    if (!value || typeof value !== "object") return value;
    return new Proxy(value, {
      get(target, property, receiver) {
        if (excluded.has(property)) throw new Error(`raw field read: ${property}`);
        const selected = Reflect.get(target, property, receiver);
        return typeof selected === "object" && selected !== null ? wrap(selected) : selected;
      },
    });
  };
  const built = buildScene(makeThree(), wrap(structuredClone(view)));
  assert.equal(JSON.stringify(publicSceneGraph(built.scene)).includes("handle"), false);
});

test("renaming all identities and references cannot change the public graph", () => {
  const original = planObject();
  const renamed = structuredClone(original);
  renamed.scene.id = "renamed_scene";
  const layerNames = ["z_layer", "m_layer", "a_layer"];
  renamed.terrain_layers.forEach((row, index) => { row.id = layerNames[index]; });
  renamed.terrain_layers.sort((a, b) => a.id.localeCompare(b.id));
  const actorNames = new Map();
  const reversedActorNames = ["z_actor", "y_actor", "x_actor", "w_actor", "v_actor"];
  renamed.actors.forEach((row, index) => {
    actorNames.set(row.id, reversedActorNames[index]);
    row.id = reversedActorNames[index];
  });
  renamed.actors.sort((a, b) => a.id.localeCompare(b.id));
  const actionNames = ["z_action", "a_action"];
  renamed.actions.forEach((row, index) => {
    row.id = actionNames[index];
    row.target_actor = actorNames.get(row.target_actor);
  });
  renamed.actions.sort((a, b) => a.id.localeCompare(b.id));
  const before = publicSceneGraph(buildScene(makeThree(), decodePlanBytes(bytesOf(original))).scene);
  const after = publicSceneGraph(buildScene(makeThree(), decodePlanBytes(bytesOf(renamed))).scene);
  assert.deepEqual(after, before);
});

test("same-target actions are centred siblings and never actor children", () => {
  const value = planObject();
  value.actions[0].target_actor = "actor_all";
  const built = buildScene(makeThree(), decodePlanBytes(bytesOf(value)));
  const actionGroup = built.scene.children[2];
  assert.deepEqual(actionGroup.children.map((node) => [node.userData.kind, node.position.x]), [
    ["action/disabled", 1.88],
    ["action/enabled", 2.12],
  ]);
  const actorGroup = built.scene.children[1];
  assert.equal(actorGroup.children.some((actor) => actor.children.some((node) => node.userData.kind?.startsWith("action/"))), false);
});

test("identical visual actor tuples use handles only for target association", () => {
  const value = planObject();
  const base = structuredClone(value.actors[0]);
  value.actors = [
    { ...structuredClone(base), id: "actor_a" },
    { ...structuredClone(base), id: "actor_b" },
  ];
  value.actions = [
    { availability: "disabled", id: "action_a", marker: "action/disabled", target_actor: "actor_a" },
    { availability: "enabled", id: "action_b", marker: "action/enabled", target_actor: "actor_b" },
  ];
  const view = decodePlanBytes(bytesOf(value));
  assert.deepEqual(view.actors.map((row) => row.handle), [0, 1]);
  assert.deepEqual(view.actions.map((row) => row.marker), ["action/disabled", "action/enabled"]);
  const graph = publicSceneGraph(buildScene(makeThree(), view).scene);
  assert.equal(JSON.stringify(graph).includes("target_handle"), false);
});

test("real render entrypoint uses the fixed viewport and one completed render", () => {
  const THREE = makeThree();
  const container = { replaceChildren(node) { this.child = node; } };
  const rendered = renderView(THREE, container, decodePlanBytes(planBytes()));
  assert.deepEqual(rendered.renderer.size, [1280, 720, false]);
  assert.equal(rendered.renderer.rendered.length, 2);
  assert.equal(container.child.tagName, "CANVAS");
});

test("renderer, UI, and catalog source do not access excluded raw fields", () => {
  for (const path of ["src/catalog.mjs", "src/render.mjs", "src/ui.mjs"]) {
    const source = readFileSync(`${app}/${path}`, "utf8");
    for (const raw of ["id", "role", "life_state", "controlled", "hostile", "protected", "availability", "schema", "source_sha256"]) {
      assert.doesNotMatch(source, new RegExp(`\\.${raw}\\b|[\\"']${raw}[\\"']\\s*:`), `${path}: ${raw}`);
    }
  }
});
