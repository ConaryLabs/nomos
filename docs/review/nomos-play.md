---
title: The authoritative play runtime — R1-5 design record
status: R1-5 design record; phase 1, awaiting review before implementation
date: 2026-08-25
issue: 154
branch: r1/issue-154-nomos-play
accepts_against: RUNTIME.md §5 R1-5 (revision 1)
registers: docs/evaluation/R1_SCHEMA_OWNERSHIP.md (nomos.play_state@1, nomos.play_command@1, nomos.play_receipt@1, nomos.play_session@1, nomos.presentation_state@1, nomos.rendering_plan@3 replacing @2)
depends_on: issue #126 (R1-1 effective facts), issue #139 (R1-2 plan compiler), issue #146 (R1-3 presentation source), issue #148 (R1-4 viewer)
folds_in: issue #153 (rendering_plan@3, kind→assembly out of Rust)
applies_to: RUNTIME.md §3, §5, §6, §7; docs/evaluation/R1_SCHEMA_OWNERSHIP.md; docs/review/presentation-source.md; docs/review/nomos-viewer.md; docs/review/executable-gaol-ownership-audit.md
---

# The authoritative play runtime

## Problem

`apps/nomos-viewer/src/play.mjs` is 316 lines of JavaScript that own the player's
cell, the traversal cost of a step, mass collision, the exit through a door, the
gaoler's pursuit, capture, the area transition, and both counters. The kernel
supplies a captured ladder of five scenarios per area and the browser walks it.
`RUNTIME.md` §1 criterion 2 forbids a surviving shadow resolver, and §5 R1-5 is
where the *simulation* stops being one.

This record is the design R1-5 implements: one R1 crate, `crates/nomos-play`,
that layers actors, the command batch, occupancy, pursuit, receipts, and replay
over the kernel's own transactions; the same crate compiled to
`wasm32-unknown-unknown` and running in the browser as the only authority the
player ever touches; and a smoke lane that records what the browser did and
replays it natively to prove the two ran the same code over the same bytes.

Nothing here is accepted until the implementation lands with its evidence and a
non-author rerun (`AGENTS.md`). §10 records what this design found wrong,
under-specified, or in tension with the contract; three of those findings need
an owner ruling before phase 2 starts.

---

## 1. The crate

### 1.1 The epoch decision, restated with its consequence

Issue #154 settles it: actors do not enter `nomos.runtime_state@2`. Adding a
field to the kernel's state envelope would change the canonical bytes of every
run bundle and every state hash in the tree, including worlds that declare no
actor at all, and `RUNTIME.md` §3 option (a) admits kernel surface only when
"no Gate K command, artifact, hash, or diagnostic changes". So `nomos-play`
holds the actors and treats the kernel's persisted state as an opaque embedded
authority: it hands the kernel bytes and commands, and takes back bytes, hashes,
and resolved facts.

The consequence is stated once here because every shape below depends on it:
**`nomos-play` never invents a movement disposition, a traversal cost, a reason,
or a light fact.** Every one of those comes from `nomos_sim::resolve_movement`
and `nomos_sim::resolve_light` evaluated at the embedded kernel state. What
`nomos-play` owns is *where the actors are*, *whose turn it is*, and *what the
run has cost so far* — facts the kernel has no opinion about.

### 1.2 Layout and line estimates

```text
crates/nomos-play/
  Cargo.toml                       25   deps: nomos-core, nomos-projection, nomos-sim
                                        dev-deps: nomos-compiler (equivalence fixture only)
  src/lib.rs                       95   the five schema identities, re-exports, crate doc
  src/error.rs                    130   PlayError, PlayResult, the PL#### code table
  src/semantics.rs                540   simulation-projection bytes -> SimulationPlan (§10 finding 1)
  src/plan.rs                     280   nomos.rendering_plan@3 -> the typed facts the reducer needs
  src/actor.rs                     90   Actor, Role, the stable-id ordering
  src/state.rs                    250   PlayState; nomos.play_state@1 encode and decode
  src/command.rs                  170   PlayCommand; nomos.play_command@1 encode and decode
  src/occupancy.rs                150   the occupancy predicate and the step cost
  src/batch.rs                    290   the reducer: the ordering rule, the pursuit rule, crossing
  src/receipt.rs                  220   PlayReceipt; nomos.play_receipt@1; the hash chain
  src/session.rs                  240   PlaySession; nomos.play_session@1; enter and cross
  src/presentation.rs             180   nomos.presentation_state@1
  src/replay.rs                   170   replay a session's log and compare receipts
  src/wasm.rs                     210   the extern "C" exports and the linear-memory contract
  src/bin/nomos-play.rs           190   the `replay` subcommand
                                 ----
                                 3230   source

  tests/documents.rs              260   every field of every document; the no-float schema test
  tests/reducer.rs                300   ordering, occupancy, cost, refusals, crossing
  tests/pursuit.rs                200   the pursuit rule and capture on a fixed log
  tests/session.rs                180   four-area session, counters, reset
  tests/replay.rs                 220   ten-run byte identity; a tampered log is refused
  tests/semantics.rs              200   decoder round trip; equivalence with nomos-compiler
  tests/corpus.rs                 160   the four committed areas: route, counters, capture
  tests/common/mod.rs             240   fixture builders
                                 ----
                                 1760   tests
```

No file passes the ~1,000-line rule in `AGENTS.md`. `semantics.rs` is the
largest and is the subject of §10 finding 1; if the owner rules it into the
kernel instead, this crate loses 540 lines and `nomos-projection` gains them.

### 1.3 Dependency edges

```text
nomos-play  ->  nomos-core        canonical values, ids, hashes, checked arithmetic
nomos-play  ->  nomos-projection  SimulationPlan and the projected types it is built from
nomos-play  ->  nomos-sim         commit_transaction, resolve_command, resolve_movement,
                                  resolve_light, PersistedRuntimeState, SimulationState

nomos-play  ->  nomos-compiler    DEV ONLY: tests/semantics.rs compares the decoder's
                                  SimulationPlan with open_compiled_package()'s own
```

Exactly the three edges the issue's Scope declares, plus one dev-dependency
edge. The dev edge has precedent: `nomos-render-plan` already carries dev
edges to `nomos-projection` and `nomos-sim` for the issue #132 divergence
fixture, and `RUNTIME.md` §3 records them in the declared-member entry.

Forbidden and not taken: any edge to `nomos-schema`; any parse of `.nomos`
source, Canonical World IR, or compiler receipts; any third-party crate. The
crate carries `#![forbid(unsafe_code)]` everywhere except `src/wasm.rs`, which
carries `#![allow(unsafe_code)]` on the module because an `extern "C"` pointer
ABI cannot be written without it — that is the only `unsafe` block in the R1
tree and §5.3 states exactly what it does.

### 1.4 What the crate does *not* contain

No renderer, no DOM, no filesystem access outside `src/bin/`, no clock, no
random source, no floating-point type. `tests/documents.rs` greps the crate's
own source for `f32`, `f64`, `SystemTime`, `Instant`, `now()`, and `rand` and
fails on a hit; that is the schema test `RUNTIME.md` §5 R1-5 asks for, applied
to the source as well as to the documents.

---

## 2. The documents

Five identities, all declared by the emitting code, all expressible in
`nomos_core::CanonicalValue` with no widening: seven variants, `FieldName` =
`[a-z][a-z0-9_]*`, no floating-point variant
(`crates/nomos-core/src/canonical.rs:105-121`). Canonical bytes sort object
keys, so the reading order below is not the byte order.

Every entity- or actor-keyed collection is a `nomos_core::canonical::keyed_array`
of `{id, …}` objects — the kernel's own stable-ID idiom, so ordering and
duplicate-identity refusal come from `nomos-core` rather than from this crate.
This is the same choice `nomos.rendering_plan@2` made
(`docs/review/presentation-source.md` §2.3).

### 2.1 `nomos.play_state@1`

Owner: `crates/nomos-play/src/state.rs`. The authoritative state of one area.

| Field | Shape | Owner | Constraints |
| --- | --- | --- | --- |
| `schema` | `{name, version}` | this file | exactly `nomos.play_state@1`; `PL0101` otherwise |
| `area` | text | rendering plan `area.id` | must equal the `area.id` of the plan the state was opened against; `PL0102` otherwise |
| `tick` | Uint | this file | the ordinal of the last committed batch; starts at 0, `+1` per batch through `nomos_core::arith::add_u64`, never resets while a session lives |
| `kernel` | object | `nomos-sim` | the `nomos.persisted_runtime_state@2` object, nested verbatim; see §2.6 |
| `actors` | keyed_array by `id` | this file | `[{cell, id, role}]`; ids unique; exactly one `role == "player"`; at most one `role == "pursuer"`; `PL0103` otherwise |
| `actors[].id` | text | rendering plan `actors[].id` | an `EntityId` |
| `actors[].role` | text | rendering plan `actors[].role` | `player` \| `pursuer` |
| `actors[].cell` | `{x, y, z}` | this file | `Int`; `0 ≤ x < bounds.width`, `0 ≤ y < bounds.height`, `z == 0` |
| `pursuit` | object | this file | exactly `{light, moves_since_step}` |
| `pursuit.light` | text | rendering plan `pursuit.light` | an `EntityId` naming a plan entity of kind `light` |
| `pursuit.moves_since_step` | Uint | this file | `0` or `1`; the counter the pursuit rule reads. It is not a clock: it counts accepted player moves since the pursuer last stepped |
| `outcome` | text | this file | `playing` \| `escaped` \| `caught` |
| `counters` | object | this file | exactly `{moves, traversal_cost}` |
| `counters.moves` | Uint | this file | **cumulative across the whole session**, not per area; see §2.4 |
| `counters.traversal_cost` | Uint | this file | cumulative across the whole session |

Nothing here is fractional, nothing is a duration, and nothing names a frame.
`cell` components are `Int` rather than `Uint` because
`nomos_projection::LatticeCell` is `i32` and the state must be able to spell a
cell the kernel could spell; the bounds check refuses a negative one.

### 2.2 `nomos.play_command@1`

Owner: `crates/nomos-play/src/command.rs`. Exactly one input per batch.

| Field | Shape | Owner | Constraints |
| --- | --- | --- | --- |
| `schema` | `{name, version}` | this file | exactly `nomos.play_command@1` |
| `kind` | text | this file | `move` \| `interact` \| `cross` |
| `direction` | text | this file | present **iff** `kind == "move"`; one of `east`, `north`, `south`, `west` |
| `entity` | text | this file | present **iff** `kind == "interact"`; an `EntityId` naming a plan entity |
| `action` | text | this file | present **iff** `kind == "interact"`; an `Ident` |
| `gate` | text | this file | present **iff** `kind == "cross"`; an `EntityId` naming a plan entity of kind `door` |

The field set is checked exactly per `kind`, the way
`crates/nomos-render-plan/src/source.rs:787` checks presentation source: a
missing field, an unknown field, or a field belonging to another kind is
refused with `PL0201` and the command never becomes a batch (§3.5 splits shape
refusal from rule refusal).

There is no `argument` field. `nomos_sim::CommandRequest::new` takes
`Option<CatalogValueId>` and the committed corpus declares no command with a
credential requirement (`crates/nomos-projection/src/simulation.rs:15-20`;
every `requirement` in the four `simulation.json` members is `{"kind":"none"}`).
An `interact` that resolves to a transition requiring a credential is refused
with `PL0305`, "this runtime declares no credential arguments", rather than
guessed at. A later version adds the field when content needs it.

### 2.3 `nomos.play_receipt@1`

Owner: `crates/nomos-play/src/receipt.rs`. One per batch, always — including
for a batch the rules refused (§3.5).

