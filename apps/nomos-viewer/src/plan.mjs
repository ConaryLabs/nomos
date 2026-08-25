// The strict decoder for the two published artifacts.
//
// `docs/review/nomos-viewer.md` section 4 is the contract. This module is the
// only one that reads an artifact and the only one that constructs a URL, and
// it refuses the same things `crates/nomos-render-plan/src/json.rs` refuses at
// the other end of the pipe: a foreign identity, an unknown field, a missing
// field, a number that is not an integer, a name outside a closed catalog set,
// and a cross-reference that does not resolve.
//
// Nothing is defaulted. There is no `?? fallback` and no optional chain that
// swallows an absent collection: an artifact that does not say what it must say
// is refused, with a code, before a frame is drawn.

import {
  ANCHOR_KINDS,
  DIRECTIONS,
  DISPOSITIONS,
  ENTITY_KINDS,
  ENTITY_KIND_IDS,
  OBJECTIVE_KINDS,
  ACTOR_ASSEMBLIES,
  ARCHITECTURE_ASSEMBLIES,
  EFFECT_ASSEMBLIES,
  MATERIAL_FAMILIES,
  SOCKETS,
  TRIM_FAMILIES,
} from "./catalog.mjs";

export const PLAN_SCHEMA = "nomos.rendering_plan@2";

// Declared by `experiments/executable-gaol/src/build-collection.mjs`, which is
// quarantined tooling: the four plans are accepted output, the file that
// stitches them into a route is not. Recorded in the design record as finding 2
// and ruled acceptable for R1-4; issue #152 carries promoting it.
export const COLLECTION_SCHEMA = "nomos.experiment.area_collection@2";

/// A refusal. The code space is `NV####`, disjoint by prefix from the frozen
/// Gate K `EK` space and from `nomos-render-plan`'s `RP` space.
export class ViewerError extends Error {
  constructor(code, message, artifact) {
    super(artifact ? `${code} ${message} (${artifact})` : `${code} ${message}`);
    this.name = "ViewerError";
    this.code = code;
    this.artifact = artifact ?? null;
  }
}

export const CODES = Object.freeze({
  UNREADABLE: "NV0101",
  SCHEMA_MISMATCH: "NV0102",
  DOCUMENT_SHAPE: "NV0201",
  CONSTRAINT: "NV0202",
  NUMBER_UNSUPPORTED: "NV0203",
  CATALOG_UNKNOWN: "NV0301",
  REFERENCE_UNRESOLVED: "NV0401",
});

const fail = (code, message, artifact) => {
  throw new ViewerError(code, message, artifact);
};

// ---------------------------------------------------------------------------
// Shape primitives
// ---------------------------------------------------------------------------

const isPlainObject = (value) =>
  typeof value === "object" && value !== null && !Array.isArray(value);

// Exact field sets. `optional` names fields that may be absent; anything not
// named at all is an unknown field and a refusal.
function object(value, where, required, artifact, optional = []) {
  if (!isPlainObject(value)) {
    fail(CODES.DOCUMENT_SHAPE, `expected an object at ${where}, found ${describe(value)}`, artifact);
  }
  const declared = new Set([...required, ...optional]);
  for (const key of Object.keys(value)) {
    if (!declared.has(key)) {
      fail(CODES.DOCUMENT_SHAPE, `unknown field \`${key}\` at ${where}`, artifact);
    }
  }
  for (const key of required) {
    if (!(key in value)) {
      fail(CODES.DOCUMENT_SHAPE, `missing field \`${key}\` at ${where}`, artifact);
    }
  }
  return value;
}

const describe = (value) => {
  if (value === null) return "null";
  if (Array.isArray(value)) return "an array";
  return typeof value;
};

function array(value, where, artifact) {
  if (!Array.isArray(value)) {
    fail(CODES.DOCUMENT_SHAPE, `expected an array at ${where}, found ${describe(value)}`, artifact);
  }
  return value;
}

function text(value, where, artifact) {
  if (typeof value !== "string" || value.length === 0) {
    fail(CODES.DOCUMENT_SHAPE, `expected a non-empty string at ${where}, found ${describe(value)}`, artifact);
  }
  return value;
}

function bool(value, where, artifact) {
  if (typeof value !== "boolean") {
    fail(CODES.DOCUMENT_SHAPE, `expected a boolean at ${where}, found ${describe(value)}`, artifact);
  }
  return value;
}

