---
title: The executable semantic kernel
status: Proposed contract revision 3; Gate K implementation through SW-C
gate: K
contract_revision: 2
decision_record: docs/decisions/0001-contract-repair.md
proposed_contract_revision: 3
proposed_supersedes_contract_revision: 2
proposed_decision_record: docs/decisions/0003-contract-profile-closure.md
---

# The executable semantic kernel

Gate K is the first executable artifact after the thesis. It proves the semantic
machine without graphics, networking, audio playback, hot reload, or an asset
pipeline. If the kernel is ugly, a renderer would only conceal the corpse under
some lovely palette quantization.

Gate K may happen before the visual target pack because it is cheap and
renderer-free. It does **not** authorize renderer work. Gate 0 remains mandatory
before any rendering architecture, visual primitive catalog, or Gate 1
implementation begins.

These criteria are fixed before code so code cannot silently redefine them.
They may be corrected only through the amendment process in `AGENTS.md` and a
new owner-authorized decision record. Contract repair is allowed; weakening a
criterion because an implementation failed it is not.

## 1. Exact base fixture

One source file describes exactly three world primitive instances in the base
fixture:

```text
Room contents:
  north_gate       primitive/iron_barred_door
  flooded_section  primitive/shallow_water_region
  brazier_02       primitive/extinguishable_light

Catalog values:
  credential/gaoler_key
```

These are three primitive **kinds** and three instances in the base fixture.
The formal cold-author evaluation operates on an isolated copy and may add one
second `primitive/iron_barred_door` instance. That produces four instances while
preserving the same three approved primitive kinds; it does not expand the
catalog or Gate K's semantic scope.

`credential/gaoler_key` is a catalog credential value, not a world entity and
not a fourth primitive. References to catalog values are resolved by their own
symbol table and cannot satisfy entity references.

### `north_gate`

The door is lockable, breakable, warded, and burnable. Its namespace-local
machines are independent:

```text
access:       locked | closed | open
integrity:    intact | damaged | destroyed
ward:         sealed | unsealed
combustion:   cold | burning | spent
```

Initial state:

```text
access       = locked
integrity    = intact
ward         = sealed
combustion   = cold
credential   = credential/gaoler_key
```

The ward supplies the second independent blocking claim used by the composition
test; no magical-seal primitive exists.

Required derived behavior:

```text
portal_open = access == open OR integrity == destroyed

movement_blockers =
  access_or_integrity_blocker when NOT portal_open
  ward_blocker                when ward == sealed
```

Opening the door while the ward remains sealed changes `access` but does not
change the effective ground movement disposition: the passage remains blocked
and the explanation names the ward as the surviving reason.

Required causal interaction:

```text
combustion.on_enter(burning)
  -> integrity.apply_damage(channel = fire, amount = 2)
```

For Gate K this interaction fires exactly once on entry into `burning`. Recurring
pulse or scheduler semantics are outside Gate K.

### `flooded_section`

The water region is static in Gate K. It has a lattice region binding and
contributes a ground traversal cost of `3` to otherwise traversable cells. It
must appear in simulation, navigation, persistence metadata where applicable,
and diagnostics. It may not be parsed and then ignored.

### `brazier_02`

The light has one namespace-local machine:

```text
emission: lit | extinguished
```

Initial state is `lit`. `extinguish` transitions to `extinguished`, removes the
effective light-emission fact, updates persistence and diagnostics projections,
and produces a causal receipt.

## 2. Compile-time and command-time phases

The phrase “capability resolution” is deliberately split into two operations.

### Compile time: claim and resolver preparation

```text
parse source
resolve symbolic names in typed symbol tables
expand approved primitives
compile namespace-local machines
compile typed interaction edges
compile capability claim templates
compile per-capability composition laws
compile cross-capability coherence rules
validate fact ownership and cross-references
emit Canonical World IR plus resolver plan
project versioned static subsystem artifacts
```

