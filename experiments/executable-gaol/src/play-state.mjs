import { isHunting, movementOf } from "./renderer-catalog.mjs";

export const movementKeys = {
  ArrowUp: [0, -1], KeyW: [0, -1],
  ArrowDown: [0, 1], KeyS: [0, 1],
  ArrowLeft: [-1, 0], KeyA: [-1, 0],
  ArrowRight: [1, 0], KeyD: [1, 0],
};

// No fallback. `play-state.mjs` used to carry hardcoded defaults of exactly
// North Gaol's player and gaoler cells, so generic runtime code silently
// encoded one area's coordinates as "the" defaults — the ownership audit's
// fourth double authority. The presentation source is the only authority for
// where an actor starts, and it declares both actors for every area.
const actorPosition = (plan, id) => {
  const actor = plan.actors.find((entry) => entry.id === id);
  if (!actor) throw new Error(`plan for ${plan.area.id} declares no actor ${id}`);
  return { ...actor.cell };
};

export const displayName = (id) => id.replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase());

// The objective's gate, which the compiler derives from the single authored
// `route.exit.gate`. The study authored the same string three times.
const objectiveTarget = (plan) => plan.objective.gate;

export function createPlayState(plan) {
  return {
    player: actorPosition(plan, "player"),
    gaoler: actorPosition(plan, "gaoler"),
    movementCost: 0,
    moves: 0,
    areasCleared: 0,
    pursuitClock: 0,
    escaped: false,
    caught: false,
    completed: false,
    message: `Reach ${displayName(objectiveTarget(plan))}`,
    tone: "neutral",
  };
}

// Arrival places the player at the destination area's own `route.entry`.
// The exiting area used to name a cell inside its destination; each area now
// declares the one cell a player arrives on, validated against its own bounds
// and its own masses.
export function enterArea(plan, state) {
  const entry = plan.route.entry;
  if (!entry) throw new Error(`plan for ${plan.area.id} declares no arrival cell`);
  return {
    ...createPlayState(plan),
    player: { ...entry },
    movementCost: state.movementCost,
    moves: state.moves,
    areasCleared: state.areasCleared + 1,
    message: `Entered ${plan.area.label}`,
    tone: "success",
  };
}

export function completeRun(state) {
  return {
    ...state,
    areasCleared: state.areasCleared + 1,
    completed: true,
    message: "Escaped the gaol",
    tone: "success",
  };
}

export function completionSummary(state, totalAreas) {
  return `${totalAreas} areas · ${state.moves} moves · ${state.movementCost} traversal cost`;
}

const contains = (region, point) => point.x >= region.min.x && point.x <= region.max.x
  && point.y >= region.min.y && point.y <= region.max.y;

export function terrainAt(plan, scenario, point) {
  const water = plan.entities.find((entity) => entity.kind === "water" && contains(entity.anchor, point));
  if (!water) return { kind: "stone", entity: null, cost: 1 };
  return {
    kind: "water",
    entity: water.id,
    cost: movementOf(scenario, water.id)?.cost ?? 1,
  };
}

export function masonryAt(plan, point) {
  return plan.architecture.masses.find((mass) => point.x >= mass.min.x && point.x < mass.max.x
    && point.y >= mass.min.y && point.y < mass.max.y) ?? null;
}

export function attemptMove(plan, scenarioId, state, dx, dy) {
  const scenario = plan.scenarios.find((candidate) => candidate.id === scenarioId);
  if (!scenario) throw new Error(`unknown scenario ${scenarioId}`);
  if (state.escaped || state.caught) return { state, moved: false, cost: 0 };

  const target = { x: state.player.x + dx, y: state.player.y + dy, z: 0 };
  if (target.y < 0) {
    // The north face is the entity's declared `anchor.direction`, not a
    // `cell.y === 0` inference — the same convention the WebGL renderer used
    // for its wall segments, recorded by the audit as never reading the field
    // that already carried the answer.
    const door = plan.entities.find((entity) => entity.kind === "door"
      && entity.anchor.direction === "north" && entity.anchor.cell.x === state.player.x);
    const movement = door && movementOf(scenario, door.id);
    if (!door || movement?.disposition !== "traversable") {
      const reasons = movement?.reasons?.map((reason) => reason.split("#").at(-1)).join(" + ");
      return {
        state: { ...state, message: door ? `Blocked: ${reasons || door.id}` : "The masonry has no opening here", tone: "blocked" },
        moved: false,
        cost: 0,
      };
    }
    const cost = 1;
    return {
      state: {
        ...state,
        player: target,
        movementCost: state.movementCost + cost,
        moves: state.moves + 1,
        escaped: true,
        message: `Exited through ${door.id}`,
        tone: "success",
      },
      moved: true,
      cost,
      exitGate: door.id,
    };
  }

  const { width, height } = plan.architecture.bounds;
  if (target.x < 0 || target.x >= width || target.y >= height) {
    return { state: { ...state, message: "Blocked by masonry", tone: "blocked" }, moved: false, cost: 0 };
  }

  const masonry = masonryAt(plan, target);
  if (masonry) {
    return { state: { ...state, message: `Blocked by ${masonry.id}`, tone: "blocked" }, moved: false, cost: 0 };
  }

  const terrain = terrainAt(plan, scenario, target);
  const movedState = {
      ...state,
      player: target,
      movementCost: state.movementCost + terrain.cost,
      moves: state.moves + 1,
      message: terrain.kind === "water" ? `Shallow water costs ${terrain.cost}` : "Stone costs 1",
      tone: terrain.kind === "water" ? "water" : "neutral",
  };
  return {
    state: advanceGaoler(plan, scenario, movedState),
    moved: true,
    cost: terrain.cost,
  };
}