function integer(value, where, artifact) {
  if (typeof value !== "number" || !Number.isSafeInteger(value)) {
    fail(CODES.NUMBER_UNSUPPORTED, `expected an integer at ${where}, found ${JSON.stringify(value)}`, artifact);
  }
  return value;
}

function member(set, value, where, artifact) {
  if (!set.includes(value)) {
    fail(
      CODES.CATALOG_UNKNOWN,
      `\`${value}\` at ${where} is not in the renderer catalog; expected one of ${set.join(", ")}`,
      artifact,
    );
  }
  return value;
}

function constrain(condition, message, artifact) {
  if (!condition) fail(CODES.CONSTRAINT, message, artifact);
}

function reference(condition, message, artifact) {
  if (!condition) fail(CODES.REFERENCE_UNRESOLVED, message, artifact);
}

// Every number in a published artifact is an integer. The compiler emits the
// plan through `nomos_core::CanonicalValue`, which has no floating-point
// variant; this is the same statement made about the values that reach the
// renderer, at any depth, including inside a field the shape check would
// otherwise never visit.
function refuseNonIntegerNumbers(value, artifact, where = "the document") {
  if (typeof value === "number") integer(value, where, artifact);
  else if (Array.isArray(value)) {
    value.forEach((entry, index) => refuseNonIntegerNumbers(entry, artifact, `${where}[${index}]`));
  } else if (isPlainObject(value)) {
    for (const [key, entry] of Object.entries(value)) {
      refuseNonIntegerNumbers(entry, artifact, `${where}.${key}`);
    }
  }
}

function bindSchema(value, expected, artifact) {
  if (!isPlainObject(value)) {
    fail(CODES.DOCUMENT_SHAPE, `expected an object at the document root, found ${describe(value)}`, artifact);
  }
  if (value.schema !== expected) {
    fail(
      CODES.SCHEMA_MISMATCH,
      `expected schema \`${expected}\`, found \`${value.schema ?? "none"}\``,
      artifact,
    );
  }
}

// ---------------------------------------------------------------------------
// The rendering plan
// ---------------------------------------------------------------------------

const cell = (value, where, artifact) => {
  object(value, where, ["x", "y", "z"], artifact);
  return {
    x: integer(value.x, `${where}.x`, artifact),
    y: integer(value.y, `${where}.y`, artifact),
    z: integer(value.z, `${where}.z`, artifact),
  };
};

const corner = (value, where, artifact) => {
  object(value, where, ["x", "y"], artifact);
  return {
    x: integer(value.x, `${where}.x`, artifact),
    y: integer(value.y, `${where}.y`, artifact),
  };
};

function anchor(value, kind, where, artifact) {
  const declared = ENTITY_KINDS[kind].anchorKind;
  if (!isPlainObject(value)) {
    fail(CODES.DOCUMENT_SHAPE, `expected an object at ${where}, found ${describe(value)}`, artifact);
  }
  member(ANCHOR_KINDS, value.kind, `${where}.kind`, artifact);
  constrain(
    value.kind === declared,
    `${where}.kind is \`${value.kind}\`, but a \`${kind}\` entity anchors by \`${declared}\``,
    artifact,
  );
  if (value.kind === "region") {
    object(value, where, ["kind", "min", "max"], artifact);
    const min = cell(value.min, `${where}.min`, artifact);
    const max = cell(value.max, `${where}.max`, artifact);
    constrain(min.x <= max.x && min.y <= max.y, `${where} has an inverted region`, artifact);
    return { kind: "region", min, max };
  }
  if (value.kind === "face") {
    object(value, where, ["kind", "cell", "direction"], artifact);
    return {
      kind: "face",
      cell: cell(value.cell, `${where}.cell`, artifact),
      direction: member(DIRECTIONS, value.direction, `${where}.direction`, artifact),
    };
  }
  object(value, where, ["kind", "cell"], artifact);
  return { kind: "cell", cell: cell(value.cell, `${where}.cell`, artifact) };
}

