import { CanonicalFailure, deepFreeze, encodeCanonical, parseCanonical } from "./canonical.mjs";

const SCHEMA = "nomos.observed_scene_plan@1";
const ID = /^[a-z][a-z0-9_]{0,63}$/;
const DIGEST = /^[0-9a-f]{64}$/;
const ROLES = new Map([
  ["calm_ground", ["terrain/calm_ground", "ground_muted", 0]],
  ["traversable_route", ["terrain/traversable_route", "route_worn", 10]],
  ["structure_footprint", ["terrain/structure_footprint", "structure_stone", 20]],
]);
const POSES = new Map([
  ["living", "upright_living"],
  ["dead", "prone_dead"],
]);
const PRESENCE = new Map([
  [true, "present"],
  [false, "absent"],
]);
const MARKERS = new Map([
  ["enabled", "action/enabled"],
  ["disabled", "action/disabled"],
]);
const TERRAIN_ASSEMBLIES = [...ROLES.values()].map(([assembly]) => assembly);
const MATERIAL_FAMILIES = [...ROLES.values()].map(([, material]) => material);
const ACTOR_ASSEMBLIES = ["actor/observed_figure"];
const POSE_SELECTIONS = [...POSES.values()];
const PRESENCE_SELECTIONS = [...new Set(PRESENCE.values())];
const ACTION_MARKERS = [...MARKERS.values()];
const MIN_I64 = -(1n << 63n);
const MAX_I64 = (1n << 63n) - 1n;

export const reject = (artifact, code, path, message) =>
  Object.freeze({ artifact, code, message, path });

const fail = (artifact, code, path, message) => {
  throw reject(artifact, code, path, message);
};

const object = (value, artifact, path) => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    fail(artifact, "OV0201", path, "expected object");
  }
  return value;
};

const exact = (value, keys, artifact, path) => {
  object(value, artifact, path);
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    const first = [...new Set([...actual, ...expected])].sort().find(
      (key) => actual.includes(key) !== expected.includes(key),
    );
    fail(artifact, "OV0201", first ? `${path}.${first}` : path, "field set is not exact");
  }
};

const array = (value, artifact, path) => {
  if (!Array.isArray(value)) fail(artifact, "OV0201", path, "expected array");
  return value;
};

const string = (value, artifact, path) => {
  if (typeof value !== "string") fail(artifact, "OV0201", path, "expected string");
  return value;
};

const boolean = (value, artifact, path) => {
  if (typeof value !== "boolean") fail(artifact, "OV0201", path, "expected boolean");
  return value;
};

const boundedInteger = (value, low, high, artifact, path) => {
  if (typeof value !== "bigint") fail(artifact, "OV0201", path, "expected integer");
  if (value < BigInt(low) || value > BigInt(high)) {
    fail(artifact, "OV0202", path, `integer must be in ${low}..=${high}`);
  }
  return Number(value);
};

const identity = (value, artifact, path) => {
  const id = string(value, artifact, path);
  if (!ID.test(id)) fail(artifact, "OV0202", path, "invalid local identity");
  return id;
};

const sortedUnique = (rows, field, artifact, path) => {
  let prior = null;
  rows.forEach((row, index) => {
    const current = row[field];
    if (prior !== null && current <= prior) {
      fail(artifact, "OV0202", `${path}[${index}].${field}`, "rows must be strictly ID-sorted");
    }
    prior = current;
  });
};

const enumValue = (value, choices, artifact, path) => {
  const selected = string(value, artifact, path);
  if (!choices.includes(selected)) fail(artifact, "OV0201", path, "unknown enum value");
  return selected;
};

const integerType = (value, artifact, path) => {
  if (typeof value !== "bigint") fail(artifact, "OV0201", path, "expected integer");
};

