// Synthetic `nomos.presentation_state@1` documents, for the tests that used to
// build a JavaScript play state.
//
// The real shape is `crates/nomos-play/src/presentation.rs`, and
// `test/runtime.test.mjs` drives the real runtime; these are for the pure
// functions — the readout and the guidance — which need a state to describe and
// have no business instantiating a wasm module to get one.

const cell = (x, y) => ({ x, y, z: 0 });

/// One presentation state over the `test-hall` fixture, with overrides.
export function hallView(overrides = {}) {
  return {
    schema: "nomos.presentation_state@1",
    area: "test-hall",
    tick: 0,
    kernel_state_hash: "a".repeat(64),
    machine_states: [
      { namespace: "hall_gate.access", state: "locked" },
      { namespace: "hall_gate.integrity", state: "intact" },
      { namespace: "hall_gate.ward", state: "sealed" },
      { namespace: "hall_lamp.emission", state: "lit" },
    ],
    movement: [
      { cost: null, disposition: "blocked", entity: "hall_gate", reasons: ["hall_gate.ward#blocks_ground"] },
      { cost: 3, disposition: "traversable", entity: "hall_pool", reasons: [] },
    ],
    effective_light: [{ emitting: true, entity: "hall_lamp" }],
    actors: [
      { cell: cell(2, 2), id: "gaoler", role: "pursuer" },
      { cell: cell(0, 0), id: "player", role: "player" },
    ],
    interactions: [],
    outcome: "playing",
    counters: { moves: 0, traversal_cost: 0 },
    pursuit: { hunting: false, light: "hall_lamp", moves_since_step: 0 },
    ...overrides,
  };
}

/// The same state with the pursuit light out, which is the only thing that
/// makes the pursuer hunt.
export const darkHallView = (overrides = {}) =>
  hallView({
    effective_light: [{ emitting: false, entity: "hall_lamp" }],
    pursuit: { hunting: true, light: "hall_lamp", moves_since_step: 0 },
    ...overrides,
  });

/// One `nomos.play_receipt@1`, for the message builder.
export function receipt(overrides = {}) {
  return {
    schema: "nomos.play_receipt@1",
    ordinal: 0,
    area: "test-hall",
    input: { direction: "north", kind: "move", schema: "nomos.play_command@1" },
    accepted: true,
    refusal: null,
    tick_before: 0,
    tick_after: 1,
    kernel_state_hash_before: "a".repeat(64),
    kernel_state_hash_after: "a".repeat(64),
    actor_deltas: [{ from: cell(0, 0), id: "player", to: cell(0, 1) }],
    outcome_before: "playing",
    outcome_after: "playing",
    counters_after: { moves: 1, traversal_cost: 1 },
    previous_receipt_hash: "0".repeat(64),
    play_state_hash_after: "b".repeat(64),
    ...overrides,
  };
}
