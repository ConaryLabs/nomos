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
| `nomos.presentation_source@2` | `nomos-render-plan` | `crates/nomos-render-plan/src/source.rs` | the decoded `PresentationSource`: area identity, route placement and this area's own arrival cell, pursuit light, bounded architecture in integer vertical steps, presentation actors with their declared role, and socket-anchored effects | not encoded: this is an input schema, hand-authored as pretty-printed JSON and read by `crates/nomos-render-plan/src/json.rs`, whose value type has no decimal variant | `source::read_source`, which binds the identity (`RP0104`), checks every field set exactly, checks each identifier grammar (`RP0206`), and refuses every bounded-area violation (`RP0202`) | the four `experiments/executable-gaol/areas/*/presentation.json` files; never written by any command | `nomos-render-plan`, which is its only reader | active R1-3 |
| `nomos.area_collection@2` | `nomos-render-plan` | `crates/nomos-render-plan/src/collection.rs` | the collection document `collection::build` assembles over the compiled plans: ordered areas with identity, label, start flag, exit gate, destination, arrival cell, and the published plan file name and SHA-256; the derived route chain; and the visual grammar every area shares, whose `entity_kinds` names the compiled kinds and not what they are drawn as | `nomos_render_plan::collection::build` assembles a `nomos_core::CanonicalValue` and calls `to_canonical_bytes`; there is no encoder in this crate, and `tests/collection.rs::the_document_is_canonical_and_names_the_plan_bytes` proves `parse_canonical` accepts the result and re-encodes it byte-identically | `collection::read_area`, which binds each input plan's identity (`RP0104`) through the one `plan::rendering_plan_schema` constant, and refuses every route-graph violation (`RP0301`) and any divergence from the shared visual grammar (`RP0302`) | none: written to the `--out` path the caller names, never into a package or a run bundle, and outside the state-hash domain because it is derived | `apps/nomos-viewer`, which binds the identity in `src/plan.mjs`; `experiments/executable-gaol/src/verify.mjs --collection` | active R1-5 |
| `nomos.rendering_plan@3` | `nomos-render-plan` | `crates/nomos-render-plan/src/plan.rs` | the `CanonicalValue` document `compile()` assembles: area identity, derived objective, route, pursuit, republished projection identities and digests, classified entities carrying no assembly or material family, presentation actors with their declared role, and effects, per-scenario runtime facts, and derived interaction edges | `nomos_render_plan::plan::compile` assembles a `nomos_core::CanonicalValue` and calls `to_canonical_bytes`; there is no encoder in this crate, and `tests/canonical_round_trip.rs` proves `parse_canonical` accepts the result and re-encodes it byte-identically | none in-tree: derived read-only output with no strict package reader; its consumers bind the identity string | none: written to the `--out` path the caller names, never into a package or a run bundle, and outside the state-hash domain because it is derived | `apps/nomos-viewer` and `crates/nomos-play`, which reads seven of its thirteen fields and neither `scenarios` nor `interactions`; the study's SVG capture renderer | active R1-5 |
| `nomos.play_state@1` | `nomos-play` | `crates/nomos-play/src/state.rs` | the authoritative state of one area: the play tick, the embedded kernel `PersistedRuntimeState`, the ordered actor collection with each actor's declared role and integer lattice cell, the pursuit counter and its declared light, the outcome, and the cumulative counters | `PlayState::to_canonical` assembles a `nomos_core::CanonicalValue` and calls `to_canonical_bytes`; there is no encoder in this crate, and the embedded kernel envelope is nested verbatim rather than re-encoded | `PlayState::decode`, which binds the identity (`PL0101`), checks the field set exactly, refuses an actor collection that is not one player and at most one pursuer (`PL0103`), and hands the embedded envelope to `PersistedRuntimeState::from_canonical_bytes`, which refuses `EK0813` when it belongs to different simulation semantics | none: held in memory by the runtime and carried inside `nomos.play_session@1`; never written into a package or a run bundle, and outside the Gate K state-hash domain | `crates/nomos-play` itself, and `nomos.play_session@1`, which carries one per entered area | active R1-5 |
| `nomos.play_command@1` | `nomos-play` | `crates/nomos-play/src/command.rs` | exactly one input per batch: `move {direction}`, `interact {entity, action}`, or `cross {gate}` | `PlayCommand::to_canonical`; the browser adapter builds the same document in JavaScript with its keys in byte order, and the runtime reads it with the kernel's strict canonical reader | `PlayCommand::decode`, which binds the identity (`PL0101`) and refuses a field set that does not match the declared kind exactly (`PL0201`) — a shape refusal, which produces no receipt and no tick | none directly; every committed command is carried in `nomos.play_session@1`'s log and copied verbatim into the receipt for its batch | `apps/nomos-viewer/src/play.mjs`, which constructs them; `nomos-play replay`, which re-executes them | active R1-5 |
| `nomos.play_receipt@1` | `nomos-play` | `crates/nomos-play/src/receipt.rs` | the evidence one batch produced: its ordinal and area, the input verbatim, whether it was accepted and the `PL####` code if not, the tick and kernel state hash before and after, the actor deltas, the outcome before and after, the counters after, the previous receipt's hash, and the hash of the play state it produced | `PlayReceipt::to_canonical`. The receipt's own hash is `sha256` of those bytes and is deliberately not a field of it: a canonical document cannot carry its own digest, which is the position `nomos_sim::CausalReceipt::digest` already takes | none in-tree as a standalone document; a replay compares recorded receipts to produced ones by canonical bytes and reports the first difference with its ordinal | none: carried inside `nomos.play_session@1`, which is what a caller writes out | `nomos-play replay`; the viewer, which derives the HUD message from the last receipt rather than recomputing it | active R1-5 |
| `nomos.play_session@1` | `nomos-play` | `crates/nomos-play/src/session.rs` | the run across areas: the route with a plan and projection digest per area, the entered areas' play states, the cleared-area count, the outcome, the ordered command log, the ordered receipts, and the receipt chain head | `PlaySession::to_canonical`; `PlaySession::sessionText` on the wasm surface returns those bytes so a caller records what the runtime produced rather than a re-serialization of it | `RecordedSession::decode`, which binds the identity (`PL0101`) and checks the field set exactly; `nomos_play::replay` then refuses content whose digests the session does not name (`PL0402`) before it replays anything | the smoke lane's `session.json`, written from the page and replayed natively; never into a package or a run bundle, and outside the Gate K state-hash domain | `nomos-play replay`; `apps/nomos-viewer/smoke/smoke.mjs` | active R1-5 |
| `nomos.presentation_state@1` | `nomos-play` | `crates/nomos-play/src/presentation.rs` | what the renderer draws for one tick: the tick, the kernel state hash, the machine states, the effective movement and light facts resolved at that state, the actor cells and roles, the outcome, the counters, the pursuit condition, and the interactions legal within reach | `presentation_state()` assembles a `nomos_core::CanonicalValue`; `machine_states`, `movement`, and `effective_light` are spelled exactly as `nomos.rendering_plan@3`'s `scenarios[]` spells them, including the `null` cost on a blocked subject | none in-tree: derived read-only output, emitted once per tick and never read back by this crate | none: returned across the wasm boundary and drawn; persisted nowhere and outside every hash domain, the standing `nomos.effective_facts@1` has | `apps/nomos-viewer/src/{ui,render}.mjs`, which render from it and compute none of it | active R1-5 |
Ten R1 identities have entered the accepted tree.