const validateShape = (document, artifact) => {
  const actions = array(document.actions, artifact, "$.actions");
  actions.forEach((row, index) => {
    const path = `$.actions[${index}]`;
    exact(row, ["availability", "id", "marker", "target_actor"], artifact, path);
    enumValue(row.availability, [...MARKERS.keys()], artifact, `${path}.availability`);
    string(row.id, artifact, `${path}.id`);
    enumValue(row.marker, ACTION_MARKERS, artifact, `${path}.marker`);
    string(row.target_actor, artifact, `${path}.target_actor`);
  });
  const actors = array(document.actors, artifact, "$.actors");
  actors.forEach((row, index) => {
    const path = `$.actors[${index}]`;
    exact(
      row,
      ["assembly", "cell", "controlled", "controlled_marker", "hostile", "hostile_outline", "id", "life_state", "pose", "protected", "protection_ring"],
      artifact,
      path,
    );
    enumValue(row.assembly, ACTOR_ASSEMBLIES, artifact, `${path}.assembly`);
    exact(row.cell, ["x", "y", "z"], artifact, `${path}.cell`);
    integerType(row.cell.x, artifact, `${path}.cell.x`);
    integerType(row.cell.y, artifact, `${path}.cell.y`);
    integerType(row.cell.z, artifact, `${path}.cell.z`);
    boolean(row.controlled, artifact, `${path}.controlled`);
    enumValue(row.controlled_marker, PRESENCE_SELECTIONS, artifact, `${path}.controlled_marker`);
    boolean(row.hostile, artifact, `${path}.hostile`);
    enumValue(row.hostile_outline, PRESENCE_SELECTIONS, artifact, `${path}.hostile_outline`);
    string(row.id, artifact, `${path}.id`);
    enumValue(row.life_state, [...POSES.keys()], artifact, `${path}.life_state`);
    enumValue(row.pose, POSE_SELECTIONS, artifact, `${path}.pose`);
    boolean(row.protected, artifact, `${path}.protected`);
    enumValue(row.protection_ring, PRESENCE_SELECTIONS, artifact, `${path}.protection_ring`);
  });
  exact(document.crop, ["height", "width"], artifact, "$.crop");
  integerType(document.crop.height, artifact, "$.crop.height");
  integerType(document.crop.width, artifact, "$.crop.width");
  exact(document.scene, ["id"], artifact, "$.scene");
  string(document.scene.id, artifact, "$.scene.id");
  const terrain = array(document.terrain_layers, artifact, "$.terrain_layers");
  terrain.forEach((row, index) => {
    const path = `$.terrain_layers[${index}]`;
    exact(row, ["assembly", "cells", "id", "material_family", "role", "stack"], artifact, path);
    enumValue(row.assembly, TERRAIN_ASSEMBLIES, artifact, `${path}.assembly`);
    const cells = array(row.cells, artifact, `${path}.cells`);
    cells.forEach((cell, cellIndex) => {
      const cellPath = `${path}.cells[${cellIndex}]`;
      exact(cell, ["x", "y"], artifact, cellPath);
      integerType(cell.x, artifact, `${cellPath}.x`);
      integerType(cell.y, artifact, `${cellPath}.y`);
    });
    string(row.id, artifact, `${path}.id`);
    enumValue(row.material_family, MATERIAL_FAMILIES, artifact, `${path}.material_family`);
    enumValue(row.role, [...ROLES.keys()], artifact, `${path}.role`);
    integerType(row.stack, artifact, `${path}.stack`);
  });
  return { actions, actors, terrain };
};

const outside = (value, low, high) => value < BigInt(low) || value > BigInt(high);

const validateBounds = ({ actions, actors, terrain }, document, artifact) => {
  if (actions.length > 128) fail(artifact, "OV0202", "$.actions", "invalid action count");
  if (actors.length < 1 || actors.length > 64) fail(artifact, "OV0202", "$.actors", "invalid actor count");
  actors.forEach((row, index) => {
    const path = `$.actors[${index}].cell`;
    if (outside(row.cell.x, 0, document.crop.width - 1n)) fail(artifact, "OV0202", `${path}.x`, "actor x is outside crop");
    if (outside(row.cell.y, 0, document.crop.height - 1n)) fail(artifact, "OV0202", `${path}.y`, "actor y is outside crop");
    if (row.cell.z !== 0n) fail(artifact, "OV0202", `${path}.z`, "actor z must be zero");
  });
  if (outside(document.crop.height, 1, 32)) fail(artifact, "OV0202", "$.crop.height", "invalid crop height");
  if (outside(document.crop.width, 1, 32)) fail(artifact, "OV0202", "$.crop.width", "invalid crop width");
  if (terrain.length < 3 || terrain.length > 8) fail(artifact, "OV0202", "$.terrain_layers", "invalid layer count");
  let total = 0;
  const roles = new Set();
  terrain.forEach((row, index) => {
    const path = `$.terrain_layers[${index}]`;
    if (row.cells.length < 1 || row.cells.length > 1024) fail(artifact, "OV0202", `${path}.cells`, "invalid cell count");
    total += row.cells.length;
    roles.add(row.role);
    const cells = new Set();
    row.cells.forEach((cell, cellIndex) => {
      if (outside(cell.x, 0, document.crop.width - 1n)) fail(artifact, "OV0202", `${path}.cells[${cellIndex}].x`, "cell x is outside crop");
      if (outside(cell.y, 0, document.crop.height - 1n)) fail(artifact, "OV0202", `${path}.cells[${cellIndex}].y`, "cell y is outside crop");
      const key = `${cell.x},${cell.y}`;
      if (cells.has(key)) fail(artifact, "OV0202", `${path}.cells[${cellIndex}]`, "duplicate terrain cell");
      cells.add(key);
    });
    if (row.stack < MIN_I64 || row.stack > MAX_I64) {
      fail(artifact, "OV0202", `${path}.stack`, "stack is outside the signed 64-bit range");
    }
  });
  if (total < 3 || total > 4096) fail(artifact, "OV0202", "$.terrain_layers", "invalid total cell count");
  if ([...ROLES.keys()].some((role) => !roles.has(role))) {
    fail(artifact, "OV0202", "$.terrain_layers", "all roles are required");
  }
};