function entity(value, index, artifact) {
  const where = `entities[${index}]`;
  object(
    value,
    where,
    [
      "id",
      "kind",
      "anchor",
      "visual_assembly",
      "material_family",
      "machine_namespaces",
      "provenance",
    ],
    artifact,
  );
  const id = text(value.id, `${where}.id`, artifact);
  const kind = value.kind;
  if (!ENTITY_KIND_IDS.includes(kind)) {
    // `EntityKind::Unknown` reaches a plan as `visual/marker` when the compiler
    // has no kind for a primitive (`crates/nomos-render-plan/src/catalog.rs`).
    // The study drew a marker; audit section 3 item 4 records that silent
    // fallback as a defect. A renderer that cannot draw a thing says so.
    fail(
      CODES.CATALOG_UNKNOWN,
      `entity \`${id}\` has kind \`${kind}\`, which the renderer catalog does not declare; ` +
        `expected one of ${ENTITY_KIND_IDS.join(", ")}`,
      artifact,
    );
  }
  const declared = ENTITY_KINDS[kind];
  constrain(
    value.visual_assembly === declared.visualAssembly,
    `entity \`${id}\` is a \`${kind}\` but names assembly \`${value.visual_assembly}\`, ` +
      `and the catalog draws that kind as \`${declared.visualAssembly}\``,
    artifact,
  );
  constrain(
    value.material_family === declared.materialFamily,
    `entity \`${id}\` is a \`${kind}\` but names material family \`${value.material_family}\`, ` +
      `and the catalog gives that kind \`${declared.materialFamily}\``,
    artifact,
  );
  const namespaces = array(value.machine_namespaces, `${where}.machine_namespaces`, artifact).map(
    (one, at) => text(one, `${where}.machine_namespaces[${at}]`, artifact),
  );
  const provenance = array(value.provenance, `${where}.provenance`, artifact).map((one, at) => {
    const at_where = `${where}.provenance[${at}]`;
    object(one, at_where, ["claim", "source"], artifact);
    const source = one.source;
    object(source, `${at_where}.source`, ["path", "line", "column", "byte_start", "byte_end"], artifact);
    return {
      claim: text(one.claim, `${at_where}.claim`, artifact),
      source: {
        path: text(source.path, `${at_where}.source.path`, artifact),
        line: integer(source.line, `${at_where}.source.line`, artifact),
        column: integer(source.column, `${at_where}.source.column`, artifact),
        byte_start: integer(source.byte_start, `${at_where}.source.byte_start`, artifact),
        byte_end: integer(source.byte_end, `${at_where}.source.byte_end`, artifact),
      },
    };
  });
  return Object.freeze({
    id,
    kind,
    anchor: anchor(value.anchor, kind, `${where}.anchor`, artifact),
    visual_assembly: declared.visualAssembly,
    material_family: declared.materialFamily,
    machine_namespaces: Object.freeze(namespaces),
    provenance: Object.freeze(provenance),
  });
}

function scenario(value, index, artifact) {
  const where = `scenarios[${index}]`;
  object(
    value,
    where,
    ["id", "label", "tick", "state_hash", "machine_states", "movement", "effective_light"],
    artifact,
  );
  const machineStates = array(value.machine_states, `${where}.machine_states`, artifact).map(
    (one, at) => {
      const row = `${where}.machine_states[${at}]`;
      object(one, row, ["namespace", "state"], artifact);
      return Object.freeze({
        namespace: text(one.namespace, `${row}.namespace`, artifact),
        state: text(one.state, `${row}.state`, artifact),
      });
    },
  );
  const movement = array(value.movement, `${where}.movement`, artifact).map((one, at) => {
    const row = `${where}.movement[${at}]`;
    object(one, row, ["entity", "disposition", "cost", "reasons"], artifact);
    const disposition = member(DISPOSITIONS, one.disposition, `${row}.disposition`, artifact);
    // `cost: null` on a blocked subject is the one normalization RUNTIME.md
    // section 5 R1-1 names, and it is permitted only there.
    if (one.cost !== null) integer(one.cost, `${row}.cost`, artifact);
    constrain(
      disposition === "blocked" || one.cost !== null,
      `${row} is traversable but carries no cost`,
      artifact,
    );
    return Object.freeze({
      entity: text(one.entity, `${row}.entity`, artifact),
      disposition,
      cost: one.cost,
      reasons: Object.freeze(
        array(one.reasons, `${row}.reasons`, artifact).map((reason, index_) =>
          text(reason, `${row}.reasons[${index_}]`, artifact),
        ),
      ),
    });
  });
  const light = array(value.effective_light, `${where}.effective_light`, artifact).map((one, at) => {
    const row = `${where}.effective_light[${at}]`;
    object(one, row, ["entity", "emitting"], artifact);
    return Object.freeze({
      entity: text(one.entity, `${row}.entity`, artifact),
      emitting: bool(one.emitting, `${row}.emitting`, artifact),
    });
  });
  return Object.freeze({
    id: text(value.id, `${where}.id`, artifact),
    label: text(value.label, `${where}.label`, artifact),
    tick: integer(value.tick, `${where}.tick`, artifact),
    state_hash: text(value.state_hash, `${where}.state_hash`, artifact),
    machine_states: Object.freeze(machineStates),
    movement: Object.freeze(movement),
    effective_light: Object.freeze(light),
  });
}

