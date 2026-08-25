---
title: Rust rendering-plan compiler — R1-2 design record
status: R1-2 design record; acceptance complete per RUNTIME.md §5
date: 2026-08-25
issue: 139
branch: r1/issue-139-render-plan
accepts_against: RUNTIME.md §5 R1-2 (revision 1)
registers: docs/evaluation/R1_SCHEMA_OWNERSHIP.md (nomos.rendering_plan@1)
depends_on: issue #137 (R1 workspace membership), issue #138 (nomos entity-catalog)
applies_to: RUNTIME.md §3, §4, §5, §6; docs/review/executable-gaol-ownership-audit.md; docs/review/effective-facts-spike.md
---

# Rust rendering-plan compiler

## Problem

`experiments/executable-gaol/src/build-plan.mjs` was the last piece of the
executable study that decided semantics. It classified an entity as a door by
asking whether any of its machine namespaces ended in `.access`
(`build-plan.mjs:25`), fell silently back to `visual/marker` when none of its
three heuristics matched (`:28`), assigned visual assemblies and material
families from a hardcoded kind table (`:33-38`, `:43`), reimplemented activation
evaluation (`:86-95`), and recomputed movement and light facts the kernel
resolvers had already resolved (`:111-128`).

R1-1 (`nomos effective-facts`, issue #126) and the entity catalog
(`nomos entity-catalog`, issue #138) now expose everything it was inferring, so
the inference can be deleted rather than translated.

`crates/nomos-render-plan` is that replacement: a declared R1 member with
`nomos-core` as its only build dependency, emitting `nomos.rendering_plan@1` as
canonical bytes from documents alone.

## Acceptance mapping

Every bullet of `RUNTIME.md` §5 R1-2, and every box of issue #139, against the
artifact that proves it.

| Criterion | Proof |
| --- | --- |
| Consumes the R1-1 output and presentation source only; never reads `.nomos` source, World IR, or compiler receipts | `tests/inputs.rs::world_ir_compiler_receipts_and_source_are_never_opened` compiles against a world directory whose `world-ir.json`, `compiler-receipts.json`, `manifest.json`, `schemas.json`, and `world.nomos` are unreadable bytes; `tests/inputs.rs::no_code_path_names_a_forbidden_input` greps the crate's source with comments stripped |
| Input-boundary: a temp directory of only the allowed inputs compiles | `tests/inputs.rs::a_directory_of_only_the_declared_inputs_compiles` |
| No dependency on `nomos-schema` or `nomos-compiler` | `tests/inputs.rs::the_build_dependency_graph_is_nomos_core_only` parses the manifest's declarations; `cargo xtask boundary` reports `r1 members 1`, clean |
| Binds `nomos.effective_facts@1` identity and version, refusing a mismatch with a stable diagnostic | `tests/schema_binding.rs::the_effective_facts_identity_and_version_are_bound`; the diagnostic is `RP0104` and names expected and found |
| Same for `nomos.entity_catalog@1` | `tests/schema_binding.rs::the_catalog_identity_and_version_are_bound` |
| Doors, water, and light classified from typed declarations; renaming a machine and an entity identifier changes nothing | `tests/classification.rs::renaming_every_entity_and_machine_leaves_the_kinds_unchanged` renames *every* entity id and machine namespace consistently across catalog, facts, and run bundles |
| Issue #132's three divergences, with the kernel output as the expected result | `tests/kernel_divergences.rs`, which builds a `SimulationPlan` directly and runs the real `nomos_sim::effective_facts` — no test double |
| Canonical bytes under the emitted identity; compiling twice is byte-identical | `tests/inputs.rs::compiling_twice_is_byte_identical`; `tests/canonical_profile.rs` proves the byte profile against `nomos-core`'s own encoder and strict reader |
| Byte-equal to the committed fixtures under one documented normalization, or every difference recorded | `experiments/executable-gaol/compare-rendering-plan.sh` → `4 areas compared, 0 differences`; the normalization is proved as executable properties in `tests/normalization.rs` |
| No accepted build path executes `build-plan.mjs` | The file is deleted; `experiments/executable-gaol/gaol` runs `target/debug/nomos-render-plan` |
| `gaol verify` and `gaol site` green; the three `*.test.mjs` suites pass | `gaol verify` → 26 tests, 0 failures, four `EXECUTABLE_GAOL_VERIFY PASS` receipts; `gaol site` stages |
| SVG and contact-sheet digests unchanged | [Frame digests](#frame-digests-across-the-switch) |
| Identity registered | `docs/evaluation/R1_SCHEMA_OWNERSHIP.md` row 3; `docs/evaluation/r1-schema-ownership.sh` → `schema_identities_r1 3` |
| `RUNTIME.md` §3 member list, surface table, `docs/workspace.md`, README, HANDOFF | This change |

## What the compiler reads

Documents, and only documents. There is no field on `plan::Inputs` for source,
World IR, or receipts, and no code path constructs such a path.

| Input | Identity | What it supplies |
| --- | --- | --- |
| `--catalog <entity-catalog.json>` | `nomos.entity_catalog@1` | the only source of entity kind: `primitive`, the typed `capabilities` set, `binding`, `machines`, and the resolver `claims` |
| `--facts <dir>` | `nomos.effective_facts@1`, one `<scenario>.json` per scenario | the only source of movement disposition, cost, reasons, effective light, `tick`, and `state_hash` |
| `--runs <dir>` | run bundles | `final-state.json` machine states, `result.json` status, `command-log.json` rows |
| `--world <world/>` | four projection members | their `schema` values, republished, and SHA-256 over their raw bytes |
| `--area <area.json>` | unversioned presentation source | everything R1-3 will replace |

The world directory is opened for exactly `simulation.json`,
`navigation.json`, `persistence.json`, and `diagnostics.json`.

## Convention-based classifications removed

`docs/review/executable-gaol-ownership-audit.md` §3 numbers twenty-six
convention-derived facts. This slice removes eight of them outright. Each is
listed with its prior file and line, as `RUNTIME.md` §5 R1-2's evidence bullet
requires.

| Audit §3 | Prior site | What it did | What replaced it |
| --- | --- | --- | --- |
| 1 | `experiments/executable-gaol/src/build-plan.mjs:25` | door classification via `machine.endsWith(".access")` | the catalog's declared `primitive`, cross-checked against `capabilities` (`crates/nomos-render-plan/src/catalog.rs`, `classify`) |
| 2 | `build-plan.mjs:26` | light classification via membership in the persistence light-resolver subject set | the same `primitive` table |
| 3 | `build-plan.mjs:27` | water classification via presence of a `traversal_cost_ground` navigation claim | the same `primitive` table |
| 4 | `build-plan.mjs:28` | silent `"unknown"` / `visual/marker` fallback when no heuristic matched | `EntityKind::Unknown` remains a kind, but a primitive the compiler has no kind for that nevertheless carries another kind's full capability signature is now refused with `RP0201` rather than drawn as a marker |
| 10 | `build-plan.mjs:86-95` | `activationIsActive`, a second `state_equals`/`not`/`any`/`all` evaluator | nothing: `nomos.effective_facts@1` carries the resolved facts |
| 11 | `build-plan.mjs:106` | the literal scenario directory name `"01-baseline"` special-cased as a permitted rejection | the condition that carries the meaning: a declared rejection commits zero commands (`crates/nomos-render-plan/src/runs.rs`, `read_run`). The corpus behaves identically — `01-baseline` is its only rejected scenario and it commits nothing |
| 12 | `build-plan.mjs:111-121` | movement disposition, cost, and reasons recomputed in JavaScript from raw navigation claims | the kernel document's `ground_movement` facts, copied |
| 13 | `build-plan.mjs:123-128` | effective light recomputed from raw light-resolver claims | the kernel document's `light_emission` facts, copied |

### Relocated, not removed

Audit §3 items 5 and 6 — the kind→`visualAssembly` table
(`build-plan.mjs:33-38`) and the kind→`materialFamily` table
(`build-plan.mjs:43`, which silently defaulted unknown kinds to `"stone"`) —
move into `EntityKind::visual_assembly` and `EntityKind::material_family` in
`crates/nomos-render-plan/src/catalog.rs`. They are now a closed, typed,
total enum rather than two object literals with a `?? "stone"` fallback, and the
enum's doc comment records that **this is the last place a visual assembly name
or material family is assigned to an entity kind outside the renderer catalog**.
R1-3 and R1-4 own moving them out; no later slice may add a third such table.

### Carried through unchanged, for R1-3

These are reproduced in Rust with the same semantics, because issue #139 scopes
`area.json`'s shape to R1-3 explicitly. They stay on the audit's open list.

| Audit §3 | Prior site | Now |
| --- | --- | --- |
| 7 | `build-plan.mjs:56-71` | the `{kind, target}` key-set check, the literal `exit_via`, and the literal actor ids `player` and `gaoler`, in `crates/nomos-render-plan/src/area.rs` |
| 8 | `build-plan.mjs:73-78` | the bounded 9×6 lattice and `0 < wallHeight ≤ 5`, same file |
| 9 | `build-plan.mjs:81` | the `0 < height ≤ 4` mass bound, same file |
| 14 | `build-plan.mjs:133` | `scenario.label` derived from the directory name, in `crates/nomos-render-plan/src/plan.rs` |
| 15 | `build-plan.mjs:144-162` | the O(n²) interaction-edge derivation, in `crates/nomos-render-plan/src/runs.rs` |

Audit §2 items 1 and 9 — the camera constants and the `palette` string — are
likewise reproduced verbatim, in `plan.rs`'s `look` module, with the same note.

What did change about all of these: they are now enforced by a compiler with
stable `RP####` diagnostic codes instead of by thrown `Error` strings, so R1-3
can move each one and see exactly which check it is retiring.

## Two contract findings

Recorded rather than worked around, per `AGENTS.md` ("Code may discover that the
contract is ambiguous, contradictory, impossible, or based on a falsified
assumption; it may not silently reinterpret it").

### `nomos_core::CanonicalValue` cannot express this document

`RUNTIME.md` §5 R1-2 requires the plan to be "canonical bytes under the schema
identity declared by the emitting code", and issue #139 requires the field names
and structure to match today's document so the viewer keeps working with only
its schema-string check updated. Those two requirements cannot both be met by
`CanonicalValue`:

- `nomos_core::FieldName` accepts `[a-z][a-z0-9_]*`
  (`crates/nomos-core/src/canonical.rs:54`). The plan's field names are
  camelCase (`visualAssembly`, `projectionDigests`, `inputStateHash`), and two
  of its objects are keyed by dotted identifiers — `projectionDigests` by
  projection file name and `scenarios[].machineStates` by machine namespace,
  read back at `render-core.mjs:67` and `webgl-renderer.mjs:90`.
- `CanonicalValue` has no floating-point variant, by explicit design
  (`canonical.rs:100-104`). The plan carries `architecture.wallHeight`, masonry
  mass heights, and `effects[].presentationAnchor` verbatim from `area.json`;
  the audit's §4 lists all twenty-six of those values, and R1-3 is the slice
  that removes them.

Three resolutions were available. Renaming the document's fields to snake_case
was rejected: it contradicts issue #139's scope and would move work R1-3/R1-4
own. Extending `nomos-core` was rejected: it is a kernel crate, `KERNEL.md`
§7 forbids floats in the hash domain, and the floating-point exclusion is a
stated property of the type.

What was done instead: `crates/nomos-render-plan/src/doc.rs` implements the
`KERNEL.md` §7 byte profile for this one document, widened by exactly those
three things (ASCII uppercase in a field name, `.` in a field name, and a
decimal variant) and nothing else. Two properties keep it honest:

1. `tests/canonical_profile.rs::the_two_encoders_agree_on_every_value_both_can_express`
   builds a value using every construct both types can hold — signed and
   unsigned extremes, every escape the profile names, non-ASCII, nested
   ordering — converts it to a `CanonicalValue`, and asserts the two encoders
   produce identical bytes. It then feeds those bytes to
   `nomos_core::canonical::read::parse_canonical`, the kernel's own strict
   reader, which accepts them.
2. The compiler never holds a float. A presentation number is carried as the
   verbatim source lexeme plus an exact scaled integer
   (`crates/nomos-render-plan/src/decimal.rs`), so emission is a byte copy and
   the bounds checks are integer comparisons.
   `tests/inputs.rs::no_code_path_holds_a_floating_point_type` asserts that no
   floating-point type name appears in the crate's code.

When R1-3 lands a typed presentation source with integer lattice units, this
module's reason to exist ends and the plan can be emitted from `CanonicalValue`
directly. That is the intended disposition, and `doc.rs`'s module doc says so.

### Two departures in issue #138's document shape

Both were found against the landed `nomos entity-catalog` and are handled
without asking #138 to move.

1. **The stdout envelope.** The landed document carries top-level `command` and
   `status` beside the shape issue #138's text listed, and a rejection is
   `{"diagnostics": [...], "status": "rejected"}` with no `schema` field at all.
   Binding the identity first would have reported "no `schema` field" and lost
   the kernel's own reason, so `read::require_completed` checks `status` first
   and carries the kernel's `EK####` codes into the `RP0105` rejection. The two
   envelope fields are otherwise ignored: the issue's shape is a subset.
2. **The identity spelling.** The catalog spells its `schema` as the string
   `"nomos.entity_catalog@1"`, while every Gate K artifact and `nomos
   effective-facts` spell theirs as the object `{"name": …, "version": N}`.
   `read::bind_schema` accepts both and compares them as `name@version`, so
   neither convention has to move for the other. This is a real inconsistency in
   the tree's document conventions and is worth one owner decision at R1-3 or
   R1-4; it is not worth blocking R1-2.

Nothing else in #138's shape was insufficient. `primitive` plus `capabilities`
is exactly what classification needs, and the `claims` array's `resolver` field
is what separates movement provenance from light provenance. One observation,
not a defect: a claim's `source` span is its entity's declaration span, so two
claims on one door share a span — the plan's `provenance` array carries that
verbatim, as it did before.

## Equivalence

The normalization, stated once in
`experiments/executable-gaol/compare-rendering-plan.sh`'s header and proved as
executable properties in `crates/nomos-render-plan/tests/normalization.rs`:

1. Parse both documents as JSON. Key order and insignificant whitespace are
   therefore not differences — the JavaScript wrote
   `JSON.stringify(plan, null, 2)` with insertion-ordered keys and the Rust
   writes canonical bytes with byte-sorted keys.
2. Ignore the `schema` field on both sides. The identity moves from
   `nomos.experiment.rendering_plan@1` to `nomos.rendering_plan@1`, which is the
   point of the slice.
3. Normalize nothing else. Array order is compared exactly, and `"cost": null`
   on a blocked movement subject is a value that must be present on both sides —
   never equivalent to an absent key.

Result, run at the commit where the committed fixtures were still the
JavaScript's output:

```text
OK    cistern-walk
OK    ember-vault
OK    north-gaol
OK    ossuary-reach

4 areas compared, 0 differences
```

No difference was recorded, so there is nothing to explain under the "or every
difference is recorded with its cause" branch.

After the fixtures were regenerated from the Rust output the same script
compares the pipeline against what is committed, which is the normalization-
tolerant sibling of `gaol verify`'s byte `cmp`. The historical comparison
against the JavaScript is reproducible at this branch's second commit.

### Why the fixture files are rewritten whole

The plan is now canonical bytes rather than two-space-indented JSON, so every
line of all four `rendering-plan.example.json` files changes even where the
document does not. `area-collection.example.json` changes with them: its
look-profile grammar embeds the plan's identity, so its digest moves from
`084832d5…` to `7b4f2b2c…`.

## Frame digests across the switch

Captured with `experiments/executable-gaol/gaol capture` immediately before and
immediately after the pipeline switch, over `target/executable-gaol`.

**Unchanged, byte for byte** — 27 of 31 artifacts: all twenty per-area scenario
frames, all four per-area `contact-sheet.svg`, the cross-area
`frames/contact-sheet.svg`, and `frames/contact-sheet.png`, which also equals the
committed `experiments/executable-gaol/contact-sheet.png`
(`af9c834a39e3045bca48d9209d27190978f80c7130de839ea70031de9a8b3eec`).

| Artifact | Digest, before and after |
| --- | --- |
| `frames/contact-sheet.png` | `af9c834a39e3045bca48d9209d27190978f80c7130de839ea70031de9a8b3eec` |
| `frames/contact-sheet.svg` | `d30ec3fb140c5ddad8a6aacaf3b07a348f8195a8540b900fc63b5cb350a76795` |
| `areas/cistern-walk/frames/contact-sheet.svg` | `dcca9c10f708b5693eeda86b54ee95aa992f01d45e010b64a93d8cf20e68089d` |
| `areas/ember-vault/frames/contact-sheet.svg` | `f14c2c1d721916c496b10b45b677d43dcad7ec5b4949a6e7b6f68f852fca5b73` |
| `areas/north-gaol/frames/contact-sheet.svg` | `37acf9611dc738fc62b66f532287a3a1de2f47031c5122c0855edb3e5e1ef09d` |
| `areas/ossuary-reach/frames/contact-sheet.svg` | `c54208b91906ecd24080f47b4a1a4d05bb32ead0d6a8e8ea5f922737e1219aec` |

**Changed** — the four `forensic.svg` overlays only:

| Artifact | Before | After |
| --- | --- | --- |
| `areas/cistern-walk/frames/forensic.svg` | `b558ab55994d7ad0bc9e209ac3e1de8432ba6943baa3956d8e36cdcb500cf2ca` | `778865466c2e5e0d4a5f2d40a8a49abe844d26657d837313778f1a62a8278e00` |
| `areas/ember-vault/frames/forensic.svg` | `ef6b21815c24d8185aae016615db53d849700c1776095101fa5807d02458c96f` | `3bab9cb145f503e5da93de979ebb10a1b84e7b8168a354284c25743b96295e49` |
| `areas/north-gaol/frames/forensic.svg` | `c629352af534bd317d240b883fb0c146a17d1cb147c37b65d266336c57ef1a71` | `cd953680da72de3084860133a5b4b64170b4ad80a076defb5dc3af8ea7e7d2ca` |
| `areas/ossuary-reach/frames/forensic.svg` | `7799fa2781c6c132ba64033991b4efa97d6372fe5fdc9c6957cd03d3a20d247c` | `99a4bc471c00341cb3f875033f1fde99f2c24dcd218b6f25e3e2034cf46e5529` |

Cause, and the proof of it: the forensic overlay prints the plan's own identity
(`experiments/executable-gaol/src/render-core.mjs:197`,
`renderer input: … | source/World IR unavailable`), and leaving it reading
`nomos.experiment.rendering_plan@1` would have made the overlay state something
false about its own input. Substituting that old string back into each new
`forensic.svg` reproduces its old digest exactly, so the identity string is the
only byte that moved:

```python
b = new_forensic_svg_bytes
assert b.count(b"nomos.rendering_plan@1") == 1
reverted = b.replace(b"nomos.rendering_plan@1", b"nomos.experiment.rendering_plan@1")
assert sha256(reverted).hexdigest() == digest_before   # True, all four areas
```

`forensic.svg` is not one of the four frames the contact sheet composes and is
not the contact sheet, so the acceptance criterion's subject is unaffected.

## Issue #132: the three divergences

`crates/nomos-render-plan/tests/kernel_divergences.rs` builds a `SimulationPlan`
directly — `MovementResolverPlan::new`, `MovementSubject::new`,
`MovementClaim::blocker`, `MovementClaim::traversal_cost`, all public — runs
`SimulationState::initialize`, `PersistedRuntimeState::new`, and the real
`nomos_sim::effective_facts`, writes the resulting document exactly as `nomos
effective-facts` would, and compiles a plan from it. No purpose-built `.nomos`
fixture was needed, and no kernel change was needed.

| # | Case | JavaScript would have said | Kernel says, and the plan carries |
| --- | --- | --- | --- |
| 1 | an active `blocks_ground` claim with `value: false` (`build-plan.mjs:114` filtered on capability alone) | `blocked`, reason that claim | `traversable`, cost `3` (the base cost), no reasons |
| 2 | an active cost of `1` under a `base_cost` of `3` (`build-plan.mjs:118` computed `Math.max(base_cost, …costs)`) | cost `3` | cost `1`, reason the cost claim |
| 3 | two active costs, `2` and `5` (`build-plan.mjs:119` listed every active claim as a reason) | cost `5`, both claims as reasons | cost `5`, only the `5` claim as a reason |

A second test walks the kernel document's `ground_movement` array and asserts
the plan's disposition and cost for every subject equal the kernel's, so the
claim "nothing is re-derived" is checked field by field rather than by
inspection.

## Dependency on issues #137 and #138

Both landed before this branch merged, and both are in its history.

- #137 added `R1_CRATES` to `xtask/src/boundary.rs` with length 0. This change
  makes it `["nomos-render-plan"]` and adds the matching `RUNTIME.md` §3 entry;
  `cargo xtask boundary` reports `r1 members 1`, clean.
- #137's `xtask/src/planted.rs` planted a crate literally named
  `nomos-render-plan` and asserted the shipped list did not declare it.
  Declaring the real crate falsified that premise and collided the planted crate
  with the real one. The planted members are renamed `nomos-planted-r1` and
  `nomos-planted-peer` — names the workspace will never use — and every
  `check_with` call now passes the shipped `R1_CRATES` plus the planted names,
  so a real declared member stays declared while the planted violation stays a
  violation. Fixed here rather than filed, per `AGENTS.md`.
- #138 supplies `nomos entity-catalog`; the two shape departures are recorded
  above.

## Disposition

R1-2 is complete against `RUNTIME.md` §5 as mapped above, subject to the
non-author rerun `AGENTS.md` requires. The open items it deliberately does not
close, each with its owner:

- the eleven audit rows carried through unchanged, and the two relocated
  assembly/material tables — R1-3 and R1-4;
- the widened byte profile in `doc.rs`, which R1-3's integer lattice units and
  a snake_case surface would retire;
- the `schema` spelling inconsistency between `"name@version"` and
  `{name, version}`, which wants one owner decision.
