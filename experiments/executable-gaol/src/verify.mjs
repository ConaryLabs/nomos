import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const [planPath, sheetPath] = process.argv.slice(2);
const planBytes = readFileSync(planPath);
const sheetBytes = readFileSync(sheetPath);
const plan = JSON.parse(planBytes);
const fail = (message) => { throw new Error(message); };

if (plan.schema !== "nomos.experiment.rendering_plan@1") fail("wrong plan schema");
if (plan.entities.filter((entity) => entity.kind === "door").length !== 2) fail("second content-driven door absent");
if (plan.scenarios.length !== 4) fail("expected four scenarios");
if (plan.scenarios.some((scenario) => !scenario.stateHash)) fail("scenario is not runtime-bound");
if (plan.scenarios[0].movement.north_gate.disposition !== "blocked") fail("baseline gate must be blocked");
if (plan.scenarios[2].movement.north_gate.disposition !== "traversable") fail("breached/unsealed gate must be traversable");
if (plan.scenarios[3].effectiveLight.brazier_02 !== false) fail("dark scenario must extinguish the brazier");
const forensicBytes = readFileSync(new URL("forensic.svg", `file://${sheetPath.replace(/[^/]+$/, "")}`));
if (!forensicBytes.includes(Buffer.from("FORENSIC PROJECTION OWNERSHIP"))) fail("forensic overlay missing");

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
console.log(`EXECUTABLE_GAOL_VERIFY PASS plan=${digest(planBytes)} contact_sheet=${digest(sheetBytes)}`);