| Field | Shape | Owner | Constraints |
| --- | --- | --- | --- |
| `schema` | `{name, version}` | this file | exactly `nomos.play_receipt@1` |
| `ordinal` | Uint | this file | 0-based index in the session's receipt array; strictly `previous + 1` |
| `area` | text | rendering plan | the area the batch ran in |
| `input` | object | `play_command@1` | the input verbatim, schema field and all |
| `accepted` | Bool | this file | `true` iff the batch changed any authoritative field other than `tick` |
| `refusal` | text or Null | this file | the `PL####` code when `accepted` is false; `Null` otherwise |
| `tick_before` | Uint | this file | |
| `tick_after` | Uint | this file | always `tick_before + 1` |
| `kernel_state_hash_before` | text | `nomos-sim` | 64 lowercase hex |
| `kernel_state_hash_after` | text | `nomos-sim` | equal to `_before` unless the batch committed a kernel transaction |
| `actor_deltas` | keyed_array by `id` | this file | `[{from, id, to}]`, one row per actor whose cell changed; `[]` when none did |
| `outcome_before` | text | this file | |
| `outcome_after` | text | this file | |
| `counters_after` | object | this file | `{moves, traversal_cost}`, cumulative |
| `previous_receipt_hash` | text | this file | 64 lowercase hex; 64 zeros for `ordinal == 0` |
| `play_state_hash_after` | text | this file | `StateHash::of_envelope` over the `play_state@1` value the batch produced |

The receipt's own hash is `Sha256Digest::of_bytes(receipt.to_canonical_bytes())`
and is deliberately **not a field of the receipt**: a canonical document cannot
contain its own digest, and the kernel takes the same position —
`CausalReceipt::digest()` is derived, not stored
(`crates/nomos-sim/src/receipt.rs:271`). The chain link is
`previous_receipt_hash` on the *next* receipt, and the chain's head is the
session's `receipt_chain_head`.

### 2.4 `nomos.play_session@1`

Owner: `crates/nomos-play/src/session.rs`.

| Field | Shape | Owner | Constraints |
| --- | --- | --- | --- |
| `schema` | `{name, version}` | this file | exactly `nomos.play_session@1` |
| `route` | Array, declared order | area collection | `[{area, plan_digest, semantics_digest}]`, one row per area entered so far, in the order entered |
| `position` | Uint | this file | index into `route` of the live area; `route.len() == position + 1` |
| `areas` | Array, aligned with `route` | this file | the `play_state@1` document of each entered area; `areas[position]` is live and the rest are as they were left |
| `areas_cleared` | Uint | this file | the number of areas whose outcome reached `escaped` |
| `outcome` | text | this file | `playing` \| `escaped` \| `caught` \| `completed` |
| `log` | Array, commit order | this file | every `play_command@1` committed in this session, across areas, refused ones included |
| `receipts` | Array, commit order | this file | every `play_receipt@1`, aligned index-for-index with `log` |
| `receipt_chain_head` | text | this file | the hash of the last receipt, or 64 zeros for an empty session |

There is deliberately **no session-level `tick` and no session-level `moves` or
`traversal_cost`.** The tick is `areas[position].tick` and it does not reset on
a crossing; the counters are `areas[position].counters` and they are cumulative
by construction, which is what `play.mjs:69-80` already does when it carries
`movementCost` and `moves` through `enterArea`. Restating either at session
level would be a derived second authority, which is the defect
`docs/review/executable-gaol-ownership-audit.md` §2 spends nine rows on. The
one number that is genuinely the session's and not any area's is
`areas_cleared`, and it is here.

`plan_digest` and `semantics_digest` are SHA-256 over the exact bytes the
runtime was handed. They are what makes a recorded session replayable: the
native replay refuses a session whose digests do not match the areas it is
pointed at, rather than replaying against different content and reporting a
difference that is not the runtime's.

### 2.5 `nomos.presentation_state@1`

Owner: `crates/nomos-play/src/presentation.rs`. Derived, emitted once per tick,
never persisted, outside every hash domain — the same standing
`nomos.effective_facts@1` has (`RUNTIME.md` §5 R1-1).

| Field | Shape | Owner |
| --- | --- | --- |
| `schema` | `{name, version}` | this file |
| `area` | text | rendering plan |
| `tick` | Uint | `play_state.tick` |
| `kernel_state_hash` | text | `nomos-sim` |
| `machine_states` | keyed_array by `namespace` of `{namespace, state}` | `nomos-sim`, the embedded `SimulationState`'s machines |
| `movement` | keyed_array by `entity` of `{cost, disposition, entity, reasons}` | `nomos_sim::resolve_movement` |
| `effective_light` | keyed_array by `entity` of `{emitting, entity}` | `nomos_sim::resolve_light` |
| `actors` | keyed_array by `id` of `{cell, id, role}` | `play_state.actors` |
| `outcome` | text | `play_state.outcome` |
| `counters` | `{moves, traversal_cost}` | `play_state.counters` |
| `pursuit` | `{hunting, light, moves_since_step}` | `play_state.pursuit`, with `hunting` derived (§3.3) |
| `interactions` | keyed_array by `entity` of `{action, entity}` | this file — every command legal at this state on an entity adjacent to the player (§3.6) |

`machine_states`, `movement`, and `effective_light` are spelled exactly as
`nomos.rendering_plan@2`'s `scenarios[]` spells them, and `cost` on a blocked
subject is the same `Null` normalization `RUNTIME.md` §5 R1-1 names. That is
not a coincidence and it is load-bearing: `apps/nomos-viewer/src/plan.mjs`'s
`machineState`, `doorState`, `wardSealed`, `lightOf`, and `movementOf`
accessors (`plan.mjs:888-912`) take a scenario-shaped object, so they read a
presentation state unchanged and the renderer needs no second accessor set.

`presentation_state` carries no prose. Guidance strings stay in `ui.mjs` (§8.2):
they are assembled from identifiers the plan already publishes, and putting
authored prose into an authoritative document would reopen audit §3 item 26
after R1-4 closed it.

### 2.6 How the kernel state is embedded

`play_state.kernel` is the `nomos.persisted_runtime_state@2` document as a
**nested `CanonicalValue::Object`**, not a hex or base64 string of its bytes.

Three reasons, in order of weight.

1. **It is the kernel's own idiom.** `PersistedRuntimeState::to_canonical_bytes`
   nests the runtime state as an object under `state`, and
   `from_canonical_bytes` recovers the inner bytes by re-encoding that
   sub-object: `field(fields, "state")?.to_canonical_bytes()`
   (`crates/nomos-sim/src/state_persistence.rs:69`). Canonical encoding is
   context-free — a value's bytes do not depend on where it sits — so
   `play_state.kernel.to_canonical_bytes()` is byte-for-byte the persisted
   envelope, and `PersistedRuntimeState::from_canonical_bytes` accepts it
   directly. One line of code, no second encoding.
2. **It keeps the document one document.** A hex blob would make `play_state`
   unreadable by `parse_canonical` consumers and would double the byte cost of
   the largest field.
3. **It keeps the hash single-owned.** The persisted envelope already carries
   `state_hash` and `runtime_semantics_digest`
   (`state_persistence.rs:107-118`). `play_state` therefore does **not** repeat
   either at its own level. The issue's phrase "embedded `PersistedRuntimeState`
   bytes + its `state_hash`" is satisfied by the nesting: the hash is present,
   inside, where `nomos-sim` owns it. Hoisting a copy would be exactly the kind
   of double authority `docs/review/presentation-source.md` §4.2 spent nine
   rows removing.

**No compiled static entity is copied.** The persisted state's `entities[]`
carry only `{id, binding}` — the kernel's own runtime bindings, which are its
state, not the plan's. The plan's entity records — `kind`, `anchor`,
`machine_namespaces`, `provenance` — are read from the plan and never written
into `play_state`. The actors collection holds `{cell, id, role}` and nothing
else; the assembly name stays in the plan where the renderer reads it.

Measured, for the committed corpus: north-gaol's `simulation.json` is 6,221
bytes and its `initial-state.json` is 1,196 bytes of nested object; the play
state adds roughly 300 bytes of actors, pursuit, counters, and outcome on top.

### 2.7 Every value is expressible in `CanonicalValue`

- **Field names.** Every literal above is snake_case ASCII, so every one is a
  `FieldName`. No object anywhere takes its keys from data: machine namespaces
  (`north_gate.ward`) and file names carry a dot and are therefore values, in
  `{namespace, state}` and `{file, digest}` pairs, exactly as `@2` fixed.
- **Numbers.** `tick`, `ordinal`, `moves`, `traversal_cost`, `position`,
  `areas_cleared`, and movement `cost` are `Uint`. Lattice components are
  `Int`. Nothing is fractional; `cost` on a blocked subject is `Null`.
- **Hashes.** Every hash and digest is 64 lowercase hex characters as `Text`,
  the spelling `StateHash::to_hex` and `Sha256Digest::to_hex` produce and
  `from_hex` accepts (`crates/nomos-core/src/hash.rs:44,102`; uppercase is
  refused there, so the documents inherit that refusal).
- **Ordering.** Every entity- or actor-keyed collection is `keyed_array`.
  `route`, `areas`, `log`, and `receipts` are plain arrays in declared order,
  because their order is meaning, not identity.

---

## 3. The command batch reducer

`crates/nomos-play/src/batch.rs`. §3.1 and §3.3 are quoted verbatim by that
file's module doc-comment; the paragraphs below them are commentary, not
contract.

### 3.1 The ordering rule

> **Ordering rule.** One input is one batch, one batch is one tick, and a batch
> is a total function of the input and the state it is applied to. Within a
> batch the player's action resolves first and completely — occupancy,
> traversal cost, the kernel transaction, the crossing, the outcome — and the
> state it produces is what every later step in the same batch observes. Then
> every non-player actor is offered a step, in ascending stable actor-id order,
> and each one either steps exactly once or does not step at all, according to
> its own rule read against the state the steps before it in this same batch
> have already produced. When the batch ends its tick is `tick_before + 1`,
> whether or not the player's action was accepted and whether or not any actor
> stepped, and nothing outside the batch ever observes it half-applied. No
> wall-clock reading, elapsed time, frame count, frame rate, fractional value,
> or random draw appears anywhere in this order or in anything it decides.

Resolution order, spelled out as the code runs it:

1. **Shape.** Decode the input as `play_command@1`. A shape failure returns an
   error and produces no batch, no receipt, and no tick (§3.5).
2. **Liveness.** If `outcome` is not `playing`, the batch is refused
   (`PL0301`), committed, and the tick advances. Nothing else runs.
3. **Player action.** Exactly one of:
   - `move {direction}` — §3.2 and §3.4;
   - `interact {entity, action}` — §3.6, which is the only branch that reaches
     `resolve_command` and `commit_transaction`;
   - `cross {gate}` — §3.7.
4. **Facts.** If step 3 changed the kernel state, re-resolve movement and light
   at the new state. Otherwise reuse the facts resolved at the batch's start.
   Either way the facts every later step reads are the *post-action* facts.
5. **Non-player actors.** For each actor with `role != "player"`, in ascending
   `id` order, offer it its rule. Today exactly one actor qualifies and its rule
   is §3.3.
6. **Outcome.** Capture is decided inside the pursuer's step; crossing decided
   the outcome in step 3. No other step changes it.
7. **Commit.** `tick += 1`; build the receipt; append the input to the log and
   the receipt to the receipts; update `receipt_chain_head`.

