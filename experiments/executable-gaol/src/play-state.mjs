export const movementKeys = {
  ArrowUp: [0, -1], KeyW: [0, -1],
  ArrowDown: [0, 1], KeyS: [0, 1],
  ArrowLeft: [-1, 0], KeyA: [-1, 0],
  ArrowRight: [1, 0], KeyD: [1, 0],
};

export function createPlayState() {
  return {
    player: { x: 2, y: 4, z: 0 },
    movementCost: 0,
    moves: 0,
    escaped: false,
    message: "Reach the north gate",
    tone: "neutral",
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

export function attemptMove(plan, scenarioId, state, dx, dy) {
  const scenario = plan.scenarios.find((candidate) => candidate.id === scenarioId);
  if (!scenario) throw new Error(`unknown scenario ${scenarioId}`);
  if (state.escaped) return { state, moved: false, cost: 0 };

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
    };
  }

  if (target.x < 0 || target.x > 8 || target.y > 5) {
    return { state: { ...state, message: "Blocked by masonry", tone: "blocked" }, moved: false, cost: 0 };
  }

  const terrain = terrainAt(plan, scenario, target);
  return {
    state: {
      ...state,
      player: target,
      movementCost: state.movementCost + terrain.cost,
      moves: state.moves + 1,
      message: terrain.kind === "water" ? `Shallow water costs ${terrain.cost}` : "Stone costs 1",
      tone: terrain.kind === "water" ? "water" : "neutral",
    },
    moved: true,
    cost: terrain.cost,
  };
}
