import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const [planPath, sheetPath] = process.argv.slice(2);
const planBytes = readFileSync(planPath);
const sheetBytes = readFileSync(sheetPath);
const plan = JSON.parse(planBytes);
const fail = (message) => { throw new Error(message); };

if (plan.schema !== "nomos.experiment.rendering_plan@1") fail("wrong plan schema");
if (plan.entities.filter((entity) => entity.kind === "door").length !== 2) fail("second content-driven door absent");
if (plan.scenarios.length !== 5) fail("expected five scenarios");
if (plan.interactions.length !== 3) fail("expected three verified in-world interactions");
if (plan.interactions.some((interaction) => interaction.inputStateHash !== plan.scenarios.find((scenario) => scenario.id === interaction.fromScenario)?.stateHash)) fail("interaction input hash is not scenario-bound");
if (plan.interactions.some((interaction) => interaction.resultingStateHash !== plan.scenarios.find((scenario) => scenario.id === interaction.toScenario)?.stateHash)) fail("interaction result hash is not scenario-bound");
if (plan.scenarios.some((scenario) => !scenario.stateHash)) fail("scenario is not runtime-bound");
if (plan.scenarios[0].movement[plan.presentation.primaryGate].disposition !== "blocked") fail("baseline gate must be blocked");
if (plan.scenarios[2].movement[plan.presentation.primaryGate].disposition !== "traversable") fail("breached/unsealed gate must be traversable");
if (plan.scenarios[3].effectiveLight[plan.presentation.pursuitLight] !== false) fail("breached dark scenario must extinguish the pursuit light");
const forensicBytes = readFileSync(new URL("forensic.svg", `file://${sheetPath.replace(/[^/]+$/, "")}`));
if (!forensicBytes.includes(Buffer.from("FORENSIC PROJECTION OWNERSHIP"))) fail("forensic overlay missing");

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
console.log(`EXECUTABLE_GAOL_VERIFY PASS plan=${digest(planBytes)} contact_sheet=${digest(sheetBytes)}`);