Step 5 is a loop over a stable-ordered collection even though the corpus has
one pursuer, because the ordering rule has to be total for a collection, not for
a special case. `RUNTIME.md` §5 R1-5's "Not in scope" line rules out a second
pursuer as *content*; it does not license a rule that could not order one.

### 3.2 Occupancy

A cell is available to the player when all four hold:

1. it is inside `architecture.bounds` — `0 ≤ x < width`, `0 ≤ y < height`,
   `z == 0`;
2. it is not inside any `architecture.masses[]` rectangle, which is **half-open**
   — `min.x ≤ x < max.x` and `min.y ≤ y < max.y`, reproducing
   `play.mjs:110-114` exactly;
3. every plan entity whose anchor covers the cell has effective movement
   disposition `traversable` at the batch's kernel state, as resolved by
   `nomos_sim::resolve_movement`;
4. no other actor stands on it.

The step's traversal cost is the **maximum** `cost` over the entities covering
the cell, or `1` when none covers it. Maximum rather than first-match because
maximum-of-active is the kernel's own composition rule
(`crates/nomos-sim/src/resolver.rs:57-71`) and because "first in array order"
is not a rule, it is an accident of iteration. For the committed corpus the two
agree: each area declares exactly one water region and the regions do not
overlap, so every covered cell has one covering entity. That is measured, not
assumed, and `tests/corpus.rs` pins it.

Sources, one owner each: bounds and masses from the rendering plan's
`architecture`; disposition and cost from the kernel resolvers at the embedded
state; actor cells from `play_state.actors`. Rule 3 is where the shadow
resolver dies — `play.mjs:101-108` reads `movementOf(scenario, water.id).cost`
off a captured scenario today, and the same number now comes from
`resolve_movement` at the live state.

**Rule 4 is a behaviour change and it is deliberate.** `play.mjs` lets the
player walk onto the gaoler's cell without consequence; capture happens only
when the gaoler steps onto the player. Under rule 4 the player cannot. Measured
against the committed corpus: the solved four-area route never enters the
pursuer's cell in any area (paths in §8.4), so the change moves no committed
number. It is stated here rather than left implicit because the issue's Scope
names "other actors" as an occupancy source and a reader is entitled to know it
changes something.

### 3.3 The pursuit rule

> **Pursuit rule.** The pursuer is the single actor whose declared role is
> `pursuer`. It is offered a step only at the end of a batch whose player action
> was an accepted `move` inside the lattice — never after a refused move, never
> after an `interact`, and never after a `cross`. It declines the step unless
> the outcome is still `playing` and it is hunting, and it is hunting exactly
> when the area's declared pursuit light is not emitting at the batch's
> post-action kernel state, as resolved by `nomos_sim::resolve_light`. When it
> is offered a step and does not decline, it increments
> `pursuit.moves_since_step`; if the result is less than 2 the batch ends there
> with the counter raised and the pursuer where it was. Otherwise it takes
> exactly one step and sets `pursuit.moves_since_step` back to 0. The step is
> greedy along the dominant axis: let `dx` be the player's `x` minus the
> pursuer's `x`, and `dy` the player's `y` minus the pursuer's `y`; if
> `|dx| > |dy|` the pursuer moves one cell by `signum(dx)` along `x`; otherwise
> if `dy ≠ 0` it moves one cell by `signum(dy)` along `y`; otherwise it moves by
> `signum(dx)` along `x`, which in that branch is necessarily zero because
> `|dx| ≤ |dy| = 0`. The tie `|dx| = |dy| ≠ 0` therefore resolves to the `y`
> axis, and the only branch that does not move is the one in which the pursuer
> already stands on the player's cell. The step consults nothing else: not the
> lattice bounds, not the architecture's masses, not traversal cost, not any
> other actor. If after the step the pursuer's cell equals the player's cell the
> outcome becomes `caught`, and every later command in that area is refused
> until a new session begins.

This is `play.mjs:198-220` reproduced line for line, with three things made
explicit that the JavaScript left to be read out of control flow:

- **When it fires.** `advanceGaoler` is called from exactly one place,
  `attemptMove` line 195, on the branch where the move stayed inside the
  lattice and was accepted. The exit branch (line 160) and
  `attemptInteraction` never call it. The rule above says so in words.
- **The tie-break.** `Math.abs(dx) > Math.abs(dy)` is a strict comparison, so
  `|dx| == |dy|` falls to the `y` branch. The rule says so.
- **The degenerate branch.** `else gaoler.x += Math.sign(dx)` is reachable only
  when `dy == 0` and `|dx| ≤ |dy|`, hence `dx == 0`, hence a zero step onto a
  cell the pursuer already shares with the player. The rule says so rather than
  leaving a reader to work out that the third branch never moves anything.

**The pursuer's step ignores occupancy, and that is a faithful port.** It walks
through masonry, through water at no cost, and out of bounds if the geometry
ever asked it to. R1-5's job is to move the authority for this rule from
JavaScript into a deterministic, replayable, receipt-producing reducer — not to
change what the rule does. Making the pursuer respect §3.2 would change capture
outcomes on any log that reaches a mass, and that is a content and gameplay
decision with its own evidence, filed rather than smuggled in here (§10
finding 6).

**Hunting is resolved at the post-action state.** For a `move` the kernel state
does not change, so the post-action state and the pre-action state are the same
state and this is equivalent to what `attemptMove` does when it resolves
`isHunting` against the scenario it was called with. The rule is written
against the post-action state anyway, because that is the state the rest of the
batch is ordered against and a rule that reads a different one would be a
second ordering.

### 3.4 `move`

`move {direction}` translates the direction to a delta through the same table
the renderer uses — `north = (0, -1)`, `south = (0, +1)`, `west = (-1, 0)`,
`east = (+1, 0)`, `apps/nomos-viewer/src/catalog.mjs:266-271`, restated as a
Rust `const` in `src/plan.rs` and pinned against the JavaScript by
`tests/documents.rs`.

If the target cell is inside the lattice, §3.2 decides: available means the
player moves, `counters.moves += 1`, `counters.traversal_cost += cost`, and the
pursuer is offered its step. Unavailable means the batch is refused with the
code naming which of the four conditions failed — `PL0302` masonry, `PL0303`
a blocked entity, `PL0304` another actor — and the pursuer is **not** offered a
step.

If the target cell is outside the lattice, the move is resolved as a crossing
through the door bound to the face of the player's own cell in the direction of
travel, and it calls exactly the same function `cross` calls (§3.7). This
reproduces R1-4's deliberate divergence from the study — "an exit is a move that
leaves the lattice through a door on the player's own cell whose declared
`anchor.direction` is the direction of travel", `docs/review/nomos-viewer.md`
§2 row 22 — rather than the study's `target.y < 0` special case. If no such
door exists the batch is refused with `PL0306`, "the masonry has no opening
here".

### 3.5 What a refused input does

**Decision: the batch commits. The tick always advances. Every input produces
exactly one receipt.**

An input the rules decline — a move into masonry, a move into a blocked entity,
an `interact` where nothing responds, a `cross` at a gate that is not
traversable, any command at all once the outcome is `caught` or `escaped` —
produces a receipt with `accepted: false`, a `refusal` code, no actor delta, no
counter change, no kernel transaction, and `tick_after == tick_before + 1`. It
does not offer the pursuer a step, because the pursuit rule counts accepted
player moves and not ticks.

Three reasons, in order of weight.

1. **The proof depends on it.** The smoke lane records what the browser did and
   replays it natively, and the claim it establishes is that the browser ran the
   same authority. A refusal is a decision the authority made. If refused inputs
   left no trace, the recorded log would contain only the inputs that happened to
   succeed, and the native replay could not show that the browser refused the
   same things at the same points — which is the interesting half of the claim,
   because a divergence in the rules shows up first as one side accepting what
   the other refuses.
2. **It matches the kernel.** `nomos_sim::execute_requests` does not return
   `Err` when a command is rejected; it returns a valid `RunExecution` whose
   `rejection` field carries the diagnostic and whose evidence is complete
   (`crates/nomos-sim/src/execution.rs:68,172`). A rejection is evidence in this
   tree, not an absence.
3. **"Each input resolves as exactly one deterministic command batch"** is
   `RUNTIME.md` §5 R1-5's own wording. One input, one batch, always. A batch
   whose effect set is empty is still a batch.

The consequence, stated so nobody misreads it: **`play_state.tick` counts
committed batches, which is to say inputs; it is not the kernel's tick.** The
kernel's tick lives inside `play_state.kernel.state.tick` and counts committed
kernel transactions, which for the four-area route is at most three per area.
Two ticks, two meanings, one owner each. `tests/documents.rs` asserts they are
different numbers on a log that contains a refusal.

The split that keeps this honest is **shape refusal versus rule refusal**. A
malformed document, an unknown `kind`, a `direction` that is not one of four, a
field belonging to another kind, an `entity` that names nothing in the plan —
these are refused at step 1 with a `PL02##` code, produce no receipt, and do not
advance the tick, because a document that is not a `play_command@1` is not an
input. A well-formed command the rules decline is a `PL03##` refusal and a
committed batch. The two code ranges make the split visible in the receipt.

### 3.6 `interact`, and how the available interactions are enumerated

`interact {entity, action}` is the only branch that touches the kernel's
transaction machinery:

```text
CommandRequest::new(action, entity, None)
  -> nomos_sim::resolve_command(&plan, &request)        // ambiguity, declaration, argument
  -> nomos_sim::commit_transaction(&plan, state, &cmd)  // atomic; causal settlement included
  -> PersistedRuntimeState::new(&plan, committed.into_snapshot())
```

It is refused when the target entity is not within Manhattan distance 1 of the
player (`PL0307`), and it is refused with the kernel's own diagnostic mapped to
`PL0308` when `resolve_command` or `commit_transaction` declines — an
undeclared action, an ambiguous one, a transition whose source state is not the
current one. Adjacency is defined for all three binding kinds so the rule is
total: distance to a `Cell` binding is Manhattan on `(x, y)`; to a `Face`
binding, the distance to its owning cell; to a `Region` binding, the distance to
the nearest cell of the region, taken component-wise. The corpus exercises the
first two.

The `interactions` field of `presentation_state` is the authoritative
enumeration: every command legal at this state on an entity adjacent to the
player, ordered by `(entity, action)` ascending. Legal means the entity owns a
machine whose current state is the `source` of a command transition whose
`requirement` is `none`. The viewer offers the first row; `E` sends it.

**This replaces `plan.interactions[]` as gameplay, and it had to be measured
before it could be designed.** The plan's interaction edges are derived from the
committed command *logs* (`crates/nomos-render-plan/src/runs.rs:181-221`), so
they encode the order a human authored the `.commands` scripts, not a rule.
Enumerating legal commands instead is a rule — but only if it picks the same
edge the route needs. Measured over all four committed areas and all five
scenarios:

- Ignoring adjacency, "first by `(entity, action)`" disagrees with the authored
  ladder at 6 of the 12 edges — for instance north-gaol `01-baseline` offers
  `brazier_02 extinguish` before `north_gate ignite`.
- **With the Manhattan ≤ 1 filter applied at the cell the route actually stands
  on, it agrees at every edge the four-area route uses.** In each area the
  player stands on the objective gate's own cell when pressing `E`, and in each
  area every other machine-owning entity is at least 3 cells away: cistern-walk
  `sluice_gate (2,0)` versus `service_gate (6,0)` and `watch_brazier (6,2)`;
  ember-vault `vault_gate (4,0)` versus `ossuary_gate (8,0)` and
  `ember_brazier (1,1)`; ossuary-reach `bone_gate (6,0)` versus
  `reliquary_gate (1,0)` and `ossuary_brazier (7,4)`; north-gaol
  `north_gate (5,0)` versus `north_gate_02 (7,0)` and `brazier_02 (3,1)`. Only
  the gate is adjacent, and its first legal action is `ignite` at
  `01-baseline` and `unseal` at `02-breached-warded`, which is the ladder.

