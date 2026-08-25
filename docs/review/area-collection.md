---
title: The area collection — issue #152 record
status: R1 implementation record
date: 2026-08-25
issue: 152
branch: r1/issue-152-area-collection
registers: docs/evaluation/R1_SCHEMA_OWNERSHIP.md (nomos.area_collection@1)
resolves: docs/review/nomos-viewer.md finding 2
applies_to: RUNTIME.md §2, §3, §7; apps/nomos-viewer; experiments/executable-gaol
---

# The area collection

## Problem

R1-4 shipped an accepted viewer that bound `nomos.experiment.area_collection@2`,
an identity declared at `experiments/executable-gaol/src/build-collection.mjs:88`.
`RUNTIME.md` §2 makes everything under `experiments/` non-authoritative, so the
route graph — which area starts the run, which gate leads where, that every area
is visited, that the chain terminates — had JavaScript as its only authority
while accepted code depended on the result. `docs/review/nomos-viewer.md`
finding 2 recorded that as tolerated for R1-4 and named this issue as the repair.

## Disposition

`crates/nomos-render-plan/src/collection.rs` declares and emits
`nomos.area_collection@1` through `nomos_core::CanonicalValue`, and the binary
gained a second mode:

```text
nomos-render-plan collection --plans <dir-or-plan> [--plans …] --out <areas.json>
```

`--plans` is repeatable. A directory value is the study's published layout — one
subdirectory per area, each holding `rendering-plan.json`, exactly what
`build-collection.mjs:20-26` scanned — and a file value is one plan. The plan
compiler's own invocation is unchanged: the first word selects the mode, and
there is no `plan` subcommand to add to existing call sites.

The document, for the four committed areas, is
`experiments/executable-gaol/area-collection.example.json`, committed verbatim
as the compiler emits it (canonical bytes plus one `LF`), sha256
`53bba38ca42119f13263b20530a490379b9fba3f0290a455eb736593ee5cf4f4`.

## Acceptance mapping

