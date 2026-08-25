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
  simulationProjection,
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
  assert.equal(decodePlan(hallPlan()).schema, "nomos.rendering_plan@3");
  const error = refuses(CODES.SCHEMA_MISMATCH, () =>
    decodePlan(edited(hallPlan(), "schema", "nomos.rendering_plan@2"), "areas/test-hall.json"),
  );
  assert.match(error.message, /expected schema `nomos\.rendering_plan@3`, found `nomos\.rendering_plan@2`/);
  assert.match(error.message, /areas\/test-hall\.json/);
  refuses(CODES.SCHEMA_MISMATCH, () => decodePlan(edited(hallPlan(), "schema", undefined)));
});

test("collection binds its identity and its route", () => {
  const collection = decodeCollection(collectionDocument());
  assert.equal(collection.schema, "nomos.area_collection@2");
  assert.deepEqual([...collection.order], ["test-hall", "test-yard"]);
  // The identity issue #152 retired. It was declared by quarantined tooling, and
  // an artifact still carrying it is refused by name rather than half-read.
  const stale = refuses(CODES.SCHEMA_MISMATCH, () =>
    decodeCollection(
      edited(collectionDocument(), "schema", "nomos.experiment.area_collection@2"),
      "areas.json",
    ),
  );
  assert.match(
    stale.message,
    /expected schema `nomos\.area_collection@2`, found `nomos\.experiment\.area_collection@2`/,
  );
  assert.match(stale.message, /areas\.json/);
  refuses(CODES.SCHEMA_MISMATCH, () =>
    decodeCollection(edited(collectionDocument(), "schema", "nomos.area_collection@1")),
  );
  // A route that does not reach every declared area is a broken run, not a
  // viewer that quietly plays three of four rooms.
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "route.0.to_area", null)),
  );
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "visual_grammar.rendering_plan_schema", "nomos.rendering_plan@2")),
  );
});

test("the collection names the entity kinds, not what they are drawn as", () => {
  // `@1` carried `entity_assemblies`: one row per kind naming an assembly and a
  // material family, which was the compiler's copy of a mapping this app's
  // catalog held as well. `@2` replaces those rows with the bare kind list,
  // because issue #153 left the catalog as the only place the mapping lives.
  assert.deepEqual(collectionDocument().visual_grammar.entity_kinds, ["door", "light", "water"]);
  assert.equal(decodeCollection(collectionDocument()).schema, "nomos.area_collection@2");
  const missing = refuses(CODES.DOCUMENT_SHAPE, () =>
    decodeCollection(edited(collectionDocument(), "visual_grammar.entity_kinds", undefined)),
  );
  assert.match(missing.message, /missing field `entity_kinds` at visual_grammar/);
  // A grammar still carrying the retired rows is refused by name, so a document
  // emitted against `@1` cannot be read as though it were `@2`.
  const stale = refuses(CODES.DOCUMENT_SHAPE, () =>
    decodeCollection(
      edited(collectionDocument(), "visual_grammar.entity_assemblies", [
        { kind: "door", material_family: "iron_oxidized", visual_assembly: "visual/iron_barred_door" },
      ]),
    ),
  );
  assert.match(stale.message, /unknown field `entity_assemblies` at visual_grammar/);
});

test("plan refuses an unknown field", () => {
  const error = refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "camera", { width: 1200 })));
  assert.match(error.message, /unknown field `camera`/);
  refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "area.palette", "gaol_bounded_01")));
  refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "entities.0.anchor.socket", "ward")));
  // `@2` carried `visual_assembly` and `material_family` on every entity, and
  // this decoder refused a plan whose pair disagreed with the catalog row for
  // the kind. `@3` drops both fields (issue #153), so there is one table and
  // that disagreement can no longer be spelled; what is left to prove is that a
  // plan still emitting either is refused outright rather than half-read.
  const retired = refuses(CODES.DOCUMENT_SHAPE, () =>
    decodePlan(edited(hallPlan(), "entities.0.visual_assembly", "visual/iron_barred_door")),
  );
  assert.match(retired.message, /unknown field `visual_assembly` at entities\[0\]/);
  refuses(CODES.DOCUMENT_SHAPE, () =>
    decodePlan(edited(hallPlan(), "entities.0.material_family", "iron_oxidized")),
  );
  // And an anchor whose kind is not the one its entity kind uses is a
  // constraint, not an unknown field.
  refuses(CODES.CONSTRAINT, () => decodePlan(edited(hallPlan(), "entities.0.anchor.kind", "cell")));
});