function architecture(value, artifact) {
  const where = "architecture";
  object(value, where, ["bounds", "wall_height_steps", "style", "masses"], artifact);
  object(value.bounds, `${where}.bounds`, ["width", "height"], artifact);
  const bounds = {
    width: integer(value.bounds.width, `${where}.bounds.width`, artifact),
    height: integer(value.bounds.height, `${where}.bounds.height`, artifact),
  };
  constrain(bounds.width >= 1 && bounds.height >= 1, `${where}.bounds is empty`, artifact);
  object(value.style, `${where}.style`, ["assembly", "material_family", "trim_family"], artifact);
  const style = Object.freeze({
    assembly: member(ARCHITECTURE_ASSEMBLIES, value.style.assembly, `${where}.style.assembly`, artifact),
    material_family: member(
      MATERIAL_FAMILIES,
      value.style.material_family,
      `${where}.style.material_family`,
      artifact,
    ),
    trim_family: member(TRIM_FAMILIES, value.style.trim_family, `${where}.style.trim_family`, artifact),
  });
  const steps = integer(value.wall_height_steps, `${where}.wall_height_steps`, artifact);
  constrain(steps > 0, `${where}.wall_height_steps must be positive`, artifact);
  const seen = new Set();
  const masses = array(value.masses, `${where}.masses`, artifact).map((one, at) => {
    const row = `${where}.masses[${at}]`;
    object(one, row, ["id", "min", "max", "height_steps"], artifact);
    const id = text(one.id, `${row}.id`, artifact);
    constrain(!seen.has(id), `${where} declares mass \`${id}\` twice`, artifact);
    seen.add(id);
    const min = corner(one.min, `${row}.min`, artifact);
    const max = corner(one.max, `${row}.max`, artifact);
    constrain(min.x < max.x && min.y < max.y, `${row} is an empty rectangle`, artifact);
    constrain(
      min.x >= 0 && min.y >= 0 && max.x <= bounds.width && max.y <= bounds.height,
      `${row} leaves the declared bounds`,
      artifact,
    );
    const height = integer(one.height_steps, `${row}.height_steps`, artifact);
    constrain(height > 0, `${row}.height_steps must be positive`, artifact);
    return Object.freeze({ id, min: Object.freeze(min), max: Object.freeze(max), height_steps: height });
  });
  return Object.freeze({
    bounds: Object.freeze(bounds),
    wall_height_steps: steps,
    style,
    masses: Object.freeze(masses),
  });
}

