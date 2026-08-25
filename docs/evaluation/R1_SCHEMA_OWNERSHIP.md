---
title: R1 canonical-schema ownership register
status: R1 register; revision 1
date: 2026-08-25
issue: 133
authority: RUNTIME.md §3
---

# R1 canonical-schema ownership register

This is the live canonical-schema register for the R1 epoch. It records every
persisted or contractual schema identity declared in `crates/*/src` **after**
the Gate K freeze commit `eb86f25f5084a5da83cdd4f26e42e68089367a11`.

The twenty Gate K identities remain owned exactly as recorded in
`docs/evaluation/SCHEMA_OWNERSHIP.md` at `eb86f25`, and are not repeated here.
That receipt is final historical evidence at its freeze commit; this register is
the additive R1 continuation of it. `docs/evaluation/r1-schema-ownership.sh`
re-asserts the twenty Gate K identities and their owner-source assertions on
every run, so an identity is owned if and only if it appears in exactly one of
the two documents.

Authority is `RUNTIME.md` §3, under which kernel crates may gain read-only R1
surface — so a new identity may be declared inside a kernel crate — provided no
Gate K command, artifact, hash, or diagnostic changes. Schema ownership stays
exact: one canonical identity, one owner crate, one owner file.

## Inventory

Columns are those of the Gate K receipt, plus an explicit **Owner file** column
so that `r1-schema-ownership.sh` can match a declaration site mechanically
rather than by prose. The identity and the owner file are read from this table
verbatim; both are wrapped in backticks and the owner file is repository-
relative.

| Canonical identity | Owner | Owner file | Authoritative type set | Encoder | Strict reader / verifier | Persisted boundary | Primary consumers | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `nomos.effective_facts@1` | `nomos-sim` | `crates/nomos-sim/src/effective_facts.rs` | the `effective_facts()` document, embedding the `ResolvedMovementFacts` and `ResolvedLightFacts` renderings it composes | `nomos_sim::effective_facts` composes the `CanonicalValue`; canonical entity-sorted bytes written to stdout by `nomos effective-facts` | none in-tree: derived read-only output, never re-read by the kernel and with no strict package reader; its first consumer binds identity and version | none: stdout only, never written into a run bundle or package, and outside the state-hash domain because it is derived | R1-2 Rust rendering-plan compilation; today `experiments/executable-gaol/compare-effective-facts.sh` | active R1-1 |
| `nomos.entity_catalog@1` | `nomos-compiler` | `crates/nomos-compiler/src/entity_catalog.rs` | the `entity_catalog()` document, joining the stable World IR `IrEntity` records with the `ProjectedEntity`, `MachineDefinition`, `MovementSubject`, and `LightSubject` values of the verified plans | `nomos_compiler::entity_catalog` composes the `CanonicalValue`; canonical entity-sorted bytes written to stdout by `nomos entity-catalog` | none in-tree: derived read-only output, never re-read by the kernel and with no strict package reader; its first consumer binds identity and version | none: stdout only, never written into a package or a run bundle, and outside the state-hash domain because it is derived | R1-2 Rust rendering-plan compilation, which needs the entity kind and capability set the four projections do not carry | active R1-2 input |
| `nomos.presentation_source@1` | `nomos-render-plan` | `crates/nomos-render-plan/src/source.rs` | the decoded `PresentationSource`: area identity, route placement and this area's own arrival cell, pursuit light, bounded architecture in integer vertical steps, presentation actors, and socket-anchored effects | not encoded: this is an input schema, hand-authored as pretty-printed JSON and read by `crates/nomos-render-plan/src/json.rs`, whose value type has no decimal variant | `source::read_source`, which binds the identity (`RP0104`), checks every field set exactly, checks each identifier grammar (`RP0206`), and refuses every bounded-area violation (`RP0202`) | the four `experiments/executable-gaol/areas/*/presentation.json` files; never written by any command | `nomos-render-plan`, which is its only reader | active R1-3 |
| `nomos.area_collection@1` | `nomos-render-plan` | `crates/nomos-render-plan/src/collection.rs` | the collection document `collection::build` assembles over the compiled plans: ordered areas with identity, label, start flag, exit gate, destination, arrival cell, and the published plan file name and SHA-256; the derived route chain; and the visual grammar every area shares | `nomos_render_plan::collection::build` assembles a `nomos_core::CanonicalValue` and calls `to_canonical_bytes`; there is no encoder in this crate, and `tests/collection.rs::the_document_is_canonical_and_names_the_plan_bytes` proves `parse_canonical` accepts the result and re-encodes it byte-identically | `collection::read_area`, which binds each input plan's identity (`RP0104`) through the one `plan::rendering_plan_schema` constant, and refuses every route-graph violation (`RP0301`) and any divergence from the shared visual grammar (`RP0302`) | none: written to the `--out` path the caller names, never into a package or a run bundle, and outside the state-hash domain because it is derived | `apps/nomos-viewer`, which binds the identity in `src/plan.mjs` and refuses `nomos.experiment.area_collection@2`; `experiments/executable-gaol/src/verify.mjs --collection` | active R1-4 |
| `nomos.rendering_plan@2` | `nomos-render-plan` | `crates/nomos-render-plan/src/plan.rs` | the `CanonicalValue` document `compile()` assembles: area identity, derived objective, route, pursuit, republished projection identities and digests, classified entities, presentation actors and effects, per-scenario runtime facts, and derived interaction edges | `nomos_render_plan::plan::compile` assembles a `nomos_core::CanonicalValue` and calls `to_canonical_bytes`; there is no encoder in this crate, and `tests/canonical_round_trip.rs` proves `parse_canonical` accepts the result and re-encodes it byte-identically | none in-tree: derived read-only output with no strict package reader; its consumers are the quarantined study's JavaScript, which checks the identity string | none: written to the `--out` path the caller names, never into a package or a run bundle, and outside the state-hash domain because it is derived | the executable-gaol viewer, `render-core.mjs`, `play-state.mjs`, `build-collection.mjs`; R1-4's promoted viewer | active R1-3 |

