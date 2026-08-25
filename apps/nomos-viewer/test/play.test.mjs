// The adapter, which is all that is left of what used to be the game.
//
// Nineteen of this file's twenty-two cases became Rust when the JavaScript
// reducer was deleted: movement, cost, masonry, the exit through a door,
// pursuit, capture, arrival, and the counters are `crates/nomos-play`'s, and
// `docs/review/nomos-play.md` section 8.3 is the migration table naming where
// each one went. What is left here is what stayed in JavaScript — the key
// table, the command documents, and the prose the HUD shows — plus the one
// property the two sides have to agree on.

import test from "node:test";
import assert from "node:assert/strict";

import { DIRECTION_DELTAS } from "../src/catalog.mjs";
import { decodePlan } from "../src/plan.mjs";
import {
  crossCommand,
  completionSummary,
  firstInteraction,
  guidanceFor,
  interactCommand,
  messageFor,
  moveCommand,
  movementKeys,
} from "../src/play.mjs";
import { hallPlan } from "./fixtures.mjs";
import { darkHallView, hallView, receipt } from "./views.mjs";

const hall = decodePlan(hallPlan());
const flatten = (segments) => segments.map((one) => one.text).join("");

test("movement keys name the four directions the runtime declares", () => {
  assert.deepEqual(new Set(Object.values(movementKeys)), new Set(Object.keys(DIRECTION_DELTAS)));
  assert.equal(movementKeys.ArrowUp, "north");
  assert.equal(movementKeys.KeyW, "north");
  assert.equal(movementKeys.ArrowDown, "south");
  assert.equal(movementKeys.KeyS, "south");
  assert.equal(movementKeys.ArrowLeft, "west");
  assert.equal(movementKeys.KeyA, "west");
  assert.equal(movementKeys.ArrowRight, "east");
  assert.equal(movementKeys.KeyD, "east");
  assert.equal(movementKeys.KeyQ, undefined);
});

test("a command document is canonical bytes, not an object in insertion order", () => {
  // The runtime reads a command with the kernel's strict canonical reader,
  // which requires object keys in byte order; `JSON.stringify` takes its order
  // from insertion. A command whose keys came out unsorted is refused, so this
  // is the property that keeps the adapter's front door open.
  const sorted = (command) => {
    const keys = Object.keys(command);
    return keys.every((key, at) => at === 0 || keys[at - 1] < key);
  };
  for (const command of [
    moveCommand("north"),
    interactCommand("hall_gate", "unseal"),
    crossCommand("hall_gate"),
  ]) {
    assert.ok(sorted(command), `${JSON.stringify(command)} has its keys in byte order`);
    assert.equal(command.schema, "nomos.play_command@1");
  }
  assert.equal(
    JSON.stringify(moveCommand("north")),
    '{"direction":"north","kind":"move","schema":"nomos.play_command@1"}',
  );
  assert.equal(
    JSON.stringify(interactCommand("hall_gate", "unseal")),
    '{"action":"unseal","entity":"hall_gate","kind":"interact","schema":"nomos.play_command@1"}',
  );
  assert.equal(
    JSON.stringify(crossCommand("hall_gate")),
    '{"gate":"hall_gate","kind":"cross","schema":"nomos.play_command@1"}',
  );
});

test("the offered interaction is the runtime's first row, and nothing else", () => {
  // `presentation_state.interactions` is ordered by `(entity, action)`, which
  // is the rule the runtime states. The viewer takes the first; it does not
  // filter, rank, or look at the plan's `interactions[]`, which after R1-5 is
  // captured evidence rather than a rule.
  assert.equal(firstInteraction(hallView()), null);
  const offered = firstInteraction(
    hallView({
      interactions: [
        { action: "ignite", entity: "hall_gate" },
        { action: "unseal", entity: "hall_gate" },
      ],
    }),
  );
  assert.deepEqual(offered, { action: "ignite", entity: "hall_gate" });
});

