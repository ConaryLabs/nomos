// The decoder refuses what `docs/review/nomos-viewer.md` section 4 says it does.

import test from "node:test";
import assert from "node:assert/strict";

import {
  CODES,
  ViewerError,
  decodeCollection,
  decodePlan,
  doorState,
  initialScenario,
  interactionFrom,
  lightOf,
  loadArtifacts,
  machineState,
  movementOf,
  scenarioOf,
  wardSealed,
} from "../src/plan.mjs";
import {
  HASHES,
  collectionDocument,
  edited,
  fetchFrom,
  hallPlan,
  stagedFiles,
  yardPlan,
} from "./fixtures.mjs";

const caught = (run) => {
  try {
    run();
  } catch (error) {
    return error;
  }
  return null;
};

const refuses = (code, run) => {
  const error = caught(run);
  assert.ok(error instanceof ViewerError, `expected a ViewerError, got ${error}`);
  assert.equal(error.code, code, `expected ${code}, got ${error.code}: ${error.message}`);
  return error;
};

test("plan binds its identity and refuses a mismatch", () => {
  assert.equal(decodePlan(hallPlan()).schema, "nomos.rendering_plan@2");
  const error = refuses(CODES.SCHEMA_MISMATCH, () =>
    decodePlan(edited(hallPlan(), "schema", "nomos.rendering_plan@1"), "areas/test-hall.json"),
  );
  assert.match(error.message, /expected schema `nomos\.rendering_plan@2`, found `nomos\.rendering_plan@1`/);
  assert.match(error.message, /areas\/test-hall\.json/);
  refuses(CODES.SCHEMA_MISMATCH, () => decodePlan(edited(hallPlan(), "schema", undefined)));
});

test("collection binds its identity and its route", () => {
  const collection = decodeCollection(collectionDocument());
  assert.equal(collection.schema, "nomos.experiment.area_collection@2");
  assert.deepEqual([...collection.order], ["test-hall", "test-yard"]);
  refuses(CODES.SCHEMA_MISMATCH, () =>
    decodeCollection(edited(collectionDocument(), "schema", "nomos.experiment.area_collection@1")),
  );
  // A route that does not reach every declared area is a broken run, not a
  // viewer that quietly plays three of four rooms.
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "route.0.to_area", null)),
  );
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "visual_grammar.rendering_plan_schema", "nomos.rendering_plan@1")),
  );
});

test("plan refuses an unknown field", () => {
  const error = refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "camera", { width: 1200 })));
  assert.match(error.message, /unknown field `camera`/);
  refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "area.palette", "gaol_bounded_01")));
  refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "entities.0.anchor.socket", "ward")));
  // And an anchor whose kind is not the one its entity kind uses is a
  // constraint, not an unknown field.
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(hallPlan(), "entities.0.anchor.kind", "cell")));
});

test("plan refuses a missing field", () => {
  const error = refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "pursuit", undefined)));
  assert.match(error.message, /missing field `pursuit`/);
  refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "scenarios.0.state_hash", undefined)));
});

test("plan refuses a fractional number, at any depth", () => {
  refuses(CODES.NUMBER_UNSUPPORTED, () => decodePlan(edited(hallPlan(), "architecture.wall_height_steps", 4.5)));
  refuses(CODES.NUMBER_UNSUPPORTED, () => decodePlan(edited(hallPlan(), "actors.0.cell.x", 0.5)));
  const deep = refuses(CODES.NUMBER_UNSUPPORTED, () =>
    decodePlan(edited(hallPlan(), "entities.2.provenance.0.source.line", 7.5)),
  );
  assert.match(deep.message, /expected an integer/);
  refuses(CODES.NUMBER_UNSUPPORTED, () => decodePlan(edited(hallPlan(), "scenarios.0.movement.1.cost", 3.5)));
});

test("an unclassified entity is refused", () => {
  // `EntityKind::Unknown` reaches a plan as `visual/marker`. The study drew a
  // marker; this refuses to pretend.
  const document = structuredClone(hallPlan());
  document.entities[1] = {
    ...document.entities[1],
    kind: "unknown",
    visual_assembly: "visual/marker",
    material_family: "stone",
  };
  const error = refuses(CODES.CATALOG_UNKNOWN, () => decodePlan(document));
  assert.match(error.message, /kind `unknown`, which the renderer catalog does not declare/);
});

