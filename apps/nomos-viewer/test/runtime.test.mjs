// The wasm runtime, loaded and driven under node.
//
// The browser lane proves the artifact plays in Chrome; this proves the same
// binary, loaded by the same loader, plays the same route without a browser —
// so a failure has somewhere smaller to be found than a headless run, and the
// identity assertion the smoke lane makes is reproducible on a machine with no
// Chrome at all.
//
// It runs against `dist/`, so it is a test of the staged artifact rather than
// of the working tree. It skips with a message when `dist/` has not been built.

import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { loadArtifacts } from "../src/plan.mjs";
import { interactCommand, moveCommand, firstInteraction } from "../src/play.mjs";
import { ABI_VERSION, loadRuntime } from "../src/runtime.mjs";
import { solveRoute } from "../smoke/route.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, "..", "dist");
const repo = join(here, "..", "..", "..");
const ready = existsSync(join(dist, "nomos_play.wasm"));

// A `fetch` over the staged directory. The loader takes one so that neither it
// nor the decoder has to know whether it is in a page.
const fetchFromDist = async (url) => {
  const path = join(dist, new URL(url).pathname.replace(/^\//, ""));
  if (!existsSync(path)) return { ok: false, status: 404 };
  const bytes = readFileSync(path);
  return {
    ok: true,
    status: 200,
    text: async () => bytes.toString("utf8"),
    arrayBuffer: async () => bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.length),
  };
};

const base = "http://localhost/";

const open = async () => {
  const artifacts = await loadArtifacts(base, fetchFromDist);
  const runtime = await loadRuntime(`${base}nomos_play.wasm`, fetchFromDist);
  return { ...artifacts, runtime };
};

// The route is solved from the published artifacts, exactly as the smoke lane
// solves it, so adding an area changes content and fixtures only — never this
// test. The solver speaks the page's key names: the four arrows move, and
// `KeyE` takes whatever interaction the runtime offers first at that cell.
const DIRECTIONS = { ArrowUp: "north", ArrowDown: "south", ArrowLeft: "west", ArrowRight: "east" };

const play = async () => {
  const { collection, plans, runtimeInputs, runtime } = await open();
  const route = solveRoute(collection, plans);
  let areaId = collection.start_area;
  let bytes = runtimeInputs.get(areaId);
  let view = runtime.start(bytes.plan, bytes.semantics);

  for (const leg of route.legs) {
    for (const key of leg.keys) {
      const direction = DIRECTIONS[key];
      if (direction) {
        view = runtime.step(moveCommand(direction));
      } else {
        const offered = firstInteraction(view);
        assert.ok(offered, `the route expects an interaction in ${areaId}`);
        view = runtime.step(interactCommand(offered.entity, offered.action));
      }
    }
    const plan = plans.get(areaId);
    if (view.outcome === "escaped" && plan.route.to_area !== null) {
      areaId = plan.route.to_area;
      bytes = runtimeInputs.get(areaId);
      view = runtime.enter(bytes.plan, bytes.semantics);
    }
  }
  return { runtime, view, collection, route };
};

test("the module declares no import at all", { skip: !ready && "dist/ is not built" }, async () => {
  const { runtime } = await open();
  const module_ = new WebAssembly.Module(readFileSync(join(dist, "nomos_play.wasm")));
  assert.deepEqual(WebAssembly.Module.imports(module_), []);
  const exported = WebAssembly.Module.exports(module_).map((one) => one.name).sort();
  assert.deepEqual(exported, [
    "memory",
    "nomos_play_abi_version",
    "nomos_play_alloc",
    "nomos_play_command_log",
    "nomos_play_enter",
    "nomos_play_free",
    "nomos_play_last_error",
    "nomos_play_presentation_state",
    "nomos_play_receipts",
    "nomos_play_session",
    "nomos_play_start",
    "nomos_play_step",
  ]);
  assert.equal(runtime.instance.exports.nomos_play_abi_version(), ABI_VERSION);
});

test("the runtime opens the start area", { skip: !ready && "dist/ is not built" }, async () => {
  const { collection, runtimeInputs, runtime } = await open();
  const bytes = runtimeInputs.get(collection.start_area);
  const view = runtime.start(bytes.plan, bytes.semantics);
  assert.equal(view.schema, "nomos.presentation_state@1");
  assert.equal(view.area, collection.start_area);
  assert.equal(view.tick, 0);
  assert.equal(view.outcome, "playing");
  assert.equal(view.counters.moves, 0);
  assert.equal(view.pursuit.hunting, false);
  assert.equal(view.actors.length, 2);
  assert.deepEqual(view.actors.map((one) => one.role).sort(), ["player", "pursuer"]);
});

test(
  "a refusal reaches the caller as a ViewerError carrying the runtime's code",
  { skip: !ready && "dist/ is not built" },
  async () => {
    const { collection, runtimeInputs, runtime } = await open();
    const bytes = runtimeInputs.get(collection.start_area);
    runtime.start(bytes.plan, bytes.semantics);
    // A command document the runtime will not treat as an input at all. The
    // keys are in byte order, so this is refused for the kind it names and not
    // for the shape of the JSON around it.
    assert.throws(() => runtime.step({ kind: "levitate", schema: "nomos.play_command@1" }), {
      code: "NV0501",
      message: /PL0201/,
    });
    // The session is untouched: a shape refusal is not a batch.
    assert.equal(runtime.session().receipts.length, 0);
  },
);

test(
  "the wasm runtime plays the solved route to the escape",
  { skip: !ready && "dist/ is not built" },
  async () => {
    const { runtime, view, collection, route } = await play();
    const inputs = route.legs.reduce((sum, leg) => sum + leg.keys.length, 0);
    assert.equal(view.outcome, "escaped");
    assert.equal(view.counters.moves, route.moves);
    assert.equal(view.counters.traversal_cost, route.cost);

    const session = runtime.session();
    assert.equal(session.schema, "nomos.play_session@1");
    assert.equal(session.outcome, "completed");
    assert.equal(session.areas_cleared, collection.areas.length);
    assert.equal(session.log.length, inputs);
    assert.equal(session.receipts.length, inputs);
    assert.notEqual(session.receipt_chain_head, "0".repeat(64));

    // The two array exports are windows onto the session, not second
    // authorities. If they ever disagree, one of them is lying.
    assert.deepEqual(runtime.commandLog(), session.log);
    assert.deepEqual(runtime.receipts(), session.receipts);
  },
);

test(
  "the session the browser runtime produced replays clean natively",
  { skip: !ready && "dist/ is not built" },
  async () => {
    const { runtime, route } = await play();
    const inputs = route.legs.reduce((sum, leg) => sum + leg.keys.length, 0);
    const scratch = mkdtempSync(join(tmpdir(), "nomos-play-"));
    const path = join(scratch, "session.json");
    // The runtime's own bytes, not a re-serialization of the parsed value.
    writeFileSync(path, runtime.sessionText());

    const binary = join(repo, "target", "debug", "nomos-play");
    if (!existsSync(binary)) {
      // The native replay needs the CLI. Building it here would make a unit
      // test compile Rust; the lane builds it first, and locally
      // `cargo build -p nomos-play` is the one step.
      return;
    }
    const output = execFileSync(
      binary,
      ["replay", join(repo, "target", "executable-gaol", "areas"), "--session", path],
      { encoding: "utf8" },
    );
    assert.match(
      output,
      new RegExp(`^NOMOS_PLAY_REPLAY PASS areas=${route.areas} commands=${inputs} receipts=${inputs} `),
    );
  },
);
