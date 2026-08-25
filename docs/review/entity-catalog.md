---
title: Kernel entity-catalog projection — design record
status: R1 slice design record; an R1-2 input, acceptance disposition with the owner
date: 2026-08-25
issue: 138
branch: r1/issue-138-entity-catalog
accepts_against: issue #138 acceptance; RUNTIME.md §3 (revision 1, option (a))
registers: docs/evaluation/R1_SCHEMA_OWNERSHIP.md (nomos.entity_catalog@1)
applies_to: RUNTIME.md §3, §5 R1-2, §6; docs/evaluation/R1_SCHEMA_OWNERSHIP.md
unblocks: issue #139 (R1-2 Rust rendering-plan compilation)
---

# Kernel entity-catalog projection

## Problem

`RUNTIME.md` §5 R1-2 requires doors, water, and lights to be classified from
typed declarations, and forbids the plan compiler from reading `.nomos` source,
World IR, or compiler receipts. Nothing in the four package projections lets it
obey both at once. A `simulation.json` entity is `{id, binding, machines}`; the
entity kind (`primitive/iron_barred_door`) and the typed capability set
(`expansion.capabilities`) exist only in World IR, which R1-2 may not open. That
gap is why `experiments/executable-gaol/src/build-plan.mjs:25` classifies doors
by `machine.endsWith(".access")` — a convention, not a declaration.

This is the same shape as R1-1: a read-only kernel command exposing what the
kernel already knows, so no downstream code has to infer it. R1-1 removed a
shadow *resolver*; this removes a shadow *taxonomy*.

## Owner crate, and why

The builder is `crates/nomos-compiler/src/entity_catalog.rs`, and the schema
identity literal `SchemaId::new("nomos.entity_catalog", 1)` is declared there.

`nomos-compiler` is not merely the recommended home — under `KERNEL.md`
section 10's permitted edges it is the **only** kernel crate that can build this
document at all. The catalog is a join of two things:

| Half | Lives in | Reachable from |
| --- | --- | --- |
| `primitive`, `expansion.capabilities` | `nomos_schema::IrEntity` via `StableWorldIr::construction()` | `nomos-compiler` only |
| `binding`, `machines`, resolver claims and spans | `nomos_projection` plan types | `nomos-compiler`, `nomos-sim`, `nomos-cli` |

`nomos-sim`, which owns R1-1's document, has no edge to `nomos-schema` and
therefore cannot name an entity's primitive kind; `nomos-projection` is denied
`nomos-schema` by an explicit `compile_fail` doctest at
`crates/nomos-projection/src/lib.rs:27`, on the stated ground that a projection
type able to name Canonical World IR would invite a runtime subsystem to reach
the IR through it. `nomos-cli` can see both but owns "command-line surface and
artifact orchestration", not document semantics. `nomos-compiler` owns World IR
decoding and projection generation, already crosses that boundary in
`inspect.rs`, and is the crate `KERNEL.md` names as *the only crossing between
IR and projections*.

## Rust entry points reused

Nothing new decodes, classifies, or resolves anything. The whole builder is a
join and a canonical rendering over values the package opener has already
verified.

| Entry point | File | What it supplies |
| --- | --- | --- |
| `nomos_cli::open_compiled_world` → `nomos_compiler::open_compiled_package` | `crates/nomos-compiler/src/opened.rs:136` | strict package verification, as used by `inspect`/`explain-entity`/`effective-facts` |
| `StableWorldIr::construction().entities()` | `crates/nomos-schema/src/ir.rs:637` | `IrEntity::primitive()`, `IrEntity::expansion().capabilities()` |
| `SimulationPlan::entities()` | `crates/nomos-projection/src/simulation.rs:501` | `ProjectedEntity::binding()`, `ProjectedEntity::machines()` |
| `SimulationPlan::machines()` | `crates/nomos-projection/src/simulation.rs:507` | `MachineDefinition::states()`, `MachineDefinition::initial()` |
| `NavigationPlan::movement_resolver().subjects()` | `crates/nomos-projection/src/movement.rs:466` | `MovementSubject::claims()` with each claim's `id()` and `source()` |
| `SimulationPlan::light_resolver().subjects()` | `crates/nomos-projection/src/light.rs:228` | `LightSubject::claims()` with each claim's `id()` and `source()` |
| `CapabilityKind::as_str` | `crates/nomos-schema/src/ir.rs:45` | the one wire spelling of every capability, including a claim's |