The compiler may precompute affected facts, candidate consequences, dependency
sets, and resolver plans. It may not claim that final subsystem deltas are known
from a local transition alone.

### Command time: effective-fact resolution

Every accepted external command resolves as one deterministic transaction:

```text
validate command against current state
apply the machine-local transition
emit typed causal events
settle declared interactions in fixed phase order
resolve effective capability facts from all active claims
apply cross-capability coherence rules
compare before/after effective facts
derive subsystem deltas
commit runtime state atomically
write state, command log, causal receipt, and state hash
publish externally visible events only after commit
```

A machine never writes another machine's state. It emits a typed event; the
target machine owns the resulting transition. Undeclared cross-machine writes
fail. Interaction cycles fail unless a future contract explicitly defines a
fixed-point rule; Gate K defines none.

## 3. Capability composition and cross-capability coherence

Per-capability composition laws are necessary but insufficient. Gate K uses the
following relevant laws:

```text
Blocks<movement_channel>   = any_active_claim
TraversalCost<mode>        = maximum_applicable_cost
EmitsLight                 = union
Authority                  = exactly_one
Persisted                  = compatible_union_or_error
```

Movement projections do not consume independent `Blocks` and `Traversable`
answers. The resolver must emit exactly one composite fact per movement channel:

```text
MovementDisposition<ground> =
    Blocked {
      reasons: nonempty ordered list<ClaimRef>
    }
  | Traversable {
      cost: positive integer
      reasons: ordered list<ClaimRef>
    }
```

Rules:

- any active ground blocker produces `Blocked`;
- traversal cost is considered only when no blocker remains;
- an open portal does not itself imply traversability;
- traversability requires valid lattice connectivity;
- reasons are stable, ordered, and source-mapped;
- contradictory unresolved movement facts fail with a stable diagnostic rather
  than being left to simulation and navigation to interpret independently.

Simulation and navigation projections consume the same resolved
`MovementDisposition`. Neither chooses which raw claim wins.

## 4. Canonical World IR and projection ownership

The Canonical World IR is versioned, canonically serializable semantic truth. It
contains:

- typed lattice declarations and bindings;
- graph identities and relations;
- stable symbolic IDs for the fixture;
- primitive expansions;
- namespace-machine definitions and initial state;
- typed interactions and phase order;
- capability claim templates and composition laws;
- cross-capability coherence rules;
- resolver plan;
- source maps and fact-ownership receipts;
- provenance and schema/compiler/catalog versions.

The ownership rule is precise:

> Every projection compiler consumes the Canonical World IR. Runtime subsystems
> consume only their own versioned projection artifacts. No subsystem reparses
> `.estate` source or independently invents semantic meaning.

Gate K emits these projections as JSON:

- simulation;
- navigation;
- persistence;
- diagnostics.

A rendering stub is permitted only if it emits declarative data a future
renderer would need; it cannot link a renderer or satisfy any visual gate.

## 5. Immutable packages and mutable runtime state

A compiled world package is immutable evidence. Commands and migrations never
modify it in place.

Gate K uses an inspectable directory package:

```text
build/gaol.world/
  manifest.json
  world-ir.json
  simulation.json
  navigation.json
  persistence.json
  diagnostics.json
  schemas.json
  receipts/
```

A deterministic archive format may come later. Gate K deliberately favors
`ls`, `cat`, and `diff`.

Runtime execution writes a separate run directory:

```text
runs/unlock-gate/
  initial-state.json
  final-state.json
  command-log.json
  causal-receipts.json
  state-hashes.json
  result.json
```

Runtime state has its own versioned schema. The package contains initial-state
material sufficient to create a runtime snapshot; it is not itself the mutable
snapshot.

Migration always writes a new package:

```text
build/gaol-v1.world/ -> build/gaol-v2.world/
```

The source package remains intact as evidence.

## 6. Versioning and the required migration

Version from the first commit:

- authoring source schema;
- Canonical World IR;
- simulation projection;
- navigation projection;
- persistence projection;
- diagnostics projection;
- runtime state;
- replay/command-log format;
- package manifest.

