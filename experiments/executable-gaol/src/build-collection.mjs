import { createHash } from "node:crypto";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// The area collection: the route graph, and the visual grammar every area is
// required to share.
//
// `nomos.experiment.area_collection@2` differs from `@1` in what it no longer
// carries. `camera`, `palette`, `ui_anchors`, and `deterministic` are gone
// because they left the rendering plan: they were renderer-catalog constants
// re-typed into every content artifact, or dead flags nothing read. The
// look-profile *id* is gone too, so "which visual look is active" has one
// identifier scheme — the renderer catalog's LOOK_PROFILE_IDS — instead of the
// four the ownership audit found. What remains is renamed `visual_grammar`,
// which is what it always was, and spelled snake_case to match the plan.

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
  rendering_plan_schema: plan.schema,
  projection_schemas: plan.projection_schemas,
  architecture_style: plan.architecture.style,
  entity_assemblies: uniqueRows(plan.entities.map((entity) => [entity.kind, entity.visual_assembly, entity.material_family])),
  actor_assemblies: uniqueRows(plan.actors.map((actor) => actor.assembly)),
  effect_assemblies: uniqueRows(plan.effects.map((effect) => effect.assembly)),
});

const grammar = visualGrammar(plans[0].plan);
const grammarBytes = JSON.stringify(grammar);
const byId = new Map(plans.map(({ plan }) => [plan.area.id, plan]));
for (const { directory, plan } of plans) {
  if (plan.area.id !== directory) throw new Error(`${directory} does not match plan area identity ${plan.area.id}`);
  if (JSON.stringify(visualGrammar(plan)) !== grammarBytes) {
    throw new Error(`${directory} diverges from the shared visual grammar`);
  }
  // Each area validates its own arrival cell against its own bounds and its own
  // masses, inside the compiler. What is left for the collection is the one
  // check no single area can make: that the area a gate leads to exists and can
  // actually receive an arrival.
  if (plan.area.start !== (plan.route.entry === undefined)) {
    throw new Error(`${directory} must declare an arrival cell if and only if it is not the start area`);
  }
  if (plan.route.to_area !== null) {
    const target = byId.get(plan.route.to_area);
    if (!target) throw new Error(`${directory} targets unknown area ${plan.route.to_area}`);
    if (!target.route.entry) {
      throw new Error(`${directory} leads to ${plan.route.to_area}, which declares no arrival cell`);
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
  const toArea = plan.route.to_area;
  route.push({
    from_area: current,
    gate: plan.objective.gate,
    to_area: toArea,
    // The arrival cell is the destination's own declaration, read here so the
    // viewer can follow one edge without loading the next plan first.
    entry: toArea === null ? null : byId.get(toArea).route.entry,
  });
  current = toArea;
}
if (visited.size !== plans.length) throw new Error("area route does not visit every declared area");

const collection = {
  schema: "nomos.experiment.area_collection@2",
  visual_grammar: {
    digest: createHash("sha256").update(grammarBytes).digest("hex"),
    ...grammar,
  },
  start_area: startArea,
  route,
  areas: plans.map(({ plan }) => ({
    id: plan.area.id,
    label: plan.area.label,
    plan: `areas/${plan.area.id}.json`,
  })),
};

writeFileSync(outputPath, `${JSON.stringify(collection, null, 2)}\n`);
console.log(`AreaCollection@2 ${plans.length} areas grammar=${collection.visual_grammar.digest} -> ${outputPath}`);
