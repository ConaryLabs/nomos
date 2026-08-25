// Play state over a decoded plan.
//
// Ported from `experiments/executable-gaol/src/play-state.mjs` with its tests;
// `docs/review/nomos-viewer.md` section 2 rows 15 to 28 name the lines each
// rule reproduces. Two rules deliberately differ from the study:
//
//  * an exit is a move that leaves the lattice through a door on the player's
//    own cell whose declared `anchor.direction` is the direction of travel,
//    rather than the `target.y < 0` special case the study carried;
//  * no identifier is re-cased into prose. Guidance is returned as segments,
//    each either authored words or an identifier shown verbatim, so the DOM can
//    set an identifier in its own style and nothing invents a name.
//
// Positions here are presentation state: RUNTIME.md section 5 R1-5 owns making
// actors authoritative, and until it does this module is what moves them.

import { DIRECTION_DELTAS, directionOf } from "./catalog.mjs";
import { lightOf, movementOf, scenarioOf } from "./plan.mjs";

export const movementKeys = Object.freeze({
  ArrowUp: DIRECTION_DELTAS.north,
  KeyW: DIRECTION_DELTAS.north,
  ArrowDown: DIRECTION_DELTAS.south,
  KeyS: DIRECTION_DELTAS.south,
  ArrowLeft: DIRECTION_DELTAS.west,
  KeyA: DIRECTION_DELTAS.west,
  ArrowRight: DIRECTION_DELTAS.east,
  KeyD: DIRECTION_DELTAS.east,
});

// Guidance segments. `words` is authored prose; `identifier` is a value out of
// the plan, shown as it is written.
export const words = (text) => Object.freeze({ kind: "words", text });
export const identifier = (value) => Object.freeze({ kind: "identifier", text: value });

// Whether the gaoler is hunting in this scenario. One helper: the study
// computed this twice, once for gameplay and once for the HUD, with opposite
// comparison operators.
export const isHunting = (plan, scenario) => lightOf(scenario, plan.pursuit.light) === false;

// No fallback. The study carried hardcoded defaults of exactly one area's
// player and gaoler cells, so generic code encoded one room's coordinates as
// "the" defaults - the ownership audit's fourth double authority.
const actorPosition = (plan, id) => {
  const actor = plan.actors.find((one) => one.id === id);
  if (!actor) throw new Error(`plan for ${plan.area.id} declares no actor ${id}`);
  return { ...actor.cell };
};

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
    message: `Reach ${plan.objective.gate}`,
    tone: "neutral",
  };
}

