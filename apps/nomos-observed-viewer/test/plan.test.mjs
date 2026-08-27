import assert from "node:assert/strict";
import test from "node:test";

import { decodePlanBytes } from "../src/plan.mjs";
import { bytesOf, capture, planBytes, planObject } from "./helpers.mjs";

test("strict decoding emits only the frozen render view", () => {
  const view = decodePlanBytes(planBytes(), "plans/scene_one.json");
  assert.deepEqual(Object.keys(view), ["crop", "terrain", "actors", "actions"]);
  assert.equal(Object.isFrozen(view), true);
  assert.equal(view.terrain.length, 3);
  assert.equal(view.actors.length, 5);
  assert.equal(view.actions.length, 2);
  const text = JSON.stringify(view);
  for (const excluded of [
    '"schema"', '"source_sha256"', '"life_state"', '"controlled"', '"hostile"',
    '"protected"', '"availability"', '"role"', '"id"',
  ]) assert.equal(text.includes(excluded), false, excluded);
});

test("all stable semantic decoder codes are directly planted", () => {
  const cases = [
    ["OV0102", Buffer.from("{")],
    ["OV0103", Buffer.concat([Buffer.from(" "), planBytes()])],
    ["OV0104", (() => { const v = planObject(); v.schema = "wrong"; return bytesOf(v); })()],
    ["OV0201", (() => { const v = planObject(); v.actors[0].extra = false; return bytesOf(v); })()],
    ["OV0202", (() => { const v = planObject(); v.crop.height = 0; return bytesOf(v); })()],
    ["OV0203", (() => { const v = planObject(); v.actions[0].target_actor = "missing"; return bytesOf(v); })()],
    ["OV0204", (() => { const v = planObject(); v.actors[0].pose = "prone_dead"; return bytesOf(v); })()],
  ];
  for (const [code, bytes] of cases) {
    const failure = capture(() => decodePlanBytes(bytes, "fixture.json"));
    assert.deepEqual(Object.keys(failure), ["artifact", "code", "message", "path"]);
    assert.equal(failure.code, code);
    assert.equal(failure.artifact, "fixture.json");
  }
});

test("field and lexical bound precedence beat later faults", () => {
  const fields = planObject();
  fields.actors[0].extra = true;
  fields.actions = Array.from({ length: 129 }, (_, index) => ({
    availability: "enabled",
    id: `q${String(index).padStart(3, "0")}`,
    marker: "action/enabled",
    target_actor: "actor_all",
  }));
  assert.equal(capture(() => decodePlanBytes(bytesOf(fields))).code, "OV0201");

  const bounds = planObject();
  bounds.actions = Array.from({ length: 129 }, (_, index) => ({
    availability: "enabled",
    id: `q${String(index).padStart(3, "0")}`,
    marker: "action/enabled",
    target_actor: "actor_all",
  }));
  bounds.crop.height = 99;
  bounds.crop.width = 99;
  const failure = capture(() => decodePlanBytes(bytesOf(bounds)));
  assert.equal(failure.code, "OV0202");
  assert.equal(failure.path, "$.actions");
});

