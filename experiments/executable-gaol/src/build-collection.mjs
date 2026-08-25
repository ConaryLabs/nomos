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
  architectureStyle: plan.architecture.style,
  entityAssemblies: uniqueRows(plan.entities.map((entity) => [entity.kind, entity.visualAssembly, entity.materialFamily])),
  actorAssemblies: uniqueRows(plan.actors.map((actor) => actor.assembly)),
  effectAssemblies: uniqueRows(plan.effects.map((effect) => effect.assembly)),
  uiAnchors: plan.uiAnchors,
});

const grammar = visualGrammar(plans[0].plan);
const grammarBytes = JSON.stringify(grammar);
const byId = new Map(plans.map(({ plan }) => [plan.area.id, plan]));
for (const { directory, plan } of plans) {
  if (plan.area.id !== directory) throw new Error(`${directory} does not match plan area identity ${plan.area.id}`);
  if (JSON.stringify(visualGrammar(plan)) !== grammarBytes) {
    throw new Error(`${directory} diverges from the shared visual grammar`);
  }
  const exit = plan.presentation.exit;
  if (exit.gate !== plan.presentation.primaryGate) throw new Error(`${directory} exit is not its primary gate`);
  if (exit.toArea !== null) {
    const target = byId.get(exit.toArea);
    if (!target) throw new Error(`${directory} targets unknown area ${exit.toArea}`);
    const { width, height } = target.architecture.bounds;
    if (!exit.entry || exit.entry.x < 0 || exit.entry.x >= width || exit.entry.y < 0 || exit.entry.y >= height) {
      throw new Error(`${directory} has an invalid entry into ${exit.toArea}`);
    }
    if (target.architecture.masses.some((mass) => exit.entry.x >= mass.min.x && exit.entry.x < mass.max.x
      && exit.entry.y >= mass.min.y && exit.entry.y < mass.max.y)) {
      throw new Error(`${directory} enters ${exit.toArea} inside masonry`);
    }
  }
}

const starts = plans.filter(({ plan }) => plan.area.start);
if (starts.length !== 1) throw new Error("area collection requires exactly one start area");
const startArea = starts[0].plan.area.id;
const route = [];
const visited = new Set();
let current = startArea;
while (current !== null) {
  if (visited.has(current)) throw new Error(`area route cycles at ${current}`);
  visited.add(current);
  const plan = byId.get(current);
  route.push({ fromArea: current, ...plan.presentation.exit });
  current = plan.presentation.exit.toArea;
}
if (visited.size !== plans.length) throw new Error("area route does not visit every declared area");

const collection = {
  schema: "nomos.experiment.area_collection@1",
  deterministic: true,
  lookProfile: {
    id: "gaol_bounded_01",
    digest: createHash("sha256").update(grammarBytes).digest("hex"),
    grammar,
  },
  startArea,
  route,
  areas: plans.map(({ plan }) => ({
    id: plan.area.id,
    label: plan.area.label,
    plan: `areas/${plan.area.id}.json`,
  })),
};

writeFileSync(outputPath, `${JSON.stringify(collection, null, 2)}\n`);
console.log(`AreaCollection@1 ${plans.length} areas look=${collection.lookProfile.digest} -> ${outputPath}`);
