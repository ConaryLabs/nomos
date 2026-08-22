---
title: Gate K runtime commit and persisted evidence
status: Implementation reference through SW-I
date: 2026-08-22
applies_to: KERNEL.md sections 2, 3, 5, 7, and 9; acceptance 5, 9, 10, and 12
---

# Gate K runtime commit and persisted evidence

SW-F closes the in-memory transaction boundary. SW-G subsequently assigns
stable World IR and packages the simulation plan that supplies initial-state
material. SW-I defines the strict persisted values needed by later filesystem
execution without publishing run outputs or adding runtime CLI commands. The
path is:

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

SW-I adds the inverse boundary. `CausalReceipt::from_canonical_bytes` strictly
reconstructs every nested typed command, transition cause and payload,
effective movement/light fact, claim reason, projection target and delta, tick,
and state hash. It refuses unknown or missing fields, incompatible schemas,
wrong variants, noncanonical ordering, invalid IDs, and numeric overflow. It
also re-derives projection deltas from the decoded before/after facts so a
canonical JSON tree is not accepted merely because it parses.

## Persisted state binding

`nomos.runtime_state@1` and its constitutional hash domain remain byte-for-byte
unchanged. A standalone state file uses a separate
`nomos.persisted_runtime_state@1` envelope containing the inner typed state, its
state hash, and SHA-256 of the exact canonical `simulation.json` bytes. Opening
requires a caller-supplied typed `SimulationPlan`; strict state reconstruction
checks entity identity and binding, namespace ownership, legal machine states,
empty Gate K counter/event collections, the inner state hash, and the complete
simulation digest. Same-shape state from different semantics therefore fails.

## Command and run evidence types

The schema-headed `nomos.command_script@1` language preserves the exact request
text semantics accepted by issue #56. Resolution searches only external
commands on machines owned by the requested entity and produces one explicit
typed namespace command; it never reads source or guesses among namespaces.

Three independently versioned canonical evidence types prepare the later run
publisher:

- `nomos.command_log@1` records zero-based contiguous committed rows. Each row
  binds the unresolved request, resolved typed command, input/result state
  hashes, and SHA-256 of one strictly decoded causal receipt.
- `nomos.state_hash_sequence@1` records snapshot ordinal zero for the initial
  state and one following hash for each committed command. Validation checks
  every command-log input and result rather than trusting an untyped hash list.
- `nomos.run_result@1` binds the input-package digest, simulation-semantics
  digest, completed/rejected status, first/final hashes, committed count,
  optional stable rejection code, and exact hashes for `initial-state.json`,
  `final-state.json`, `command-log.json`, `causal-receipts.json`, and
  `state-hashes.json`. `result.json` cannot hash itself, so it is deliberately
  the one run artifact not listed in its own binding rows.

Constructors and decoders enforce ordinal, hash-chain, status/diagnostic,
artifact-set, count, endpoint, command/receipt, receipt-digest, and tick-chain
agreement. Human diagnostic wording remains outside `RunResult`; its rejection
identity is the stable diagnostic code.

## Evidence boundary

SW-F proves in-memory snapshot immutability and commit evidence; SW-G proves the
package contains deterministic material for the same initial snapshot; SW-H
exposes filesystem validation, compilation, and inspection; SW-I proves the
strict typed values and their cross-object integrity rules. No slice yet writes
run directories, executes runtime commands through the CLI, replays, performs
the required World IR migration, runs the multi-target ten-run matrix, or
performs formal cold-agent gates.
