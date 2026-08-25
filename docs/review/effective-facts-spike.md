---
title: Kernel effective-facts projection at a runtime state — R1-1 design record
status: R1-1 design record; acceptance complete per RUNTIME.md §5
date: 2026-08-25
issue: 126
branch: spike/issue-126-effective-facts
accepts_against: RUNTIME.md §5 R1-1 (revision 1)
registers: docs/evaluation/R1_SCHEMA_OWNERSHIP.md (nomos.effective_facts@1)
applies_to: RUNTIME.md §5, §6; KERNEL.md sections 3, 8, 10; docs/movement.md; docs/runtime.md
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

## Acceptance mapping

Every bullet of `RUNTIME.md` §5 R1-1 against the artifact that proves it. Test
names are in `crates/nomos-cli/tests/effective_facts.rs`.

| R1-1 criterion | Proved by | State |
| --- | --- | --- |
| Command is `nomos effective-facts <world/> --state <state.json>`, read-only, writes no artifact | `the_argument_grammar_is_exact` — exact grammar, `--help`, four usage rejections, missing-state environment failure | ✅ |
| Mutates no input package or state file | `the_projection_mutates_no_input` — byte comparison of every package member and the state file | ✅ |
| Adds no file to a run bundle | `the_projection_mutates_no_input` — asserts the run bundle is the six-file set before and byte-identical after | ✅ |
| All resolution from `resolve_movement` / `resolve_light` | [Rust entry points reused](#rust-entry-points-reused) is the source-review receipt; `the_projection_agrees_with_explain_entity_at_the_initial_state` forces two surfaces to one answer | ✅ |
| `activation_is_true` stays private | `crates/nomos-sim/src/resolver.rs:155` is unexported: `grep activation_is_true` over `nomos-sim/src/lib.rs` and `nomos-cli/src` returns zero hits | ✅ |
| Schema identity `nomos.effective_facts@1` declared in `nomos-sim` | `the_projection_names_its_schema_world_and_state`; declared at `crates/nomos-sim/src/effective_facts.rs` | ✅ |
| Canonical entity-sorted bytes | `every_resolver_subject_is_composed_at_the_supplied_state` asserts the exact canonical byte string of the `effective_facts` subtree at two states | ✅ |
| Stays outside the state-hash domain | The command writes nothing, and the reported `state_hash` equals the input state's — asserted in `the_projection_names_its_schema_world_and_state` | ✅ |
| Source-review receipt names the reused pair | [Rust entry points reused](#rust-entry-points-reused) in this record | ✅ |
| Identity registered in `docs/evaluation/R1_SCHEMA_OWNERSHIP.md` as `nomos.effective_facts@1` / `nomos-sim` / `crates/nomos-sim/src/effective_facts.rs` | Registered; `docs/evaluation/r1-schema-ownership.sh` reports `schema_identities_r1 1` | ✅ |
| Not added to the frozen `docs/evaluation/SCHEMA_OWNERSHIP.md` | `git diff origin/main HEAD -- docs/evaluation/SCHEMA_OWNERSHIP.md` is empty | ✅ |
| `compare-effective-facts.sh` reports `20 scenarios compared, 0 differences` | [Comparison](#comparison-against-the-four-areas-committed-plans), with `"cost": null` the only normalization | ✅ |
| Byte identity across ten runs for the fixed triple | `the_same_world_and_state_produce_byte_identical_output`, with three anti-vacuous guards | ✅ |
| A world compiled from a different path fails closed with `EK0813`, exit 1 | Same test's second half — compiles the identical source at `nested/gaol.nomos` and asserts exit 1 plus `EK0813` | ✅ |
| Differing simulation semantics fail closed with `EK0813` | `a_state_from_another_world_is_rejected_rather_than_resolved` — exit 1, `status: "rejected"`, code `EK0813` | ✅ |
| The four §6 kernel commands pass | [Proof commands](#proof-commands); `verify` lane green on PR #130 | ✅ |
| No Gate K command, artifact, hash, or diagnostic changes | 214 workspace tests pass with every pre-existing suite unchanged; `determinism` matrix green on all three targets; no new diagnostic code added | ✅ |

Must-nots:

| Must not | Evidence | State |
| --- | --- | --- |
| Add a second implementation of activation evaluation or movement/light composition | This slice adds none and **deletes** `explanation.rs`'s byte-identical disposition renderer. One pre-existing duplicate is disclosed below | ⚠️ see note |
| Add a third-party dependency to a kernel crate | `Cargo.lock` unchanged versus `origin/main`, still seven local entries; `cargo xtask boundary` clean | ✅ |
| Edit `KERNEL.md` | `git diff origin/main HEAD -- KERNEL.md` is empty (`THESIS.md` likewise) | ✅ |
| Write a seventh file into a run bundle | `the_projection_mutates_no_input` asserts the six-file set is unchanged | ✅ |

### Disclosed pre-existing duplicate: `projected_activation_is_true`

`crates/nomos-compiler/src/projection.rs:620` contains a second activation
evaluator over the same `ProjectedActivation` tree, called from lines 125 and
165 to compute the initial movement and light shape recorded in the stable v2
IR at compile time. **It predates this slice** — it is on `origin/main` and this
branch does not touch it — so R1-1's "must not *add*" is satisfied. But a
reviewer checking "no second implementation of activation evaluation anywhere
in the accepted tree" will find it, so it is disclosed here rather than left to
be discovered.

It is not reachable from the resolvers this slice uses, and the two cannot
currently share code: `nomos-compiler` has no edge to `nomos-sim`, and section
10 does not permit one. The boundary-legal unification is to move the evaluator
down into `nomos-projection`, which already owns `ProjectedActivation` and which
both crates may depend on. That is a separate change with its own evidence, not
something to smuggle into an R1-1 slice; recommended to be filed as its own
issue.

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

**These three divergences are now recorded as issue #132.**

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
total passed: 214  total failed: 0

$ cargo xtask boundary
boundary: clean
  kernel crates      nomos-core, nomos-schema, nomos-projection, nomos-compiler, nomos-sim, nomos-cli
  tooling crates     xtask
  rules checked      membership, permitted-edges, cycles, forbidden-dependency, tooling-isolation
  forbidden entries  64 exact names, 8 prefixes
EXIT=0
```

The seven tests in `crates/nomos-cli/tests/effective_facts.rs`:

```console
running 7 tests
test the_argument_grammar_is_exact ... ok
test the_projection_mutates_no_input ... ok
test the_projection_names_its_schema_world_and_state ... ok
test every_resolver_subject_is_composed_at_the_supplied_state ... ok
test a_state_from_another_world_is_rejected_rather_than_resolved ... ok
test the_projection_agrees_with_explain_entity_at_the_initial_state ... ok
test the_same_world_and_state_produce_byte_identical_output ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
```

Two are load-bearing.

`the_projection_agrees_with_explain_entity_at_the_initial_state` forces the new
command and `explain-entity` — two independently composed surfaces — to produce
identical dispositions and reasons for every subject. A future shadow resolver
in either path breaks it.

`the_same_world_and_state_produce_byte_identical_output` covers **`RUNTIME.md`
§5 R1-1**: it invokes the command ten times against one package and one state
and asserts all ten stdout byte strings are identical, then asserts a second
freshly compiled copy of the same source produces the same bytes, so nothing
about the package's location on disk reaches canonical output. Three guards
keep it from passing vacuously — a length floor, a trailing-newline check, and
a schema-substring check — and the assertion was mutation-checked: recompiling
the copy from a *different* source path makes it fail.

That last point is worth stating precisely, because it constrains what R1-1 can
claim. "The same source" means the same bytes **at the same source path**. The
path appears in claim source spans and is therefore inside the hash domain, so
a state fails closed against a world compiled from elsewhere — the observed
failure is `EK0813 persisted state belongs to different simulation semantics`,
exit 1, not a silent byte difference. Byte identity is a property of a fixed
(source bytes, source path, state) triple, not of source bytes alone.

### CI: all lanes green

All seven PR checks pass:

```console
$ gh pr checks 130
canonical schema ownership (R1)     pass   8s
determinism (aarch64-release)       pass   28s
determinism (cross-target)          pass   10s
determinism (x86_64-debug)          pass   23s
determinism (x86_64-release)        pass   33s
measured budgets (x86_64 release)   pass   37s
verify                              pass   3m36s
```

`verify` runs the four proof commands. Worth noting for the R1 contract: the
whole **determinism matrix passes**, including the cross-target comparison, so
the byte-identity property R1-1 asks for holds across x86_64 debug, x86_64
release, and aarch64 release with this change in the tree — independent
corroboration of the new test.

`r1 schema ownership` is the lane that replaced `gate-k-evidence`'s
`canonical schema ownership` job as the pull-request gate. Locally:

```console
$ docs/evaluation/r1-schema-ownership.sh
R1_SCHEMA_OWNERSHIP PASS
schema_identities_gate_k 20
schema_identities_r1 1
evidence_head 0089985...
```

#### Why the old lane could not pass, and what replaced it

Before #135, `docs/evaluation/gate-k-schema-ownership.sh` failed this branch
for two independent reasons, and both are recorded here because they shaped
the design:

1. **The frozen-source guard.** Its last check is
   `git diff --name-only eb86f25f5084a5da83cdd4f26e42e68089367a11 -- crates`,
   which must be empty. `origin/main` has a zero-file diff from that commit
   under `crates/`, so *any* branch that edits kernel source fails this check
   by construction — this spike, or any other. It is a freeze proof, not a
   correctness test.
2. **The schema-constructor count.** The script requires exactly 16 literal
   `SchemaId::new("nomos.` constructors across `crates/*/src`, and exactly
   twenty rows in `docs/evaluation/SCHEMA_OWNERSHIP.md`. Adding
   `nomos.effective_facts@1` makes it 17 and would make it twenty-one. It fails
   with `literal schema constructor set changed outside the reviewed
   inventory`.

Reason 2 is the acceptance-15 receipt requirement predicted in
[Recorded caveat](#recorded-caveat-this-is-a-new-convention), arriving as a
live check rather than a paper obligation. That is a good outcome: the
inventory is enforced, not merely written down.

**Neither was worked around**, and both are now resolved — by an owner
decision and a new lane, not by editing the receipt that failed.

`docs/evaluation/SCHEMA_OWNERSHIP.md` **stays frozen**: it is Gate K evidence
at commit `eb86f25`, `nomos.effective_facts@1` is not a Gate K identity, and
the twenty-row inventory is not reopened. The identity registers instead in
`docs/evaluation/R1_SCHEMA_OWNERSHIP.md`, the additive R1 continuation created
under issue #133 and merged as #135, as `nomos.effective_facts@1` / `nomos-sim`
/ `crates/nomos-sim/src/effective_facts.rs`. An identity is owned if and only
if it appears in exactly one of the two documents, which the new lane enforces
by enumerating every declaration under `crates/*/src`.

Reason 1, the freeze-commit guard, is inherent to any accepted kernel change.
`docs/evaluation/r1-schema-ownership.sh` drops it deliberately, because under
`RUNTIME.md` §3 option (a) kernel crates may gain read-only R1 surface, so
every R1 slice makes that diff non-empty; "no Gate K command, artifact, hash,
or diagnostic changes" is instead proved by the `verify` and determinism
lanes.

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

This began as a spike and is now the accepted R1-1 slice. It is a small,
boundary-clean change that reuses the existing resolvers exactly and deletes a
28-line shadow resolver. Of the three items that needed an owner decision, the
first is settled and the other two remain open:

1. **Schema identity — settled and registered.** `RUNTIME.md` §5 R1-1
   requires `nomos.effective_facts@1`, declared in `nomos-sim`. The Gate K
   receipt stays frozen; the identity is registered in
   `docs/evaluation/R1_SCHEMA_OWNERSHIP.md` and the lane reports
   `schema_identities_r1 1`. See [CI](#ci-all-lanes-green).
2. **`machine_states` in the document.** Include it and the plan builder stops
   reading `final-state.json`; exclude it and the document stays purely
   composed facts.
3. **The three latent JavaScript bugs.** Filed as issue #132. They are
   unreachable today; if the gaol experiment consumes this command, they
   evaporate rather than needing a JavaScript fix.

This branch is the R1-1 slice under `RUNTIME.md` §5, with every acceptance
bullet proved and the identity registered. Merge disposition is the owner's.