`tests/corpus.rs` pins that result per area and per scenario, so a content
change that made two machine-owning entities adjacent would fail a test rather
than silently change which command `E` sends.

The entities that own no machine — every water region in the corpus — never
appear in the enumeration, because the enumeration is over command transitions.
That removes `play.mjs:230`'s `entity.anchor.cell` filter, which existed only to
skip region-bound entities.

### 3.7 `cross`, and the session transition

`cross {gate}` is the exit and the transition, one command with one effect. It
is accepted when all of:

- `gate` names a plan entity whose kind is `door`;
- that entity's anchor is a `face` whose `cell` equals the player's cell;
- the entity's effective movement disposition at the current kernel state is
  `traversable`, as resolved by `nomos_sim::resolve_movement` — the exact
  condition `play.mjs:145` reads off a scenario today;
- the outcome is `playing`.

On acceptance: `counters.moves += 1`, `counters.traversal_cost += 1` — the exit
step costs 1, as `play.mjs:159` fixes it — the area's outcome becomes `escaped`,
`areas_cleared += 1`, and the session records the plan's `route.to_area` as the
area it expects next. The pursuer is not offered a step. If `route.to_area` is
`Null` the session outcome becomes `completed` and the run is over.

Arrival is a separate call, not a command, because the destination area's bytes
have to be fetched first and because arrival is not something the player does.
`session::enter(plan, semantics)` refuses (`PL0401`) unless the live area's
outcome is `escaped` and the offered plan's `area.id` equals the recorded
`route.to_area`. On acceptance it opens a fresh `play_state` for the
destination with:

- the player at the destination plan's **own** `route.entry` — owner ruling 3
  of `docs/review/presentation-source.md` §6, the reason the exiting area no
  longer names a cell inside its neighbour;
- every other actor at the destination plan's declared `actors[].cell`;
- a fresh kernel state, `SimulationState::initialize(&destination_plan)`;
- `tick` carried, not reset;
- `counters` carried, because they are cumulative;
- `pursuit.moves_since_step` reset to 0 and `outcome` reset to `playing`.

Reset is `session::start(plan, semantics)` against the collection's start area
— a new session, not a command, exactly as the issue's Scope says. It discards
the log and the receipts. Nothing in the log can express a reset, which is what
makes a recorded log replayable as one continuous run.

**One rule, two spellings.** A `move` that leaves the lattice derives the gate
from the door bound to the face of the player's cell in the direction of travel
and then calls the same `crossing()` function `cross` calls. There is one
implementation of the crossing rule and one place the exit cost is written down.
`tests/reducer.rs` asserts that the two spellings produce byte-identical
receipts apart from the `input` field. The browser only ever emits the `move`
spelling; `cross {gate}` exists for a scripted log that would rather name the
gate than depend on geometry, and for the CLI.

---

## 4. Receipts, the log, and replay

### 4.1 What the log contains

The replayable artifact is a `nomos.play_session@1` document. It carries, in
one place: the route with a `plan_digest` and a `semantics_digest` per area,
the ordered `log` of `play_command@1` inputs, the ordered `receipts`, and the
`receipt_chain_head`. It does not carry the play states — those are recomputed;
carrying them would let a replay agree with itself.

The chain is: `receipts[0].previous_receipt_hash` is 64 zeros;
`receipts[n].previous_receipt_hash == sha256(receipts[n-1].to_canonical_bytes())`;
`receipt_chain_head == sha256(receipts.last().to_canonical_bytes())`. Each
receipt also carries `play_state_hash_after` and both kernel state hashes, so a
divergence is localised to a batch and to which of the three states diverged,
rather than reported as "the final hash differs".

### 4.2 `nomos-play replay`

```text
nomos-play replay <areas-dir> --session <session.json> [--emit <path>]
```

`<areas-dir>` is a directory of the shape `gaol capture` writes:
`<areas-dir>/<area-id>/rendering-plan.json` and
`<areas-dir>/<area-id>/world/simulation.json`. The command is read-only; it
writes nothing unless `--emit` is given, and it never edits an input.

1. Decode the session. Refuse a schema mismatch (`PL0101`).
2. For each `route[]` row, read the two files, hash their bytes, and refuse
   (`PL0402`) unless both digests equal the row's. A replay against different
   content is a harness error, not a runtime difference, and it fails loudly
   rather than reporting a false divergence.
3. `session::start` at `route[0]`.
4. For each input in `log`, in order: `batch::step`. When the resulting outcome
   is `escaped` and the next route row exists, `session::enter` that area.
5. Compare, in this order, and report the **first** difference with its
   ordinal: the receipt count; then per ordinal, the receipt's canonical bytes;
   then `receipt_chain_head`; then the final `play_state`'s hash; then the final
   `kernel_state_hash` of each area.
6. Print one line and exit 0 or 1:

```text
NOMOS_PLAY_REPLAY PASS areas=4 commands=52 receipts=52 chain=<64 hex> final_kernel=<64 hex>
NOMOS_PLAY_REPLAY FAIL ordinal=17 field=receipt_bytes area=ossuary-reach
```

The stdout line is harness output, not a canonical document, and is deliberately
not spelled `name@version` — the same position `docs/review/nomos-viewer.md` §5.5
takes about the smoke receipt, and for the same reason: a canonical-looking
identity would invite it into a register it does not belong in.

### 4.3 The ten-run byte-identity test

`tests/replay.rs`, `replaying_a_committed_log_is_byte_identical_across_ten_runs`:
build a session over the four committed areas from a fixed command log; replay
it ten times in one process; assert that all ten produce identical receipt
bytes, an identical chain head, and identical final state hashes, and that the
first run's receipt vector is non-empty and its chain head is not the
all-zero hash. The non-vacuity assertions are there because `RUNTIME.md` §5 R1-1
records that exact trap — a byte-identity test that passes on empty output
proves nothing — and PR #130's
`the_same_world_and_state_produce_byte_identical_output` guards against it the
same way.

Determinism has no seed to fix and no clock to freeze. The reducer reads no
clock, draws no random number, and iterates only `BTreeMap`, `BTreeSet`, and
`keyed_array`, so run-to-run identity is a property of the code rather than of
the environment. What the ten-run test actually catches is an accidental
`HashMap`, an accidental pointer-order iteration, and an accidental
`Instant::now()`; `tests/documents.rs`'s source grep catches the third before
the test does.

A second test, `a_tampered_log_is_refused`, flips one byte of one receipt in a
recorded session and requires `replay` to report `FAIL` at that ordinal — so the
comparison is proved to be able to fail.

---

## 5. The wasm ABI

### 5.1 Feasibility, measured

Before any of the below was designed, a stub crate depending on `nomos-core`,
`nomos-projection`, and `nomos-sim` and calling `SimulationState::initialize`,
`PersistedRuntimeState::new`, `PersistedRuntimeState::from_canonical_bytes`,
`parse_canonical`, `resolve_movement`, `resolve_light`, `resolve_command`, and
`commit_transaction` was built for `wasm32-unknown-unknown` and run.

| Measurement | Result |
| --- | --- |
| Build | succeeds; no link error, no missing symbol |
| Imports the module declares | **none** — `WebAssembly.Module.imports()` is `[]`, so `WebAssembly.instantiate(bytes, {})` is the whole loader contract; no WASI, no JS glue, no shim |
| Exports | `memory`, plus the three `#[no_mangle]` functions |
| A full kernel transaction in the browser engine | runs; the probe returns the length of the committed state hash's hex form, 64 |
| Size, `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true` | **211,650 bytes** |
| Two clean builds (`cargo clean` between) | byte-identical, sha256 `a12986d8…` |
| Build-machine paths in the binary, after `--remap-path-prefix` | **0** |

Size against the profile knobs, same stub, same toolchain:

| Profile | Bytes |
| --- | --- |
| workspace `[profile.release]` as it stands (`debug = false`) | 554,732 |
| `opt-level = 3`, lto, cgu=1, strip | 391,159 |
| `opt-level = "s"`, lto, cgu=1, strip | 232,708 |
| `opt-level = "z"`, lto, cgu=1, strip | **211,650** |
| `opt-level = "z"`, lto, cgu=1, no strip | 437,863 |
| `opt-level = "z"`, no lto, cgu=1, strip | 289,346 |
| `opt-level = "z"`, lto, cgu=1, strip, `panic = "unwind"` | 211,650 — identical, because `wasm32-unknown-unknown`'s `std` is built without unwinding, so `panic = "abort"` buys nothing here and is declared for clarity rather than for bytes |

`nomos-play` itself will be larger than the stub: the stub reaches
`commit_transaction` but not the simulation decoder, the reducer, or the
document encoders. §7 records the real number when phase 2 measures it; the
stub is the floor, and the ceiling this design will accept without a finding is
400 KB.

The reason the kernel links at all is that it never touches the host:
`crates/nomos-sim/src` and `crates/nomos-projection/src` contain no `std::fs`,
`std::io`, `std::time`, `std::env`, `std::process`, `std::thread`, and no
randomness at all; `crates/nomos-core/src` confines all of it to
`package.rs`, the on-disk world-package reader, which `nomos-play` does not
call. SHA-256 is hand-written in `crates/nomos-core/src/hash/sha256.rs`, so
there is no C dependency to fail to link.

### 5.2 The exports and the memory contract

```rust
#[unsafe(no_mangle)] pub extern "C" fn nomos_play_abi_version() -> u32;
#[unsafe(no_mangle)] pub unsafe extern "C" fn nomos_play_alloc(len: usize) -> *mut u8;
#[unsafe(no_mangle)] pub unsafe extern "C" fn nomos_play_free(ptr: *mut u8, len: usize);

#[unsafe(no_mangle)] pub unsafe extern "C" fn nomos_play_start(
    plan: *const u8, plan_len: usize, semantics: *const u8, semantics_len: usize) -> u64;
#[unsafe(no_mangle)] pub unsafe extern "C" fn nomos_play_enter(
    plan: *const u8, plan_len: usize, semantics: *const u8, semantics_len: usize) -> u64;
#[unsafe(no_mangle)] pub unsafe extern "C" fn nomos_play_step(
    command: *const u8, command_len: usize) -> u64;

#[unsafe(no_mangle)] pub extern "C" fn nomos_play_presentation_state() -> u64;
#[unsafe(no_mangle)] pub extern "C" fn nomos_play_session() -> u64;
#[unsafe(no_mangle)] pub extern "C" fn nomos_play_command_log() -> u64;
#[unsafe(no_mangle)] pub extern "C" fn nomos_play_receipts() -> u64;
#[unsafe(no_mangle)] pub extern "C" fn nomos_play_last_error() -> u64;
```

**Arguments are `(ptr, len)` pairs.** The caller allocates with
`nomos_play_alloc`, writes UTF-8 canonical JSON into linear memory, and calls.
The callee copies what it needs and does not take ownership; the caller frees
with `nomos_play_free(ptr, len)`. Ownership never crosses in that direction.

**Results are one packed `u64`**, `(ptr as u64) << 32 | (len as u64)`, pointing
at a freshly allocated UTF-8 canonical JSON document in linear memory. The
**caller** now owns it and must free it with `nomos_play_free(ptr, len)`.
Returning one scalar rather than writing into an out-parameter keeps the ABI to
a single calling convention and keeps the loader to two lines of bit
arithmetic.