test("bounds, identity, order, reference, and selection phases are strict", () => {
  const boundAndIdentity = planObject();
  boundAndIdentity.terrain_layers[0].cells[1] = { ...boundAndIdentity.terrain_layers[0].cells[0] };
  boundAndIdentity.actions[0].id = "Action_invalid";
  let failure = capture(() => decodePlanBytes(bytesOf(boundAndIdentity)));
  assert.equal(failure.code, "OV0202");
  assert.equal(failure.path, "$.terrain_layers[0].cells[1]");

  const identityAndOrder = planObject();
  identityAndOrder.actions[0].target_actor = "Actor_invalid";
  identityAndOrder.actions[1].id = "Action_later";
  identityAndOrder.terrain_layers[1].id = "a_route_layer";
  failure = capture(() => decodePlanBytes(bytesOf(identityAndOrder)));
  assert.equal(failure.code, "OV0202");
  assert.equal(failure.path, "$.actions[0].target_actor");

  const orderAndReference = planObject();
  orderAndReference.actions.reverse();
  orderAndReference.actions[0].target_actor = "actor_missing";
  orderAndReference.actors[0].pose = "prone_dead";
  failure = capture(() => decodePlanBytes(bytesOf(orderAndReference)));
  assert.equal(failure.code, "OV0202");
  assert.equal(failure.path, "$.actions[1].id");

  const referenceAndSelection = planObject();
  referenceAndSelection.actions[0].target_actor = "actor_missing";
  referenceAndSelection.actors[0].pose = "prone_dead";
  failure = capture(() => decodePlanBytes(bytesOf(referenceAndSelection)));
  assert.equal(failure.code, "OV0203");
  assert.equal(failure.path, "$.actions[0].target_actor");
});

test("identity and order phases walk canonical collection paths", () => {
  const identities = planObject();
  identities.actions[0].target_actor = "Actor_first";
  identities.actions[1].id = "Action_second";
  let failure = capture(() => decodePlanBytes(bytesOf(identities)));
  assert.equal(failure.path, "$.actions[0].target_actor");

  const duplicate = planObject();
  duplicate.actors[1].id = duplicate.actors[0].id;
  failure = capture(() => decodePlanBytes(bytesOf(duplicate)));
  assert.equal(failure.path, "$.actors[1].id");

  const orders = planObject();
  orders.actions.reverse();
  orders.actors.reverse();
  orders.terrain_layers.reverse();
  failure = capture(() => decodePlanBytes(bytesOf(orders)));
  assert.equal(failure.path, "$.actions[1].id");
});

test("every plan field is required and representative wrong types are refused", () => {
  const fields = [
    ["actions", (v) => delete v.actions, "OV0201"],
    ["actors", (v) => delete v.actors, "OV0201"],
    ["crop", (v) => delete v.crop, "OV0201"],
    ["scene", (v) => delete v.scene, "OV0201"],
    ["schema", (v) => delete v.schema, "OV0104"],
    ["source_sha256", (v) => delete v.source_sha256, "OV0104"],
    ["terrain_layers", (v) => delete v.terrain_layers, "OV0201"],
    ...["availability", "id", "marker", "target_actor"].map((key) => [
      `action.${key}`, (v) => delete v.actions[0][key], "OV0201",
    ]),
    ...[
      "assembly", "cell", "controlled", "controlled_marker", "hostile", "hostile_outline",
      "id", "life_state", "pose", "protected", "protection_ring",
    ].map((key) => [`actor.${key}`, (v) => delete v.actors[0][key], "OV0201"]),
    ...["x", "y", "z"].map((key) => [`actor.cell.${key}`, (v) => delete v.actors[0].cell[key], "OV0201"]),
    ...["height", "width"].map((key) => [`crop.${key}`, (v) => delete v.crop[key], "OV0201"]),
    ["scene.id", (v) => delete v.scene.id, "OV0201"],
    ...["assembly", "cells", "id", "material_family", "role", "stack"].map((key) => [
      `terrain.${key}`, (v) => delete v.terrain_layers[0][key], "OV0201",
    ]),
    ...["x", "y"].map((key) => [`terrain.cell.${key}`, (v) => delete v.terrain_layers[0].cells[0][key], "OV0201"]),
  ];
  for (const [label, mutate, code] of fields) {
    const value = planObject();
    mutate(value);
    assert.equal(capture(() => decodePlanBytes(bytesOf(value))).code, code, label);
  }

  const types = [
    ["actions", (v) => { v.actions = {}; }],
    ["action", (v) => { v.actions[0] = false; }],
    ["action.id", (v) => { v.actions[0].id = 1; }],
    ["actor", (v) => { v.actors[0] = false; }],
    ["actor.cell", (v) => { v.actors[0].cell = []; }],
    ["actor.cell.x", (v) => { v.actors[0].cell.x = "0"; }],
    ["actor.controlled", (v) => { v.actors[0].controlled = 0; }],
    ["crop", (v) => { v.crop = []; }],
    ["crop.height", (v) => { v.crop.height = "6"; }],
    ["scene", (v) => { v.scene = []; }],
    ["terrain", (v) => { v.terrain_layers[0] = false; }],
    ["terrain.cells", (v) => { v.terrain_layers[0].cells = {}; }],
    ["terrain.cell", (v) => { v.terrain_layers[0].cells[0] = false; }],
    ["terrain.stack", (v) => { v.terrain_layers[0].stack = "0"; }],
  ];
  for (const [label, mutate] of types) {
    const value = planObject();
    mutate(value);
    assert.equal(capture(() => decodePlanBytes(bytesOf(value))).code, "OV0201", label);
  }
});