export function advanceGaoler(plan, scenario, state) {
  // One `isHunting` helper, shared with the HUD. The audit recorded these as
  // two logical mirrors written with opposite comparison operators and tied
  // together by nothing.
  if (state.escaped || state.caught || !isHunting(plan, scenario)) return state;

  const pursuitClock = state.pursuitClock + 1;
  if (pursuitClock < 2) return { ...state, pursuitClock };

  const dx = state.player.x - state.gaoler.x;
  const dy = state.player.y - state.gaoler.y;
  const gaoler = { ...state.gaoler };
  if (Math.abs(dx) > Math.abs(dy)) gaoler.x += Math.sign(dx);
  else if (dy !== 0) gaoler.y += Math.sign(dy);
  else gaoler.x += Math.sign(dx);

  const caught = gaoler.x === state.player.x && gaoler.y === state.player.y;
  return {
    ...state,
    gaoler,
    pursuitClock: 0,
    caught,
    message: caught ? "The gaoler caught you — press R to reset" : "The gaoler advances in the dark",
    tone: caught ? "blocked" : "danger",
  };
}

export function interactionAt(plan, scenarioId, state) {
  return plan.interactions
    .filter((interaction) => interaction.from_scenario === scenarioId)
    .map((interaction) => ({
      interaction,
      entity: plan.entities.find((entity) => entity.id === interaction.target_entity),
    }))
    .filter(({ entity }) => entity?.anchor?.cell)
    .find(({ entity }) => Math.abs(entity.anchor.cell.x - state.player.x)
      + Math.abs(entity.anchor.cell.y - state.player.y) <= 1)?.interaction ?? null;
}

export function guidanceFor(plan, scenarioId, state) {
  const target = objectiveTarget(plan);
  const targetLabel = displayName(target);
  if (state.completed) {
    return { objective: "Escape complete", prompt: "R · Begin a new run", tone: "success" };
  }
  if (state.caught) {
    return { objective: `Exit via ${targetLabel}`, prompt: "R · Restart the run", tone: "danger" };
  }
  if (state.escaped) {
    return { objective: `Exited via ${targetLabel}`, prompt: "Entering the next area", tone: "success" };
  }

  const interaction = interactionAt(plan, scenarioId, state);
  if (interaction) {
    return {
      objective: `Exit via ${targetLabel}`,
      prompt: `E · ${displayName(interaction.action)} ${displayName(interaction.target_entity)}`,
      tone: "action",
    };
  }

  const scenario = plan.scenarios.find((candidate) => candidate.id === scenarioId);
  const movement = scenario && movementOf(scenario, target);
  return {
    objective: `Exit via ${targetLabel}`,
    prompt: movement?.disposition === "traversable"
      ? `The way through ${targetLabel} is open`
      : `Reach ${targetLabel}`,
    tone: movement?.disposition === "traversable" ? "success" : "neutral",
  };
}

export function attemptInteraction(plan, scenarioId, state) {
  if (state.caught) {
    return {
      state: { ...state, message: "The gaoler caught you — press R to reset", tone: "blocked" },
      scenarioId,
      changed: false,
    };
  }
  const interaction = interactionAt(plan, scenarioId, state);
  if (!interaction) {
    return {
      state: { ...state, message: "Nothing responds here", tone: "neutral" },
      scenarioId,
      changed: false,
    };
  }
  return {
    state: {
      ...state,
      message: `${interaction.action} ${interaction.target_entity}`,
      tone: "success",
    },
    scenarioId: interaction.to_scenario,
    changed: true,
    interaction,
  };
}
