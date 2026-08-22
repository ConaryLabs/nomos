---
title: Gate K light resolution and runtime commit evidence
status: Implementation reference for SW-F
date: 2026-08-22
applies_to: KERNEL.md sections 2, 3, 5, 7, and 9; acceptance 5, 9, 10, and 12
---

# Gate K light resolution and runtime commit evidence

SW-F closes the in-memory transaction boundary. SW-G subsequently assigns
stable World IR and packages the simulation plan that supplies initial-state
material; CLI commands, run outputs, replay, and migration remain later. The path is:

```text
nomos-schema construction@3 light-union plan
  -> nomos-compiler validation
  -> simulation@3 + persistence@1 + diagnostics@1
  -> nomos-sim resolve after settlement
  -> nomos.runtime_state@1 snapshot
  -> SHA-256 + nomos.causal_receipt@1
```

## Compiler-owned light semantics

Construction IR declares `EmitsLight = union`, the exact light subjects and
claim references, and the simulation, persistence, and diagnostics consumers.
Only positive claims are legal. A false claim is not an alternate way to say
dark; absence of an active positive claim is dark. The compiler validates every
activation namespace and state before projecting one byte-identical
`LightResolverPlan` to all three consumers.

`nomos_compiler::produced_schemas()` now reports construction evidence, stable
IR, simulation, navigation, persistence, diagnostics, the package schema
registry, and compiler receipts. `planned_output_schemas()` remains the
ownership inventory; the lists currently contain the same eight schema
identities but retain different meanings.

## Runtime snapshot and hash domain

`SimulationState` is the immutable `nomos.runtime_state@1` snapshot. Its
canonical envelope contains only:

- schema identity and deterministic tick;
- stable entity identities and authoritative lattice bindings;
- namespace-machine states;
- empty authoritative counter and scheduled-event collections, because Gate K
  currently defines neither.

Source spans, display text, build paths, projection caches, and cosmetic state
are unrepresentable in this envelope. `StateHash` is SHA-256 of exactly those
canonical bytes. `verify_hash` fails with `EK0810` when a recorded digest and
snapshot disagree.

## Atomic commit

`prepare_transaction` remains available as the SW-E staging boundary. SW-F adds:

```rust
nomos_sim::commit_transaction(plan, current, command)
nomos_sim::commit_transaction_with_budget(plan, current, command, budget)
```

Commit resolves movement and light before the local transition and after all
causal settlement, checks tick addition, creates a new snapshot, hashes it, and
constructs the receipt. Only then does it return `CommittedTransaction`. Any
command, handler, resolver, budget, arithmetic, or evidence-construction failure
returns only a diagnostic. The borrowed input snapshot remains byte-identical.

## Typed causal receipt

`nomos.causal_receipt@1` records the typed command, ordered local and causal
steps, complete movement and light facts before and after, active claim reasons,
typed fact identities, independently versioned projection targets, resulting
tick, and resulting state hash. Extinguishing `brazier_02` emits one light fact
change to simulation, persistence, and diagnostics. Human-readable explanation
remains downstream and is not part of the canonical semantic receipt.

## Evidence boundary

SW-F proves in-memory snapshot immutability and commit evidence; SW-G proves the
package contains deterministic material for the same initial snapshot; SW-H
exposes filesystem validation, compilation, and inspection. No slice yet writes
run directories or command logs, executes runtime commands through the CLI,
replays, performs the required World IR migration, runs the multi-target ten-run
matrix, or performs formal cold-agent gates.
