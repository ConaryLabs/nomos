---
title: Gate K transitions and transaction preparation
status: Implementation reference for SW-D
date: 2026-08-21
applies_to: KERNEL.md sections 1, 2, 4; acceptance 4-6
---

# Gate K transitions and transaction preparation

SW-D makes machine behavior compiler-owned and executable without allowing the
runtime to read authoring source or Canonical World IR. The boundary is:

```text
estate-schema WorldIr construction@3
  -> estate-compiler validation and projection
  -> estate-projection SimulationPlan@1
  -> estate-sim immutable transaction preparation
```

`estate-projection` depends only on `estate-core`; `estate-sim` depends only on
that projection and core. Neither crate can name `estate-schema`.

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

`estate_compiler::compile_simulation_plan` rejects before returning an artifact:

- absent source or target namespaces;
- absent initial, transition source/target, or `on_enter` states;
- absent target-owned handlers or payload mismatches;
- duplicate transition signatures or causal-edge identities;
- every cycle in the possible state-entry interaction graph.

These failures use stable `EK07xx` diagnostics. Construction builders also
reject duplicates before canonical encoding, so stable sorting cannot turn a
validation defect into last-write-wins behavior.

`SimulationPlan` contains only machine states, initial values, external command
requirements, internal handlers, typed causal edges, and phase/order data. It
contains no source AST, World IR type, source span, claim result, movement
disposition, or precomputed subsystem delta.

## Runtime preparation

The runtime entry points are:

```rust
estate_sim::SimulationState::initialize(&plan)
estate_sim::prepare_transaction(&plan, &current, &command)
```

Preparation borrows `current`, clones it privately, validates and stages the
local transition, queues matching state-entry events in projection order, and
lets each target machine apply its own handler. A successful result contains a
staged after-state, ordered `Local` then `Causal` transition steps, and
effective movement facts resolved before and after complete causal settlement.
It is not a committed snapshot.

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

SW-E now proves the ground `MovementDisposition` facts described in
[`movement.md`](movement.md). It still does not prove light removal,
persistence/diagnostics deltas, committed state, hashes, replay, migration,
receipts, package/CLI orchestration, the ten-run target matrix, or whole-Gate-K
cold-agent acceptance. Acceptance 5, 6, and 9 and Gate K remain open.