test("guidance derives the objective and prompt from plan data", () => {
  const sealed = guidanceFor(hall, hallView(), false);
  assert.equal(flatten(sealed.objective), "Exit via hall_gate");
  assert.equal(flatten(sealed.prompt), "Reach hall_gate");
  assert.equal(sealed.tone, "neutral");

  const open = guidanceFor(
    hall,
    hallView({
      movement: [{ cost: 1, disposition: "traversable", entity: "hall_gate", reasons: [] }],
    }),
    false,
  );
  assert.equal(flatten(open.prompt), "The way through hall_gate is open");
  assert.equal(open.tone, "success");

  const acting = guidanceFor(
    hall,
    hallView({ interactions: [{ action: "unseal", entity: "hall_gate" }] }),
    false,
  );
  assert.equal(flatten(acting.prompt), "E · unseal hall_gate");
  assert.equal(acting.tone, "action");
});

test("guidance follows the outcome the runtime reported", () => {
  assert.equal(flatten(guidanceFor(hall, hallView(), true).objective), "Escape complete");
  const caught = guidanceFor(hall, hallView({ outcome: "caught" }), false);
  assert.equal(flatten(caught.prompt), "R · Restart the run");
  assert.equal(caught.tone, "danger");
  const escaped = guidanceFor(hall, hallView({ outcome: "escaped" }), false);
  assert.equal(flatten(escaped.prompt), "Entering the next area");
});

test("no identifier is re-cased into prose", () => {
  // The study title-cased every identifier it printed. Guidance returns
  // segments instead, each either authored words or an identifier shown as it
  // is written, so the DOM can style an identifier and nothing invents a name.
  const guidance = guidanceFor(
    hall,
    hallView({ interactions: [{ action: "unseal", entity: "hall_gate" }] }),
    false,
  );
  const identifiers = [...guidance.objective, ...guidance.prompt]
    .filter((one) => one.kind === "identifier")
    .map((one) => one.text);
  assert.deepEqual(identifiers, ["hall_gate", "unseal", "hall_gate"]);
  for (const segment of [...guidance.objective, ...guidance.prompt]) {
    assert.ok(["words", "identifier"].includes(segment.kind));
    if (segment.kind === "identifier") assert.match(segment.text, /^[a-z][a-z0-9_]*$/);
  }
});

test("the message names what the runtime did, and never recomputes it", () => {
  const before = { moves: 0, traversal_cost: 0 };
  assert.deepEqual(messageFor(hallView(), receipt(), before), {
    text: "Stone costs 1",
    tone: "neutral",
  });
  // The step's cost is the difference between two cumulative totals the
  // runtime reported. The viewer knows nothing about terrain.
  assert.deepEqual(
    messageFor(hallView(), receipt({ counters_after: { moves: 1, traversal_cost: 3 } }), before),
    { text: "Shallow water costs 3", tone: "water" },
  );
  assert.deepEqual(
    messageFor(hallView(), receipt({ accepted: false, refusal: "PL0302" }), before),
    { text: "Blocked by masonry", tone: "blocked" },
  );
  assert.deepEqual(
    messageFor(
      hallView(),
      receipt({
        input: { action: "unseal", entity: "hall_gate", kind: "interact", schema: "nomos.play_command@1" },
      }),
      before,
    ),
    { text: "unseal hall_gate", tone: "success" },
  );
  assert.equal(
    messageFor(hallView({ outcome: "caught" }), receipt(), before).text,
    "The gaoler caught you — press R to reset",
  );
});

test("a refusal the viewer has no words for is reported by its code", () => {
  // Better a `PL####` on screen than a message that claims to know why.
  const seen = messageFor(hallView(), receipt({ accepted: false, refusal: "PL9999" }), {
    moves: 0,
    traversal_cost: 0,
  });
  assert.equal(seen.text, "PL9999");
  assert.equal(seen.tone, "blocked");
});

test("the pursuer's advance is named only when it actually advanced", () => {
  const before = { moves: 0, traversal_cost: 0 };
  const both = receipt({
    actor_deltas: [
      { from: { x: 2, y: 2, z: 0 }, id: "gaoler", to: { x: 2, y: 1, z: 0 } },
      { from: { x: 0, y: 0, z: 0 }, id: "player", to: { x: 0, y: 1, z: 0 } },
    ],
  });
  assert.equal(messageFor(darkHallView(), both, before).text, "The gaoler advances in the dark");
  // The same batch with only the player moving is not the gaoler advancing.
  assert.equal(messageFor(darkHallView(), receipt(), before).text, "Stone costs 1");
});

test("completion reports cumulative run state", () => {
  const done = hallView({ counters: { moves: 44, traversal_cost: 60 } });
  assert.equal(completionSummary(done, 4), "4 areas · 44 moves · 60 traversal cost");
});
