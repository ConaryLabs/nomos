---
title: Kernel effective-facts projection at a runtime state — spike design
status: Spike design and prototype record; non-authoritative, not merged
date: 2026-08-25
issue: 126
branch: spike/issue-126-effective-facts
informs: docs/decisions/0017 (first R1 target)
applies_to: KERNEL.md sections 3, 8, 10; docs/movement.md; docs/runtime.md
---

# Kernel effective-facts projection at a runtime state

## Problem

`experiments/executable-gaol/src/build-plan.mjs` lines 86–128 contain a
JavaScript reimplementation of the kernel's effective-fact resolution:
activation evaluation over machine states, movement blocker/cost composition,
and light union. It exists only because no kernel command emits composed
effective facts for a runtime state. `explain-entity` composes them for **one**
entity at the **initial** state; `explain-transition` reports one entity at one
tick. Neither answers "what are the effective facts for every resolver subject
at *this* state".

This is a shadow resolver, and it has already drifted. See
[Divergence found](#divergence-found-in-the-existing-shadow-resolver).

## Rust entry points reused

Nothing new resolves anything. The whole prototype is composition and I/O
around three existing functions.

| Entry point | File | Signature |
| --- | --- | --- |
| `nomos_sim::resolve_movement` | `crates/nomos-sim/src/resolver.rs:21` | `fn(&SimulationPlan, &SimulationState) -> Result<ResolvedMovementFacts, Diagnostic>` |
| `nomos_sim::resolve_light` | `crates/nomos-sim/src/resolver.rs:82` | `fn(&SimulationPlan, &SimulationState) -> Result<ResolvedLightFacts, Diagnostic>` |
| `nomos_sim::PersistedRuntimeState::from_canonical_bytes` | `crates/nomos-sim/src/state_persistence.rs:46` | `fn(&[u8], &SimulationPlan) -> Result<Self, Diagnostic>` |
| `nomos_cli::open_compiled_world` | `crates/nomos-cli/src/package.rs` | strict package verification, as used by `run`/`command`/`explain-*` |

`activation_is_true` (`crates/nomos-sim/src/resolver.rs:155`) is the private
recursion that evaluates `Always` / `StateEquals` / `Any` / `All` / `Not`. It
stays private: it is reached only through the two public resolvers, which also
enforce the projected law flags (`blockers_any_active`, `costs_maximum_active`,
`blockers_before_cost`, `requires_connectivity`, non-zero `base_cost`,
`union_active`, and the exact light consumer set). Calling the resolvers rather
than the recursion is what keeps the plan guards in the path.

`explain-entity` already demonstrates the exact call pair at
`crates/nomos-cli/src/explanation.rs:25-27`; the spike generalises it from one
entity at the initial state to every subject at a supplied state.

## Chosen CLI shape

```text
nomos effective-facts <world/> --state <state.json>
```

Read-only, writes no artifact, mirrors `explain-entity`'s strict world
verification and `command`'s `--state` loading. Rejected alternative: writing
the same document into every `run`/`command` output directory.

**Justification (one sentence, as asked):** the run bundle is a closed six-file
set — `docs/runtime.md:156` says the publisher "writes exactly" those six files
and the strict reopener fails closed on "missing/extra entries", and
`nomos.run_result@1` binds five of them by digest — so a seventh artifact would
break the run-bundle opener, every `RUN_FILES` assertion, and the committed
`rendering-plan.example.json` byte comparisons, whereas a new read-only
subcommand adds a command to a surface KERNEL.md section 8 never closes.

Supporting detail: KERNEL.md section 8 lists eight invocations with no
"exactly"/"only these" closure language, unlike section 5's "No unmanifested
file or directory is permitted inside a verified package" or section 10's
"fails closed when ... an undeclared workspace member appears". The three
normative constraints in section 8 are all satisfied: structured JSON to
stdout, the four exit codes, and no mutation of an input package or state file.

## Output schema

Proposed identity: **`nomos.effective_facts@1`**, declared in `nomos-sim`,
whose section 10 charter is "runtime state, command transactions, replay,
**effective-fact resolution**".

```json
{
  "command": "effective-facts",
  "effective_facts": {
    "ground_movement": [
      {"disposition": {"kind": "blocked", "reasons": ["north_gate.ward#blocks_ground"]},
       "entity": "north_gate"},
      {"disposition": {"cost": 3, "kind": "traversable",
                       "reasons": ["flooded_section.region#traversal_cost_ground"]},
       "entity": "flooded_section"}
    ],
    "light_emission": [
      {"emitting": true, "entity": "brazier_02",
       "reasons": ["brazier_02.emission#emits_light"]}
    ]
  },
  "package_digest": "<hex>",
  "runtime_semantics_digest": "<hex>",
  "schema": {"name": "nomos.effective_facts", "version": 1},
  "state_hash": "<hex>",
  "status": "completed",
  "subject_count": {"ground_movement": 3, "light_emission": 1},
  "tick": 3
}
```

Both arrays are entity-sorted by `keyed_array`, so the document is canonical
and byte-stable. The per-subject shapes are **not** new: they are exactly what
`ResolvedMovement::to_canonical` and `ResolvedLight::to_canonical` already emit
inside `ResolvedMovementFacts::to_canonical_bytes` /
`ResolvedLightFacts::to_canonical_bytes`. The spike promotes those two private
methods to public `to_canonical()` accessors rather than writing a third
spelling of the same object.

### Recorded caveat: this is a new convention

No `nomos` stdout document currently carries a schema identity. `explain-entity`
and `explain-transition` emit bare canonical objects with `command`/`status` and
a `schemas` sub-object naming their *inputs*; `docs/explanations.md:79` states
SW-N "adds no persisted schema". Schema identities have so far been reserved for
persisted artifacts.

Giving this document one is deliberate and is the point of the issue: unlike the
explanation commands, this output exists to be **consumed and persisted by a
downstream tool** (today `build-plan.mjs`, tomorrow any renderer bridge), and a
consumer needs a versioned identity to bind against. That is the difference
between forensics for a human and an interface for a program.

Two consequences an owner must dispose of before this leaves spike status:

1. KERNEL.md section 10 and acceptance item 15 require an explicit
   source-review receipt enumerating each canonical schema identity, its owner
   crate, and its authoritative Rust type set. A new identity adds a row:
   `nomos.effective_facts@1` / `nomos-sim` / the document builder in
   `crates/nomos-sim/src/effective_facts.rs`.
2. If the owner prefers zero new schema identities, dropping the `schema` field
   is a two-line change and the rest of the design is unaffected.

## Divergence found in the existing shadow resolver

The JavaScript is not merely duplicated — it is **wrong by the documented law**
in three ways the gaol corpus happens not to exercise. `docs/movement.md:43`
says `Traversable` "contains the positive maximum active cost and the sorted
claims that supplied that maximum; its reason list is empty only when the base
cost applies."

| # | `build-plan.mjs` | Kernel (`resolver.rs:56-70`) | Bites when |
| --- | --- | --- | --- |
| 1 | `reasons: active.map(c => c.id)` — every active claim | only active cost claims equal to the maximum | a subject has two cost claims at different values, or an active non-blocking claim alongside a cost claim |
| 2 | `blockers = active.filter(c => c.capability === "blocks_ground")` — ignores `claim.value` | `MovementClaim::Blocker { value: true }` only | a `blocks_ground` claim with `value: false` is active; the JS blocks, the kernel does not |
| 3 | `cost: Math.max(base_cost, ...costs)` | `max(active costs).unwrap_or(base_cost)` | an active cost is *below* `base_cost`; the JS floors at base, the kernel does not |

Verified against all four compiled areas: every gaol subject is either a single
`always`-active cost claim of `3` (> `base_cost` 1) or two `value: true`
blockers, so none of the three conditions occurs. The twenty scenarios must
therefore agree exactly — which is what makes them a usable equivalence
baseline, not what makes the JavaScript correct.

The JS also emits `"cost": null` on a blocked subject; the kernel's `Blocked`
variant carries no `cost` key at all. That is a plan-builder presentation
choice, not a semantic difference, and the comparison harness normalises it.

## Estimate

| File | Change | Lines |
| --- | --- | --- |
| `crates/nomos-projection/src/movement.rs` | publish `MovementDisposition::to_canonical`, `ResolvedMovement::to_canonical`, add `ResolvedMovementFacts::to_canonical` | +14 / −4 |
| `crates/nomos-projection/src/light.rs` | publish `ResolvedLight::to_canonical`, add `ResolvedLightFacts::to_canonical` | +12 / −3 |
| `crates/nomos-sim/src/effective_facts.rs` | **new** — document builder over the two resolvers | +75 |
| `crates/nomos-sim/src/lib.rs` | `mod`/`pub use`/`effective_facts_schema()` + schema-uniqueness test row | +18 |
| `crates/nomos-cli/src/command.rs` | help text, arg grammar, dispatch, loader (779 → ~840 lines, stays under the ~1,000 rule) | +58 |
| `crates/nomos-cli/src/explanation.rs` | delete the duplicated `movement_to_canonical` | −19 |
| `crates/nomos-cli/tests/effective_facts.rs` | **new** — in-tree fixture end-to-end test | +140 |
| `experiments/executable-gaol/compare-effective-facts.sh` | **new** — twenty-scenario comparison harness | +60 |

**Total ≈ 350 lines of Rust across 7 files, plus a 60-line quarantined shell
harness.** Under the ~500-line ceiling, so the spike proceeds to prototype.
No file crosses ~1,000 lines, so no decomposition is triggered. No new
dependency edge: `nomos-sim -> nomos-projection` and
`nomos-cli -> nomos-sim` are both already permitted by section 10.

### Estimate versus actual

The prototype was built. Production code landed close to the estimate; the test
came in well over it, because the strongest available property — agreement with
`explain-entity` — needed structural comparison rather than a byte assertion.

| File | Estimated | Actual |
| --- | --- | --- |
| `crates/nomos-projection/src/movement.rs` | +14 / −4 | +22 / −5 |
| `crates/nomos-projection/src/light.rs` | +12 / −3 | +15 / −4 |
| `crates/nomos-sim/src/effective_facts.rs` | +75 | +87 |
| `crates/nomos-sim/src/lib.rs` | +18 | +7 / −2 |
| `crates/nomos-cli/src/command.rs` | +58 | +35 / −1 (779 → 812 lines) |
| `crates/nomos-cli/src/explanation.rs` | −19 | +2 / −21 |
| `crates/nomos-cli/tests/effective_facts.rs` | +140 | **+355** |
| `experiments/executable-gaol/compare-effective-facts.sh` | +60 | +84 |
| **Rust total** | **≈350** | **510 added / 33 removed = 477 net** |

Non-test Rust is **122 net lines**. No file crosses ~1,000 lines
(`command.rs` is the largest touched, at 812), so no decomposition was
triggered. `cargo xtask boundary` stays clean: no new dependency edge was
needed, because `nomos-sim -> nomos-projection` and `nomos-cli -> nomos-sim`
already exist.

## Proof commands

All four pass on this branch. Run from the worktree root, Linux x86_64.

```console
$ cargo fmt --all -- --check
EXIT=0

$ cargo clippy --workspace --all-targets --locked -- -D warnings
    Checking nomos-core v0.0.0 (.../crates/nomos-core)
    Checking xtask v0.0.0 (.../xtask)
    Checking nomos-projection v0.0.0 (.../crates/nomos-projection)
    Checking nomos-schema v0.0.0 (.../crates/nomos-schema)
    Checking nomos-sim v0.0.0 (.../crates/nomos-sim)
    Checking nomos-compiler v0.0.0 (.../crates/nomos-compiler)
    Checking nomos-cli v0.0.0 (.../crates/nomos-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s

$ cargo test --workspace --locked
total passed: 213  total failed: 0

$ cargo xtask boundary
boundary: clean
  kernel crates      nomos-core, nomos-schema, nomos-projection, nomos-compiler, nomos-sim, nomos-cli
  tooling crates     xtask
  rules checked      membership, permitted-edges, cycles, forbidden-dependency, tooling-isolation
  forbidden entries  64 exact names, 8 prefixes
EXIT=0
```

The six new tests in `crates/nomos-cli/tests/effective_facts.rs`:

```console
running 6 tests
test every_resolver_subject_is_composed_at_the_supplied_state ... ok
test the_projection_names_its_schema_world_and_state ... ok
test the_projection_agrees_with_explain_entity_at_the_initial_state ... ok
test the_projection_mutates_no_input ... ok
test the_argument_grammar_is_exact ... ok
test a_state_from_another_world_is_rejected_rather_than_resolved ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`the_projection_agrees_with_explain_entity_at_the_initial_state` is the load-
bearing one: it forces the new command and `explain-entity` — two independently
composed surfaces — to produce identical dispositions and reasons for every
subject. A future shadow resolver in either path breaks it.

### Build time

Measured on this branch, worktree root, after `cargo clean`:

| Build | Wall clock | User |
| --- | --- | --- |
| `cargo build --workspace --locked` | **2.888 s** | 10.851 s |
| `cargo build --workspace --locked --release` | **7.070 s** | 63.808 s |

## Comparison against the four areas' committed plans

`experiments/executable-gaol/compare-effective-facts.sh` compiles each area,
executes all five scenarios, runs `nomos effective-facts` on each resulting
`final-state.json`, and compares the result against the `movement`,
`effectiveLight`, `tick`, and `stateHash` blocks of the **committed**
`rendering-plan.example.json` — the artifact the JavaScript produced.

```console
$ experiments/executable-gaol/compare-effective-facts.sh
OK    cistern-walk   01-baseline
OK    cistern-walk   02-breached-warded
OK    cistern-walk   03-breached-unsealed
OK    cistern-walk   04-breached-unsealed-dark
OK    cistern-walk   05-open-dark
OK    ember-vault    01-baseline
OK    ember-vault    02-breached-warded
OK    ember-vault    03-breached-unsealed
OK    ember-vault    04-breached-unsealed-dark
OK    ember-vault    05-open-dark
OK    north-gaol     01-baseline
OK    north-gaol     02-breached-warded
OK    north-gaol     03-breached-unsealed
OK    north-gaol     04-breached-unsealed-dark
OK    north-gaol     05-open-dark
OK    ossuary-reach  01-baseline
OK    ossuary-reach  02-breached-warded
OK    ossuary-reach  03-breached-unsealed
OK    ossuary-reach  04-breached-unsealed-dark
OK    ossuary-reach  05-open-dark

20 scenarios compared, 0 differences
```

**Twenty of twenty agree exactly.** The only normalisation the harness applies
is the `"cost": null` spelling described above, and it is a presentation
difference in the plan builder, not a resolved-fact difference.

That agreement is a property of the corpus, not a vindication of the
JavaScript. See [Divergence found](#divergence-found-in-the-existing-shadow-resolver):
none of the three latent bugs is reachable from any current gaol world, so the
twenty scenarios are a valid equivalence baseline and nothing more.

## What becomes deletable in `build-plan.mjs`

Once the plan builder consumes `nomos effective-facts` output per scenario,
**28 lines of shadow resolver delete outright**:

| Lines | What | Replaced by |
| --- | --- | --- |
| **86–95** | `activationIsActive` — the whole `always`/`state_equals`/`not`/`any`/`all` evaluator | `nomos_sim::resolver::activation_is_true`, reached through the two resolvers |
| **111–122** | per-subject movement composition: blocker filter, cost max, disposition, reasons | `effective_facts.ground_movement` |
| **123–128** | per-subject light union | `effective_facts.light_emission` |

Two more lines change source rather than disappearing: **134** (`tick`) and
**135** (`stateHash`) can read the kernel document's `tick` and `state_hash`
instead of `final-state.json`, which also gets the plan builder a
`package_digest` binding it does not currently carry.

### What this cannot replace

- **Lines 144–164, interaction-edge derivation.** The edges are built by
  matching each scenario's `command-log.json` row prefix against every other
  scenario's, then checking `input_state_hash` against the shorter run's final
  state hash. That is a relation *between runs*, not an effective fact *at a
  state*. This command answers one state at a time and cannot see the other
  nineteen runs. Replacing it would need a different projection over a set of
  run bundles — a separate piece of work, and arguably not a kernel concern at
  all since the viewer's notion of an "interaction edge" is presentation.
- **Lines 108–110, `machineStates`.** A plain read of `final-state.json`'s
  machine list, used for the forensic overlay. Not resolution, so not a shadow
  resolver — but it could be folded in by adding a `machine_states` field to
  the document, which would let the plan builder stop opening `final-state.json`
  entirely. Deliberately left out of the spike to keep the document to
  *composed facts*; worth an owner opinion.
- **Lines 17–29, `classify` / `navByEntity` / `lightEntities`.** These read the
  resolver *tables* (a claim's `capability`, a subject's presence) to pick a
  visual assembly per entity. Reading the projected table is legitimate; it is
  evaluating activations against state that was not.
- **Lines 103–107**, the run-status guard, still needs `result.json`.

## Disposition

The spike answers the sizing question: this is a small, boundary-clean change
that reuses the existing resolvers exactly and deletes a 28-line shadow
resolver. Three items need an owner decision before any of it is promoted:

1. **Schema identity.** Whether `nomos.effective_facts@1` should exist at all,
   given that no `nomos` stdout document currently carries one, and if so the
   acceptance-15 source-review receipt row it requires.
2. **`machine_states` in the document.** Include it and the plan builder stops
   reading `final-state.json`; exclude it and the document stays purely
   composed facts.
3. **The three latent JavaScript bugs.** They are unreachable today. If the
   gaol experiment is going to keep its own resolver for any period, they
   should be filed; if it is going to consume this command, they evaporate.

This branch is a spike. It is not to be merged before decision 0017.