test("a name outside a closed set is refused", () => {
  refuses(CODES.CATALOG_UNKNOWN, () => decodePlan(edited(hallPlan(), "actors.0.assembly", "visual/hero")));
  refuses(CODES.CATALOG_UNKNOWN, () => decodePlan(edited(hallPlan(), "effects.0.assembly", "visual/red_crescent")));
  refuses(CODES.CATALOG_UNKNOWN, () => decodePlan(edited(hallPlan(), "architecture.style.trim_family", "fine_mortar")));
  refuses(CODES.CATALOG_UNKNOWN, () => decodePlan(edited(hallPlan(), "scenarios.0.movement.0.disposition", "maybe")));
  refuses(CODES.CATALOG_UNKNOWN, () => decodePlan(edited(hallPlan(), "entities.0.anchor.direction", "up")));
  // The socket vocabulary is the catalog's, not the compiler's.
  refuses(CODES.CATALOG_UNKNOWN, () => decodePlan(edited(hallPlan(), "effects.0.anchor.socket", "lintel")));
  // An assembly that disagrees with the kind the compiler classified is a
  // disagreement between the two tables, and the catalog wins.
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(hallPlan(), "entities.0.visual_assembly", "visual/brazier")));
});

test("movement and light are indexed by entity", () => {
  const plan = decodePlan(hallPlan());
  const sealed = scenarioOf(plan, "01-sealed");
  assert.equal(movementOf(sealed, "hall_gate").disposition, "blocked");
  assert.equal(movementOf(sealed, "hall_gate").cost, null);
  assert.equal(movementOf(sealed, "hall_pool").cost, 4);
  assert.equal(movementOf(sealed, "absent"), null);
  assert.equal(lightOf(sealed, "hall_lamp"), true);
  assert.equal(lightOf(scenarioOf(plan, "03-dark"), "hall_lamp"), false);
  assert.equal(interactionFrom(plan, "01-sealed").action, "unseal");
  assert.equal(interactionFrom(plan, "03-dark"), null);
});

test("an absent machine namespace is refused", () => {
  const plan = decodePlan(hallPlan());
  const sealed = scenarioOf(plan, "01-sealed");
  assert.deepEqual(doorState(sealed, "hall_gate"), {
    access: "locked",
    integrity: "intact",
    ward: "sealed",
  });
  assert.equal(wardSealed(sealed, "hall_gate"), true);
  assert.equal(wardSealed(scenarioOf(plan, "02-unsealed"), "hall_gate"), false);
  // No fallback. The study had four of these, each with its own default, so an
  // absent machine silently drew a sealed ward.
  const error = caught(() => machineState(sealed, "hall_gate", "combustion"));
  assert.ok(error instanceof ViewerError);
  assert.equal(error.code, CODES.REFERENCE_UNRESOLVED);
  assert.match(error.message, /carries no machine `hall_gate\.combustion`/);
});

test("the initial scenario is the unique lowest tick", () => {
  const plan = decodePlan(hallPlan());
  assert.equal(initialScenario(plan).id, "01-sealed");
  // Not array position: the same plan with its scenarios reversed still starts
  // where the authoritative tick says.
  const reversed = structuredClone(hallPlan());
  reversed.scenarios.reverse();
  assert.equal(initialScenario(decodePlan(reversed)).id, "01-sealed");
});

test("a tie on tick is refused", () => {
  const error = refuses(CODES.CONSTRAINT, () => decodePlan(edited(hallPlan(), "scenarios.1.tick", 0)));
  assert.match(error.message, /two scenarios share the lowest tick 0/);
});

test("a cross-reference that does not resolve is refused", () => {
  refuses(CODES.REFERENCE_UNRESOLVED, () => decodePlan(edited(hallPlan(), "objective.gate", "no_such_gate")));
  refuses(CODES.REFERENCE_UNRESOLVED, () => decodePlan(edited(hallPlan(), "pursuit.light", "no_such_lamp")));
  refuses(CODES.REFERENCE_UNRESOLVED, () => decodePlan(edited(hallPlan(), "effects.0.anchor.entity", "no_such_gate")));
  refuses(CODES.REFERENCE_UNRESOLVED, () =>
    decodePlan(edited(hallPlan(), "interactions.0.target_entity", "no_such_gate")),
  );
  refuses(CODES.REFERENCE_UNRESOLVED, () => decodePlan(edited(hallPlan(), "interactions.0.to_scenario", "04-none")));
  // An objective that points at something that is not a door.
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(hallPlan(), "objective.gate", "hall_lamp")));
  // And an interaction whose hashes are not the scenarios' own.
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(hallPlan(), "interactions.0.input_state_hash", HASHES.dark)));
});