| Issue #152 acceptance | Proof |
| --- | --- |
| `nomos.area_collection@1` registered, owned by `nomos-render-plan`; `r1-schema-ownership.sh` passes | `docs/evaluation/R1_SCHEMA_OWNERSHIP.md` row 1; the script reports `schema_identities_r1 5` |
| Route-chain refusals tested (cycle, unvisited area, missing entry, two starts) | `crates/nomos-render-plan/tests/collection.rs`, [below](#refusals) — all four, plus three more |
| `build-collection.mjs` deleted | Deleted in this change; `experiments/executable-gaol/gaol` calls `target/debug/nomos-render-plan collection` |
| `apps/nomos-viewer` binds only `nomos.*` identities declared under `crates/` | `src/plan.mjs` binds `nomos.rendering_plan@2` and `nomos.area_collection@1`; `test/plan.test.mjs::collection binds its identity and its route` asserts the retired `nomos.experiment.area_collection@2` is refused with `NV0102` naming both sides |
| `gaol verify` green | `AREA_COLLECTION_VERIFY PASS areas=4` plus four `EXECUTABLE_GAOL_VERIFY PASS` receipts |
| Non-author rerun | Outstanding; this record is the author's run |

## What `build-collection.mjs` did, reproduced

Every check the deleted file performed, with the line it performed it at and
where it lives now. `RUNTIME.md` §2 requires exactly this citation: the study is
a specification, never a source of truth.

| `build-collection.mjs` | Check | Now |
| --- | --- | --- |
| `:20-26` | Scan a directory of area directories for `rendering-plan.json`, ordered by name | `collection::expand` |
| `:28` | At least two areas | `collection::build`, `one_area_is_not_a_collection` |
| `:30` | Deduplicate and order the assembly rows | `collection::visual_grammar`, by `BTreeSet` |
| `:31-38` | The visual grammar: plan schema, projection schemas, architecture style, entity assemblies, actor assemblies, effect assemblies | `collection::visual_grammar`, field for field, tabled in its doc comment |
| `:40, :45-47` | Every area's grammar is byte-identical to the first area's | `collection::build`, `an_area_that_diverges_from_the_shared_grammar_is_refused` (`RP0302`) |
| `:32` | The plan identity the grammar publishes | `plan::rendering_plan_schema()`, the one constant, also bound per plan by `read::bind_schema` (`RP0104`, `the_plan_identity_and_version_are_bound`) |
| `:44` | The directory name is the area identity | `collection::read_area`, `a_directory_that_is_not_the_area_identity_is_refused` |
| `:52-54` | An area declares an arrival cell if and only if it is not the start area | `collection::build`, `an_area_that_is_both_a_start_and_an_arrival_is_refused` |
| `:56-57` | A non-null `to_area` names a declared area | `collection::build`, `a_destination_that_is_not_a_declared_area_is_refused` |
| `:58-60` | That area declares an arrival cell | `collection::build`, `a_destination_that_declares_no_arrival_cell_is_refused` |
| `:64-66` | Exactly one start area | `collection::build`, `a_collection_with_no_start_area_is_refused`, `a_collection_with_two_start_areas_is_refused` |
| `:70-84` | One walk from the start, refusing a revisit | `collection::build`, `a_route_that_cycles_is_refused` |
| `:75-83` | One route row per hop: `from_area`, `gate`, `to_area`, `entry` | `collection::build` |
| `:79-81` | Each hop's arrival cell is read from the destination's own plan | `collection::build`, asserted in `four_areas_compile_to_one_ordered_chain` |
| `:85` | The walk visits every declared area | `collection::build`, `an_area_the_chain_never_visits_is_refused` |
| `:87-100` | The emitted document: schema, `visual_grammar` with its digest, `start_area`, `route`, `areas` | `collection::assemble` |
| `:90` | The grammar digest is SHA-256 over the grammar | `collection::build`, `nomos_core::hash::Sha256Digest::of_bytes` over the grammar's canonical bytes |
| `:95-99` | One row per area: identity, label, plan file | `collection::assemble`, plus `start`, `exit`, `entry`, and the plan's SHA-256 |
| `:102` | Write the document | `write_atomically` in the binary, through a temporary sibling |

Nothing it checked was dropped.

## Differences, and why

1. **The emitted bytes are canonical, not pretty-printed.** `:102` wrote
   `JSON.stringify(collection, null, 2)`. `nomos.area_collection@1` is
   `nomos_core::CanonicalValue` bytes plus one `LF`, like the plan, so the strict
   reader accepts the artifact and re-encodes it byte-identically
   (`the_document_is_canonical_and_names_the_plan_bytes`).

2. **`entity_assemblies` rows are objects, not triples.** `:35` built
   `[kind, visual_assembly, material_family]`. A positional triple has no field
   names and `CanonicalValue` gives no reason to keep one, so each row is
   `{kind, material_family, visual_assembly}`. The ordering is a `BTreeSet` over
   the same three components rather than `Array.prototype.sort`'s
   comma-joined-string comparison; for this corpus the order is identical, and
   the object spelling is what makes that checkable rather than incidental.

3. **A repeated area identity is refused.** `:42` built its lookup with
   `new Map(...)`, which resolves a repeated identity by keeping whichever plan
   was read last. `a_repeated_area_identity_is_refused` proves the refusal.

4. **The chain's termination is named.** `build-collection.mjs` left "the chain
   ends somewhere" implied by its every-area check at `:85`: a second area with
   no destination could only be unreachable, and was reported as "does not visit
   every declared area". The collection requires exactly one area declaring no
   destination and says so
   (`a_chain_that_does_not_terminate_at_one_exit_area_is_refused`).

5. **Each area row carries its own exit, arrival, start flag, plan file, and
   plan digest.** `:95-99` published identity, label, and a hand-built path
   string. The area row now carries what an area declares, the route stays the
   derived chain, and the two halves are checked against each other by the
   viewer's decoder rather than trusted because one emitter wrote both.

6. **The plan file is a name, not a path.** `:98` emitted `areas/<id>.json`,
   which is the viewer's staged layout. The collection publishes
   `plan.file = "<id>.json"` and `apps/nomos-viewer/src/plan.mjs` derives
   `areas/<file>` itself: the compiler has no business knowing where a consumer
   files the plan, and the app keeps its one URL-safety constraint.

7. **The plan's SHA-256 is published.** New. `apps/nomos-viewer/build.mjs`
   checks it against the bytes it stages (`plan-digest` rule, planted test
   `staging refuses a plan whose bytes are not the ones the collection names`),
   so the collection decides which bytes are an area's plan. The page does not
   hash: `crypto.subtle` is absent outside a secure context, and a plan that
   failed to publish is a build failure rather than a runtime one.

## Refusals

Each is one test in `crates/nomos-render-plan/tests/collection.rs`.

| Refusal | Test | Code |
| --- | --- | --- |
| No start area | `a_collection_with_no_start_area_is_refused` | `RP0301` |
| Two start areas | `a_collection_with_two_start_areas_is_refused` | `RP0301` |
| `to_area` names an undeclared area | `a_destination_that_is_not_a_declared_area_is_refused` | `RP0301` |
| Destination declares no arrival cell | `a_destination_that_declares_no_arrival_cell_is_refused` | `RP0301` |
| The chain cycles | `a_route_that_cycles_is_refused` | `RP0301` |
| An area the chain never visits | `an_area_the_chain_never_visits_is_refused` | `RP0301` |
| The chain does not terminate at one exit area | `a_chain_that_does_not_terminate_at_one_exit_area_is_refused` | `RP0301` |
| A start area that also declares an arrival cell | `an_area_that_is_both_a_start_and_an_arrival_is_refused` | `RP0301` |
| Fewer than two areas | `one_area_is_not_a_collection` | `RP0301` |
| A directory name that is not the area identity | `a_directory_that_is_not_the_area_identity_is_refused` | `RP0301` |
| A repeated area identity | `a_repeated_area_identity_is_refused` | `RP0301` |
| An area that diverges from the shared grammar | `an_area_that_diverges_from_the_shared_grammar_is_refused` | `RP0302` |
| A plan carrying a foreign identity or version | `the_plan_identity_and_version_are_bound` | `RP0104` |
| A plan that is not canonical bytes | `a_plan_that_is_not_canonical_bytes_is_refused` | `RP0102` |

The mode itself is proved by two more:
`the_collection_mode_writes_the_document_and_reports_it` runs the built binary
and checks that the file it wrote is what the library produces and that its
stdout document names the identity, and
`the_collection_mode_fails_closed_on_stdout` checks that a usage error is a
canonical rejection on stdout, exit status 1, and no output file.

`RP0301` is the route-graph code and `RP0302` the grammar code, both added to
`crates/nomos-render-plan/src/error.rs`. The `RP` space is this crate's own and
is disjoint from the frozen Gate K `EK` space by its prefix.

## Where the JavaScript went

| Was | Now |
| --- | --- |
| `src/build-collection.mjs` | Deleted. `crates/nomos-render-plan/src/collection.rs` |
| The collection assertions in `src/area-collection.test.mjs` (its `collection.route` arrival-cell check) | `crates/nomos-render-plan/tests/collection.rs`. The file keeps its plan-level checks over the four committed plans, which are about the plans and not the collection |
| `cmp "$output_dir/areas.json" area-collection.example.json` in `gaol` | `verify.mjs --collection`, which byte-compares the committed example, walks the chain, and re-derives every plan digest the collection publishes from the file on disk |
| The viewer's `nomos.experiment.area_collection@2` binding | `nomos.area_collection@1`, with the retired identity refused by name |

## Proof

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
docs/evaluation/r1-schema-ownership.sh          → schema_identities_r1 5
node --test apps/nomos-viewer/test/*.test.mjs
experiments/executable-gaol/gaol verify
node apps/nomos-viewer/build.mjs --from target/executable-gaol --out apps/nomos-viewer/dist
node apps/nomos-viewer/smoke/smoke.mjs --dist apps/nomos-viewer/dist --out target/nomos-viewer-smoke --require-chrome
```

The pull request records each one's output. Under `AGENTS.md` nothing is green
until someone other than its author reruns it.
