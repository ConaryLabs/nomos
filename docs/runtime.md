---
title: Gate K runtime commit and persisted evidence
status: Implementation reference through SW-M
date: 2026-08-22
applies_to: KERNEL.md sections 2, 3, 5, 7, and 9; acceptance 5, 9, 10, and 12
---

# Gate K runtime commit and persisted evidence

SW-F closes the in-memory transaction boundary. SW-G subsequently assigns
stable World IR and packages the simulation plan that supplies initial-state
material. SW-I defines the strict persisted values needed by filesystem
execution. SW-J publishes those values as verified immutable run bundles and
exposes the runtime CLI commands. The path is:

```text
nomos-schema construction@3 light-union plan
  -> nomos-compiler validation
  -> simulation@3 + persistence@1 + diagnostics@1
  -> nomos-sim resolve after settlement
  -> nomos.runtime_state@2 snapshot
  -> SHA-256 + nomos.causal_receipt@1
  -> six-file atomic run bundle
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

`SimulationState` is the immutable `nomos.runtime_state@2` snapshot. Its
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

SW-M advances the active snapshot to `nomos.runtime_state@2` so legacy and
migrated stable-IR meaning share one explicit normalization boundary; v1 bytes
are never accepted or relabelled as v2. A standalone state file uses a separate
`nomos.persisted_runtime_state@2` envelope containing the inner typed state, its
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

Four independently versioned canonical evidence types underpin the run
publisher:

- `nomos.command_log@1` records zero-based contiguous committed rows. Each row
  binds the unresolved request, resolved typed command, input/result state
  hashes, and SHA-256 of one strictly decoded causal receipt.
- `nomos.causal_receipt_sequence@1` is the schema-headed canonical value for
  `causal-receipts.json`. It strictly reconstructs each nested receipt, preserves
  commit order, and anchors receipt ticks to the supplied initial state's tick
  when checked against a command log.
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
agreement. `RunResult` accepts all five typed artifacts rather than caller-made
digest rows and derives every binding from their exact canonical bytes; later
validation recomputes all five digests. Human diagnostic wording remains
outside `RunResult`; its rejection identity is the stable diagnostic code. A
rejected bundle intentionally binds only the committed prefix and the terminal
code, as issue #58 requires; it does not persist the rejected request. Status is
therefore a canonical content claim, not an authenticity claim. Publication
also checks it against the in-memory terminal outcome before writing.

## Filesystem execution and run bundles

`nomos_sim::execute_requests` resolves and commits requests in order without
knowing about packages or the filesystem. It stops at the first rejection and
returns one `RunExecution` whose initial/final state, committed log, causal
receipts, state hashes, result binding, and optional terminal diagnostic agree.
The CLI supplies only the digest and typed simulation plan from an already
verified `OpenedCompiledWorld`.

`nomos run` reads the exact schema-headed command script and begins from the
package-derived initial state. `nomos command` strictly decodes a persisted
state against the package simulation semantics and executes one exact request
line. Runtime rejection is published evidence: only successful commits enter
the log, the final state is the last committed snapshot, and `result.json`
records the stable rejection code. CLI input, environment, or publication
failure returns no partial output.

The publisher writes exactly `initial-state.json`, `final-state.json`,
`command-log.json`, `causal-receipts.json`, `state-hashes.json`, and
`result.json` in a fresh sibling staging directory. It strictly reopens all six
typed artifacts against the input package, recomputes all bindings, then uses
one rename. Existing destinations are preserved. Before execution, the CLI
resolves existing path ancestors and rejects an output at or below the immutable
input package, including paths that enter it through a symlinked ancestor. When
`nomos command` consumes a state from a verified run bundle, that input bundle
receives the same protection. Roots and entries must be the expected directory
and regular files; symlinks, special files, missing/extra
entries, noncanonical bytes, digest changes, cross-package evidence, and
cross-semantics state reuse fail closed. Opening re-executes every committed log
row from the persisted initial state and requires byte-identical receipts,
hashes, and final state, including exact tick continuity. As with compiled
packages, callers own a quiescent supported local-filesystem tree; this is
integrity, not hostile filesystem race safety or authenticity. The generic
opener accepts both package-initial `run` bundles and nonzero-tick `command`
bundles; the `run` command itself derives and tests its initial state against the
opened package.

## Strict replay input and reproduction

SW-K assigns `nomos.replay_log@1` to a canonical replay input that binds the
exact package digest, simulation-semantics digest, package-derived initial state
hash, expected committed `CommandLog`, and expected final state hash. The nested
log carries each unresolved request, resolved typed command, input/result state
hash, and causal-receipt digest. `ReplayLog::from_execution` derives every field
from one completed `RunExecution`; rejected or empty histories cannot become
replay fixtures.

`nomos replay` strictly decodes `fixtures/gaol.replay`, opens the compiled world,
derives its initial persisted state, and checks all three input identities before
executing. It re-resolves and commits the five requests under the opened package
semantics, then requires the regenerated complete command log and final state
hash to match the fixture. `EK0822` identifies malformed replay bytes, `EK0823`
identifies the wrong input package/semantics/state, and `EK0824` identifies a
deterministic reproduction mismatch. None of those paths enters publication.
A successful replay uses the unchanged SW-J six-file staging, verification,
rename, and strict-open boundary, and is byte-identical to the corresponding
ordinary `run` output.

## Evidence boundary

SW-F proves in-memory snapshot immutability and commit evidence; SW-G proves the
package contains deterministic material for the same initial snapshot; SW-H
exposes filesystem validation, compilation, and inspection; SW-I proves the
strict typed values and their cross-object integrity rules; SW-J publishes and
reopens those exact values through `run` and `command`; SW-K binds and reproduces
them through `replay`. SW-M proves that strict legacy-v1 migration produces the
same normalized runtime-v2 state, command, receipt, movement, light, and
hash-chain evidence as active v2 meaning. Decision 0009 requires a future
transition explanation to open the world and strictly re-execute the run before
rendering causal evidence; it does not implement that command or change the
six-file bundle. No slice yet explains causal evidence, runs the multi-target
ten-run matrix, or performs formal cold-agent gates.