test("the arrival cell belongs to the area that receives it", () => {
  // Present exactly when the area is not the start area.
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(hallPlan(), "route.entry", { x: 1, y: 1, z: 0 })));
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(yardPlan(), "route.entry", undefined)));
  // Inside its own bounds, on the floor, and not inside its own masonry.
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(yardPlan(), "route.entry", { x: 9, y: 1, z: 0 })));
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(yardPlan(), "route.entry", { x: 1, y: 1, z: 1 })));
  const walled = structuredClone(yardPlan());
  walled.architecture.masses = [{ height_steps: 20, id: "block", max: { x: 2, y: 2 }, min: { x: 1, y: 1 } }];
  refuses(CODES.CONSTRAINT, () => decodePlan(walled));
});

test("a traversable subject carries a cost and a blocked one may not", () => {
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(hallPlan(), "scenarios.1.movement.0.cost", null)));
  const plan = decodePlan(edited(hallPlan(), "scenarios.0.movement.0.cost", null));
  assert.equal(movementOf(scenarioOf(plan, "01-sealed"), "hall_gate").cost, null);
});

test("every scenario answers for every subject", () => {
  const missing = structuredClone(hallPlan());
  missing.scenarios[0].movement = missing.scenarios[0].movement.filter((row) => row.entity !== "hall_pool");
  refuses(CODES.REFERENCE_UNRESOLVED, () => decodePlan(missing));
  const dark = structuredClone(hallPlan());
  dark.scenarios[1].effective_light = [];
  refuses(CODES.REFERENCE_UNRESOLVED, () => decodePlan(dark));
});

test("loading joins only relative paths the collection declared", async () => {
  const { collection, plans } = await loadArtifacts(
    "http://127.0.0.1:8080/",
    fetchFrom(stagedFiles()),
  );
  assert.equal(collection.start_area, "test-hall");
  assert.deepEqual([...plans.keys()], ["test-hall", "test-yard"]);
  assert.equal(plans.get("test-hall").objective.gate, "hall_gate");

  const requested = [];
  const recording = fetchFrom(stagedFiles());
  await loadArtifacts("http://127.0.0.1:8080/", async (url) => {
    requested.push(url);
    return recording(url);
  });
  assert.deepEqual(requested, [
    "http://127.0.0.1:8080/areas.json",
    "http://127.0.0.1:8080/areas/test-hall.json",
    "http://127.0.0.1:8080/areas/test-yard.json",
  ]);
});

test("a plan path that is not the staged layout is refused", () => {
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "areas.0.plan", "https://example.invalid/plan.json")),
  );
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "areas.0.plan", "../../etc/passwd")),
  );
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "areas.0.plan", "//example.invalid/plan.json")),
  );
});

test("the collection and the plans must agree", async () => {
  const files = stagedFiles();
  files["areas/test-hall.json"] = edited(files["areas/test-hall.json"], "route.to_area", null);
  await assert.rejects(
    () => loadArtifacts("http://127.0.0.1:8080/", fetchFrom(files)),
    (error) => error instanceof ViewerError && error.code === CODES.CONSTRAINT,
  );

  const relabelled = stagedFiles();
  relabelled["areas/test-yard.json"] = edited(relabelled["areas/test-yard.json"], "area.label", "Elsewhere");
  await assert.rejects(
    () => loadArtifacts("http://127.0.0.1:8080/", fetchFrom(relabelled)),
    (error) => error instanceof ViewerError && error.code === CODES.CONSTRAINT,
  );
});

test("an artifact that cannot be read is refused with its name", async () => {
  await assert.rejects(
    () => loadArtifacts("http://127.0.0.1:8080/", fetchFrom({})),
    (error) => error instanceof ViewerError && error.code === CODES.UNREADABLE,
  );
  await assert.rejects(
    () =>
      loadArtifacts("http://127.0.0.1:8080/", async () => ({
        ok: true,
        status: 200,
        text: async () => "{not json",
      })),
    (error) => error instanceof ViewerError && /not well-formed JSON/.test(error.message),
  );
});
