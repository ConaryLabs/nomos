import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  ACTOR_ASSEMBLIES,
  ARCHITECTURE_ASSEMBLIES,
  EFFECT_ASSEMBLIES,
  MATERIAL_FAMILIES,
  SOCKETS,
  TRIM_FAMILIES,
  assemblyOf,
  lightOf,
  movementOf,
} from "./renderer-catalog.mjs";

const digestOf = (bytes) => createHash("sha256").update(bytes).digest("hex");

// The area collection, which `crates/nomos-render-plan/src/collection.rs` emits
// and `build-collection.mjs` used to. The Rust tests own the route-graph
// semantics; this is the comparison the study keeps: the committed example is
// byte-equal to what the compiler just produced, and every digest the collection
// publishes is a digest of the plan file actually on disk.
if (process.argv[2] === "--collection") {
  const [collectionPath, examplePath, areasDir] = process.argv.slice(3);
  if (!collectionPath || !examplePath || !areasDir) {
    throw new Error("usage: verify.mjs --collection <areas.json> <example> <areas-dir>");
  }
  const bytes = readFileSync(collectionPath);
  const expected = readFileSync(examplePath);
  if (!bytes.equals(expected)) {
    throw new Error(`${collectionPath} is not byte-equal to the committed ${examplePath}`);
  }
  const collection = JSON.parse(bytes);
  if (collection.schema !== "nomos.area_collection@2") throw new Error("wrong collection schema");
  const chain = [];
  let current = collection.start_area;
  while (current !== null) {
    const area = collection.areas.find((one) => one.id === current);
    if (!area) throw new Error(`the route names undeclared area ${current}`);
    if (chain.includes(current)) throw new Error(`the route cycles at ${current}`);
    chain.push(current);
    const planBytes = readFileSync(join(areasDir, area.id, "rendering-plan.json"));
    if (area.plan.file !== `${area.id}.json`) {
      throw new Error(`${area.id} publishes its plan as ${area.plan.file}`);
    }
    if (area.plan.sha256 !== digestOf(planBytes)) {
      throw new Error(`${area.id} does not carry the digest of its own plan`);
    }
    const plan = JSON.parse(planBytes);
    if (plan.area.label !== area.label) throw new Error(`${area.id} disagrees about its label`);
    if (plan.objective.gate !== area.exit.gate) throw new Error(`${area.id} disagrees about its gate`);
    if ((plan.route.to_area ?? null) !== area.exit.to_area) {
      throw new Error(`${area.id} disagrees about its destination`);
    }
    current = area.exit.to_area;
  }
  if (chain.length !== collection.areas.length) {
    throw new Error(`the route visits ${chain.length} of ${collection.areas.length} areas`);
  }
  console.log(
    `AREA_COLLECTION_VERIFY PASS areas=${chain.length} ` +
      `grammar=${collection.visual_grammar.digest} collection=${digestOf(bytes)}`,
  );
  process.exit(0);
}

const [planPath, sheetPath] = process.argv.slice(2);
const planBytes = readFileSync(planPath);
const sheetBytes = readFileSync(sheetPath);
const plan = JSON.parse(planBytes);
const fail = (message) => { throw new Error(message); };

if (plan.schema !== "nomos.rendering_plan@3") fail("wrong plan schema");
if (plan.entities.filter((entity) => entity.kind === "door").length !== 2) fail("second content-driven door absent");
if (plan.scenarios.length !== 5) fail("expected five scenarios");
if (plan.interactions.length !== 3) fail("expected three verified in-world interactions");
if (plan.interactions.some((interaction) => interaction.input_state_hash !== plan.scenarios.find((scenario) => scenario.id === interaction.from_scenario)?.state_hash)) fail("interaction input hash is not scenario-bound");
if (plan.interactions.some((interaction) => interaction.resulting_state_hash !== plan.scenarios.find((scenario) => scenario.id === interaction.to_scenario)?.state_hash)) fail("interaction result hash is not scenario-bound");
if (plan.scenarios.some((scenario) => !scenario.state_hash)) fail("scenario is not runtime-bound");
if (plan.objective?.kind !== "exit_via") fail("bounded area objective missing");
if (plan.entities.find((entity) => entity.id === plan.objective.gate)?.kind !== "door") fail("objective does not target a compiled door");
if (movementOf(plan.scenarios[0], plan.objective.gate).disposition !== "blocked") fail("baseline gate must be blocked");
if (movementOf(plan.scenarios[2], plan.objective.gate).disposition !== "traversable") fail("breached/unsealed gate must be traversable");
if (lightOf(plan.scenarios[3], plan.pursuit.light) !== false) fail("breached dark scenario must extinguish the pursuit light");

// No decimal reaches the plan. `nomos.presentation_source@2` is integer-only by
// the type its reader parses into and `nomos.rendering_plan@3` is emitted
// through `nomos_core::CanonicalValue`, which has no floating-point variant;
// this is the same statement checked against the artifact rather than the code.
if (/[-\d]\.\d/.test(planBytes.toString("utf8").replaceAll(/"[^"]*"/g, '""'))) {
  fail("the rendering plan carries a decimal literal");
}

// The renderer catalog defines the closed sets; the presentation source selects
// from them. The Rust decoder checks that each name is well formed, and this is
// the other half: that the name the author chose is one the renderer can
// actually draw. A name legal to the compiler but unknown here fails the build
// rather than a frame.
const member = (set, value, what) => {
  if (!set.includes(value)) fail(`${what} ${value} is not in the renderer catalog`);
};
member(ARCHITECTURE_ASSEMBLIES, plan.architecture.style.assembly, "architecture assembly");
member(MATERIAL_FAMILIES, plan.architecture.style.material_family, "material family");
member(TRIM_FAMILIES, plan.architecture.style.trim_family, "trim family");
for (const actor of plan.actors) {
  member(ACTOR_ASSEMBLIES, actor.assembly, "actor assembly");
  member(["player", "pursuer"], actor.role, "actor role");
}
if (plan.actors.filter((actor) => actor.role === "player").length !== 1) {
  fail("a plan declares exactly one player actor");
}
if (plan.actors.filter((actor) => actor.role === "pursuer").length > 1) {
  fail("a plan declares at most one pursuer actor");
}
// The plan carries no assembly or material family for a compiled entity any
// more; the catalog owns both, and every kind the plan uses must resolve.
for (const entity of plan.entities) assemblyOf(entity.kind);
if (plan.entities.some((entity) => "visual_assembly" in entity || "material_family" in entity)) {
  fail("a rendering_plan@3 entity carries no visual_assembly and no material_family");
}
for (const effect of plan.effects) {
  member(EFFECT_ASSEMBLIES, effect.assembly, "effect assembly");
  const anchor = plan.entities.find((entity) => entity.id === effect.anchor.entity);
  if (!anchor) fail(`effect ${effect.id} anchors to absent entity ${effect.anchor.entity}`);
  if (!SOCKETS[anchor.kind]?.[effect.anchor.socket]) {
    fail(`effect ${effect.id} names socket ${effect.anchor.socket}, which kind ${anchor.kind} does not declare`);
  }
}

const forensicBytes = readFileSync(new URL("forensic.svg", `file://${sheetPath.replace(/[^/]+$/, "")}`));
if (!forensicBytes.includes(Buffer.from("FORENSIC PROJECTION OWNERSHIP"))) fail("forensic overlay missing");

console.log(`EXECUTABLE_GAOL_VERIFY PASS plan=${digestOf(planBytes)} contact_sheet=${digestOf(sheetBytes)}`);
