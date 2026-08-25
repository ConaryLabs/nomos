import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { basename, join } from "node:path";

const [worldDir, runsDir, areaPath, outputPath] = process.argv.slice(2);
if (!worldDir || !runsDir || !areaPath || !outputPath) {
  throw new Error("usage: build-plan.mjs <world> <runs> <area> <output>");
}

const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));
const area = readJson(areaPath);
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

const entityIds = new Set(entities.map((entity) => entity.id));
if (!area.id || !area.label) throw new Error("area identity is required");
if (!entityIds.has(area.primaryGate)) throw new Error(`primary gate ${area.primaryGate} is not a compiled entity`);
if (!entityIds.has(area.pursuitLight)) throw new Error(`pursuit light ${area.pursuitLight} is not a compiled entity`);
if (!area.actors.some((actor) => actor.id === "player") || !area.actors.some((actor) => actor.id === "gaoler")) {
  throw new Error("area requires player and gaoler presentation anchors");
}
if (area.effects.some((effect) => !entityIds.has(effect.anchorEntity))) {
  throw new Error("effect anchor must reference a compiled entity");
}
const { width, height } = area.architecture.bounds;
if (!Number.isInteger(width) || !Number.isInteger(height) || width < 1 || width > 9 || height < 1 || height > 6) {
  throw new Error("architecture bounds must fit the bounded 9x6 lattice");
}
if (area.architecture.wallHeight <= 0 || area.architecture.wallHeight > 5) {
  throw new Error("architecture wall height must be in (0, 5]");
}
for (const mass of area.architecture.masses) {
  if (mass.min.x < 0 || mass.min.y < 0 || mass.max.x > width || mass.max.y > height
    || mass.min.x >= mass.max.x || mass.min.y >= mass.max.y || mass.height <= 0 || mass.height > 4) {
    throw new Error(`masonry mass ${mass.id} exceeds the bounded architecture profile`);
  }
}

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
  area: { id: area.id, label: area.label },
  projectionSchemas: [simulation.schema, navigation.schema, persistence.schema, diagnostics.schema],
  projectionDigests,
  camera: { identity: "gaol_oblique_01", projection: "fixed_oblique", width: 1200, height: 540, tileWidth: 96, tileHeight: 50 },
  palette: "gaol_bounded_01",
  architecture: area.architecture,
  entities,
  actors: area.actors,
  effects: area.effects,
  presentation: {
    primaryGate: area.primaryGate,
    pursuitLight: area.pursuitLight,
    forensicScenario: area.forensicScenario,
  },
  uiAnchors: ["vitals", "abilities", "gate_state", "water_cost"],
  scenarios,
  interactions,
};

writeFileSync(outputPath, `${JSON.stringify(plan, null, 2)}\n`);
console.log(`RenderingPlan@1 ${entities.length} entities ${scenarios.length} scenarios ${interactions.length} interactions -> ${outputPath}`);