Every persisted artifact names its schema and version. An incompatible change
requires a migration or an explicit recorded epoch break. Successful
parsing/deserialization alone never implies compatibility.

Gate K implements one real Canonical World IR migration:

```text
v1 movement representation:
  blocked_ground: boolean
  traversal_cost_ground: integer | null

v2 movement representation:
  movement_disposition_ground:
    Blocked { reasons }
    | Traversable { cost, reasons }
```

The migration must preserve the fixture's semantic runtime behavior and replay
hashes after both versions are normalized into the v2 runtime-state schema. A v1
package presented directly to a v2 runtime without migration is refused.

## 7. Determinism contract

### Canonical byte profile

Persisted semantic artifacts use UTF-8 JSON under this profile:

- no byte-order mark;
- stable identifier segments and canonical object field names are ASCII and
  match `[a-z][a-z0-9_]*`;
- composite stable IDs use only their schema-declared separators between
  validated segments;
- the accepted identifier alphabet is invariant under Unicode NFC, so
  validation establishes normalization by construction; non-ASCII identifiers
  and field names are refused;
- object keys sorted by ascending UTF-8 byte sequence of their validated names;
- arrays emitted in schema-declared semantic order;
- authoritative numbers are signed or unsigned integers only;
- integers use base-10 with no leading plus sign or redundant leading zeroes;
- string values accept any valid UTF-8 and are not normalized or restricted to
  the identifier alphabet;
- non-ASCII string characters are emitted as UTF-8, not optional `\u` escapes;
- quotation mark and reverse solidus use `\"` and `\\`; backspace, form feed,
  line feed, carriage return, and tab use `\b`, `\f`, `\n`, `\r`, and `\t`;
- every other code point below `U+0020` uses `\u00xx` with lowercase
  hexadecimal digits; solidus is emitted raw and `\/` is refused; `U+007F` is
  emitted raw;
- booleans and `null` use lowercase JSON spelling;
- no insignificant whitespace;
- the hashed byte sequence has no trailing newline.

Human-facing pretty views may be emitted separately, but hashes and package
manifests use canonical bytes.

### State hash

- algorithm: SHA-256;
- display: lowercase hexadecimal;
- hash domain: canonical bytes of the versioned authoritative runtime-state
  envelope only;
- object members use the canonical key ordering above;
- entity collections are arrays ordered by stable entity ID;
- machine collections are arrays ordered by canonical namespace ID.

Included:

- runtime-state schema name and version;
- tick;
- authoritative entity identities;
- namespace-machine states;
- authoritative lattice bindings required by runtime state;
- authoritative counters and scheduled semantic events, if present.

Excluded:

- timestamps and wall-clock values;
- absolute paths;
- source spans and display diagnostics;
- provenance presentation text;
- compiler build paths;
- projection caches;
- pretty-print formatting;
- renderer, audio, and cosmetic state.

Integer arithmetic is checked. Overflow rejects the transaction with a stable
error; no authoritative arithmetic wraps implicitly.

### Execution matrix

The proof uses a pinned Rust toolchain and committed dependency lockfile. The
same command log runs ten times on each target in this initial matrix:

```text
Linux x86_64 debug
Linux x86_64 release
Linux aarch64 release
```

All runs must produce identical semantic state hashes. If the available CI
cannot provide one target, the missing target is recorded as unproved; Gate K is
not called green until the matrix is completed or an owner-authorized contract
revision changes it.

### RNG isolation

Gate K contains no random behavior. Any later authoritative RNG must version its
algorithm and derive each draw from a key containing at least:

```text
world_seed
stream_id
tick
entity_id_or_zero
local_occurrence
```

`local_occurrence` is scoped to that stream/tick/entity tuple. A global event
counter is forbidden because unrelated systems must not perturb one another's
random sequences.

## 8. Command surface

