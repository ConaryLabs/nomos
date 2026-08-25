// The DOM binding, and the presentation model it paints.
//
// `readout` is a pure function of the decoded artifacts and one
// `nomos.presentation_state@1` document: it returns every visible string, so
// the presentation model is tested in node and only the wiring below needs a
// browser. The wiring also writes the readout onto the root element as `data-`
// attributes, which is the contract the smoke lane reads — the same state the
// HUD paints, not a test hook bolted on beside it.
//
// Nothing here decides anything about the run. Every authoritative fact comes
// from `crates/nomos-play` through `runtime.mjs`: where the actors are, what a
// step cost, which interaction `E` sends, whether a gate opened, whether the
// gaoler caught anyone. This module turns a key into a command document, hands
// it over, and paints what comes back.
//
// Authoritative state advances synchronously on the key event. The tween is
// presentation-only, between two authoritative endpoints, and gates nothing:
// the study held input and the area transition inside a
// `requestAnimationFrame` completion callback, which `RUNTIME.md` section 5
// R1-5 forbids for state and which would make a headless lane depend on frames
// being scheduled.

import {
  DEFAULT_LOOK_PROFILE,
  LOOK_PROFILE_IDS,
  PALETTE,
  hex,
} from "./catalog.mjs";
import {
  ViewerError,
  initialScenario,
  loadArtifacts,
  scenarioOf,
  semanticsFile,
} from "./plan.mjs";
import {
  completionSummary,
  firstInteraction,
  guidanceFor,
  interactCommand,
  messageFor,
  moveCommand,
  movementKeys,
} from "./play.mjs";
import { createGaolRenderer } from "./render.mjs";
import { loadRuntime } from "./runtime.mjs";

/// The staged name of the authoritative runtime.
export const RUNTIME_FILE = "nomos_play.wasm";

const TWEEN_MS_PER_COST = 105;

/// Every visible string, from the artifacts and one presentation state alone.
///
/// `view` is a `nomos.presentation_state@1` document. `session` is the two
/// facts that belong to the run rather than to the area: how many areas have
/// been cleared and whether the route is finished.
export function readout(collection, plan, view, session, message) {
  const total = collection.areas.length;
  const guidance = guidanceFor(plan, view, session.completed);
  const pursuit =
    view.outcome === "caught" ? "caught" : view.pursuit.hunting ? "hunting" : "dormant";
  return {
    area: view.area,
    tick: view.tick,
    kernelStateHash: view.kernel_state_hash,
    outcome: session.completed ? "completed" : view.outcome,
    progress: `Area ${Math.min(session.areasCleared + 1, total)} / ${total} · ${plan.area.label}`,
    objective: guidance.objective,
    prompt: guidance.prompt,
    tone: guidance.tone,
    message: message.text,
    messageTone: message.tone,
    meter:
      `areas ${session.areasCleared}/${total} · moves ${view.counters.moves} · ` +
      `cost ${view.counters.traversal_cost} · gaoler ${pursuit}`,
    pursuit,
    moves: view.counters.moves,
    cost: view.counters.traversal_cost,
    areasCleared: session.areasCleared,
    completed: session.completed,
    summary: completionSummary(view, total),
    arrival: `Area ${Math.min(session.areasCleared + 1, total)} of ${total}`,
    title: plan.area.label,
  };
}

/// The scenario a number key selects: the nth declared scenario, by identity.
/// The study indexed the DOM's button list, which was correct only because the
/// buttons happened to be built in plan order.
export const scenarioByIndex = (plan, index) =>
  index >= 1 && index <= plan.scenarios.length ? plan.scenarios[index - 1] : null;

const paintSegments = (document, element, segments) => {
  element.replaceChildren();
  for (const segment of segments) {
    if (segment.kind === "identifier") {
      const code = document.createElement("code");
      code.textContent = segment.text;
      element.append(code);
    } else element.append(document.createTextNode(segment.text));
  }
};

