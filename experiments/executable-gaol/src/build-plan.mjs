import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { basename, join } from "node:path";

const [worldDir, runsDir, outputPath] = process.argv.slice(2);
if (!worldDir || !runsDir || !outputPath) {
  throw new Error("usage: build-plan.mjs <world> <runs> <output>");
}

const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));
const simulation = readJson(join(worldDir, "simulation.json"));
const navigation = readJson(join(worldDir, "navigation.json"));
const persistence = readJson(join(worldDir, "persistence.json"));
const diagnostics = readJson(join(worldDir, "diagnostics.json"));

const navByEntity = new Map(
  navigation.movement_resolver.subjects.map((subject) => [subject.entity, subject]),
);
const lightEntities = new Set(
  persistence.light_resolver.subjects.map((subject) => subject.entity),
);

const classify = (entity) => {
  if (entity.machines.some((machine) => machine.endsWith(".access"))) return "door";
  if (lightEntities.has(entity.id)) return "light";
  if (navByEntity.get(entity.id)?.claims.some((claim) => claim.capability === "traversal_cost_ground")) return "water";
  return "unknown";
};

const entities = simulation.entities.map((entity) => {
  const kind = classify(entity);
  const visualAssembly = {
    door: "visual/iron_barred_door",
    light: "visual/brazier",
    water: "visual/shallow_water",
    unknown: "visual/marker",
  }[kind];
  return {
    id: entity.id,
    kind,
    visualAssembly,
    materialFamily: { door: "iron_oxidized", light: "iron_brazier", water: "water_cold" }[kind] ?? "stone",
    anchor: entity.binding,
    machineNamespaces: entity.machines,
    provenance: (navByEntity.get(entity.id)?.claims ?? []).map((claim) => ({
      claim: claim.id,
      source: claim.source,
    })),
  };
});

const activationIsActive = (activation, states) => {
  switch (activation.kind) {
    case "always": return true;
    case "state_equals": return states[activation.namespace] === activation.state;
    case "not": return !activationIsActive(activation.child, states);
    case "any": return activation.children.some((child) => activationIsActive(child, states));
    case "all": return activation.children.every((child) => activationIsActive(child, states));
    default: throw new Error(`unsupported activation ${activation.kind}`);
  }
};

const scenarioDirs = readdirSync(runsDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort();

const scenarioRecords = scenarioDirs.map((name) => {
  const finalState = readJson(join(runsDir, name, "final-state.json"));
  const result = readJson(join(runsDir, name, "result.json"));
  const commandLog = readJson(join(runsDir, name, "command-log.json"));
  const expectedBaselineRejection = name === "01-baseline" && result.status === "rejected" && result.committed_command_count === 0;
  if (result.status !== "completed" && !expectedBaselineRejection) throw new Error(`${name} did not reach its declared state`);
  const states = Object.fromEntries(
    finalState.state.machines.map((machine) => [machine.namespace, machine.state]),
  );
  const movement = Object.fromEntries(
    navigation.movement_resolver.subjects.map((subject) => {
      const active = subject.claims.filter((claim) => activationIsActive(claim.activation, states));
      const blockers = active.filter((claim) => claim.capability === "blocks_ground").map((claim) => claim.id);
      const costs = active.filter((claim) => claim.capability === "traversal_cost_ground").map((claim) => claim.value);
      return [subject.entity, {
        disposition: blockers.length ? "blocked" : "traversable",
        cost: blockers.length ? null : Math.max(navigation.movement_resolver.base_cost, ...costs),
        reasons: blockers.length ? blockers : active.map((claim) => claim.id),
      }];
    }),
  );
  const effectiveLight = Object.fromEntries(
    simulation.light_resolver.subjects.map((subject) => [
      subject.entity,
      subject.claims.some((claim) => activationIsActive(claim.activation, states)),
    ]),
  );
  return {
    committedRows: commandLog.rows,
    scenario: {
    id: name,
    label: name.replace(/^\d+-/, "").replaceAll("-", " "),
    tick: finalState.state.tick,
    stateHash: finalState.state_hash,
    machineStates: states,
    movement,
    effectiveLight,
    },
  };
});

const scenarios = scenarioRecords.map((record) => record.scenario);
const interactions = [];
for (const from of scenarioRecords) for (const to of scenarioRecords) {
  if (to.committedRows.length !== from.committedRows.length + 1) continue;
  const prefixMatches = from.committedRows.every((row, index) =>
    JSON.stringify(row.request) === JSON.stringify(to.committedRows[index].request)
    && row.resulting_state_hash === to.committedRows[index].resulting_state_hash);
  if (!prefixMatches) continue;
  const next = to.committedRows.at(-1);
  if (next.input_state_hash !== from.scenario.stateHash) continue;
  interactions.push({
    id: `${from.scenario.id}:${next.request.action}:${next.request.entity}`,
    fromScenario: from.scenario.id,
    toScenario: to.scenario.id,
    targetEntity: next.request.entity,
    action: next.request.action,
    inputStateHash: next.input_state_hash,
    resultingStateHash: next.resulting_state_hash,
  });
}

const projectionDigests = Object.fromEntries(
  ["simulation.json", "navigation.json", "persistence.json", "diagnostics.json"].map((file) => [
    file,
    createHash("sha256").update(readFileSync(join(worldDir, file))).digest("hex"),
  ]),
);

const plan = {
  schema: "nomos.experiment.rendering_plan@1",
  deterministic: true,
  projectionSchemas: [simulation.schema, navigation.schema, persistence.schema, diagnostics.schema],
  projectionDigests,
  camera: { identity: "gaol_oblique_01", projection: "fixed_oblique", width: 1200, height: 540, tileWidth: 96, tileHeight: 50 },
  palette: "gaol_bounded_01",
  entities,
  actors: [
    { id: "player", assembly: "visual/player_silhouette", anchor: { kind: "cell", cell: { x: 2, y: 4, z: 0 } } },
    { id: "gaoler", assembly: "visual/gaoler_silhouette", anchor: { kind: "cell", cell: { x: 5, y: 3, z: 0 } } },
  ],
  effects: [{ id: "ward_crescent", assembly: "visual/cyan_crescent", anchorEntity: "north_gate" }],
  uiAnchors: ["vitals", "abilities", "gate_state", "water_cost"],
  scenarios,
  interactions,
};

writeFileSync(outputPath, `${JSON.stringify(plan, null, 2)}\n`);
console.log(`RenderingPlan@1 ${entities.length} entities ${scenarios.length} scenarios ${interactions.length} interactions -> ${outputPath}`);