/// Decodes one `nomos.rendering_plan@2` document.
export function decodePlan(document, artifact = "the rendering plan") {
  bindSchema(document, PLAN_SCHEMA, artifact);
  refuseNonIntegerNumbers(document, artifact);
  object(
    document,
    "the plan",
    [
      "schema",
      "area",
      "objective",
      "route",
      "pursuit",
      "projection_schemas",
      "projection_digests",
      "architecture",
      "entities",
      "actors",
      "effects",
      "scenarios",
      "interactions",
    ],
    artifact,
  );

  object(document.area, "area", ["id", "label", "start"], artifact);
  const area = Object.freeze({
    id: text(document.area.id, "area.id", artifact),
    label: text(document.area.label, "area.label", artifact),
    start: bool(document.area.start, "area.start", artifact),
  });

  object(document.objective, "objective", ["kind", "gate"], artifact);
  const objective = Object.freeze({
    kind: member(OBJECTIVE_KINDS, document.objective.kind, "objective.kind", artifact),
    gate: text(document.objective.gate, "objective.gate", artifact),
  });

  object(document.route, "route", ["to_area"], artifact, ["entry"]);
  const hasEntry = "entry" in document.route;
  constrain(
    area.start !== hasEntry,
    "route.entry is present exactly when the area is not the start area",
    artifact,
  );
  const route = Object.freeze({
    to_area: document.route.to_area === null ? null : text(document.route.to_area, "route.to_area", artifact),
    entry: hasEntry ? Object.freeze(cell(document.route.entry, "route.entry", artifact)) : null,
  });

  object(document.pursuit, "pursuit", ["light"], artifact);
  const pursuit = Object.freeze({ light: text(document.pursuit.light, "pursuit.light", artifact) });

  const projectionSchemas = array(document.projection_schemas, "projection_schemas", artifact).map(
    (one, at) => {
      const row = `projection_schemas[${at}]`;
      object(one, row, ["name", "version"], artifact);
      return Object.freeze({
        name: text(one.name, `${row}.name`, artifact),
        version: integer(one.version, `${row}.version`, artifact),
      });
    },
  );
  const projectionDigests = array(document.projection_digests, "projection_digests", artifact).map(
    (one, at) => {
      const row = `projection_digests[${at}]`;
      object(one, row, ["file", "digest"], artifact);
      return Object.freeze({
        file: text(one.file, `${row}.file`, artifact),
        digest: text(one.digest, `${row}.digest`, artifact),
      });
    },
  );
  constrain(
    projectionSchemas.length === projectionDigests.length && projectionSchemas.length > 0,
    "projection_schemas and projection_digests must describe the same members",
    artifact,
  );

  const built = architecture(document.architecture, artifact);
  const entities = array(document.entities, "entities", artifact).map((one, at) =>
    entity(one, at, artifact),
  );
  const byId = new Map();
  for (const one of entities) {
    constrain(!byId.has(one.id), `entities declares \`${one.id}\` twice`, artifact);
    byId.set(one.id, one);
  }

  const actors = array(document.actors, "actors", artifact).map((one, at) => {
    const row = `actors[${at}]`;
    object(one, row, ["id", "assembly", "cell"], artifact);
    return Object.freeze({
      id: text(one.id, `${row}.id`, artifact),
      assembly: member(ACTOR_ASSEMBLIES, one.assembly, `${row}.assembly`, artifact),
      cell: Object.freeze(cell(one.cell, `${row}.cell`, artifact)),
    });
  });

  const effects = array(document.effects, "effects", artifact).map((one, at) => {
    const row = `effects[${at}]`;
    object(one, row, ["id", "assembly", "anchor"], artifact);
    object(one.anchor, `${row}.anchor`, ["entity", "socket"], artifact);
    return Object.freeze({
      id: text(one.id, `${row}.id`, artifact),
      assembly: member(EFFECT_ASSEMBLIES, one.assembly, `${row}.assembly`, artifact),
      anchor: Object.freeze({
        entity: text(one.anchor.entity, `${row}.anchor.entity`, artifact),
        socket: text(one.anchor.socket, `${row}.anchor.socket`, artifact),
      }),
    });
  });

  const scenarios = array(document.scenarios, "scenarios", artifact).map((one, at) =>
    scenario(one, at, artifact),
  );
  constrain(scenarios.length > 0, "the plan declares no scenario", artifact);
  const scenarioIds = new Set();
  for (const one of scenarios) {
    constrain(!scenarioIds.has(one.id), `scenarios declares \`${one.id}\` twice`, artifact);
    scenarioIds.add(one.id);
  }

  const interactions = array(document.interactions, "interactions", artifact).map((one, at) => {
    const row = `interactions[${at}]`;
    object(
      one,
      row,
      [
        "id",
        "action",
        "from_scenario",
        "to_scenario",
        "target_entity",
        "input_state_hash",
        "resulting_state_hash",
      ],
      artifact,
    );
    return Object.freeze({
      id: text(one.id, `${row}.id`, artifact),
      action: text(one.action, `${row}.action`, artifact),
      from_scenario: text(one.from_scenario, `${row}.from_scenario`, artifact),
      to_scenario: text(one.to_scenario, `${row}.to_scenario`, artifact),
      target_entity: text(one.target_entity, `${row}.target_entity`, artifact),
      input_state_hash: text(one.input_state_hash, `${row}.input_state_hash`, artifact),
      resulting_state_hash: text(one.resulting_state_hash, `${row}.resulting_state_hash`, artifact),
    });
  });

  const plan = Object.freeze({
    schema: PLAN_SCHEMA,
    area,
    objective,
    route,
    pursuit,
    projection_schemas: Object.freeze(projectionSchemas),
    projection_digests: Object.freeze(projectionDigests),
    architecture: built,
    entities: Object.freeze(entities),
    actors: Object.freeze(actors),
    effects: Object.freeze(effects),
    scenarios: Object.freeze(scenarios),
    interactions: Object.freeze(interactions),
  });

  checkPlanReferences(plan, artifact);
  return plan;
}

