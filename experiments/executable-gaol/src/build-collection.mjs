import { createHash } from "node:crypto";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [areasDir, outputPath] = process.argv.slice(2);
if (!areasDir || !outputPath) throw new Error("usage: build-collection.mjs <areas-dir> <output>");

const plans = readdirSync(areasDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => ({
    directory: entry.name,
    plan: JSON.parse(readFileSync(join(areasDir, entry.name, "rendering-plan.json"), "utf8")),
  }))
  .sort((left, right) => left.directory.localeCompare(right.directory));

if (plans.length < 2) throw new Error("the consistency proof requires at least two areas");

const uniqueRows = (rows) => [...new Set(rows.map((row) => JSON.stringify(row)))].map((row) => JSON.parse(row)).sort();
const visualGrammar = (plan) => ({
  renderingPlanSchema: plan.schema,
  projectionSchemas: plan.projectionSchemas,
  camera: plan.camera,
  palette: plan.palette,
  entityAssemblies: uniqueRows(plan.entities.map((entity) => [entity.kind, entity.visualAssembly, entity.materialFamily])),
  actorAssemblies: uniqueRows(plan.actors.map((actor) => actor.assembly)),
  effectAssemblies: uniqueRows(plan.effects.map((effect) => effect.assembly)),
  uiAnchors: plan.uiAnchors,
});

const grammar = visualGrammar(plans[0].plan);
const grammarBytes = JSON.stringify(grammar);
for (const { directory, plan } of plans) {
  if (plan.area.id !== directory) throw new Error(`${directory} does not match plan area identity ${plan.area.id}`);
  if (JSON.stringify(visualGrammar(plan)) !== grammarBytes) {
    throw new Error(`${directory} diverges from the shared visual grammar`);
  }
}

const collection = {
  schema: "nomos.experiment.area_collection@1",
  deterministic: true,
  lookProfile: {
    id: "gaol_bounded_01",
    digest: createHash("sha256").update(grammarBytes).digest("hex"),
    grammar,
  },
  areas: plans.map(({ plan }) => ({
    id: plan.area.id,
    label: plan.area.label,
    plan: `areas/${plan.area.id}.json`,
  })),
};

writeFileSync(outputPath, `${JSON.stringify(collection, null, 2)}\n`);
console.log(`AreaCollection@1 ${plans.length} areas look=${collection.lookProfile.digest} -> ${outputPath}`);
