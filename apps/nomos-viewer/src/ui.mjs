// The DOM binding, and the presentation model it paints.
//
// `readout` is a pure function of the decoded artifacts and the play state: it
// returns every visible string, so the presentation model is tested in node and
// only the wiring below needs a browser. The wiring also writes the readout onto
// the root element as `data-` attributes, which is the contract the smoke lane
// reads — the same state the HUD paints, not a test hook bolted on beside it.
//
// Authoritative state advances synchronously on the key event. The tween is
// presentation-only, between authoritative endpoints, and gates nothing: the
// study held input and the area transition inside a `requestAnimationFrame`
// completion callback, which RUNTIME.md section 5 R1-5 forbids for state and
// which would make a headless lane depend on frames being scheduled.

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
} from "./plan.mjs";
import {
  attemptInteraction,
  attemptMove,
  completeRun,
  completionSummary,
  createPlayState,
  enterArea,
  guidanceFor,
  isHunting,
  movementKeys,
} from "./play.mjs";
import { createGaolRenderer } from "./render.mjs";

const TWEEN_MS_PER_COST = 105;

/// Every visible string, from the artifacts and the play state alone.
export function readout(collection, plan, play, scenarioId) {
  const scenario = scenarioOf(plan, scenarioId);
  const total = collection.areas.length;
  const guidance = guidanceFor(plan, scenarioId, play);
  const pursuit = play.caught ? "caught" : isHunting(plan, scenario) ? "hunting" : "dormant";
  return {
    area: plan.area.id,
    scenario: scenarioId,
    progress: `Area ${Math.min(play.areasCleared + 1, total)} / ${total} · ${plan.area.label}`,
    objective: guidance.objective,
    prompt: guidance.prompt,
    tone: guidance.tone,
    message: play.message,
    messageTone: play.tone,
    meter: `areas ${play.areasCleared}/${total} · moves ${play.moves} · cost ${play.movementCost} · gaoler ${pursuit}`,
    pursuit,
    completed: play.completed,
    summary: completionSummary(play, total),
    arrival: `Area ${Math.min(play.areasCleared + 1, total)} of ${total}`,
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
  try {
    ({ collection, plans } = await loadArtifacts(document.baseURI, host.fetch.bind(host)));
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

  const renderer = createGaolRenderer(frame, three, host);

  let areaId = collection.start_area;
  let plan = plans.get(areaId);
  let selected = initialScenario(plan).id;
  let play = createPlayState(plan);
  let visualPlayer = { ...play.player };
  let visualGaoler = { ...play.gaoler };
  let tween = null;
  let arrivalTimer;
  let lookId = DEFAULT_LOOK_PROFILE;

  const draw = () => {
    renderer.present(plan, selected, forensic.ariaPressed === "true", {
      actorPositions: { player: visualPlayer, gaoler: visualGaoler },
    });
    const view = readout(collection, plan, play, selected);
    message.textContent = view.message;
    message.dataset.tone = view.messageTone;
    meter.textContent = view.meter;
    progress.textContent = view.progress;
    paintSegments(document, objective, view.objective);
    paintSegments(document, prompt, view.prompt);
    prompt.dataset.tone = view.tone;
    completion.hidden = !view.completed;
    if (view.completed) completionSummaryElement.textContent = view.summary;
    root.dataset.area = view.area;
    root.dataset.scenario = view.scenario;
    root.dataset.moves = String(play.moves);
    root.dataset.cost = String(play.movementCost);
    root.dataset.areasCleared = String(play.areasCleared);
    root.dataset.completed = String(play.completed);
    root.dataset.caught = String(play.caught);
    root.dataset.pursuit = view.pursuit;
    root.dataset.message = view.message;
    root.dataset.ready = "true";
  };

  const showArrival = () => {
    const view = readout(collection, plan, play, selected);
    host.clearTimeout?.(arrivalTimer);
    arrivalKicker.textContent = view.arrival;
    arrivalTitle.textContent = view.title;
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
      button.ariaPressed = String(scenario.id === selected);
      button.onclick = () => selectScenario(scenario.id);
      statesBar.append(button);
    }
  };

  const selectScenario = (scenarioId) => {
    selected = scenarioId;
    for (const child of statesBar.children) {
      child.ariaPressed = String(child.dataset.scenario === selected);
    }
    play = { ...play, message: `Loaded ${scenarioOf(plan, scenarioId).label}`, tone: "neutral" };
    draw();
  };

  const enterPlan = (nextPlan, nextPlay) => {
    areaId = nextPlan.area.id;
    plan = nextPlan;
    selected = initialScenario(plan).id;
    play = nextPlay;
    visualPlayer = { ...play.player };
    visualGaoler = { ...play.gaoler };
    tween = null;
    for (const child of areasBar.children) child.ariaPressed = String(child.dataset.area === areaId);
    buildScenarioButtons();
    draw();
    showArrival();
  };

  const selectArea = (nextAreaId, forensicJump = true) => {
    const nextPlan = plans.get(nextAreaId);
    const fresh = createPlayState(nextPlan);
    enterPlan(nextPlan, forensicJump ? { ...fresh, message: `Forensic jump to ${nextPlan.area.label}` } : fresh);
  };

  const reset = () => selectArea(collection.start_area, false);

  // The tween interpolates between two authoritative endpoints and is read only
  // by the renderer. If frames never come, the run still completes.
  const startTween = (fromPlayer, fromGaoler, cost) => {
    tween = {
      fromPlayer,
      fromGaoler,
      toPlayer: { ...play.player },
      toGaoler: { ...play.gaoler },
      started: host.performance?.now?.() ?? 0,
      duration: TWEEN_MS_PER_COST * Math.max(cost, 1),
    };
    const step = () => {
      if (!tween) return;
      const now = host.performance?.now?.() ?? tween.started + tween.duration;
      const t = Math.min(1, (now - tween.started) / tween.duration);
      const eased = 1 - (1 - t) ** 3;
      visualPlayer = {
        x: tween.fromPlayer.x + (tween.toPlayer.x - tween.fromPlayer.x) * eased,
        y: tween.fromPlayer.y + (tween.toPlayer.y - tween.fromPlayer.y) * eased,
        z: Math.sin(Math.PI * t) * 0.08,
      };
      visualGaoler = {
        x: tween.fromGaoler.x + (tween.toGaoler.x - tween.fromGaoler.x) * eased,
        y: tween.fromGaoler.y + (tween.toGaoler.y - tween.fromGaoler.y) * eased,
        z: 0,
      };
      if (t >= 1) {
        visualPlayer = { ...tween.toPlayer };
        visualGaoler = { ...tween.toGaoler };
        tween = null;
      }
      draw();
      if (tween) host.requestAnimationFrame?.(step);
    };
    host.requestAnimationFrame?.(step);
  };

  const move = (delta) => {
    const fromPlayer = { ...play.player };
    const fromGaoler = { ...play.gaoler };
    const result = attemptMove(plan, selected, play, delta.dx, delta.dy);
    play = result.state;
    if (!result.moved) {
      draw();
      return;
    }
    if (result.exitGate) {
      const edge = collection.route.find(
        (one) => one.from_area === areaId && one.gate === result.exitGate,
      );
      if (edge?.to_area) {
        const destination = plans.get(edge.to_area);
        enterPlan(destination, enterArea(destination, play));
      } else {
        play = completeRun(play);
        draw();
      }
      return;
    }
    startTween(fromPlayer, fromGaoler, result.cost);
    draw();
  };

  grammar.textContent = `GRAMMAR ${collection.visual_grammar.digest.slice(0, 8)}`;
  for (const area of collection.areas) {
    const button = document.createElement("button");
    button.textContent = area.label;
    button.title = "Forensic shortcut — resets run progress";
    button.dataset.area = area.id;
    button.ariaPressed = String(area.id === areaId);
    button.onclick = () => selectArea(area.id);
    areasBar.append(button);
  }
  buildScenarioButtons();

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
      selectArea(collection.areas[next].id);
      return;
    }
    if (event.code === "KeyE") {
      event.preventDefault();
      const result = attemptInteraction(plan, selected, play);
      play = result.state;
      if (result.changed) selected = result.scenarioId;
      for (const child of statesBar.children) {
        child.ariaPressed = String(child.dataset.scenario === selected);
      }
      draw();
      return;
    }
    const scenario = scenarioByIndex(plan, Number(event.key));
    if (scenario) {
      selectScenario(scenario.id);
      return;
    }
    const delta = movementKeys[event.code];
    if (!delta) return;
    event.preventDefault();
    move(delta);
  });

  draw();
  showArrival();
  return { readout: () => readout(collection, plan, play, selected) };
}