Two consequences worth stating, because they are what keep this from being a
second implementation of anything:

- **The capability spelling has one owner.** A claim's `capability` is not a
  string literal in the new file; it is `CapabilityKind::BlocksGround`,
  `TraversalCostGround`, or `EmitsLight` rendered through `as_str`, the same
  function World IR renders `expansion.capabilities` through. A renamed
  capability changes both together or neither.
- **The movement plan is read once.** `validate_member_integrity`
  (`crates/nomos-compiler/src/package.rs:414`) proves the simulation and
  navigation projections carry byte-identical `movement_resolver` values before
  the package can open, so reading it from the navigation plan and light from
  the simulation plan is a choice of spelling, not of source.

### Disclosed refactor: one owner for the source-span rendering

The claim `source` object is the `byte_end`/`byte_start`/`column`/`line`/`path`
shape every artifact already writes. Before this change that shape was written
out **five** separate times, byte-identically, in `nomos-core`
(`Diagnostic::to_canonical`), `nomos-schema` (`ir::span_to_canonical`, used by
`IrEntity`, `IrRelation`, and `FactOwnershipReceipt`), `nomos-projection`
(`movement::span_to_canonical` and `light::span_to_canonical`), and `nomos-cli`
(`explanation.rs`'s `source_mapping`). Adding a sixth copy for the catalog was
the obvious wrong answer, so this change adds `SourceSpan::to_canonical` in
`nomos-core` — the crate that owns the type — and deletes all five copies.

The rendering is unchanged field for field, so no artifact byte moves; the 222
workspace tests, which include byte assertions on every compiled package member
and on the committed run-bundle evidence, are the proof.

## Chosen CLI shape

```text
nomos entity-catalog <world/>
```

Read-only: it opens and strictly verifies the package exactly as `inspect` and
`explain-entity` do, writes canonical entity-sorted bytes to stdout, and touches
nothing. It writes no artifact, adds no file to a run bundle — whose strict
reopener fails closed on a seventh entry — and is outside the state-hash domain
because it is derived. The argument grammar matches `effective-facts` in
strictness: one exact arity, no optional arguments, `--help`, and a usage error
for anything else.

No `--state` argument exists, and that is deliberate. Everything in this
document is a compile-time fact: the entity kind, its capabilities, its binding,
its machine *definitions*, and the claims declared against it. What a machine's
state happens to be right now, and what the claims therefore compose to, is
exactly what `nomos effective-facts <world/> --state <state.json>` already
answers. The two documents partition cleanly — this one is the world's shape,
that one is the world's condition — and a consumer joins them on `id`.

## The document

Verbatim stdout for `experiments/executable-gaol/areas/north-gaol/world.nomos`,
compiled at `world.nomos`, abbreviated to two of its four entities; the full
document is in the pull request.

```json
{
  "command": "entity-catalog",
  "entities": [
    {
      "binding": {"cell": {"x": 3, "y": 1, "z": 0}, "kind": "cell"},
      "capabilities": ["authority", "emits_light", "interactable", "machine", "persisted"],
      "claims": [
        {
          "capability": "emits_light",
          "id": "brazier_02.emission#emits_light",
          "resolver": "light",
          "source": {"byte_end": 437, "byte_start": 365, "column": 1, "line": 18, "path": "world.nomos"}
        }
      ],
      "id": "brazier_02",
      "light_subject": true,
      "machines": [
        {"initial": "lit", "namespace": "brazier_02.emission", "states": ["extinguished", "lit"]}
      ],
      "movement_subject": false,
      "primitive": "primitive/extinguishable_light"
    },
    {
      "binding": {"cell": {"x": 5, "y": 0, "z": 0}, "direction": "north", "kind": "face"},
      "capabilities": ["authority", "blocks_ground", "boundary", "interactable", "machine", "persisted", "portal"],
      "claims": [
        {
          "capability": "blocks_ground",
          "id": "north_gate.portal#blocks_ground",
          "resolver": "movement",
          "source": {"byte_end": 162, "byte_start": 53, "column": 1, "line": 4, "path": "world.nomos"}
        },
        {
          "capability": "blocks_ground",
          "id": "north_gate.ward#blocks_ground",
          "resolver": "movement",
          "source": {"byte_end": 162, "byte_start": 53, "column": 1, "line": 4, "path": "world.nomos"}
        }
      ],
      "id": "north_gate",
      "light_subject": false,
      "machines": [
        {"initial": "locked", "namespace": "north_gate.access", "states": ["closed", "locked", "open"]},
        {"initial": "cold", "namespace": "north_gate.combustion", "states": ["burning", "cold", "spent"]},
        {"initial": "intact", "namespace": "north_gate.integrity", "states": ["damaged", "destroyed", "intact"]},
        {"initial": "sealed", "namespace": "north_gate.ward", "states": ["sealed", "unsealed"]}
      ],
      "movement_subject": true,
      "primitive": "primitive/iron_barred_door"
    }
  ],
  "schema": "nomos.entity_catalog@1",
  "status": "completed",
  "world": {
    "manifest_digest": "fabc8398300df90a6bb28b445cc36f2cc4507bbf2a7cc332f575729d4bca0977",
    "world_ir_schema": "nomos.world_ir@2"
  }
}
```

Every field name is issue #138's, unchanged. Every array is sorted: entities by
`id`, machines by namespace, claims by `(id, resolver)`, capabilities and
machine states by their wire spelling. There is no floating point anywhere in
the document — the only numbers are lattice coordinates, byte offsets, and line
and column positions, all integers.

### Four notes on faithfulness

1. **`command` and `status` are additions.** Issue #138's sketch shows
   `schema`, `world`, and `entities`. Every other `nomos` stdout document
   carries `command` and `status`, and a rejection is
   `{"diagnostics": [...], "status": "rejected"}` — so a consumer that cannot
   read `status` cannot tell a catalog from a refusal. They are added for that
   reason. Nothing #139 codes against is renamed, moved, or retyped; the
   contract shape is a subset of what is emitted.
2. **`capabilities` is sorted, and World IR's is not.** World IR emits the set
   in `CapabilityKind` declaration order (`["machine", "interactable",
   "emits_light", "authority", "persisted"]`). Issue #138 asks for sorted
   arrays and its own example is lexicographic, so the catalog sorts. The
   members are identical, which is what the acceptance criterion asserts and
   what the test compares — as sets, in both directions.
3. **A claim's `source` is its entity's declaration span.** Every claim in the
   gaol corpus is a compiler expansion of a sealed catalog primitive, so it has
   no source text of its own; the projected claim carries the entity
   declaration's span, and the catalog copies it verbatim rather than inventing
   a narrower one. Two claims on the same door therefore share a span. This is
   the plan's own data, unmodified.
4. **`machines` carries definitions, not states.** `states` is the machine's
   legal state set and `initial` its initial state, both from the simulation
   plan. The current state of a machine is runtime data and is not in this
   document; `nomos effective-facts --state` is where it lives.

Nothing in issue #138's shape could not be populated truthfully. No field was
dropped, and none was invented.

## Acceptance mapping

Test names are in `crates/nomos-cli/tests/entity_catalog.rs` unless stated.

| Issue #138 criterion | Proved by | State |
| --- | --- | --- |
| Command shape and strict verification | `the_argument_grammar_is_exact` — `--help`, root-help listing, four usage rejections, `EK0002` on an escaping path, `EK0405` on an absent world | ✅ |
| Read-only, stdout only; no input mutation | `the_catalog_mutates_no_input` — byte comparison of every package member and of the workspace's top-level entries before and after | ✅ |
| No run-bundle file | Same test — asserts the run bundle is the six-file set before, and byte-identical after | ✅ |
| Never reads `.nomos` source | `the_catalog_reads_no_source` — deletes the source the package was compiled from and asserts byte-identical output | ✅ |
| `primitive` matches the source declaration, for the fixture and all four areas | `every_entity_carries_its_declared_primitive_and_world_ir_capabilities` — the *test* parses each `entity <id> <primitive/kind>` line; five worlds, with an anti-vacuous count assertion | ✅ |
| `capabilities` equal World IR's `expansion.capabilities` | Same test — reads `world/world-ir.json` from the compiled package and compares sets per entity, with an anti-vacuous non-empty assertion | ✅ |
| `binding`, `machines`, `claims`, resolver flags come from the plans | `resolver_subjects_claims_and_machines_come_from_the_plans` — exact canonical byte strings for the door's binding, four machines, and both claims; the water region's cost claim; the brazier's light claim | ✅ |
| Byte-identical across ten runs for the same package | `the_same_world_produces_byte_identical_output` — ten invocations, three anti-vacuous guards, plus a second compilation of the same source at the same path | ✅ |
| Arrays sorted; no floating point | Same test's byte assertions plus `resolver_subjects_...`; the document contains no `CanonicalValue::Float`, which the canonical encoder does not offer | ✅ |
| Schema identity `nomos.entity_catalog@1` declared in the owner crate | `the_catalog_identity_is_versioned_and_is_not_a_package_artifact` in `crates/nomos-compiler/src/entity_catalog.rs`; it also asserts the identity never enters `produced_schemas()` and so cannot reach a package member | ✅ |
| Identity registered; `r1-schema-ownership.sh` → `schema_identities_r1 2` | [Proof commands](#proof-commands) | ✅ |
| `RUNTIME.md` §3 table row added | Three rows: `nomos-compiler`, `nomos-cli`, `nomos-core` | ✅ |
| Four proof commands pass | [Proof commands](#proof-commands) | ✅ |
| No Gate K command, artifact, hash, or diagnostic changes | 222 workspace tests pass with every pre-existing suite unchanged and no assertion edited; no diagnostic code added; `produced_schemas()` and the package registry are untouched | ✅ |
| Non-author rerun recorded | Owner's, on the pull request | ⏳ |

Must-nots:

| Must not | Evidence | State |
| --- | --- | --- |
| Classify by string convention | The builder contains no `ends_with`, no substring test, and no entity or namespace literal; `primitive` is copied from `IrEntity::primitive()` | ✅ |
| Add a second decoder for `.nomos`, World IR, or receipts | The command opens a package through the one existing strict opener and decodes nothing itself | ✅ |
| Add a third-party dependency to a kernel crate | `Cargo.lock` unchanged versus `origin/main`; `cargo xtask boundary` clean | ✅ |
| Add a new dependency edge | None; `nomos-compiler → nomos-schema` and `nomos-cli → nomos-compiler` both already exist | ✅ |
| Edit `KERNEL.md`, `THESIS.md`, or the frozen `SCHEMA_OWNERSHIP.md` | `git diff origin/main HEAD` over all three is empty | ✅ |
| Write a seventh file into a run bundle | `the_catalog_mutates_no_input` | ✅ |
| Leave a file over ~1,000 lines | Largest touched file is `crates/nomos-cli/src/command.rs` at 833 lines | ✅ |

### Disclosed finding, filed rather than folded in: issue #141

`docs/evaluation/test-gate-k-eval-tooling.sh`, a step in the `verify` lane,
fails locally on this branch — and equally on a pristine `origin/main` checkout.
`tree_sha` at `docs/evaluation/gate-k-eval-packet.sh:164` sorts paths with
`sort -z` and pins no collation, so its digest depends on the caller's locale:
under `LC_ALL=C` it reproduces the frozen `artifactsTreeSha256` receipt exactly,
and under `en_US.UTF-8` it does not. That is the same latent defect issue #134
records in the Gate K schema-ownership script, and the reason
`docs/evaluation/r1-schema-ownership.sh` already sets `export LC_ALL=C`.

It is pre-existing, unrelated to this slice — the branch touches no file under
`docs/evaluation/runs/` — and lives in frozen Gate K evaluation tooling rather
than in R1 surface, so it is filed as **issue #141** with its evidence rather
than repaired here.

## Proof commands

All five pass on this branch. Run from the worktree root, Linux x86_64,
toolchain 1.98.0.

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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.60s

$ cargo test --workspace --locked
total passed: 222  total failed: 0

$ cargo xtask boundary
boundary: clean
  kernel crates      nomos-core, nomos-schema, nomos-projection, nomos-compiler, nomos-sim, nomos-cli
  tooling crates     xtask
  rules checked      membership, permitted-edges, cycles, forbidden-dependency, tooling-isolation
  forbidden entries  64 exact names, 8 prefixes
EXIT=0

$ docs/evaluation/r1-schema-ownership.sh
R1_SCHEMA_OWNERSHIP PASS
schema_identities_gate_k 20
schema_identities_r1 2
```

The seven tests in `crates/nomos-cli/tests/entity_catalog.rs`:

```console
running 7 tests
test the_argument_grammar_is_exact ... ok
test the_catalog_reads_no_source ... ok
test resolver_subjects_claims_and_machines_come_from_the_plans ... ok
test the_catalog_names_its_schema_and_the_world_it_catalogued ... ok
test the_catalog_mutates_no_input ... ok
test the_same_world_produces_byte_identical_output ... ok
test every_entity_carries_its_declared_primitive_and_world_ir_capabilities ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
```

Three are load-bearing.

`every_entity_carries_its_declared_primitive_and_world_ir_capabilities` is the
acceptance criterion itself, over all five committed worlds. Its expected values
come from two places the command cannot reach: the `.nomos` source text, parsed
in the test, and the package's `world-ir.json` member. If the catalog ever
starts deriving a kind instead of copying one, this fails.

`the_catalog_reads_no_source` deletes the source file and demands byte-identical
output. It is the mechanical form of R1-2's "never reads `.nomos` source" for
the command R1-2 will consume.

`the_same_world_produces_byte_identical_output` covers the ten-run criterion and
was mutation-checked: recompiling the identical bytes from a *different* source
path produces a different document, which the test asserts, because the source
path appears in every claim span. Byte identity is a property of a fixed
(source bytes, source path) pair, not of source bytes alone — the same bound
R1-1 recorded.

## What this unblocks

Issue #139 (R1-2) can now classify from declarations. The three convention-based
classifications in `experiments/executable-gaol/src/build-plan.mjs` lines 17–29
have a typed replacement:

| `build-plan.mjs` | Convention | Replaced by |
| --- | --- | --- |
| line 25 | `machine.endsWith(".access")` means "door" | `primitive == "primitive/iron_barred_door"`, or the capability set `{boundary, portal, blocks_ground}` |
| lines 17–24, `classify` | subject presence in the movement table means "water" | `primitive == "primitive/shallow_water_region"`, or `traversal_cost_ground` in `capabilities` |
| lines 26–29, `lightEntities` | presence in the light table means "light" | `light_subject`, or `emits_light` in `capabilities` |

Two of those are exactly R1-2's "a test renames a machine and an entity
identifier and the classification is unchanged": neither the primitive kind nor
the capability set moves when an identifier does.

## Disposition

This is a small, boundary-clean, read-only slice that adds one command, one
schema identity, and one canonical accessor, and deletes five duplicated
renderings. It resolves nothing, classifies nothing, and reads no source. It is
an input to R1-2 rather than an R1 target in its own right, so `RUNTIME.md` §5
gains no new subsection; its acceptance is issue #138's list above.

Merge and acceptance disposition are the owner's, and nothing here is green
until a non-author reruns the five proof commands.
