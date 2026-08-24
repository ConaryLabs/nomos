export const movementKeys = {
  ArrowUp: [0, -1], KeyW: [0, -1],
  ArrowDown: [0, 1], KeyS: [0, 1],
  ArrowLeft: [-1, 0], KeyA: [-1, 0],
  ArrowRight: [1, 0], KeyD: [1, 0],
};

const actorPosition = (plan, id, fallback) => ({
  ...(plan?.actors.find((actor) => actor.id === id)?.anchor?.cell ?? fallback),
});

export function createPlayState(plan) {
  return {
    player: actorPosition(plan, "player", { x: 2, y: 4, z: 0 }),
    gaoler: actorPosition(plan, "gaoler", { x: 5, y: 3, z: 0 }),
    movementCost: 0,
    moves: 0,
    areasCleared: 0,
    pursuitClock: 0,
    escaped: false,
    caught: false,
    completed: false,
    message: "Reach the north gate",
    tone: "neutral",
  };
}

export function enterArea(plan, state, entry) {
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

const contains = (region, point) => point.x >= region.min.x && point.x <= region.max.x
  && point.y >= region.min.y && point.y <= region.max.y;

export function terrainAt(plan, scenario, point) {
  const water = plan.entities.find((entity) => entity.kind === "water" && contains(entity.anchor, point));
  if (!water) return { kind: "stone", entity: null, cost: 1 };
  return {
    kind: "water",
    entity: water.id,
    cost: scenario.movement[water.id]?.cost ?? 1,
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
    const door = plan.entities.find((entity) => entity.kind === "door"
      && entity.anchor.cell.x === state.player.x && entity.anchor.cell.y === 0);
    const movement = door && scenario.movement[door.id];
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
  const pursuitLight = plan.presentation.pursuitLight;
  if (state.escaped || state.caught || scenario.effectiveLight[pursuitLight] !== false) return state;

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
    .filter((interaction) => interaction.fromScenario === scenarioId)
    .map((interaction) => ({
      interaction,
      entity: plan.entities.find((entity) => entity.id === interaction.targetEntity),
    }))
    .filter(({ entity }) => entity?.anchor?.cell)
    .find(({ entity }) => Math.abs(entity.anchor.cell.x - state.player.x)
      + Math.abs(entity.anchor.cell.y - state.player.y) <= 1)?.interaction ?? null;
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
      message: `${interaction.action} ${interaction.targetEntity}`,
      tone: "success",
    },
    scenarioId: interaction.toScenario,
    changed: true,
    interaction,
  };
}