/// Writes the palette into CSS custom properties. One table, two consumers.
export function applyPalette(document) {
  const style = document.documentElement.style;
  for (const [role, value] of Object.entries(PALETTE)) {
    style.setProperty(`--nomos-${role.replaceAll("_", "-")}`, hex(value));
  }
}

const showFailure = (document, error) => {
  const root = document.documentElement;
  root.dataset.error = error instanceof ViewerError ? error.code : "NV0000";
  const panel = document.querySelector("#failure");
  panel.hidden = false;
  panel.textContent = error.message;
};

/// Boots the viewer. `three` is the vendored namespace, imported by the page.
export async function start(document, three, host = globalThis) {
  applyPalette(document);
  const root = document.documentElement;

  let collection;
  let plans;
  let runtimeInputs;
  let runtime;
  try {
    const fetchImpl = host.fetch.bind(host);
    ({ collection, plans, runtimeInputs } = await loadArtifacts(document.baseURI, fetchImpl));
    runtime = await loadRuntime(new URL(RUNTIME_FILE, document.baseURI).href, fetchImpl);
  } catch (error) {
    showFailure(document, error);
    throw error;
  }

  const element = (id) => document.querySelector(id);
  const areasBar = element("#areas");
  const statesBar = element("#states");
  const frame = element("#frame");
  const forensic = element("#forensic");
  const lookMode = element("#look-mode");
  const grammar = element("#grammar");
  const message = element("#message");
  const meter = element("#meter");
  const progress = element("#progress");
  const objective = element("#objective");
  const prompt = element("#prompt");
  const arrival = element("#arrival");
  const arrivalKicker = element("#arrival-kicker");
  const arrivalTitle = element("#arrival-title");
  const completion = element("#completion");
  const completionSummaryElement = element("#completion-summary");
  const restart = element("#restart");
  const sessionElement = element("#session");

  const renderer = createGaolRenderer(frame, three, host);

  // Everything below is either a copy of what the runtime last said, or a
  // presentation-only value the runtime has no opinion about.
  let areaId;
  let plan;
  let view;
  let run = { areasCleared: 0, completed: false };
  let note = { text: "", tone: "neutral" };
  let sessionDocument;
  let sessionText = "{}";
  // A captured scenario the number keys put on screen instead of the live
  // state. Forensic only: it changes nothing authoritative and the next input
  // clears it.
  let inspecting = null;
  let visualActors = new Map();
  let tween = null;
  let arrivalTimer;
  let lookId = DEFAULT_LOOK_PROFILE;

  const inputsFor = (id) => runtimeInputs.get(id);
  const actorCells = (from) =>
    new Map(from.actors.map((actor) => [actor.id, { ...actor.cell }]));
  const positions = () => Object.fromEntries(visualActors);

  const adopt = (next) => {
    view = next;
    areaId = view.area;
    plan = plans.get(areaId);
  };

  const refreshSession = () => {
    // Kept as text as well as parsed: the text is the runtime's own canonical
    // bytes, and that is what the smoke lane writes out and replays natively.
    sessionText = runtime.sessionText();
    sessionDocument = JSON.parse(sessionText);
    run = {
      areasCleared: sessionDocument.areas_cleared,
      completed: sessionDocument.outcome === "completed",
    };
  };

  const draw = () => {
    renderer.present(plan, inspecting ?? view, forensic.ariaPressed === "true", {
      actorPositions: positions(),
    });
    const seen = readout(collection, plan, view, run, note);
    message.textContent = seen.message;
    message.dataset.tone = seen.messageTone;
    meter.textContent = seen.meter;
    progress.textContent = seen.progress;
    paintSegments(document, objective, seen.objective);
    paintSegments(document, prompt, seen.prompt);
    prompt.dataset.tone = seen.tone;
    completion.hidden = !seen.completed;
    if (seen.completed) completionSummaryElement.textContent = seen.summary;
    root.dataset.area = seen.area;
    root.dataset.tick = String(seen.tick);
    root.dataset.kernelStateHash = seen.kernelStateHash;
    root.dataset.outcome = seen.outcome;
    root.dataset.moves = String(seen.moves);
    root.dataset.cost = String(seen.cost);
    root.dataset.areasCleared = String(seen.areasCleared);
    root.dataset.pursuit = seen.pursuit;
    root.dataset.message = seen.message;
    root.dataset.ready = "true";
    // The whole `nomos.play_session@1` document, as page state. The smoke lane
    // reads it, writes it out, and replays it through `nomos-play replay`; that
    // is what proves the browser ran the same authority as the native runtime.
    if (sessionElement) sessionElement.textContent = sessionText;
  };

  const showArrival = () => {
    const seen = readout(collection, plan, view, run, note);
    host.clearTimeout?.(arrivalTimer);
    arrivalKicker.textContent = seen.arrival;
    arrivalTitle.textContent = seen.title;
    arrival.classList.remove("active");
    host.requestAnimationFrame?.(() => arrival.classList.add("active"));
    arrivalTimer = host.setTimeout?.(() => arrival.classList.remove("active"), 1400);
  };

  const buildScenarioButtons = () => {
    statesBar.replaceChildren();
    for (const scenario of plan.scenarios) {
      const button = document.createElement("button");
      button.textContent = scenario.label;
      button.dataset.scenario = scenario.id;
      button.ariaPressed = String(scenario.id === inspecting?.id);
      button.onclick = () => inspect(scenario.id);
    statesBar.append(button);
    }
  };

  // A captured scenario, drawn instead of the live state. The plan's scenarios
  // are the SVG capture ladder and the evidence that the compiler consumed
  // committed run bundles; after R1-5 they are not gameplay, and looking at one
  // moves nothing.
  const inspect = (scenarioId) => {
    inspecting = scenarioOf(plan, scenarioId);
    for (const child of statesBar.children) {
      child.ariaPressed = String(child.dataset.scenario === scenarioId);
    }
    note = { text: `Inspecting ${inspecting.label}`, tone: "neutral" };
    draw();
  };

  const clearInspection = () => {
    if (!inspecting) return;
    inspecting = null;
    for (const child of statesBar.children) child.ariaPressed = "false";
  };

  const settle = (next, cost) => {
    const from = visualActors;
    adopt(next);
    refreshSession();
    startTween(from, actorCells(view), cost);
    for (const child of areasBar.children) {
      child.ariaPressed = String(child.dataset.area === areaId);
    }
  };

  const arrive = (next) => {
    adopt(next);
    refreshSession();
    visualActors = actorCells(view);
    tween = null;
    clearInspection();
    for (const child of areasBar.children) {
      child.ariaPressed = String(child.dataset.area === areaId);
    }
    buildScenarioButtons();
    draw();
    showArrival();
  };

  // The tween interpolates between two authoritative endpoints and is read only
  // by the renderer. If frames never come, the run still completes.
  const startTween = (from, to, cost) => {
    tween = {
      from,
      to,
      started: host.performance?.now?.() ?? 0,
      duration: TWEEN_MS_PER_COST * Math.max(cost, 1),
    };
    const step = () => {
      if (!tween) return;
      const now = host.performance?.now?.() ?? tween.started + tween.duration;
      const t = Math.min(1, (now - tween.started) / tween.duration);
      const eased = 1 - (1 - t) ** 3;
      const hop = Math.sin(Math.PI * t) * 0.08;
      const at = new Map();
      for (const [id, target] of tween.to) {
        const origin = tween.from.get(id) ?? target;
        at.set(id, {
          x: origin.x + (target.x - origin.x) * eased,
          y: origin.y + (target.y - origin.y) * eased,
          z: id === playerId() ? hop : 0,
        });
      }
      visualActors = at;
      if (t >= 1) {
        visualActors = new Map(tween.to);
        tween = null;
      }
      draw();
      if (tween) host.requestAnimationFrame?.(step);
    };
    host.requestAnimationFrame?.(step);
  };

  // The declared role, not an identity. `@3` added it precisely so no consumer
  // has to know that the player happens to be called `player`.
  const playerId = () => view.actors.find((actor) => actor.role === "player")?.id;

  const commit = (command) => {
    clearInspection();
    const before = view.counters;
    let next;
    try {
      next = runtime.step(command);
    } catch (error) {
      // A shape refusal: the runtime declined to treat this as an input at all,
      // so nothing moved and there is no receipt. Report it and stop.
      showFailure(document, error);
      throw error;
    }
    refreshSession();
    const receipt = sessionDocument.receipts.at(-1);
    note = messageFor(next, receipt, before, run.completed);
    const cost = receipt.counters_after.traversal_cost - before.traversal_cost;
    settle(next, cost);

    if (next.outcome === "escaped" && plan.route.to_area !== null) {
      const destination = plans.get(plan.route.to_area);
      const bytes = inputsFor(destination.area.id);
      arrive(runtime.enter(bytes.plan, bytes.semantics));
      return;
    }
    draw();
  };

  const begin = (id, message_) => {
    const bytes = inputsFor(id);
    adopt(runtime.start(bytes.plan, bytes.semantics));
    refreshSession();
    note = message_ ?? { text: `Reach ${plan.objective.gate}`, tone: "neutral" };
    visualActors = actorCells(view);
    tween = null;
    clearInspection();
    for (const child of areasBar.children) {
      child.ariaPressed = String(child.dataset.area === areaId);
    }
    buildScenarioButtons();
    draw();
    showArrival();
  };

  const reset = () => begin(collection.start_area, null);

  grammar.textContent = `GRAMMAR ${collection.visual_grammar.digest.slice(0, 8)}`;
  for (const area of collection.areas) {
    const button = document.createElement("button");
    button.textContent = area.label;
    button.title = "Forensic shortcut — resets run progress";
    button.dataset.area = area.id;
    button.onclick = () =>
      begin(area.id, { text: `Forensic jump to ${area.label}`, tone: "neutral" });
    areasBar.append(button);
  }

  forensic.onclick = () => {
    forensic.ariaPressed = String(forensic.ariaPressed !== "true");
    draw();
  };
  const [BASELINE, PROCEDURAL] = LOOK_PROFILE_IDS;
  lookMode.textContent = `Look: ${lookId}`;
  lookMode.onclick = () => {
    lookId = lookId === PROCEDURAL ? BASELINE : PROCEDURAL;
    lookMode.ariaPressed = String(lookId === PROCEDURAL);
    lookMode.textContent = `Look: ${lookId}`;
    renderer.setLookProfile(lookId);
    draw();
  };
  restart.onclick = reset;

  host.addEventListener("keydown", (event) => {
    if (event.code === "KeyR") {
      event.preventDefault();
      reset();
      return;
    }
    if (event.code === "BracketLeft" || event.code === "BracketRight") {
      event.preventDefault();
      const direction = event.code === "BracketLeft" ? -1 : 1;
      const index = collection.areas.findIndex((one) => one.id === areaId);
      const next = (index + direction + collection.areas.length) % collection.areas.length;
      const area = collection.areas[next];
      begin(area.id, { text: `Forensic jump to ${area.label}`, tone: "neutral" });
      return;
    }
    if (event.code === "KeyE") {
      event.preventDefault();
      const offered = firstInteraction(view);
      if (!offered) {
        note = { text: "Nothing responds here", tone: "neutral" };
        clearInspection();
        draw();
        return;
      }
      commit(interactCommand(offered.entity, offered.action));
      return;
    }
    const scenario = scenarioByIndex(plan, Number(event.key));
    if (scenario) {
      inspect(scenario.id);
      return;
    }
    const direction = movementKeys[event.code];
    if (!direction) return;
    event.preventDefault();
    commit(moveCommand(direction));
  });

  begin(collection.start_area, null);
  return {
    readout: () => readout(collection, plan, view, run, note),
    session: () => sessionDocument,
    sessionText: () => sessionText,
  };
}