Five R1 identities have entered the accepted tree.

`nomos.effective_facts@1` is the read-only effective-fact projection accepted as
R1-1 under `RUNTIME.md` §5 (issue #126, PR #130): given a strictly verified world
package and a runtime state it composes, for every resolver subject, the
effective movement disposition, cost, ordered reason claim IDs, and effective
light. It resolves nothing itself — `nomos_sim::resolve_movement` and
`nomos_sim::resolve_light` do that — and it is derived output, so it is persisted
nowhere and enters no hash domain.

`nomos.presentation_source@1` is the typed presentation source accepted as R1-3
(issue #146): one `presentation.json` per area, replacing the unversioned
`area.json`. It is the epoch's first *input* schema — every other R1 identity
names derived output — so its acceptance is about refusal rather than about
bytes: a version mismatch, an unknown field, a decimal literal anywhere in the
file, an identifier outside its declared grammar, or a bounded-area violation is
refused with a stable `RP####` code. Its owner file declares it, decodes it, and
is the only reader of it.

`nomos.rendering_plan@2` is the rendering plan. `@1` (issue #139) reproduced the
study's camelCase, dotted-key, decimal-carrying document, which
`nomos_core::CanonicalValue` cannot express, so R1-2 shipped a private canonical
encoder in `crates/nomos-render-plan/src/doc.rs` — a second implementation of the
`KERNEL.md` section 7 byte profile in the accepted tree, recorded as a drift risk
by issue #144. `@2` is designed to fit inside `CanonicalValue` with no widening:
snake_case field names, the kernel's own stable-ID arrays in place of the two
dotted-key and two entity-keyed objects, and integer vertical steps in place of
every decimal. `doc.rs` is deleted, and the plan is now the kernel's canonical
bytes rather than a second encoder's agreement with them. `@1`'s row is replaced
rather than kept: it was never persisted anywhere, never entered a hash domain,
and had no consumer outside this repository.

`nomos.entity_catalog@1` is the read-only entity catalog added under issue #138:
given a strictly verified world package it emits, for every entity, the World IR
primitive kind and `expansion.capabilities` beside the simulation projection's
binding and machines and the movement and light resolver claims whose subject
that entity is. It classifies nothing and resolves nothing; every field is
copied from typed evidence the package opener has already verified, so that no
downstream compiler has to infer an entity's kind from a naming convention. It
is declared in `nomos-compiler` because that crate owns World IR decoding and
projection generation and is the only kernel crate that can see both halves of
the join: `nomos-sim` has no edge to `nomos-schema` and therefore cannot name an
entity's primitive kind at all. Like the effective-fact projection it is derived
output, persisted nowhere, and outside every hash domain.

`nomos.area_collection@1` is the area collection added under issue #152. It is
the fifth identity and the second one `nomos-render-plan` emits: given the four
compiled plans it publishes the route graph — one start area, one gate per hop,
each hop's arrival cell read from the destination's own plan, and a chain that
visits every declared area exactly once and terminates — plus the visual grammar
all four areas are required to share, and one row per area naming the plan file
and its SHA-256. It replaces `nomos.experiment.area_collection@2`, declared by
`experiments/executable-gaol/src/build-collection.mjs`, which
`docs/review/nomos-viewer.md` finding 2 recorded as the last identity accepted
code bound whose declaration was quarantined. That file is deleted, and the
viewer refuses the retired identity by name. The two identities never coexist:
`@2` was never persisted anywhere, never entered a hash domain, and had no
consumer outside this repository.

## How a row is added

A row is added in the same change that adds the identity to `crates/*/src`,
naming the owner crate and the exact owner file that declares it, so that
`docs/evaluation/r1-schema-ownership.sh` passes on that change's head and the
identity is owned from the moment it exists.