**Error signalling is `ptr == 0`.** A packed result of `0` means the call
failed; the caller then calls `nomos_play_last_error()`, which returns a packed
pair pointing at a **plain UTF-8 string**, not a document — `"PL0302 the target
cell is inside masonry mass `channel_buttress`"`, the same shape the kernel's
`Diagnostic` renders. Deliberately not a document: an error envelope would be a
sixth canonical identity to declare, own, and register, and an ABI failure
channel is not a contract with content. A `0` from `nomos_play_alloc` is
out-of-memory and is the one failure the loader reports as fatal rather than as
a runtime refusal.

**`nomos_play_abi_version()` returns `1`.** The loader refuses any other value
before it calls anything else, so a stale `.wasm` in a browser cache fails with
a named error instead of a trap.

**Panics.** `panic = "abort"` compiles a Rust panic to a wasm `unreachable`,
which surfaces in JS as a `RuntimeError` from the call. The loader catches it
and rethrows a `ViewerError`, so the smoke lane sees a console error and fails
rather than hanging. Everything reachable from an export returns
`Result<_, PlayError>`; the only `panic!`s left are the `expect()`s on schema-id
literals that the crate's own tests rule out, which is the same standing they
have in the kernel.

**Memory growth.** Every read of linear memory re-reads
`instance.exports.memory.buffer`, because an allocation can detach the previous
`ArrayBuffer`. The loader has no cached view.

Return shapes, one line each:

| Export | Returns |
| --- | --- |
| `nomos_play_start` / `nomos_play_enter` | `nomos.presentation_state@1` for the new tick |
| `nomos_play_step` | `nomos.presentation_state@1` for the tick the batch produced |
| `nomos_play_presentation_state` | the same, without stepping |
| `nomos_play_session` | `nomos.play_session@1`, complete |
| `nomos_play_command_log` | a bare `CanonicalValue::Array` of `play_command@1` objects |
| `nomos_play_receipts` | a bare `CanonicalValue::Array` of `play_receipt@1` objects |

The last two are windows onto the session, not second authorities: they exist so
the adapter and the smoke lane do not have to parse a session document that
carries every state to read the log. `tests/documents.rs` asserts each equals
the corresponding field of `nomos_play_session()`'s output.

### 5.3 The `unsafe` block

`src/wasm.rs` is the only module in the R1 tree that is not `forbid(unsafe_code)`.
What it does with it, exhaustively: `std::alloc::alloc` and `dealloc` with a
1-byte-aligned `Layout`; `std::slice::from_raw_parts` to read an argument;
`std::ptr::copy_nonoverlapping` to write a result. No pointer arithmetic, no
transmute, no static mut — the runtime singleton is a
`std::cell::RefCell<Option<PlaySession>>` in a `thread_local!`, which is sound
on wasm32's single thread and needs no `unsafe`. The whole module is behind
`#[cfg(target_arch = "wasm32")]`, so the native build, the CLI, and every test
compile with `unsafe` forbidden.

### 5.4 The JS loader, and `play.mjs` as an adapter

`apps/nomos-viewer/src/runtime.mjs`, about 100 lines, no dependency:

```js
const dec = new TextDecoder();
const enc = new TextEncoder();
const unpack = (packed) => [Number(packed >> 32n), Number(packed & 0xffffffffn)];

export async function loadRuntime(url, fetchImpl = fetch) {
  const bytes = await (await fetchImpl(url)).arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, {});   // no imports; measured
  const x = instance.exports;
  if (x.nomos_play_abi_version() !== 1) throw new ViewerError(CODES.RUNTIME_ABI, "...");
  const bytesOf = (p, n) => new Uint8Array(x.memory.buffer).slice(p, n === 0 ? p : p + n);
  const fail = () => { const [p, n] = unpack(x.nomos_play_last_error());
                       const text = dec.decode(bytesOf(p, n)); x.nomos_play_free(p, n);
                       throw new ViewerError(CODES.RUNTIME_REFUSED, text); };
  const take = (packed) => { const [p, n] = unpack(packed); if (p === 0) fail();
                             const out = JSON.parse(dec.decode(bytesOf(p, n)));
                             x.nomos_play_free(p, n); return out; };
  const put = (value) => { const b = enc.encode(JSON.stringify(value));
                           const p = x.nomos_play_alloc(b.length);
                           if (p === 0) throw new ViewerError(CODES.RUNTIME_MEMORY, "...");
                           new Uint8Array(x.memory.buffer).set(b, p); return [p, b.length]; };
  // start/enter/step wrap put + take + free-the-argument in a finally
  ...
}
```

`WebAssembly.instantiate(arrayBuffer)` rather than `instantiateStreaming`, so
the loader does not depend on the server sending `application/wasm`.
`smoke/server.mjs`'s extension table gains `.wasm` anyway, because the browser
lane should serve the artifact the way a real host would.

`apps/nomos-viewer/src/play.mjs` shrinks to an adapter of roughly 60 lines. It
keeps exactly one thing it has today: the key-code table, rewritten to produce a
direction name rather than a delta —

```js
export const movementKeys = Object.freeze({
  ArrowUp: "north", KeyW: "north",  ArrowDown: "south", KeyS: "south",
  ArrowLeft: "west", KeyA: "west",  ArrowRight: "east", KeyD: "east",
});
export const moveCommand = (direction) =>
  ({ schema: { name: "nomos.play_command", version: 1 }, kind: "move", direction });
export const interactCommand = (entity, action) => ({ ..., kind: "interact", entity, action });
```

— and everything else in it is deleted. §8 is the deletion table.

### 5.5 Building it

```text
cargo build -p nomos-play --target wasm32-unknown-unknown --profile wasm
```

**Not `--release`, and that is a finding, not a preference.** Cargo profiles are
workspace-global and `lto`, `panic`, and `strip` cannot be set per package
(`[profile.release.package.*]` accepts only a subset). Setting them on
`[profile.release]` would change every native release build in the workspace,
including giving native binaries `panic = "abort"`, which stops a test from
catching a panic. So the root `Cargo.toml` gains

```toml
[profile.wasm]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

which changes no existing build and is never selected by any kernel command.
§10 finding 3 records the deviation from the issue's stated command.

Reproducibility needs one flag beyond the profile. Even with `strip = true` and
`panic = "abort"`, `core::panic::Location` embeds the **source path of every
`expect()` and every slice index** in the binary. Measured on the stub: 10
distinct absolute paths beginning `/work/signed-dev/...`. That is two defects at
once — the binary is not reproducible across checkouts, and
`build.mjs`'s scan rule 6 refuses `/work/` in any staged file. The fix is a
remap, applied by the build script and recorded in the receipt:

```text
RUSTFLAGS="--remap-path-prefix=$(git rev-parse --show-toplevel)=/nomos"
```

After the remap the only absolute strings left in the binary are
`/nomos/crates/...`, `/rustc/<toolchain-hash>/library/...`, and
`/rust/deps/dlmalloc-0.2.13/src/dlmalloc.rs` — all toolchain-relative virtual
paths, none matching `build.mjs`'s `MACHINE_PATHS`, and all stable across
machines at a pinned toolchain. Measured: 0 machine paths, and two clean builds
byte-identical.

`dlmalloc` is worth naming so nobody finds it later and thinks a dependency was
smuggled in: it is `wasm32-unknown-unknown`'s allocator inside the pinned
`std`, not a Cargo dependency. `cargo tree --target wasm32-unknown-unknown` for
`nomos-play` lists exactly `nomos-core`, `nomos-projection`, `nomos-sim` and
nothing else, and `cargo xtask boundary` sees the same graph. It is not a
`RUNTIME.md` §4 addition, because §4 records "each crate or package addition"
and the toolchain's own standard library is not one; the toolchain pin in
`rust-toolchain.toml` is what governs it.

`rust-toolchain.toml` gains the target so a clean checkout can build it without
a separate `rustup target add`:

```toml
[toolchain]
channel = "1.98.0"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
profile = "minimal"
```

### 5.6 `build.mjs`, the staged tree, and the digest

`build.mjs` gains three things.

1. **A `--wasm <path>` input** and one more staged file, `nomos_play.wasm`.
   `play.mjs` stays, as the adapter, and `runtime.mjs` joins it: both
   `APP_MODULES` (`build.mjs:36`) and `EXPECTED_FILES` (`build.mjs:115`) gain
   `src/runtime.mjs`, and `EXPECTED_FILES` gains `nomos_play.wasm`.
2. **Per-area semantics.** `areas/<area-id>.simulation.json`, copied from
   `<from>/areas/<area-id>/world/simulation.json`, one per declared area, added
   to the rule 8 shape list the same way the plans are.
3. **A build receipt.** `build.mjs` today only prints a stdout line and records
   no digest of anything except the vendored renderer; there is no receipt file
   at all. It gains `--receipt <path>`, default
   `target/nomos-viewer-build/receipt.json`, written **outside `dist/`** so the
   shape rule is unaffected:

```json
{
  "receipt": "nomos-viewer-build/1",
  "generated_by": "apps/nomos-viewer/build.mjs",
  "commit": "<git rev-parse HEAD>",
  "node": "v22.x.y",
  "files": [ { "path": "nomos_play.wasm", "bytes": 0, "sha256": "…" } ],
  "wasm": { "path": "nomos_play.wasm", "bytes": 0, "sha256": "…",
            "toolchain": "1.98.0", "target": "wasm32-unknown-unknown",
            "profile": "wasm", "remap": "<repo>=/nomos" },
  "semantics": [ { "area": "north-gaol", "sha256": "…", "bytes": 6221 } ],
  "total_bytes": 0,
  "outcome": "pass"
}
```

Not spelled `name@version`, for the reason §4.2 gives.

**The scan needs two amendments, and both are bounded.**

- **Rule 4, forbidden inputs.** `PROJECTION_FILES` currently permits the string
  `simulation.json` only when preceded by `"file":"` — the plan's
  `projection_digests` idiom. A staged `areas/<id>.simulation.json` is a file,
  not a string, so the amendment is to rule 8's shape list, not rule 4's
  content check. The *content* passes rule 4 unchanged, and that is measured:
  a projection member carries `"path":"experiments/executable-gaol/areas/<id>/world.nomos"`
  inside each claim's source span, which already satisfies rule 4's `.nomos`
  exception (preceded by `"path":"`, matching `PROVENANCE_PATH`); it contains
  none of `FORBIDDEN_INPUTS`; and its canonical single-line JSON matches none of
  `SOURCE_MARKERS`, which need `schema` followed by whitespace at a line start.
  `world-ir.json`, `compiler-receipts.json`, `manifest.json`, `schemas.json`,
  and the other three projections stay refused everywhere, and a new planted
  case in `test/scan.test.mjs` proves each still is.
- **Binary files.** Rules 4, 5, and 6 read every staged file as UTF-8 and match
  regexes over it. A 200 KB binary read that way can match
  `AIza[0-9A-Za-z_-]{35}` or a `MACHINE_PATHS` pattern by coincidence, and a
  build that fails on a coin flip is worse than no check. The fix is the
  pattern rule 3 already uses for the vendored renderer: **a staged binary is
  pinned by digest, not scanned as text.** `nomos_play.wasm` is verified against
  the build receipt's sha256 and byte count and is exempted from the text rules,
  and a new rule 9, "every staged file is either text-scanned or
  digest-pinned, and nothing is neither", makes the exemption a closed set
  rather than a hole. The *text* checks that matter for a wasm binary — no
  external origin, no build-machine path — are then enforced where they belong,
  at build time, by the `--remap-path-prefix` measurement in §5.5 and a
  `strings`-equivalent assertion in `test/scan.test.mjs` over the committed
  digest.

### 5.7 The smoke lane

Three changes, all inside `apps/nomos-viewer/smoke/`.