const validateIdentityCollection = (rows, artifact, path, actionTargets = false) => {
  const seen = new Set();
  rows.forEach((row, index) => {
    const rowPath = `${path}[${index}]`;
    if (!ID.test(row.id)) fail(artifact, "OV0202", `${rowPath}.id`, "invalid local identity");
    if (seen.has(row.id)) fail(artifact, "OV0202", `${rowPath}.id`, "duplicate local identity");
    seen.add(row.id);
    if (actionTargets && !ID.test(row.target_actor)) {
      fail(artifact, "OV0202", `${rowPath}.target_actor`, "invalid target identity");
    }
  });
};

const validateIdentities = ({ actions, actors, terrain }, document, artifact) => {
  validateIdentityCollection(actions, artifact, "$.actions", true);
  validateIdentityCollection(actors, artifact, "$.actors");
  if (!ID.test(document.scene.id)) fail(artifact, "OV0202", "$.scene.id", "invalid scene identity");
  validateIdentityCollection(terrain, artifact, "$.terrain_layers");
};

const validateOrder = ({ actions, actors, terrain }, artifact) => {
  sortedUnique(actions, "id", artifact, "$.actions");
  sortedUnique(actors, "id", artifact, "$.actors");
  sortedUnique(terrain, "id", artifact, "$.terrain_layers");
  terrain.forEach((row, index) => {
    for (let cellIndex = 1; cellIndex < row.cells.length; cellIndex += 1) {
      const before = row.cells[cellIndex - 1];
      const after = row.cells[cellIndex];
      if (after.y < before.y || (after.y === before.y && after.x <= before.x)) {
        fail(artifact, "OV0202", `$.terrain_layers[${index}].cells[${cellIndex}]`, "cells must be unique row-major");
      }
    }
  });
};

const validateReferences = ({ actions, actors }, artifact) => {
  const actorIds = new Set(actors.map((row) => row.id));
  actions.forEach((row, index) => {
    if (!actorIds.has(row.target_actor)) fail(artifact, "OV0203", `$.actions[${index}].target_actor`, "dangling target");
  });
};

const parseCell = (value, crop, artifact, path, actor = false) => {
  exact(value, actor ? ["x", "y", "z"] : ["x", "y"], artifact, path);
  const x = boundedInteger(value.x, 0, crop.width - 1, artifact, `${path}.x`);
  const y = boundedInteger(value.y, 0, crop.height - 1, artifact, `${path}.y`);
  if (!actor) return { x, y };
  const z = boundedInteger(value.z, 0, 0, artifact, `${path}.z`);
  return { x, y, z };
};