```text
estate validate fixtures/gaol.estate

estate compile fixtures/gaol.estate \
  --out build/gaol.world/

estate inspect build/gaol.world/

estate run build/gaol.world/ \
  --commands fixtures/gaol.commands \
  --out runs/gaol/

estate command build/gaol.world/ \
  --state runs/current/final-state.json \
  "unlock north_gate with credential/gaoler_key" \
  --out runs/after-unlock/

estate explain-entity build/gaol.world/ north_gate
estate explain-entity build/gaol.world/ flooded_section
estate explain-entity build/gaol.world/ brazier_02

estate explain-transition runs/gaol/ north_gate --tick 4
estate explain-transition runs/gaol/ brazier_02 --tick 7

estate replay build/gaol.world/ \
  --log fixtures/gaol.replay \
  --out runs/replay/

estate migrate build/gaol-v1.world/ \
  --to 2 \
  --out build/gaol-v2.world/
```

Every command writes structured JSON to standard output plus artifact paths.
Exit codes:

```text
0  completed successfully
1  rejected with structured diagnostics
2  invalid CLI usage
3  could not execute because of environment or I/O failure
```

No command mutates an input package or input state file.

## 9. Diagnostics and receipts

Diagnostics use stable codes, source spans where source exists, the rejected
fact or command, and legal repair classes. The exact wording may improve without
changing the code's meaning.

At minimum, mutation tests cover:

- dangling entity ID;
- dangling catalog value;
- relation encoded as a lattice cell property;
- authored raw transform;
- derived fact supplied by content;
- two canonical owners for one fact;
- undeclared cross-machine write;
- interaction cycle;
- simultaneous unresolved blocking and traversal;
- projection artifact version mismatch;
- v1 package loaded by v2 runtime without migration;
- attempted in-place command or migration output.

`explain-entity` names source declaration, primitive expansion, machines,
active claims, effective facts, fact owners, projection consumers, and relevant
schema versions.

`explain-transition` names command, actor or system cause, local transition,
typed interactions, claims added/removed, effective facts before/after,
projection deltas, tick, source mapping, and resulting state hash.

## 10. Workspace and dependency boundaries

Gate K uses one Rust workspace containing six kernel crates and one isolated
tooling member:

```text
estate-core        stable IDs, deterministic primitives, canonical bytes, hashing, diagnostics
estate-schema      authoring and Canonical World IR schemas
estate-projection  versioned simulation/navigation/persistence/diagnostic schemas
estate-compiler    parse, link, expand, validate, migrate, and project
estate-sim         runtime state, command transactions, replay, effective-fact resolution
estate-cli         command-line surface and artifact orchestration
xtask              workspace tooling; dependency-boundary proof only
```

`xtask` builds no kernel artifact, depends on no kernel crate, and is
unreachable from every kernel crate. The boundary check fails closed when a
listed kernel crate is missing or an undeclared workspace member appears.

Permitted dependency edges:

```text
estate-schema      -> estate-core
estate-projection  -> estate-core
estate-compiler    -> estate-core, estate-schema, estate-projection
estate-sim         -> estate-core, estate-projection
estate-cli         -> estate-core, estate-compiler, estate-sim, estate-projection
```

Forbidden:

- dependency cycles;
- `estate-sim` depending on `estate-schema` or the source parser;
- any `wgpu`, windowing, renderer, audio, networking, watcher, or hot-reload
  dependency anywhere in Gate K;
- canonical schema types defined in more than one crate;
- runtime subsystems parsing `.estate` files.

Cargo-metadata automation proves workspace membership, permitted dependency
edges, cycles, forbidden dependencies, and tooling isolation. It cannot infer
whether two Rust types duplicate canonical schema semantics. The prohibition on
cross-crate schema-type duplication therefore also requires an explicit source-
review receipt that enumerates each canonical schema identity, its owner crate,
and its authoritative Rust type set, then confirms no second crate defines that
schema. Local schema-ID uniqueness tests, compile-fail visibility tests at
forbidden boundaries, and compiler-crossing tests support that review; none is
misrepresented as a semantic-uniqueness proof by itself.