1. **Record the command log.** `press()` (`smoke.mjs:231`) is the single choke
   point every input passes through. It gains a recorder: after each key, read
   `document.documentElement.dataset` as it already does, and at the end of the
   run read the session once via `Runtime.evaluate`. The session is exposed the
   way everything else the lane reads is exposed — as page state, not a test
   hook: `ui.mjs` writes `data-tick`, `data-outcome`, and
   `data-kernel-state-hash` onto the root element beside the counters it already
   writes, and the full session comes from one `Runtime.evaluate` of a
   `#session` `<script type="application/json">` element the app refreshes on
   every tick. That keeps `ui.test.mjs:70`'s "the readout is the page's data
   contract" claim true and adds no page global. The recorded session is written
   to `target/nomos-viewer-smoke/session.json`.
   Note: the literal string `command-log` is in `build.mjs`'s
   `FORBIDDEN_INPUTS`, so nothing staged may contain it — the field is `log`,
   the file is `session.json`, and the element id is `session`.
2. **Replay it natively and assert identity.** After the browser run, the lane
   shells `nomos-play replay target/executable-gaol/areas --session
   target/nomos-viewer-smoke/session.json` and requires exit 0. The receipt
   gains a `native_replay` block with the command, the exit status, the stdout
   line, and the compared chain head. **This is the assertion `RUNTIME.md` §5
   R1-5 and the issue both ask for**: the same command log run natively and in
   the browser yields the same final state hash — and, because the comparison is
   over the whole receipt chain, the same everything else.
3. **Stop predicting counters in JavaScript.** `smoke/route.mjs` today
   re-implements terrain cost, mass blocking, and move counting to predict
   `moves` and `cost`, which after this slice is a third implementation of rules
   the wasm runtime owns. It keeps the part that is genuinely a harness concern
   — the Dijkstra path search over bounds, masses, and actor cells, which
   decides *which cells to walk* — and stops emitting `moves` and `cost`. The
   exact four-area counters move to `tests/corpus.rs`, where they are pinned
   against the committed plans in the language that owns them. The lane keeps
   one cheap invariant that a cost regression would still break —
   `data-cost > data-moves`, which holds only if water was actually paid for —
   and gets its real strength from the replay identity in (2).

`smoke.mjs`'s pass list changes at three points: the per-key barrier keys on
`data-tick` changing rather than `data-moves`, because a refused input now
advances the tick and not the move count; the final assertion is
`data-outcome === "completed"` with `data-areas-cleared === "4"` rather than
`data-completed === "true"`; and a thirteenth failure condition is added, "the
native replay of the recorded session did not agree".

---

## 6. `nomos.rendering_plan@3`

One bump, two changes, folding issue #153 in so the fixtures are regenerated
once. Owner file stays `crates/nomos-render-plan/src/plan.rs`; the register row
replaces `@2`'s.

### 6.1 The delta

| # | `@2` | `@3` | Why |
| --- | --- | --- | --- |
| 1 | `actors[]: {assembly, cell, id}` | `actors[]: {assembly, cell, id, role}` | The declared role R1-5 needs. Resolves audit §3 rows 7 (remainder) and 21. |
| 2 | `entities[].visual_assembly` | removed | Renderer-catalog data. `crates/nomos-render-plan/src/catalog.rs:60-63` says so itself: "This is the last place in the tree where a visual assembly name or a material family is assigned to an entity kind outside the renderer catalog… the correct change is to move these two out." Issue #153. |
| 3 | `entities[].material_family` | removed | Same table, same reason; the two move together. |

Nothing else changes. `scenarios[]` and `interactions[]` **stay in the plan**,
unchanged in shape and in bytes, and their standing changes rather than their
content: they are the SVG capture ladder and the evidence that the compiler
consumed committed run bundles, and they are no longer gameplay. §7 rows 14, 15,
and 23 record that.

`presentation_source@1` gains the matching field:
`actors[].role`, text, `player | pursuer`, exact-field-set
`["assembly", "cell", "id", "role"]` at
`crates/nomos-render-plan/src/source.rs:549`, refused with `RP0202`. The
`REQUIRED_ACTORS` constraint at `source.rs:89-95` — which the R1-3 record
explicitly parked for this slice — is replaced by a role constraint: exactly one
actor with `role: "player"`, at most one with `role: "pursuer"`, ids unique and
free. `player` and `gaoler` stay the authored ids in the four committed areas
because renaming them is content churn with no evidence attached, but nothing
in the compiler or the runtime depends on the strings any more, and
`tests/source.rs` proves it by renaming both in a fixture and asserting the
plan compiles and the runtime plays it.

The source schema does **not** bump. Adding a required field to an input schema
is a breaking change for the four authored files, and all four are edited in
this commit; `nomos.presentation_source@1` has one reader, one writer, and no
persisted consumer outside this repository, and `RP0104` already refuses a
version mismatch. §10 finding 7 flags this for the owner: the conservative
alternative is `presentation_source@2`, and it costs one register row.

### 6.2 No drawn field changes

The claim to prove is that the SVG frames and the contact sheet are
byte-identical at `@3`. Field by field, over
`experiments/executable-gaol/src/render-core.mjs`, which is the only SVG
generator:

- `entities[].material_family` is **read by no renderer at all**. Its only
  consumers are the collection grammar fingerprint
  (`build-collection.mjs:35`), the viewer's equality constraint
  (`plan.mjs:260-262`), and fixtures. Dropping it changes no drawn byte.
- `entities[].visual_assembly` is read on the drawn path in exactly one place,
  `renderer-catalog.mjs:100`, as the key into `SOCKETS` when placing the cyan
  crescent. `SOCKETS` is re-keyed by entity `kind` — one door kind, one `ward`
  socket, the same `{x: 5, y: 0, z: 17}` offset — so the crescent resolves to
  the same coordinates and the same bytes. The equivalent viewer sites,
  `catalog.mjs:242` and `render.mjs:546`, are re-keyed the same way;
  `plan.mjs:288-289` already rebuilds both fields from the catalog and discards
  the plan's strings, so the viewer's internal entity object is unchanged.
- `actors[].role` is read by no renderer. `render-core.mjs:205` selects the
  player glyph with `actor.id === "player"` and that line is **not changed** in
  this slice: switching it to `role` would be correct and would also be a drawn
  edit, so it is deferred to the change that regenerates frames for another
  reason. The viewer's `render.mjs:432` already dispatches on
  `actor.assembly` and holds no actor identifier at all.

**One SVG artifact does change bytes, and it is not a frame.**
`render-core.mjs:234` prints the literal `nomos.rendering_plan@2` into the
forensic overlay, so each area's `forensic.svg` changes. `capture.mjs:28` builds
the contact sheet from `frames.slice(0, 4)`, which excludes the forensic frame,
and `verify.mjs:66` digests only the plan and the contact sheet. So the
**verified** digests are: contact sheet unchanged, plan changed. That is exactly
the accounting `experiments/executable-gaol/CAPTURE.md:64-87` records for the
`@1 → @2` bump, and the `@3` receipt follows the same template — including its
proof form: substituting `@2` back into the version string reproduces each old
forensic digest exactly.

### 6.3 What has to change with it

| Site | Change |
| --- | --- |
| `crates/nomos-render-plan/src/plan.rs:56` | the one authoritative literal, `2` → `3` |
| `crates/nomos-render-plan/src/plan.rs:152-160` | delete the two emitted fields |
| `crates/nomos-render-plan/src/plan.rs:392-408` | emit `actors[].role` |
| `crates/nomos-render-plan/src/catalog.rs:88-108` | delete `visual_assembly()` and `material_family()` |
| `crates/nomos-render-plan/src/source.rs:89-95, 542-572` | role decoding; `REQUIRED_ACTORS` retired |
| `apps/nomos-viewer/src/plan.mjs:29, 231-232, 253-264, 288-289, 493-501, 623-628, 722` | schema literal; entity field set; drop the equality check; keep the catalog re-derivation; `role` on actors; socket lookup by kind; collection cross-check |
| `apps/nomos-viewer/src/catalog.mjs:229-262` | `SOCKETS` re-keyed by kind |
| `apps/nomos-viewer/src/render.mjs:546` | `ENTITY_BUILDERS` keyed by kind |
| `apps/nomos-viewer/test/catalog.test.mjs:100-132` | `the_catalog_knows_every_assembly_the_compiler_can_emit` is **deleted**: it regex-parses two Rust functions that no longer exist, and with the mapping gone there is nothing left to drift |
| `experiments/executable-gaol/src/verify.mjs:20, 52, 57-58` | schema literal; add `role ∈ {player, pursuer}` membership; socket lookup by kind |
| `experiments/executable-gaol/src/renderer-catalog.mjs:100` | `SOCKETS` by kind |
| `experiments/executable-gaol/src/render-core.mjs:234` | version string in the forensic overlay |
| `experiments/executable-gaol/src/build-collection.mjs:32, 35, 88` | `visual_grammar.entity_assemblies` loses two columns and becomes the kind list; the grammar digest changes; collection identity → `nomos.experiment.area_collection@3` |
| `experiments/executable-gaol/src/area-collection.test.mjs:24, 90, 123` | grammar fingerprint; socket lookup; the player-cell assertion keyed by role |
| Fixtures | four `rendering-plan.example.json`, `area-collection.example.json`, four `presentation.json`, regenerated and committed |
| Register | `nomos.rendering_plan@3` replaces `@2`'s row |
| Docs | `README.md`, `RUNTIME.md:137`, `docs/workspace.md:39`, `docs/HANDOFF.md`, `apps/nomos-viewer/README.md`, `experiments/executable-gaol/{README,CAPTURE}.md` |

The area collection is quarantined tooling until issue #152 promotes it; bumping
it to `@3` here is unavoidable because its `visual_grammar` carries both the
plan version and the assembly rows. That is one more reason to land #152 next,
and this record does not fold it in — it is a separate identity with its own
route-chain refusals to test.

---

## 7. The five deferred audit rows

`docs/review/presentation-source.md` §4.3 deferred five rows of the ownership
audit's §3 to R1-5. Dispositions:

| Row | Convention | Disposition | Reason |
| --- | --- | --- | --- |
| 7 (remainder) | the required literal actor ids `player` and `gaoler` | **resolved** | `presentation_source@1` and `rendering_plan@3` carry `actors[].role`; `REQUIRED_ACTORS` at `source.rs:95` is deleted and replaced by a role constraint. The ids stay in the four authored files as content, and a test renames both and plays the area, which is the proof that no code depends on them. |
| 14 | `scenario.label` regex-stripped from a directory name | **resolved by removal from the gameplay path; the convention stays where it belongs** | The deferral said a scenario's label attaches to "the ordered collection R1-5 introduces". It does not, and the reason is worth stating: R1-5's ordered collection is the *actors*, and a scenario is not a runtime object at all any more. Scenarios are captured evidence produced by `gaol capture` from a directory of `.commands` scripts, and the label is derived by the capture tooling from the script's file name — tooling deriving a label for its own output from its own input file, which is the "tooling only" owner category, not a gameplay convention. It is recorded as resolved in that category rather than carried forward as a debt against a slice that cannot discharge it. |
| 15 | `interactions[]` reconstructed by diffing command logs, O(n²) | **resolved for gameplay; the reconstruction stays as evidence** | Gameplay no longer reads `plan.interactions[]`. What the player can do is enumerated authoritatively from the projection's command transitions at the live kernel state (§3.6), which is a rule rather than a reconstruction. `runs.rs:181-221` and its doc comment — which names R1-5 as the owner of "the declared successor pointer that would replace it" — stay, retitled: the edges are now the evidence that the plan's scenarios came from committed command logs, and they are what the SVG ladder walks. The comment is amended to say so instead of promising a successor pointer that this design decided not to introduce. |
| 21 | `actor.id === "player"` as the only role signal | **resolved** | Same field as row 7. Every Rust and JavaScript site that branched on the id is listed in §6.3. One site is knowingly left: `render-core.mjs:205`, because changing it is a drawn edit and §6.2 needs the frames byte-identical; it is filed with the next change that regenerates frames. |
| 23 | `plan.scenarios[0]` treated as the default by array position | **resolved** | There is no default scenario any more. The runtime's initial state is `SimulationState::initialize(&plan)` at tick 0, which is a kernel fact and not an array index. Measured: for north-gaol that state's hash is `9d81ddaf…`, byte-equal to the run bundles' `initial-state.json` and to the `01-baseline` scenario's `state_hash` in the committed plan — so the position-0 convention was *describing* the kernel's initial state, and the kernel now supplies it directly. `plan.mjs:915-919`'s `initialScenario` survives as a forensic control for the number keys, where "the unique lowest tick" is a stated rule and not an array position. |