// Everything one plan can be asked about itself. A plan that fails any of these
// is broken in a way that would otherwise surface as a blank frame.
function checkPlanReferences(plan, artifact) {
  const byId = new Map(plan.entities.map((one) => [one.id, one]));
  const { width, height } = plan.architecture.bounds;
  const insideBounds = (point) =>
    point.x >= 0 && point.y >= 0 && point.x < width && point.y < height;
  const insideMass = (point) =>
    plan.architecture.masses.some(
      (mass) =>
        point.x >= mass.min.x && point.x < mass.max.x && point.y >= mass.min.y && point.y < mass.max.y,
    );

  const gate = byId.get(plan.objective.gate);
  reference(gate !== undefined, `objective.gate names absent entity \`${plan.objective.gate}\``, artifact);
  constrain(gate.kind === "door", `objective.gate \`${gate.id}\` is a \`${gate.kind}\`, not a door`, artifact);

  const light = byId.get(plan.pursuit.light);
  reference(light !== undefined, `pursuit.light names absent entity \`${plan.pursuit.light}\``, artifact);
  constrain(
    light.kind === "light",
    `pursuit.light \`${light.id}\` is a \`${light.kind}\`, not a light`,
    artifact,
  );

  if (plan.route.entry) {
    constrain(insideBounds(plan.route.entry), "route.entry lies outside the declared bounds", artifact);
    constrain(plan.route.entry.z === 0, "route.entry is not on the floor", artifact);
    constrain(!insideMass(plan.route.entry), "route.entry lies inside a masonry mass", artifact);
  }

  const actorIds = new Set();
  for (const actor of plan.actors) {
    constrain(!actorIds.has(actor.id), `actors declares \`${actor.id}\` twice`, artifact);
    actorIds.add(actor.id);
    constrain(insideBounds(actor.cell), `actor \`${actor.id}\` starts outside the bounds`, artifact);
    constrain(!insideMass(actor.cell), `actor \`${actor.id}\` starts inside a masonry mass`, artifact);
  }

  const effectIds = new Set();
  for (const effect of plan.effects) {
    constrain(!effectIds.has(effect.id), `effects declares \`${effect.id}\` twice`, artifact);
    effectIds.add(effect.id);
    const host = byId.get(effect.anchor.entity);
    reference(
      host !== undefined,
      `effect \`${effect.id}\` anchors to absent entity \`${effect.anchor.entity}\``,
      artifact,
    );
    const declared = SOCKETS[host.visual_assembly];
    if (!declared?.[effect.anchor.socket]) {
      fail(
        CODES.CATALOG_UNKNOWN,
        `effect \`${effect.id}\` names socket \`${effect.anchor.socket}\`, which ` +
          `\`${host.visual_assembly}\` does not declare`,
        artifact,
      );
    }
  }

  const scenarioById = new Map(plan.scenarios.map((one) => [one.id, one]));
  const subjects = plan.entities.filter((one) => one.kind === "door" || one.kind === "water");
  const lights = plan.entities.filter((one) => one.kind === "light");
  for (const one of plan.scenarios) {
    for (const subject of subjects) {
      reference(
        one.movement.some((row) => row.entity === subject.id),
        `scenario \`${one.id}\` carries no movement for \`${subject.id}\``,
        artifact,
      );
    }
    for (const subject of lights) {
      reference(
        one.effective_light.some((row) => row.entity === subject.id),
        `scenario \`${one.id}\` carries no effective light for \`${subject.id}\``,
        artifact,
      );
    }
    for (const row of one.machine_states) {
      const [entityId] = row.namespace.split(".");
      reference(
        byId.has(entityId),
        `scenario \`${one.id}\` carries machine \`${row.namespace}\` for absent entity \`${entityId}\``,
        artifact,
      );
    }
  }

  for (const interaction of plan.interactions) {
    const from = scenarioById.get(interaction.from_scenario);
    const to = scenarioById.get(interaction.to_scenario);
    reference(from !== undefined, `interaction \`${interaction.id}\` leaves absent scenario \`${interaction.from_scenario}\``, artifact);
    reference(to !== undefined, `interaction \`${interaction.id}\` enters absent scenario \`${interaction.to_scenario}\``, artifact);
    reference(
      byId.has(interaction.target_entity),
      `interaction \`${interaction.id}\` targets absent entity \`${interaction.target_entity}\``,
      artifact,
    );
    constrain(
      interaction.input_state_hash === from.state_hash,
      `interaction \`${interaction.id}\` is not bound to the state hash of \`${from.id}\``,
      artifact,
    );
    constrain(
      interaction.resulting_state_hash === to.state_hash,
      `interaction \`${interaction.id}\` is not bound to the state hash of \`${to.id}\``,
      artifact,
    );
  }

  // The initial scenario is the one with the lowest authoritative tick, not the
  // one that happens to sort first. Audit section 3 item 23 recorded
  // `scenarios[0]` as a convention resting on a filename prefix; a tie would put
  // it back, so a tie is refused.
  const lowest = Math.min(...plan.scenarios.map((one) => one.tick));
  constrain(
    plan.scenarios.filter((one) => one.tick === lowest).length === 1,
    `two scenarios share the lowest tick ${lowest}, so the initial scenario is undecided`,
    artifact,
  );
}