The old `~1,000 lines` criterion is removed. File length is advisory. Acceptance
uses dependency boundaries, measured build time, measured peak disk, test
coverage, and observable behavior rather than crate confetti.

## 11. Acceptance

Gate K passes only when all of the following are observed, not asserted:

1. **Source is understandable.** The base fixture fits on one normal screen and
   a reader who has not seen the thesis can identify the three instances, three
   primitive kinds, and credential value.
2. **Typed references resolve.** Entity and catalog namespaces are distinct;
   `credential/gaoler_key` resolves without becoming a fourth entity.
3. **Primitives expand.** `inspect` prints each primitive's capability bundle,
   namespace machines, claim templates, and source map.
4. **Machines stay independent.** `access`, `integrity`, `ward`, `combustion`,
   and `emission` are not flattened into a product-state table.
5. **Causal interaction is deterministic.** Entering `burning` applies exactly
   one typed fire-damage event in fixed phase order; ten runs per target produce
   identical hashes.
6. **Effective facts are runtime-resolved.** Opening the warded door changes
   `access` but leaves `MovementDisposition<ground>` blocked until the ward is
   removed; the receipt explains the surviving claim.
7. **Cross-capability contradictions fail closed.** A mutation that supplies
   unresolved simultaneous block/traverse claims receives a stable diagnostic;
   simulation and navigation never choose independently.
8. **Water is real.** `flooded_section` has an inspectable region binding,
   contributes traversal cost `3`, and simulation/navigation projections agree.
9. **Light is real.** Extinguishing `brazier_02` changes its local machine,
   removes effective emission, updates persistence/diagnostics, and emits a
   useful receipt.
10. **Projections agree.** Simulation, navigation, persistence, and diagnostics
    are derived from one IR and move together when a relevant state change is
    applied.
11. **Ownership fails closed.** Every ownership and cross-reference mutation in
    section 9 is rejected with a stable code and useful source context.
12. **Packages stay immutable.** Compile, command, run, replay, and migrate write
    new outputs; input package and state hashes remain unchanged.
13. **Migration works.** The defined v1-to-v2 movement migration preserves
    normalized behavior and replay hashes; direct incompatible loading is
    refused.
14. **Explanations are useful.** Door, water, and light explanations expose
    semantic causality rather than only implementation state.
15. **Workspace boundaries hold.** Automated checks prove the workspace
    membership, dependency graph, cycles, forbidden dependencies, and tooling
    isolation in section 10. An explicit source-review receipt verifies that no
    canonical schema type is defined in more than one crate.
16. **Budgets are measured.** Build time, peak disk, validation latency, command
    latency, and replay throughput are recorded; no unmeasured “fast enough”
    claim satisfies acceptance.
17. **Cold author succeeds.** Under
    `docs/evaluation/COLD_AGENT_PROTOCOL.md`, a model from a different family
    adds a second instance of the approved door kind to an isolated fixture copy
    and reaches a clean compile without editing kernel source, adding a primitive
    kind, or changing unrelated packages.
18. **Cold debugger succeeds.** Under the same protocol, a different-family
    model receives a seeded failing replay and names the true cause using docs,
    CLI, packages, and forensic output without reading kernel source.
19. **A non-author reruns the proof.** The receipt records commit, commands,
    environment, outputs, and reviewer; the author's own run is insufficient.

## 12. Non-goals

No `wgpu`. No window. No renderer. No network. No audio playback. No Workbench
UI. No hot-reload daemon. No asset pipeline. No arbitrary scene tree. No plugin
system. No fourth primitive kind. No recurring-effect scheduler. No production
save compatibility policy beyond the one migration fixture. No visual claim.

The cold-author evaluation may add a second instance of an existing approved
kind in its isolated copy; that is an authoring proof, not a catalog expansion.

Passing Gate K proves only that the semantic architecture is coherent enough to
deserve the next experiment. It does not prove that the game will look good,
play well, scale, ship, or avoid becoming a very educated swamp.