test("every finite plan enum rejects an unknown spelling in the field phase", () => {
  const mutations = [
    ["action availability", (v) => { v.actions[0].availability = "unknown"; }],
    ["action marker", (v) => { v.actions[0].marker = "action/unknown"; }],
    ["actor assembly", (v) => { v.actors[0].assembly = "actor/unknown"; }],
    ["controlled marker", (v) => { v.actors[0].controlled_marker = "unknown"; }],
    ["hostile outline", (v) => { v.actors[0].hostile_outline = "unknown"; }],
    ["life state", (v) => { v.actors[0].life_state = "unknown"; }],
    ["pose", (v) => { v.actors[0].pose = "unknown"; }],
    ["protection ring", (v) => { v.actors[0].protection_ring = "unknown"; }],
    ["terrain assembly", (v) => { v.terrain_layers[0].assembly = "terrain/unknown"; }],
    ["material family", (v) => { v.terrain_layers[0].material_family = "unknown"; }],
    ["terrain role", (v) => { v.terrain_layers[0].role = "unknown"; }],
  ];
  for (const [label, mutate] of mutations) {
    const value = planObject();
    mutate(value);
    const failure = capture(() => decodePlanBytes(bytesOf(value)));
    assert.equal(failure.code, "OV0201", label);
  }
});

test("every lower and upper semantic bound is planted", () => {
  const bounds = [
    ["action upper", (v) => { v.actions = Array.from({ length: 129 }, (_, i) => ({ ...v.actions[0], id: `q${String(i).padStart(3, "0")}` })); }],
    ["actor lower", (v) => { v.actors = []; }],
    ["actor upper", (v) => { v.actors = Array.from({ length: 65 }, (_, i) => ({ ...structuredClone(v.actors[0]), id: `a${String(i).padStart(2, "0")}` })); }],
    ["crop height lower", (v) => { v.crop.height = 0; }],
    ["crop height upper", (v) => { v.crop.height = 33; }],
    ["crop width lower", (v) => { v.crop.width = 0; }],
    ["crop width upper", (v) => { v.crop.width = 33; }],
    ["layer lower", (v) => { v.terrain_layers.length = 2; }],
    ["layer upper", (v) => { v.terrain_layers = Array.from({ length: 9 }, (_, i) => ({ ...structuredClone(v.terrain_layers[i % 3]), id: `layer_${i}` })); }],
    ["cell lower", (v) => { v.terrain_layers[0].cells = []; }],
    ["cell upper", (v) => { v.terrain_layers[0].cells = Array.from({ length: 1025 }, () => ({ x: 0, y: 0 })); }],
    ["actor x lower", (v) => { v.actors[0].cell.x = -1; }],
    ["actor x upper", (v) => { v.actors[0].cell.x = v.crop.width; }],
    ["actor y lower", (v) => { v.actors[0].cell.y = -1; }],
    ["actor y upper", (v) => { v.actors[0].cell.y = v.crop.height; }],
    ["actor z lower", (v) => { v.actors[0].cell.z = -1; }],
    ["actor z upper", (v) => { v.actors[0].cell.z = 1; }],
    ["terrain x lower", (v) => { v.terrain_layers[0].cells[0].x = -1; }],
    ["terrain x upper", (v) => { v.terrain_layers[0].cells[0].x = v.crop.width; }],
    ["terrain y lower", (v) => { v.terrain_layers[0].cells[0].y = -1; }],
    ["terrain y upper", (v) => { v.terrain_layers[0].cells[0].y = v.crop.height; }],
    ["stack signed upper", (v) => { v.terrain_layers[0].stack = 1n << 63n; }],
  ];
  for (const [label, mutate] of bounds) {
    const value = planObject();
    mutate(value);
    assert.equal(capture(() => decodePlanBytes(bytesOf(value))).code, "OV0202", label);
  }

  const total = planObject();
  const allCells = Array.from({ length: 1024 }, (_, index) => ({ x: index % 32, y: Math.floor(index / 32) }));
  total.crop = { height: 32, width: 32 };
  total.terrain_layers = Array.from({ length: 5 }, (_, index) => ({
    ...structuredClone(total.terrain_layers[index % 3]),
    cells: structuredClone(allCells),
    id: `layer_${index}`,
  }));
  assert.equal(capture(() => decodePlanBytes(bytesOf(total))).code, "OV0202", "total cell upper");
});

