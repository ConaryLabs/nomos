// The adapter between a key press and the authoritative runtime.
//
// This file used to be the game. It owned the player's cell, the traversal cost
// of a step, mass collision, the exit through a door, the gaoler's pursuit,
// capture, the area transition, and both counters, over a ladder of captured
// scenarios the kernel had precomputed — 316 lines of JavaScript deciding
// everything the player experienced. `RUNTIME.md` section 1 criterion 2 forbids
// a surviving shadow resolver, and R1-5 is where the simulation stopped being
// one.
//
// What is left is a key table, three command constructors, and the prose the
// HUD shows. Every rule the deleted code carried is now in
// `crates/nomos-play`, compiled to wasm and reached through `runtime.mjs`;
// `docs/review/nomos-play.md` section 8 is the table of what went where.
//
// Nothing here decides anything. In particular it does not decide whether a
// move is legal, what it costs, or which interaction `E` sends: the runtime
// answers all three, and this module only asks.

/// Key code to the direction the runtime names. `@3`'s `actors[].role` and this
/// table are the two places a direction or a role is spelled in the viewer; the
/// lattice deltas themselves live in `catalog.mjs`, beside the renderer that
/// draws with them, and `crates/nomos-play`'s `tests/documents.rs` pins the two
/// against each other.
export const movementKeys = Object.freeze({
  ArrowUp: "north",
  KeyW: "north",
  ArrowDown: "south",
  KeyS: "south",
  ArrowLeft: "west",
  KeyA: "west",
  ArrowRight: "east",
  KeyD: "east",
});

/// The identity every command declares. `RUNTIME.md` section 3: an R1 document
/// spells `schema` as the string `name@version`.
export const PLAY_COMMAND_SCHEMA = "nomos.play_command@1";

// A command crosses the wasm boundary as `JSON.stringify` of one flat object of
// strings, and the runtime reads it with the kernel's strict canonical reader,
// which requires object keys in byte order. `JSON.stringify` takes its order
// from insertion, so the keys are sorted here.
//
// This is not a canonical encoder and must not grow into one. Every value in
// every command is an ASCII identifier already, so the only thing that could
// differ from canonical form is key order; nothing else about the byte profile
// is being reimplemented, and a command that got it wrong would be refused by
// the runtime rather than accepted as something else.
const command = (fields) =>
  Object.freeze(
    Object.fromEntries(
      Object.entries({ ...fields, schema: PLAY_COMMAND_SCHEMA }).sort(([left], [right]) =>
        left < right ? -1 : 1,
      ),
    ),
  );

/// `move {direction}`. A move that would leave the lattice is resolved by the
/// runtime as the crossing through the door on the player's own cell facing
/// that way; the viewer neither knows the bounds nor needs to.
export const moveCommand = (direction) => command({ kind: "move", direction });

/// `interact {entity, action}`. The pair comes from the runtime's own
/// enumeration of what is legal and within reach, never from the plan's
/// `interactions[]`, which is captured evidence rather than a rule.
export const interactCommand = (entity, action) =>
  command({ kind: "interact", entity, action });

/// `cross {gate}`. The browser never sends this — a lattice-leaving `move`
/// reaches the same rule — but it is the spelling a scripted log uses, and the
/// adapter offers it so nothing has to construct a command document by hand.
export const crossCommand = (gate) => command({ kind: "cross", gate });

/// The first interaction the runtime offers at this state, or `null`.
///
/// `presentation_state.interactions` is ordered by `(entity, action)`, which is
/// the rule `crates/nomos-play/src/batch.rs` states; taking the first is the
/// whole of the viewer's part in choosing.
export const firstInteraction = (view) => view.interactions[0] ?? null;

// ---------------------------------------------------------------------------
// Guidance
// ---------------------------------------------------------------------------
//
// Prose, assembled from identifiers the plan already publishes. It stays here
// rather than in `presentation_state@1` because authored prose inside an
// authoritative document would reopen the ownership audit's item 26 after R1-4
// closed it. No identifier is ever re-cased into a name: `north_gate` is shown
// as `north_gate`, in the identifier style, and the only authored prose in the
// content model is `area.label`.