// ---------------------------------------------------------------------------
// The area collection
// ---------------------------------------------------------------------------

/// Decodes one `nomos.experiment.area_collection@2` document.
export function decodeCollection(document, artifact = "the area collection") {
  bindSchema(document, COLLECTION_SCHEMA, artifact);
  refuseNonIntegerNumbers(document, artifact);
  object(document, "the collection", ["schema", "visual_grammar", "start_area", "route", "areas"], artifact);

  const grammar = document.visual_grammar;
  object(
    grammar,
    "visual_grammar",
    [
      "digest",
      "rendering_plan_schema",
      "projection_schemas",
      "architecture_style",
      "entity_assemblies",
      "actor_assemblies",
      "effect_assemblies",
    ],
    artifact,
  );
  constrain(
    grammar.rendering_plan_schema === PLAN_SCHEMA,
    `visual_grammar.rendering_plan_schema is \`${grammar.rendering_plan_schema}\`, ` +
      `and this viewer reads \`${PLAN_SCHEMA}\``,
    artifact,
  );

  const areaIds = new Set();
  const areas = array(document.areas, "areas", artifact).map((one, at) => {
    const row = `areas[${at}]`;
    object(one, row, ["id", "label", "plan"], artifact);
    const id = text(one.id, `${row}.id`, artifact);
    constrain(!areaIds.has(id), `areas declares \`${id}\` twice`, artifact);
    areaIds.add(id);
    const planPath = text(one.plan, `${row}.plan`, artifact);
    // The only path the app ever joins onto its own base. Anything that could
    // reach another origin, or climb out of the staged tree, is refused here
    // rather than by the browser.
    constrain(
      planPath === `areas/${id}.json`,
      `${row}.plan is \`${planPath}\`, and the staged layout names it \`areas/${id}.json\``,
      artifact,
    );
    return Object.freeze({ id, label: text(one.label, `${row}.label`, artifact), plan: planPath });
  });

  const startArea = text(document.start_area, "start_area", artifact);
  reference(areaIds.has(startArea), `start_area names absent area \`${startArea}\``, artifact);

  const route = array(document.route, "route", artifact).map((one, at) => {
    const row = `route[${at}]`;
    object(one, row, ["from_area", "gate", "to_area", "entry"], artifact);
    const toArea = one.to_area === null ? null : text(one.to_area, `${row}.to_area`, artifact);
    const entry = one.entry === null ? null : Object.freeze(cell(one.entry, `${row}.entry`, artifact));
    constrain(
      (toArea === null) === (entry === null),
      `${row} names a destination exactly when it carries an arrival cell`,
      artifact,
    );
    const fromArea = text(one.from_area, `${row}.from_area`, artifact);
    reference(areaIds.has(fromArea), `${row}.from_area names absent area \`${fromArea}\``, artifact);
    if (toArea !== null) {
      reference(areaIds.has(toArea), `${row}.to_area names absent area \`${toArea}\``, artifact);
    }
    return Object.freeze({
      from_area: fromArea,
      gate: text(one.gate, `${row}.gate`, artifact),
      to_area: toArea,
      entry,
    });
  });

  // One walk from the start, visiting every area exactly once and terminating.
  const edges = new Map(route.map((one) => [one.from_area, one]));
  constrain(edges.size === route.length, "route declares two edges leaving one area", artifact);
  const visited = [];
  let current = startArea;
  while (current !== null) {
    constrain(!visited.includes(current), `route cycles at \`${current}\``, artifact);
    visited.push(current);
    const edge = edges.get(current);
    reference(edge !== undefined, `route has no edge leaving \`${current}\``, artifact);
    current = edge.to_area;
  }
  constrain(
    visited.length === areas.length,
    `route visits ${visited.length} of ${areas.length} declared areas`,
    artifact,
  );

  return Object.freeze({
    schema: COLLECTION_SCHEMA,
    visual_grammar: Object.freeze({ digest: text(grammar.digest, "visual_grammar.digest", artifact) }),
    start_area: startArea,
    route: Object.freeze(route),
    areas: Object.freeze(areas),
    order: Object.freeze(visited),
  });
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

async function readJson(fetchImpl, base, relative) {
  const url = new URL(relative, base);
  let response;
  try {
    response = await fetchImpl(url.href);
  } catch (cause) {
    throw new ViewerError(CODES.UNREADABLE, `could not fetch \`${relative}\`: ${cause.message}`, relative);
  }
  if (!response.ok) {
    throw new ViewerError(CODES.UNREADABLE, `\`${relative}\` responded ${response.status}`, relative);
  }
  const body = await response.text();
  try {
    return JSON.parse(body);
  } catch (cause) {
    throw new ViewerError(CODES.UNREADABLE, `\`${relative}\` is not well-formed JSON: ${cause.message}`, relative);
  }
}

/// Loads and decodes the collection and every plan it names, relative to `base`.
///
/// Every URL the app ever constructs is built here, from a relative path the
/// collection declared and the decoder already constrained.
export async function loadArtifacts(base, fetchImpl) {
  const collection = decodeCollection(await readJson(fetchImpl, base, "areas.json"), "areas.json");
  const plans = new Map();
  for (const area of collection.areas) {
    const plan = decodePlan(await readJson(fetchImpl, base, area.plan), area.plan);
    reference(
      plan.area.id === area.id,
      `\`${area.plan}\` carries area \`${plan.area.id}\`, and the collection lists it as \`${area.id}\``,
      area.plan,
    );
    constrain(
      plan.area.label === area.label,
      `\`${area.plan}\` labels the area \`${plan.area.label}\`, and the collection says \`${area.label}\``,
      area.plan,
    );
    plans.set(area.id, plan);
  }
  for (const edge of collection.route) {
    const plan = plans.get(edge.from_area);
    constrain(
      plan.route.to_area === edge.to_area,
      `the collection routes \`${edge.from_area}\` to \`${edge.to_area}\`, and its plan says \`${plan.route.to_area}\``,
      "areas.json",
    );
    constrain(
      plan.objective.gate === edge.gate,
      `the collection leaves \`${edge.from_area}\` by \`${edge.gate}\`, and its plan says \`${plan.objective.gate}\``,
      "areas.json",
    );
    if (edge.to_area !== null) {
      const destination = plans.get(edge.to_area);
      constrain(
        destination.route.entry !== null,
        `\`${edge.to_area}\` receives an arrival but declares no entry cell`,
        "areas.json",
      );
    }
  }
  constrain(
    plans.get(collection.start_area).area.start === true,
    `\`${collection.start_area}\` is the collection's start area but its plan does not say so`,
    "areas.json",
  );
  return { collection, plans };
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------
//
// `nomos.rendering_plan@2` spells its stable-ID collections as arrays of
// `{entity, ...}` or `{namespace, ...}` rows rather than as objects keyed by
// data, so every lookup goes through one of these. None of them has a fallback:
// the study's four independent machine-state lookups each invented their own
// default, and an absent machine silently drew a sealed ward.

export const entityOf = (plan, id) => plan.entities.find((one) => one.id === id) ?? null;

export const scenarioOf = (plan, id) => plan.scenarios.find((one) => one.id === id) ?? null;

export function machineState(scenario_, entity_, machine) {
  const namespace = `${entity_}.${machine}`;
  const row = scenario_.machine_states.find((one) => one.namespace === namespace);
  if (!row) {
    throw new ViewerError(
      CODES.REFERENCE_UNRESOLVED,
      `scenario \`${scenario_.id}\` carries no machine \`${namespace}\``,
    );
  }
  return row.state;
}

export const doorState = (scenario_, entity_) => ({
  access: machineState(scenario_, entity_, "access"),
  integrity: machineState(scenario_, entity_, "integrity"),
  ward: machineState(scenario_, entity_, "ward"),
});

export const wardSealed = (scenario_, entity_) => machineState(scenario_, entity_, "ward") === "sealed";

export const movementOf = (scenario_, entity_) =>
  scenario_.movement.find((one) => one.entity === entity_) ?? null;

export const lightOf = (scenario_, entity_) =>
  scenario_.effective_light.find((one) => one.entity === entity_)?.emitting ?? null;

/// The scenario a run starts in: the unique lowest authoritative tick.
export function initialScenario(plan) {
  let lowest = plan.scenarios[0];
  for (const one of plan.scenarios) if (one.tick < lowest.tick) lowest = one;
  return lowest;
}

export const interactionFrom = (plan, scenarioId) =>
  plan.interactions.find((one) => one.from_scenario === scenarioId) ?? null;