const decodeTerrain = (rows, crop, artifact) => {
  if (rows.length < 3 || rows.length > 8) fail(artifact, "OV0202", "$.terrain_layers", "invalid layer count");
  let total = 0;
  const roles = new Set();
  const decoded = rows.map((value, index) => {
    const path = `$.terrain_layers[${index}]`;
    exact(value, ["assembly", "cells", "id", "material_family", "role", "stack"], artifact, path);
    const id = identity(value.id, artifact, `${path}.id`);
    const role = enumValue(value.role, [...ROLES.keys()], artifact, `${path}.role`);
    const cells = array(value.cells, artifact, `${path}.cells`);
    if (cells.length < 1 || cells.length > 1024) fail(artifact, "OV0202", `${path}.cells`, "invalid cell count");
    const parsedCells = cells.map((cell, cellIndex) =>
      parseCell(cell, crop, artifact, `${path}.cells[${cellIndex}]`),
    );
    for (let cellIndex = 1; cellIndex < parsedCells.length; cellIndex += 1) {
      const before = parsedCells[cellIndex - 1];
      const after = parsedCells[cellIndex];
      if (after.y < before.y || (after.y === before.y && after.x <= before.x)) {
        fail(artifact, "OV0202", `${path}.cells[${cellIndex}]`, "cells must be unique row-major");
      }
    }
    total += parsedCells.length;
    roles.add(role);
    const [assembly, material_family, stack] = ROLES.get(role);
    const copied = {
      id,
      role,
      cells: parsedCells,
      assembly: string(value.assembly, artifact, `${path}.assembly`),
      material_family: string(value.material_family, artifact, `${path}.material_family`),
      stack: value.stack,
    };
    if (copied.assembly !== assembly) fail(artifact, "OV0204", `${path}.assembly`, "assembly disagrees with role");
    if (copied.material_family !== material_family) fail(artifact, "OV0204", `${path}.material_family`, "material disagrees with role");
    if (copied.stack !== BigInt(stack)) fail(artifact, "OV0204", `${path}.stack`, "stack disagrees with role");
    return { ...copied, stack };
  });
  if (total < 3 || total > 4096) fail(artifact, "OV0202", "$.terrain_layers", "invalid total cell count");
  if ([...ROLES.keys()].some((role) => !roles.has(role))) fail(artifact, "OV0202", "$.terrain_layers", "all roles are required");
  sortedUnique(decoded, "id", artifact, "$.terrain_layers");
  return decoded;
};

const decodeActors = (rows, crop, artifact) => {
  if (rows.length < 1 || rows.length > 64) fail(artifact, "OV0202", "$.actors", "invalid actor count");
  const decoded = rows.map((value, handle) => {
    const path = `$.actors[${handle}]`;
    exact(
      value,
      ["assembly", "cell", "controlled", "controlled_marker", "hostile", "hostile_outline", "id", "life_state", "pose", "protected", "protection_ring"],
      artifact,
      path,
    );
    const id = identity(value.id, artifact, `${path}.id`);
    const cell = parseCell(value.cell, crop, artifact, `${path}.cell`, true);
    const life = enumValue(value.life_state, [...POSES.keys()], artifact, `${path}.life_state`);
    const controlled = boolean(value.controlled, artifact, `${path}.controlled`);
    const hostile = boolean(value.hostile, artifact, `${path}.hostile`);
    const protectedFact = boolean(value.protected, artifact, `${path}.protected`);
    const actor = {
      id,
      handle,
      cell,
      assembly: string(value.assembly, artifact, `${path}.assembly`),
      pose: string(value.pose, artifact, `${path}.pose`),
      controlled_marker: string(value.controlled_marker, artifact, `${path}.controlled_marker`),
      hostile_outline: string(value.hostile_outline, artifact, `${path}.hostile_outline`),
      protection_ring: string(value.protection_ring, artifact, `${path}.protection_ring`),
      life,
      controlled,
      hostile,
      protectedFact,
    };
    if (actor.assembly !== "actor/observed_figure") fail(artifact, "OV0204", `${path}.assembly`, "actor assembly disagrees");
    if (actor.pose !== POSES.get(life)) fail(artifact, "OV0204", `${path}.pose`, "pose disagrees with life state");
    if (actor.controlled_marker !== PRESENCE.get(controlled)) fail(artifact, "OV0204", `${path}.controlled_marker`, "controlled marker disagrees");
    if (actor.hostile_outline !== PRESENCE.get(hostile)) fail(artifact, "OV0204", `${path}.hostile_outline`, "hostile outline disagrees");
    if (actor.protection_ring !== PRESENCE.get(protectedFact)) fail(artifact, "OV0204", `${path}.protection_ring`, "protection ring disagrees");
    return actor;
  });
  sortedUnique(decoded, "id", artifact, "$.actors");
  return decoded;
};