That closes all five. Combined with `docs/review/presentation-source.md` §4.2–4.4,
the ownership audit's 61 flagged rows are now 60 resolved and 1 deferred (§4.3
item 26's remainder, held by R1-4).

---

## 8. The viewer

### 8.1 What it loses

`apps/nomos-viewer/src/play.mjs`, 316 lines, keeps 2 exports and loses 11.

| Export | Fate |
| --- | --- |
| `movementKeys` | **kept**, remapped to direction names (§5.4) |
| `words`, `identifier` | **moved** to `ui.mjs`; they are prose helpers, not play state |
| `isHunting` | **deleted**; `presentation_state.pursuit.hunting` |
| `createPlayState` | **deleted**; `nomos_play_start` |
| `enterArea` | **deleted**; `nomos_play_enter` |
| `completeRun` | **deleted**; the session's `completed` outcome |
| `completionSummary` | **moved** to `ui.mjs`; it is a string built from counters |
| `terrainAt` | **deleted**; `presentation_state.movement` |
| `masonryAt` | **deleted**; occupancy is `nomos-play`'s (`smoke/route.mjs` keeps its own for path search, which is a harness concern) |
| `attemptMove` | **deleted**; `nomos_play_step` |
| `advanceGaoler` | **deleted**; the pursuit rule |
| `interactionAt` | **deleted**; `presentation_state.interactions` |
| `guidanceFor` | **moved** to `ui.mjs`, rewritten against `presentation_state` |
| `attemptInteraction` | **deleted**; `nomos_play_step` |

### 8.2 What it keeps

- **Interpolation.** The tween lives in `ui.mjs:225-258` — `TWEEN_MS_PER_COST`,
  the cubic ease-out, the sine hop — and is unchanged. It runs between two
  authoritative endpoints the runtime produced and feeds `render.mjs` through
  `presentation.actorPositions` (`ui.mjs:146`, `render.mjs:561-566`), which is
  already the only fractional value in the app and already presentation-only.
  `RUNTIME.md` §5 R1-5's "interpolate inside authoritative state" prohibition is
  satisfied by construction: authoritative state is now in another language, in
  another memory, behind an integer-only document schema.
- **Rendering.** `render.mjs` keeps its geometry, materials, palette, camera,
  and shadow work untouched. Two changes only: `ENTITY_BUILDERS` keys on `kind`
  (§6.3), and `present()` takes a presentation state where it took a plan
  scenario. That second one is nearly free, because
  `machine_states`/`movement`/`effective_light` are spelled identically in the
  two documents and `plan.mjs`'s `doorState`, `wardSealed`, and `lightOf`
  accessors read either without modification (§2.5).
- **UI.** `ui.mjs` keeps the DOM wiring, the palette custom properties, the
  scenario and area controls, the look-profile toggle, and the readout. It gains
  the guidance builders, `data-tick`, `data-outcome`, `data-kernel-state-hash`,
  and the `#session` JSON element; it loses `data-completed` and `data-caught`,
  which `data-outcome` subsumes. The number keys 1–5 stay as a **forensic**
  control that renders a captured plan scenario without touching authoritative
  state, which is what they already are.

### 8.3 Test migration

`apps/nomos-viewer/test/play.test.mjs`, 22 tests. Nineteen become Rust; three
stay as adapter tests; the file shrinks to roughly 60 lines.

| # | JS test | Becomes |
| --- | --- | --- |
| 1 | `movement keys map to lattice deltas` | **stays** — the key table is the adapter's, retargeted to direction names, plus a Rust `tests/documents.rs` case pinning the four direction deltas against the same table |
| 2 | `a plan without an actor is refused` | Rust `tests/documents.rs` — a plan with no `player` role is `PL0103` |
| 3 | `water uses the projected traversal cost` | Rust `tests/reducer.rs` — and strengthened: the cost now comes from `resolve_movement` at the live state, not a captured scenario |
| 4 | `a mass blocks the cells it covers` | Rust `tests/reducer.rs`, half-open rectangle |
| 5 | `the baseline gate refuses an exit` | Rust `tests/reducer.rs`, `PL0306` with the resolver's reasons in the receipt |
| 6 | `the breached and unsealed gate permits an exit` | Rust `tests/reducer.rs` |
| 7 | `the unchanged second door remains blocked` | Rust `tests/reducer.rs` |
| 8 | `a move that leaves the lattice with no door finds masonry` | Rust `tests/reducer.rs` |
| 9 | `an exit uses the door's declared direction` | Rust `tests/reducer.rs`, all four faces |
| 10 | `nearby interactions follow verified state hashes` | Rust `tests/reducer.rs` — inverted: the interaction is now *enumerated*, and the test asserts the resulting kernel state hash equals the plan scenario's, which is the stronger claim |
| 11 | `interaction range does not invent remote actions` | Rust `tests/reducer.rs`, `PL0307` |
| 12 | `the brazier interaction follows the verified extinguish receipt` | Rust `tests/corpus.rs` |
| 13 | `the gaoler hunts only when the pursuit light is out` | Rust `tests/pursuit.rs` |
| 14 | `the gaoler stays dormant while the light is lit` | Rust `tests/pursuit.rs` |
| 15 | `the dark gaoler advances every second successful move` | Rust `tests/pursuit.rs` |
| 16 | `the dark gaoler can catch and stop the player` | Rust `tests/pursuit.rs`, plus a receipt assertion that later commands are refused with `PL0301` |
| 17 | `pursuit advances only for a scenario that is hunting` | Rust `tests/pursuit.rs` |
| 18 | `guidance derives the objective and prompt from plan data` | **stays**, moved to `ui.test.mjs`, rewritten against a `presentation_state` fixture |
| 19 | `no identifier is re-cased into prose` | **stays**, moved to `ui.test.mjs` |
| 20 | `arrival uses the destination's own entry cell` | Rust `tests/session.rs` |
| 21 | `completion reports cumulative run state` | Rust `tests/session.rs` for the counters; the summary string stays in `ui.test.mjs` |
| 22 | `the unseal-and-escape route remains winnable across both areas` | Rust `tests/session.rs` |

New JavaScript tests, `test/runtime.test.mjs`: the loader instantiates the
committed `.wasm` under Node, refuses an ABI version mismatch, round-trips a
document through `alloc`/`free` without leaking, surfaces a `PL03##` refusal as
a `ViewerError`, and — the one that matters — steps the four-area route in Node
and asserts the session it produces replays clean through `nomos-play replay`.
That makes the browser lane's identity assertion reproducible without Chrome.

### 8.4 The four-area route, measured today

Run against the committed plans through `smoke/route.mjs` before any change, so
the phase-2 numbers have a baseline:

```text
52 keys · 44 moves · 60 traversal cost · "4 areas · 44 moves · 60 traversal cost"
cistern-walk   ↑↑↑←←←←←←↑ E E →↑    12 moves  16 cost   7,4 → … → 2,0 → out
ember-vault    ←←←←←↑↑↑→↑↑ E E →↑   25 moves  31 cost   7,5 → … → 4,0 → out
ossuary-reach  ↑↑→↑↑→→→↑ E E →↑     36 moves  48 cost   1,5 → … → 6,0 → out
north-gaol     ↑↑↑↑→→ E E →↑        44 moves  60 cost   2,4 → … → 5,0 → out
```

No leg's path enters the pursuer's cell — `4,3`, `1,3`, `7,3`, `5,3`
respectively — which is what makes §3.2 rule 4 free. Note that
`docs/review/nomos-viewer.md` §5.3's prose says "the lane dispatches 60 keys"
while its own table sums to 52; the table is right and the prose is a slip,
corrected in passing since this slice edits the lane.

---

## 9. Workspace and contract bookkeeping

### 9.1 `cargo xtask boundary`

`R1_CRATES` at `xtask/src/boundary.rs:64` becomes
`["nomos-play", "nomos-render-plan"]` and the array type widens to `[&str; 2]`.
The report line becomes `r1 members 2`. The permitted-edge table gains
`nomos-play -> {nomos-core, nomos-projection, nomos-sim}`; the dev edge to
`nomos-compiler` is declared the way `nomos-render-plan`'s dev edges are. A
planted-violation case in `xtask/src/planted.rs` proves the checker refuses
`nomos-play -> nomos-schema`.

### 9.2 `RUNTIME.md`

- §3, **declared R1 members**: a `nomos-play` entry naming its edges, its five
  canonical identities, its dev-dependency edge and what it is for, and the
  fact that it is the first workspace member built for a second target.
- §3, **R1 surface added to kernel crates**: no new row. This design touches no
  kernel crate — subject to §10 finding 1, which is the one place that could
  change.
- §5 R1-5: no wording change is proposed here. §10 finding 2 raises the one
  clause that cannot be satisfied as written and asks for an owner ruling
  rather than reinterpreting it.
- §6: the proof block gains `cargo build -p nomos-play --target
  wasm32-unknown-unknown --profile wasm` and `nomos-play replay`.
- §7: three rows to record, with the runner that produced them.

### 9.3 §7 budget rows

| Field | Unit | How measured | Value |
| --- | --- | --- | --- |
| Play runtime wasm size | bytes | `cargo build -p nomos-play --target wasm32-unknown-unknown --profile wasm`, `stat -c%s` | phase 2; the stub floor is 211,650 |
| Public artifact size | bytes | `apps/nomos-viewer/build.mjs` | phase 2; today 894,174, plus the wasm and four projection members totalling 24,956 bytes (6,250 + 6,217 + 6,221 + 6,268) |
| Replay throughput | commands/s | `nomos-play replay` over the recorded four-area session | phase 2 — this fills a row §7 currently records as "not measured" |

### 9.4 The register

Five new rows in `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`, plus `@3` replacing
`@2`'s row:

| Identity | Owner | Owner file |
| --- | --- | --- |
| `nomos.play_state@1` | `nomos-play` | `crates/nomos-play/src/state.rs` |
| `nomos.play_command@1` | `nomos-play` | `crates/nomos-play/src/command.rs` |
| `nomos.play_receipt@1` | `nomos-play` | `crates/nomos-play/src/receipt.rs` |
| `nomos.play_session@1` | `nomos-play` | `crates/nomos-play/src/session.rs` |
| `nomos.presentation_state@1` | `nomos-play` | `crates/nomos-play/src/presentation.rs` |
| `nomos.rendering_plan@3` | `nomos-render-plan` | `crates/nomos-render-plan/src/plan.rs` |

