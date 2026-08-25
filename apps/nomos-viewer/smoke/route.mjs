// The route solver.
//
// Test tooling: it never ships in `dist/`. It derives the key sequence from the
// artifacts, so a content change moves the route without anyone editing the
// harness - which is also the strongest form of the claim that adding an area
// needs no edit under `apps/`.
//
// It decodes with the app's own `plan.mjs`. There is no second decoder here,
// and no area identifier or cell coordinate written into this file.

import { initialScenario, interactionFrom, movementOf, scenarioOf } from "../src/plan.mjs";

const KEY_FOR = {
  north: "ArrowUp",
  south: "ArrowDown",
  west: "ArrowLeft",
  east: "ArrowRight",
};

const STEPS = [
  { dx: 0, dy: -1, key: "ArrowUp" },
  { dx: 0, dy: 1, key: "ArrowDown" },
  { dx: -1, dy: 0, key: "ArrowLeft" },
  { dx: 1, dy: 0, key: "ArrowRight" },
];

const blocked = (plan, cell) => {
  const { width, height } = plan.architecture.bounds;
  if (cell.x < 0 || cell.y < 0 || cell.x >= width || cell.y >= height) return true;
  return plan.architecture.masses.some(
    (mass) =>
      cell.x >= mass.min.x && cell.x < mass.max.x && cell.y >= mass.min.y && cell.y < mass.max.y,
  );
};

const waterAt = (plan, cell) =>
  plan.entities.find(
    (one) =>
      one.kind === "water" &&
      cell.x >= one.anchor.min.x &&
      cell.x <= one.anchor.max.x &&
      cell.y >= one.anchor.min.y &&
      cell.y <= one.anchor.max.y,
  ) ?? null;

const costOf = (plan, scenario, cell) => {
  const water = waterAt(plan, cell);
  return water ? movementOf(scenario, water.id).cost : 1;
};

// Cheapest walk to the first cell `accept` likes, tie-broken by steps and then
// by position, so the sequence is the same on every run.
function walk(plan, scenario, from, accept) {
  const key = (cell) => `${cell.x},${cell.y}`;
  const best = new Map([[key(from), 0]]);
  let frontier = [{ cell: from, cost: 0, steps: 0, keys: [] }];
  while (frontier.length > 0) {
    frontier.sort(
      (left, right) =>
        left.cost - right.cost ||
        left.steps - right.steps ||
        left.cell.y - right.cell.y ||
        left.cell.x - right.cell.x,
    );
    const node = frontier.shift();
    if (accept(node.cell)) return node;
    for (const step of STEPS) {
      const next = { x: node.cell.x + step.dx, y: node.cell.y + step.dy };
      if (blocked(plan, next)) continue;
      const cost = node.cost + costOf(plan, scenario, next);
      if (best.has(key(next)) && best.get(key(next)) <= cost) continue;
      best.set(key(next), cost);
      frontier.push({ cell: next, cost, steps: node.steps + 1, keys: [...node.keys, step.key] });
    }
  }
  throw new Error(`no walk from ${key(from)} in ${plan.area.id}`);
}

const manhattan = (left, right) => Math.abs(left.x - right.x) + Math.abs(left.y - right.y);

/// Solves the whole run: one leg per area, in the collection's route order.
export function solveRoute(collection, plans) {
  const legs = [];
  let moves = 0;
  let cost = 0;
  let areaId = collection.start_area;

  while (areaId !== null) {
    const plan = plans.get(areaId);
    let scenario = initialScenario(plan);
    let cell = plan.area.start
      ? { ...plan.actors.find((one) => one.id === "player").cell }
      : { ...plan.route.entry };
    const keys = [];
    const spend = (leg) => {
      keys.push(...leg.keys);
      moves += leg.steps;
      cost += leg.cost;
      cell = leg.cell;
    };

    // Cross the water on the way, where there is any. Without this the cheapest
    // walk avoids it, the cumulative cost equals the move count, and a
    // regression in projected traversal cost would pass unnoticed.
    const water = plan.entities.find((one) => one.kind === "water");
    if (water) {
      const target = { x: water.anchor.min.x, y: water.anchor.min.y };
      if (!blocked(plan, target)) {
        spend(walk(plan, scenario, cell, (one) => one.x === target.x && one.y === target.y));
      }
    }

    const gate = plan.entities.find((one) => one.id === plan.objective.gate);
    let guard = 0;
    while (movementOf(scenario, gate.id).disposition !== "traversable") {
      if (guard++ > plan.scenarios.length) {
        throw new Error(`no interaction chain opens ${gate.id} in ${plan.area.id}`);
      }
      const interaction = interactionFrom(plan, scenario.id);
      if (!interaction) throw new Error(`${plan.area.id} offers no interaction from ${scenario.id}`);
      const target = plan.entities.find((one) => one.id === interaction.target_entity);
      if (!target.anchor.cell) {
        throw new Error(`${interaction.id} targets ${target.id}, which has no cell to stand beside`);
      }
      spend(walk(plan, scenario, cell, (one) => manhattan(one, target.anchor.cell) <= 1));
      keys.push("KeyE");
      scenario = scenarioOf(plan, interaction.to_scenario);
    }

    spend(
      walk(
        plan,
        scenario,
        cell,
        (one) => one.x === gate.anchor.cell.x && one.y === gate.anchor.cell.y,
      ),
    );
    keys.push(KEY_FOR[gate.anchor.direction]);
    moves += 1;
    cost += 1;

    const edge = collection.route.find((one) => one.from_area === areaId);
    legs.push({
      area: areaId,
      gate: gate.id,
      scenario: scenario.id,
      keys,
      moves,
      cost,
      to: edge.to_area,
    });
    areaId = edge.to_area;
  }

  return {
    legs,
    keys: legs.flatMap((one) => one.keys),
    areas: legs.length,
    moves,
    cost,
    summary: `${legs.length} areas · ${moves} moves · ${cost} traversal cost`,
  };
}