const decodeActions = (rows, actors, artifact) => {
  if (rows.length > 128) fail(artifact, "OV0202", "$.actions", "invalid action count");
  const byId = new Map(actors.map((actor) => [actor.id, actor]));
  const decoded = rows.map((value, index) => {
    const path = `$.actions[${index}]`;
    exact(value, ["availability", "id", "marker", "target_actor"], artifact, path);
    const id = identity(value.id, artifact, `${path}.id`);
    const availability = enumValue(value.availability, [...MARKERS.keys()], artifact, `${path}.availability`);
    const target = identity(value.target_actor, artifact, `${path}.target_actor`);
    const marker = string(value.marker, artifact, `${path}.marker`);
    if (marker !== MARKERS.get(availability)) fail(artifact, "OV0204", `${path}.marker`, "marker disagrees with availability");
    return { id, target, marker };
  });
  sortedUnique(decoded, "id", artifact, "$.actions");
  for (let index = 0; index < decoded.length; index += 1) {
    if (!byId.has(decoded[index].target)) fail(artifact, "OV0203", `$.actions[${index}].target_actor`, "dangling target");
  }
  return decoded.map((action) => ({ marker: action.marker, actor: byId.get(action.target) }));
};

const tuple = (actor) =>
  encodeCanonical({
    assembly: actor.assembly,
    cell: actor.cell,
    controlled_marker: actor.controlled_marker,
    hostile_outline: actor.hostile_outline,
    pose: actor.pose,
    protection_ring: actor.protection_ring,
  });

export const decodePlanBytes = (bytes, artifact = "plan.json") => {
  let document;
  try {
    document = parseCanonical(bytes);
  } catch (error) {
    if (!(error instanceof CanonicalFailure)) throw error;
    const code = error.kind === "canonical" ? "OV0103" : error.kind === "duplicate" ? "OV0201" : "OV0102";
    fail(artifact, code, error.path, error.message);
  }
  object(document, artifact, "$");
  if (typeof document.schema !== "string" || document.schema !== SCHEMA) {
    fail(artifact, "OV0104", "$.schema", "schema identity is not exact");
  }
  if (typeof document.source_sha256 !== "string" || !DIGEST.test(document.source_sha256)) {
    fail(artifact, "OV0104", "$.source_sha256", "source digest spelling is invalid");
  }
  exact(document, ["actions", "actors", "crop", "scene", "schema", "source_sha256", "terrain_layers"], artifact, "$");
  const shaped = validateShape(document, artifact);
  validateBounds(shaped, document, artifact);
  validateIdentities(shaped, document, artifact);
  validateOrder(shaped, artifact);
  validateReferences(shaped, artifact);
  const crop = {
    height: boundedInteger(document.crop.height, 1, 32, artifact, "$.crop.height"),
    width: boundedInteger(document.crop.width, 1, 32, artifact, "$.crop.width"),
  };
  const terrain = decodeTerrain(shaped.terrain, crop, artifact);
  const actors = decodeActors(shaped.actors, crop, artifact);
  const actions = decodeActions(shaped.actions, actors, artifact);

  const terrainView = terrain
    .map((row, ordinal) => ({
      ordinal,
      cells: row.cells,
      assembly: row.assembly,
      material_family: row.material_family,
      stack: row.stack,
      order: encodeCanonical(row.cells),
    }))
    .sort((a, b) => a.stack - b.stack || a.order.localeCompare(b.order) || a.ordinal - b.ordinal)
    .map(({ cells, assembly, material_family, stack }) => ({ cells, assembly, material_family, stack }));

  const actorView = actors
    .map((actor) => ({
      handle: actor.handle,
      cell: actor.cell,
      assembly: actor.assembly,
      pose: actor.pose,
      controlled_marker: actor.controlled_marker,
      hostile_outline: actor.hostile_outline,
      protection_ring: actor.protection_ring,
      order: tuple(actor),
    }))
    .sort((a, b) => a.order.localeCompare(b.order) || a.handle - b.handle);
  const actorOrder = new Map(actors.map((actor) => [actor.handle, tuple(actor)]));
  const actionView = actions
    .map(({ marker, actor }) => ({ marker, target_handle: actor.handle, order: actorOrder.get(actor.handle) }))
    .sort((a, b) => a.order.localeCompare(b.order) || a.marker.localeCompare(b.marker) || a.target_handle - b.target_handle)
    .map(({ marker, target_handle }) => ({ marker, target_handle }));

  return deepFreeze({
    crop,
    terrain: terrainView,
    actors: actorView.map(({ order, ...actor }) => actor),
    actions: actionView,
  });
};