test("plan refuses a missing field", () => {
  const error = refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "pursuit", undefined)));
  assert.match(error.message, /missing field `pursuit`/);
  refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "scenarios.0.state_hash", undefined)));
  // `actors[].role` is `@3`'s, and both the runtime and the renderer read it, so
  // an actor that declares none is refused rather than inferred from the
  // identity string the study dispatched on.
  const role = refuses(CODES.DOCUMENT_SHAPE, () => decodePlan(edited(hallPlan(), "actors.0.role", undefined)));
  assert.match(role.message, /missing field `role` at actors\[0\]/);
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
  // `EntityKind::Unknown` reaches a plan as the kind `unknown`, for a primitive
  // the compiler has no visual kind for. The study drew a marker; this refuses
  // to pretend, and it is the last refusal standing between the two catalogs
  // now that `@3` carries no assembly name of its own.
  const document = structuredClone(hallPlan());
  document.entities[1] = { ...document.entities[1], kind: "unknown" };
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
  // A declared role is drawn from `ACTOR_ROLES`, so the identity string the
  // study told the two silhouettes apart by is not itself a role a plan may
  // name.
  refuses(CODES.CATALOG_UNKNOWN, () => decodePlan(edited(hallPlan(), "actors.1.role", "gaoler")));
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
  const { collection, plans, runtimeInputs } = await loadArtifacts(
    "http://127.0.0.1:8080/",
    fetchFrom(stagedFiles()),
  );
  assert.equal(collection.start_area, "test-hall");
  assert.deepEqual([...plans.keys()], ["test-hall", "test-yard"]);
  assert.equal(plans.get("test-hall").objective.gate, "hall_gate");
  // The bytes the authoritative runtime is handed, per area, as they were
  // staged: the app carries them rather than a re-serialization, because the
  // digests the runtime checks are over exactly these.
  assert.deepEqual([...runtimeInputs.keys()], ["test-hall", "test-yard"]);
  assert.deepEqual(
    JSON.parse(new TextDecoder().decode(runtimeInputs.get("test-hall").semantics)),
    simulationProjection("test-hall"),
  );

  const requested = [];
  const recording = fetchFrom(stagedFiles());
  await loadArtifacts("http://127.0.0.1:8080/", async (url) => {
    requested.push(url);
    return recording(url);
  });
  assert.deepEqual(requested, [
    "http://127.0.0.1:8080/areas.json",
    "http://127.0.0.1:8080/areas/test-hall.json",
    "http://127.0.0.1:8080/areas/test-hall.simulation.json",
    "http://127.0.0.1:8080/areas/test-yard.json",
    "http://127.0.0.1:8080/areas/test-yard.simulation.json",
  ]);
});

test("a plan file name that is not the staged layout is refused", () => {
  for (const name of [
    "https://example.invalid/plan.json",
    "../../etc/passwd",
    "//example.invalid/plan.json",
    "areas/test-hall.json",
  ]) {
    refuses(CODES.CONSTRAINT, () =>
      decodeCollection(edited(collectionDocument(), "areas.0.plan.file", name)),
    );
  }
  // The digest is a digest, and the app says so before it publishes one.
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "areas.0.plan.sha256", "not-a-digest")),
  );
});

test("the collection's two halves must agree with each other", () => {
  // The route is derived from the area rows by one emitter, so a document whose
  // halves disagree is a broken emitter rather than a viewer's problem to
  // reconcile.
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "route.0.gate", "yard_gate")),
  );
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "route.0.entry.x", 4)),
  );
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "areas.0.start", false)),
  );
  refuses(CODES.CONSTRAINT, () =>
    decodeCollection(edited(collectionDocument(), "areas.1.entry", null)),
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
  // Served, and not a document: the app decodes the bytes it fetched, so the
  // malformed artifact is handed over the way a server would hand it over.
  const malformed = new TextEncoder().encode("{not json");
  await assert.rejects(
    () =>
      loadArtifacts("http://127.0.0.1:8080/", async () => ({
        ok: true,
        status: 200,
        arrayBuffer: async () => malformed.buffer,
      })),
    (error) => error instanceof ViewerError && /not well-formed JSON/.test(error.message),
  );
});