test("every identity, uniqueness, order, and reference rule is planted", () => {
  const faults = [
    ["action identity", (v) => { v.actions[0].id = "Action"; }, "OV0202"],
    ["action target identity", (v) => { v.actions[0].target_actor = "Actor"; }, "OV0202"],
    ["actor identity", (v) => { v.actors[0].id = "Actor"; }, "OV0202"],
    ["scene identity", (v) => { v.scene.id = "Scene"; }, "OV0202"],
    ["layer identity", (v) => { v.terrain_layers[0].id = "Layer"; }, "OV0202"],
    ["action uniqueness", (v) => { v.actions[1].id = v.actions[0].id; }, "OV0202"],
    ["actor uniqueness", (v) => { v.actors[1].id = v.actors[0].id; }, "OV0202"],
    ["layer uniqueness", (v) => { v.terrain_layers[1].id = v.terrain_layers[0].id; }, "OV0202"],
    ["action order", (v) => { v.actions.reverse(); }, "OV0202"],
    ["actor order", (v) => { v.actors.reverse(); }, "OV0202"],
    ["layer order", (v) => { v.terrain_layers.reverse(); }, "OV0202"],
    ["cell order", (v) => { v.terrain_layers[0].cells.reverse(); }, "OV0202"],
    ["reference", (v) => { v.actions[0].target_actor = "missing_actor"; }, "OV0203"],
  ];
  for (const [label, mutate, code] of faults) {
    const value = planObject();
    mutate(value);
    assert.equal(capture(() => decodePlanBytes(bytesOf(value))).code, code, label);
  }
});

test("every copied selection disagreement is refused", () => {
  const mutations = [
    (v) => { v.terrain_layers[0].assembly = "terrain/traversable_route"; },
    (v) => { v.terrain_layers[0].material_family = "route_worn"; },
    (v) => { v.terrain_layers[0].stack = 10; },
    (v) => { v.actors[0].pose = "prone_dead"; },
    (v) => { v.actors[0].controlled_marker = "absent"; },
    (v) => { v.actors[0].hostile_outline = "absent"; },
    (v) => { v.actors[0].protection_ring = "absent"; },
    (v) => { v.actions[0].marker = "action/enabled"; },
  ];
  for (const mutate of mutations) {
    const value = planObject();
    mutate(value);
    assert.equal(capture(() => decodePlanBytes(bytesOf(value))).code, "OV0204");
  }
});