// Arrival places the player at the destination area's own `route.entry`. The
// exiting area used to name a cell inside its destination; each area now
// declares the one cell a player arrives on.
export function enterArea(plan, state) {
  if (!plan.route.entry) throw new Error(`plan for ${plan.area.id} declares no arrival cell`);
  return {
    ...createPlayState(plan),
    player: { ...plan.route.entry },
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

export const completionSummary = (state, totalAreas) =>
  `${totalAreas} areas · ${state.moves} moves · ${state.movementCost} traversal cost`;

const withinRegion = (region, point) =>
  point.x >= region.min.x &&
  point.x <= region.max.x &&
  point.y >= region.min.y &&
  point.y <= region.max.y;

export function terrainAt(plan, scenario, point) {
  const water = plan.entities.find(
    (one) => one.kind === "water" && withinRegion(one.anchor, point),
  );
  if (!water) return { kind: "stone", entity: null, cost: 1 };
  const movement = movementOf(scenario, water.id);
  return { kind: "water", entity: water.id, cost: movement.cost };
}

export const masonryAt = (plan, point) =>
  plan.architecture.masses.find(
    (mass) =>
      point.x >= mass.min.x && point.x < mass.max.x && point.y >= mass.min.y && point.y < mass.max.y,
  ) ?? null;

// The door a move leaves through, if the move leaves the lattice at all: one on
// the player's own cell, bound to the face the move is heading for. The study
// inferred this from `target.y < 0` and matched only the x coordinate, which
// worked because every corpus door faces north.
const exitDoor = (plan, state, dx, dy) => {
  const direction = directionOf(dx, dy);
  return (
    plan.entities.find(
      (one) =>
        one.kind === "door" &&
        one.anchor.direction === direction &&
        one.anchor.cell.x === state.player.x &&
        one.anchor.cell.y === state.player.y,
    ) ?? null
  );
};

export function attemptMove(plan, scenarioId, state, dx, dy) {
  const scenario = scenarioOf(plan, scenarioId);
  if (!scenario) throw new Error(`unknown scenario ${scenarioId}`);
  if (state.escaped || state.caught) return { state, moved: false, cost: 0 };

  const target = { x: state.player.x + dx, y: state.player.y + dy, z: 0 };
  const { width, height } = plan.architecture.bounds;
  const leavesLattice = target.x < 0 || target.y < 0 || target.x >= width || target.y >= height;

  if (leavesLattice) {
    const door = exitDoor(plan, state, dx, dy);
    const movement = door && movementOf(scenario, door.id);
    if (!door || movement.disposition !== "traversable") {
      const reasons = movement?.reasons.map((one) => one.split("#").at(-1)).join(" + ");
      return {
        state: {
          ...state,
          message: door
            ? `Blocked: ${reasons || door.id}`
            : "The masonry has no opening here",
          tone: "blocked",
        },
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

  const masonry = masonryAt(plan, target);
  if (masonry) {
    return {
      state: { ...state, message: `Blocked by ${masonry.id}`, tone: "blocked" },
      moved: false,
      cost: 0,
    };
  }

  const terrain = terrainAt(plan, scenario, target);
  const moved = {
    ...state,
    player: target,
    movementCost: state.movementCost + terrain.cost,
    moves: state.moves + 1,
    message:
      terrain.kind === "water" ? `Shallow water costs ${terrain.cost}` : "Stone costs 1",
    tone: terrain.kind === "water" ? "water" : "neutral",
  };
  return { state: advanceGaoler(plan, scenario, moved), moved: true, cost: terrain.cost };
}

export function advanceGaoler(plan, scenario, state) {
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
  return (
    plan.interactions
      .filter((one) => one.from_scenario === scenarioId)
      .map((one) => ({
        interaction: one,
        entity: plan.entities.find((entry) => entry.id === one.target_entity),
      }))
      .filter(({ entity }) => entity.anchor.cell)
      .find(
        ({ entity }) =>
          Math.abs(entity.anchor.cell.x - state.player.x) +
            Math.abs(entity.anchor.cell.y - state.player.y) <=
          1,
      )?.interaction ?? null
  );
}

export function guidanceFor(plan, scenarioId, state) {
  const gate = plan.objective.gate;
  if (state.completed) {
    return {
      objective: [words("Escape complete")],
      prompt: [words("R · Begin a new run")],
      tone: "success",
    };
  }
  if (state.caught) {
    return {
      objective: [words("Exit via "), identifier(gate)],
      prompt: [words("R · Restart the run")],
      tone: "danger",
    };
  }
  if (state.escaped) {
    return {
      objective: [words("Exited via "), identifier(gate)],
      prompt: [words("Entering the next area")],
      tone: "success",
    };
  }

  const interaction = interactionAt(plan, scenarioId, state);
  if (interaction) {
    return {
      objective: [words("Exit via "), identifier(gate)],
      prompt: [
        words("E · "),
        identifier(interaction.action),
        words(" "),
        identifier(interaction.target_entity),
      ],
      tone: "action",
    };
  }

  const scenario = scenarioOf(plan, scenarioId);
  const movement = movementOf(scenario, gate);
  const open = movement.disposition === "traversable";
  return {
    objective: [words("Exit via "), identifier(gate)],
    prompt: open
      ? [words("The way through "), identifier(gate), words(" is open")]
      : [words("Reach "), identifier(gate)],
    tone: open ? "success" : "neutral",
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
