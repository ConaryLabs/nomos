---
title: Gate K transitions and transaction preparation
status: Implementation reference through SW-F
date: 2026-08-21
applies_to: KERNEL.md sections 1, 2, 4; acceptance 4-6
---

# Gate K transitions and transaction preparation

SW-D makes machine behavior compiler-owned and executable without allowing the
runtime to read authoring source or Canonical World IR. The boundary is:

```text
nomos-schema WorldIr construction@3 (Nomos epoch)
  -> nomos-compiler validation and projection
  -> nomos-projection SimulationPlan@3
  -> nomos-sim immutable transaction preparation and commit
```

`nomos-projection` depends only on `nomos-core`; `nomos-sim` depends only on
that projection and core. Neither crate can name `nomos-schema`.

## Sealed transition catalog

| Trigger | Kind | Input | Source → target |
| --- | --- | --- | --- |
| `access.unlock` | command | exact resolved entity credential | `locked → closed` |
| `access.open` | command | none | `closed → open` |
| `access.close` | command | none | `open → closed` |
| `combustion.ignite` | command | none | `cold → burning` |
| `ward.unseal` | command | none | `sealed → unsealed` |
| `emission.extinguish` | command | none | `lit → extinguished` |
| `integrity.apply_damage` | internal event | `damage(channel = fire, amount = 2)` | `intact → destroyed` |

The internal damage handler is not an external command. A caller cannot use it
to write `integrity` directly. The door catalog also declares exactly one
interaction:

```text
phase causal:
  combustion.on_enter(burning)
    -> integrity.apply_damage(channel = fire, amount = 2)
```

`on_enter` fires when a successful transition actually enters `burning`; it is
not recurring and has no while-true or scheduler interpretation. Edges sort by
explicit phase ordinal, source namespace/state, and target namespace/handler.

## Compiler validation

`nomos_compiler::compile_simulation_plan` rejects before returning an artifact:

- absent source or target namespaces;
- absent initial, transition source/target, or `on_enter` states;
- absent target-owned handlers or payload mismatches;
- duplicate transition signatures or causal-edge identities;
- every cycle in the possible state-entry interaction graph.

These failures use stable `EK07xx` diagnostics. Construction builders also
reject duplicates before canonical encoding, so stable sorting cannot turn a
validation defect into last-write-wins behavior.

`SimulationPlan` contains machine states, initial values, external command
requirements, internal handlers, typed causal edges, phase/order data, and the
SW-E movement resolver: typed claim IDs, activations, values, source spans, and
connectivity. It contains no source AST, World IR type, resolved claim result,
movement disposition, or precomputed subsystem delta.

## Runtime preparation

The runtime entry points are:

```rust
nomos_sim::SimulationState::initialize(&plan)
nomos_sim::prepare_transaction(&plan, &current, &command)
```

Preparation borrows `current`, clones it privately, validates and stages the
local transition, queues matching state-entry events in projection order, and
lets each target machine apply its own handler. A successful result contains a
staged after-state, ordered `Local` then `Causal` transition steps, and
effective movement and light facts resolved before and after complete causal
settlement. It is not a committed snapshot. SW-F's `commit_transaction`
consumes the successful preparation privately, advances the tick with checked
arithmetic, and returns a new `nomos.runtime_state@1` snapshot, its state hash,
and a typed `nomos.causal_receipt@1`. No commit evidence is returned on rejection.

Any `EK08xx` failure discards the staged clone. Tests compare both Rust equality
and canonical state bytes around rejected commands. They cover missing/wrong
credentials, arguments supplied to argument-free commands, undeclared actions,
external attempts to call internal handlers, illegal source states, missing
event targets/handlers, and a malicious cyclic projection. The compiler rejects
cycles first; a 64-step runtime budget remains the last defense.

## Evidence boundary

SW-D proves the local `ignite → burning` step, exactly one typed fire-damage
event, and target-owned `integrity → destroyed` staging in deterministic order.
It also proves unlock/open/close/unseal/extinguish local changes and that opening
does not alter `ward`.

SW-E's ground `MovementDisposition` facts remain as described in
[`movement.md`](movement.md). SW-F proves light removal,
persistence/diagnostics deltas, committed in-memory state, hashes, and typed
causal receipts as described in [`runtime.md`](runtime.md). Replay, migration,
package/filesystem CLI orchestration, the ten-run target matrix, and
whole-Gate-K cold-agent acceptance remain open.