Six identities where the issue's acceptance says "four … five if all are
emitted". All five play identities are emitted, and the sixth is the plan bump
the same slice makes. `docs/evaluation/r1-schema-ownership.sh` passes on the
head of the commit that adds them, per that document's own rule.

The wasm ABI declares no identity: the error channel is a diagnostic string and
the two array exports are bare `CanonicalValue::Array`s whose elements carry
their own identity (§5.2). That is deliberate — an ABI envelope would be a
seventh row for a transport concern.

### 9.5 `rust-toolchain.toml`

Gains `targets = ["wasm32-unknown-unknown"]` (§5.5). The channel pin is
unchanged, so no Gate K determinism receipt moves.

---

## 10. Findings against the issue

Seven findings. Three need an owner ruling before phase 2 starts; four are
recorded corrections this design already makes.

### Finding 1 — nothing in the tree can turn a simulation projection back into a `SimulationPlan`, and the issue's Scope has no room for the code that would — NEEDS A RULING

This is the load-bearing one.

`nomos_sim::commit_transaction`, `resolve_command`, `resolve_movement`,
`resolve_light`, `SimulationState::initialize`, and
`PersistedRuntimeState::from_canonical_bytes` all take
`&nomos_projection::SimulationPlan`. There are exactly three ways to obtain one
(evidence below), and in the browser none of them is available under the issue's
Scope as written:

- **`SimulationPlan::new(...).with_entities(...).with_movement_resolver(...).with_light_resolver(...)`**
  — `crates/nomos-projection/src/simulation.rs:443-490`. Construction from typed
  parts. There is **no `SimulationPlan::from_canonical_bytes`**; the type has
  `to_canonical_bytes` (`simulation.rs:531`) and no inverse anywhere in the
  workspace.
- **`nomos_compiler::compile_simulation_plan(&StableWorldIr)`** —
  `crates/nomos-compiler/src/lib.rs:141`, reached from
  `open_compiled_package` and from `rehydrate_members`
  (`crates/nomos-compiler/src/opened.rs:170,192`). The compiler does not
  *decode* `simulation.json`; it **recompiles** the plan from `world-ir.json` and
  compares the bytes. `RUNTIME.md` §3 forbids "an R1 crate or the viewer parsing
  `.nomos` source, **Canonical World IR**, or compiler receipts", so this path is
  closed to `nomos-play` by name.
- **A decoder written for the purpose.** None exists.

So the browser cannot run a kernel transaction without one being written. The
issue's Scope names three dependencies and no decoder, and its `init(plan, area)`
signature suggests the rendering plan alone suffices — it does not: the plan
carries no machine, no transition, no causal edge, and no resolver.

**This design's answer: write the decoder in `crates/nomos-play/src/semantics.rs`,
keeping the kernel untouched exactly as the epoch decision says, and bound it
with three independent proofs.**

1. **Exact re-encode.** The decoder refuses (`PL0501`) unless
   `decoded.to_canonical_bytes() == input_bytes`. Since the encoder is total
   over the type, byte-identity means the decode recovered exactly the plan that
   produced those bytes. This is the kernel's own discipline for its own
   decoders, twice: `state_persistence.rs:153` and `:79`.
2. **The kernel enforces it at runtime, for free.** `PersistedRuntimeState`
   carries `runtime_semantics_digest = Sha256Digest::of_bytes(&plan.to_canonical_bytes())`
   (`state_persistence.rs:245`), and `from_canonical_bytes` refuses a mismatch
   with `EK0813`, "persisted state belongs to different simulation semantics"
   (`:62-68`). **Measured**: for north-gaol,
   `sha256(world/simulation.json)` is
   `ed4eab528fbb6a289f883f3dd80fdb98e4ee9e16e3735efa5cea6863be75eb04`, and the
   `runtime_semantics_digest` in every run bundle's `initial-state.json` is the
   same 64 characters. `simulation.json`'s bytes *are*
   `SimulationPlan::to_canonical_bytes()` verbatim, with no trailing newline. A
   mis-decoded plan therefore cannot be paired with a kernel-produced state at
   all — the kernel refuses it.
3. **Equivalence with the compiler.** `tests/semantics.rs`, over all four
   committed areas: decode `world/simulation.json`, open the same package with
   `nomos_compiler::open_compiled_package`, and assert the two
   `SimulationPlan`s are `==`. That is what the dev-dependency edge is for, and
   it is the same shape as `nomos-render-plan`'s issue #132 divergence fixture.

Cost: about 540 lines in `nomos-play`. Every constructor it needs is already
public — `MachineDefinition::new`, `CommandTransition::new`, `EventHandler::new`,
`CausalEdge::new`, `ProjectedEntity::new`, `MovementSubject::new`,
`MovementResolverPlan::new`, `MovementClaim::blocker`/`traversal_cost`,
`LightClaim::new`, `LightSubject::new`, `LightResolverPlan::new`,
`SourceSpan::new`, `SourcePath::new` — so no kernel change is needed to make it
writable.

**The alternative, and why it was not chosen.** A row in `RUNTIME.md` §3's
kernel-surface table: `SimulationPlan::from_canonical_bytes` in
`nomos-projection`, beside the encoder it inverts, ~450 lines, one round-trip
test, no Gate K command/artifact/hash/diagnostic changed. It is arguably the
better home — encoder and decoder in one file cannot drift — and §3 explicitly
authorises exactly this kind of addition. It was not chosen because issue #154's
epoch decision says "The kernel crates are untouched", and because it changes
the kernel's trust model in a way worth stating out loud: today the *only* way
to obtain executable semantics is to recompile them from verified World IR and
check the bytes, and a public projection decoder would make a projection
loadable as input without its World IR ever being seen.

**What the owner needs to rule.** Whether the decoder lives in `nomos-play`
(this design) or in `nomos-projection` as a §3 surface row. Either way, one
non-claim goes in the record: **the browser does not verify a world package.** It
replays the semantics of a package that was compiled and verified natively at
build time, whose bytes `build.mjs` digests and whose digest the recorded
session carries, and whose replay is re-verified natively by the smoke lane.

### Finding 2 — `RUNTIME.md` §5 R1-5 requires "the rendering-plan digests are unchanged", and adding `actors[].role` necessarily changes them — NEEDS A RULING

§5 R1-5's last acceptance bullet reads: "the four-area route, interactions,
water cost, capture, and reset remain green, **and the rendering-plan digests
are unchanged**." Its evidence line asks for "the rendering-plan digest
comparison".

Issue #154's Scope requires `rendering_plan@3` carrying `actors[].role`. Any
new field changes every plan's canonical bytes and therefore every plan digest.
The two cannot both hold.

Read in context the clause almost certainly means "the drawn output does not
change" — it sits beside route, water cost, and capture, all behavioural, and
R1-3 set the precedent when it moved every plan digest at the `@1 → @2` bump and
proved the *contact sheet* byte-identical instead
(`experiments/executable-gaol/CAPTURE.md:64-87`). §6.2 above is that proof for
`@3`, and it holds: frames and contact sheet byte-identical, plan digests moved,
forensic overlay moved by one version string.

But `AGENTS.md` forbids silently reinterpreting the contract, so this is not
reinterpreted. **The owner needs to either (a) rule that the clause means the
drawn-artifact digests, recording the ruling here, or (b) authorise a §8
contract repair amending §5 R1-5's wording and raising the R1 contract
revision.** Phase 2 does not start until one of the two exists, because the
alternative — dropping `actors[].role` — removes the field audit rows 7 and 21
are waiting on and puts the role back in a literal id.

### Finding 3 — the stated build command cannot carry the flags the build needs — CORRECTED HERE

The issue names `cargo build -p nomos-play --target wasm32-unknown-unknown
--release`. Cargo profiles are workspace-global, and `lto`, `panic`, and `strip`
have no per-package override, so putting them on `[profile.release]` would
change every native release build in the workspace and give native binaries
`panic = "abort"`. §5.5 uses `--profile wasm` instead, which changes no existing
build. Measured cost of not doing it: 554,732 bytes instead of 211,650, a 2.6×
artifact.

Related and also corrected: `--remap-path-prefix` is not optional. Without it
the binary embeds 10 absolute build-machine paths from panic-location metadata
even with `strip = true`, which breaks reproducibility across checkouts and
trips `build.mjs`'s scan rule 6. Both measured in §5.5.

### Finding 4 — `init(plan, area)` is not a sufficient signature, and the export set is larger than five — CORRECTED HERE

`init` needs the simulation projection's bytes as well as the plan's (finding 1),
and an `area` argument is redundant because the plan names its own area. The
export set in §5.2 is a superset of the issue's five: `alloc` and `free` are
forced by the memory contract, `enter` is the session transition (which cannot
be a command because the destination's bytes must be fetched first), `session`
is what the smoke lane records, `last_error` is the error channel, and
`abi_version` is what stops a stale cached `.wasm` from trapping. Eleven
exports, one packed-`u64` calling convention, one `unsafe` module.

### Finding 5 — interpolation is in `ui.mjs`, not `render.mjs` — CORRECTED HERE

The issue says "interpolation stays in `render.mjs`, presentation-only".
Measured: the tween is `ui.mjs:225-258` and `render.mjs` only consumes the
already-interpolated `presentation.actorPositions` at `:561-566`. The substance
is unaffected — interpolation is presentation-only, between authoritative
endpoints, and stays exactly where it is — but the file name in the issue is
wrong and §8.2 records the right one. `render.mjs`'s own time-dependence is the
brazier flicker at `:600-605`, a function of `clock.getElapsedTime()` and of
nothing authoritative.

### Finding 6 — the pursuer ignores occupancy, and the issue's occupancy sentence reads as though it should not — CORRECTED HERE, WITH A FOLLOW-UP

The issue lists occupancy sources as "masses from the plan; kernel effective
movement at the embedded state; other actors" without saying whose steps consult
them. `play.mjs:198-220`'s gaoler consults none of them: it walks through
masonry and through water at no cost. §3.2 applies occupancy to the player and
§3.3 states in the rule itself that the pursuer consults nothing, because R1-5's
job is to move the authority and not to change the game. Making the pursuer
respect occupancy would change capture outcomes on any log that reaches a mass,
which is a gameplay change needing its own evidence; it is filed as a follow-up
issue rather than folded in.

The one occupancy source that does change behaviour is "other actors" blocking
the *player* (§3.2 rule 4). Measured free against the committed corpus: the
solved route never enters the pursuer's cell in any of the four areas (§8.4).

### Finding 7 — `presentation_source@1` gains a required field without a version bump — NEEDS A RULING

`actors[].role` is a required field on an input schema whose version does not
change. Every one of its four authored files is edited in the same commit, its
only reader is `crates/nomos-render-plan/src/source.rs`, it is persisted
nowhere, and `RP0104` already refuses a version mismatch — so nothing outside
this repository can be handed a `@1` file that no longer parses. The
conservative alternative is `nomos.presentation_source@2`: one register row, one
literal, four `schema` lines in content, and a strictly honest version history.

This design proposes staying at `@1` and records the argument on both sides
rather than deciding for the owner, because R1-3's acceptance criterion — "the
accepted source is versioned, and a version mismatch is refused with a stable
diagnostic" — is about refusal rather than about incrementing, and because a
version number that moves for every field is a version number nobody reads.

### Nothing in the issue is impossible

Subject to finding 1's ruling on where the decoder lives and finding 2's ruling
on the digest clause, every part of issue #154's Scope and Acceptance is
reachable, and the two things most likely to have been impossible were measured
first rather than assumed: the kernel links and runs on `wasm32-unknown-unknown`
with zero imports and a 211,650-byte floor (§5.1), and the authoritative
interaction enumeration reproduces the authored ladder at every edge the
four-area route uses (§3.6).