`nomos.effective_facts@1` is the read-only effective-fact projection accepted as
R1-1 under `RUNTIME.md` §5 (issue #126, PR #130): given a strictly verified world
package and a runtime state it composes, for every resolver subject, the
effective movement disposition, cost, ordered reason claim IDs, and effective
light. It resolves nothing itself — `nomos_sim::resolve_movement` and
`nomos_sim::resolve_light` do that — and it is derived output, so it is persisted
nowhere and enters no hash domain.

`nomos.presentation_source@2` is the typed presentation source accepted as R1-3
(issue #146): one `presentation.json` per area, replacing the unversioned
`area.json`. It is the epoch's first *input* schema — every other R1 identity
names derived output — so its acceptance is about refusal rather than about
bytes: a version mismatch, an unknown field, a decimal literal anywhere in the
file, an identifier outside its declared grammar, or a bounded-area violation is
refused with a stable `RP####` code. Its owner file declares it, decodes it, and
is the only reader of it. `@2` (issue #154) adds `actors[].role`, `player` or
`pursuer`, and retires `REQUIRED_ACTORS` — the constant that forced the
identities `player` and `gaoler` — for the rule the runtime actually needs:
exactly one player, at most one pursuer, identities free.
`renaming_both_actors_changes_nothing` renames both and decodes, which is the
proof that nothing reads the strings. `@1`'s row is replaced: it was never
persisted outside this repository and its four authored files are edited in the
same change.

`nomos.rendering_plan@3` is the rendering plan. `@1` (issue #139) reproduced the
study's camelCase, dotted-key, decimal-carrying document, which
`nomos_core::CanonicalValue` cannot express, so R1-2 shipped a private canonical
encoder in `crates/nomos-render-plan/src/doc.rs` — a second implementation of the
`KERNEL.md` section 7 byte profile in the accepted tree, recorded as a drift risk
by issue #144. `@2` is designed to fit inside `CanonicalValue` with no widening:
snake_case field names, the kernel's own stable-ID arrays in place of the two
dotted-key and two entity-keyed objects, and integer vertical steps in place of
every decimal. `doc.rs` is deleted, and the plan is now the kernel's canonical
bytes rather than a second encoder's agreement with them. `@3` (issue #154, with
issue #153 folded in so the four fixtures are regenerated once) adds
`actors[].role` and drops `entities[].visual_assembly` and
`entities[].material_family`, whose assignment `crates/nomos-render-plan/src/catalog.rs`
said in its own comment belonged in the renderer catalog. Each superseded row is
replaced rather than kept: none was persisted anywhere, none entered a hash
domain, and none had a consumer outside this repository.

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

`nomos.area_collection@2` is the area collection added under issue #152. It is
the second identity `nomos-render-plan` emits: given the four
compiled plans it publishes the route graph — one start area, one gate per hop,
each hop's arrival cell read from the destination's own plan, and a chain that
visits every declared area exactly once and terminates — plus the visual grammar
all four areas are required to share, and one row per area naming the plan file
and its SHA-256. `@2` (issue #154) publishes `visual_grammar.entity_kinds` in
place of `entity_assemblies`, whose other two columns left the plan with `@3`. It replaces `nomos.experiment.area_collection@2`, declared by
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