/// Authored words.
export const words = (text) => Object.freeze({ kind: "words", text });

/// A value out of the plan, shown as it is written.
export const identifier = (value) => Object.freeze({ kind: "identifier", text: value });

/// The objective line, the prompt line, and the tone, for one presentation
/// state.
export function guidanceFor(plan, view, completed) {
  const gate = plan.objective.gate;
  if (completed) {
    return {
      objective: [words("Escape complete")],
      prompt: [words("R · Begin a new run")],
      tone: "success",
    };
  }
  if (view.outcome === "caught") {
    return {
      objective: [words("Exit via "), identifier(gate)],
      prompt: [words("R · Restart the run")],
      tone: "danger",
    };
  }
  if (view.outcome === "escaped") {
    return {
      objective: [words("Exited via "), identifier(gate)],
      prompt: [words("Entering the next area")],
      tone: "success",
    };
  }

  const interaction = firstInteraction(view);
  if (interaction) {
    return {
      objective: [words("Exit via "), identifier(gate)],
      prompt: [
        words("E · "),
        identifier(interaction.action),
        words(" "),
        identifier(interaction.entity),
      ],
      tone: "action",
    };
  }

  const open = view.movement.find((one) => one.entity === gate)?.disposition === "traversable";
  return {
    objective: [words("Exit via "), identifier(gate)],
    prompt: open
      ? [words("The way through "), identifier(gate), words(" is open")]
      : [words("Reach "), identifier(gate)],
    tone: open ? "success" : "neutral",
  };
}

/// The line the completion panel shows. Both counters are the runtime's and are
/// cumulative across the session by construction.
export const completionSummary = (view, totalAreas) =>
  `${totalAreas} areas · ${view.counters.moves} moves · ${view.counters.traversal_cost} traversal cost`;

/// The message the HUD shows for the batch that just committed.
///
/// Derived from the receipt, so the words follow the authority rather than
/// running beside it. A refusal names what the runtime refused; an accepted
/// step says what it cost. `before` is the cumulative counters as they stood
/// when the batch started: one step's cost is the difference between two
/// cumulative totals, and the caller has both, so the viewer never has to know
/// what terrain costs — which is exactly the authority this slice moved out.
/// `completed` is the session's, not the area's: leaving the terminal area is
/// an escape from the gaol rather than from a room, and only the session knows
/// which one it was.
export function messageFor(view, receipt, before, completed = false) {
  if (!receipt) return { text: `Reach ${view.area}`, tone: "neutral" };
  if (completed) return { text: "Escaped the gaol", tone: "success" };
  if (view.outcome === "caught") {
    return { text: "The gaoler caught you — press R to reset", tone: "blocked" };
  }
  if (!receipt.accepted) {
    return { text: REFUSALS[receipt.refusal] ?? receipt.refusal, tone: "blocked" };
  }
  if (receipt.input.kind === "interact") {
    return { text: `${receipt.input.action} ${receipt.input.entity}`, tone: "success" };
  }
  if (view.outcome === "escaped") {
    return { text: `Exited through ${plannedGate(receipt)}`, tone: "success" };
  }
  if (view.pursuit.hunting && receipt.actor_deltas.length > 1) {
    return { text: "The gaoler advances in the dark", tone: "danger" };
  }
  const cost = receipt.counters_after.traversal_cost - before.traversal_cost;
  return cost > 1
    ? { text: `Shallow water costs ${cost}`, tone: "water" }
    : { text: "Stone costs 1", tone: "neutral" };
}

// A crossing spelled as a `move` does not name the gate it went through; the
// runtime resolved that from the declared face. The message says "the gate"
// rather than inventing a name the input did not carry.
const plannedGate = (receipt) => (receipt.input.kind === "cross" ? receipt.input.gate : "the gate");

const REFUSALS = Object.freeze({
  PL0301: "The run is over — press R to reset",
  PL0302: "Blocked by masonry",
  PL0303: "Blocked",
  PL0304: "Someone is standing there",
  PL0305: "That needs something you do not have",
  PL0306: "The masonry has no opening here",
  PL0307: "Nothing responds here",
  PL0308: "Nothing responds here",
});
