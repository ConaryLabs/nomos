import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import {
  ACTOR_ASSEMBLIES,
  ARCHITECTURE_ASSEMBLIES,
  EFFECT_ASSEMBLIES,
  MATERIAL_FAMILIES,
  SOCKETS,
  TRIM_FAMILIES,
  lightOf,
  movementOf,
} from "./renderer-catalog.mjs";

const [planPath, sheetPath] = process.argv.slice(2);
const planBytes = readFileSync(planPath);
const sheetBytes = readFileSync(sheetPath);
const plan = JSON.parse(planBytes);
const fail = (message) => { throw new Error(message); };

if (plan.schema !== "nomos.rendering_plan@2") fail("wrong plan schema");
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

// No decimal reaches the plan. `nomos.presentation_source@1` is integer-only by
// the type its reader parses into and `nomos.rendering_plan@2` is emitted
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
for (const actor of plan.actors) member(ACTOR_ASSEMBLIES, actor.assembly, "actor assembly");
for (const effect of plan.effects) {
  member(EFFECT_ASSEMBLIES, effect.assembly, "effect assembly");
  const anchor = plan.entities.find((entity) => entity.id === effect.anchor.entity);
  if (!anchor) fail(`effect ${effect.id} anchors to absent entity ${effect.anchor.entity}`);
  if (!SOCKETS[anchor.visual_assembly]?.[effect.anchor.socket]) {
    fail(`effect ${effect.id} names socket ${effect.anchor.socket}, which ${anchor.visual_assembly} does not declare`);
  }
}

const forensicBytes = readFileSync(new URL("forensic.svg", `file://${sheetPath.replace(/[^/]+$/, "")}`));
if (!forensicBytes.includes(Buffer.from("FORENSIC PROJECTION OWNERSHIP"))) fail("forensic overlay missing");

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
console.log(`EXECUTABLE_GAOL_VERIFY PASS plan=${digest(planBytes)} contact_sheet=${digest(sheetBytes)}`);
