---
title: The Signed World — an LLM-native game runtime thesis
status: Exploratory design thesis
authority: Non-authoritative
scope: Greenfield / vacuum architecture exercise
implementation_commitment: None
supersedes: Thesis revision 1
superseded_by: Nothing
applies_to_the_mortal_estate: No, unless separately adopted by an explicit project decision
authors: Claude Fable 5 and GPT-5.6 Pro, in adversarial review, with Peter Permenter as owner and referee
date: 2026-08-21
revision: 2
contract_revision: 2
proposed_contract_revision: 3
decision_record: docs/decisions/0003-contract-profile-closure.md
---

# The Signed World

> **The agent names the thing. Namespaces own state. Capabilities define
> obligations. The resolver composes effective facts. Projection compilers own
> the consequences. The runtime executes a sealed world. The renderer owns every
> pixel. A cold stranger can rebuild and explain all of it.**

## 0. How to read this

This is a design thesis written in a vacuum: if the primary author of a game's
content and much of its tooling is a large language model, what runtime
architecture plays to that author's strengths and fences its weaknesses?

It is greenfield, not historically isolated. It deliberately ignores existing
implementation commitments while carrying forward lessons about consistency,
latency, provenance, and proof. The phrase “clean” in the provenance sections
means a controlled source zone; it does not claim the architecture exercise was
created without prior project knowledge.

This document is not an implementation specification and binds no other
project. [KERNEL.md](KERNEL.md) is the first executable acceptance contract.
[docs/decisions/0001-contract-repair.md](docs/decisions/0001-contract-repair.md)
records the owner-authorized repair from revision 1 to revision 2.
[docs/decisions/0003-contract-profile-closure.md](docs/decisions/0003-contract-profile-closure.md)
proposes revision 3 to close the canonical-profile and workspace-evidence gaps
found by SW-B; it remains non-authoritative pending owner disposition.

Contract changes follow `AGENTS.md`. Contradictions and falsified assumptions may
be repaired explicitly; criteria may not be silently weakened because code
failed them.

The founding review is recorded twice: a condensed primary record written in
the originating session (`docs/review/2026-08-21-founding-review.md`, with its
provenance limits stated) and the contract-revision-2 synthesis
(`docs/review/2026-08-21-founding-review-synthesis.md`).

---

## 1. Executive thesis

An LLM is good at text that compiles and bad at pixels that match. It is good at
naming a thing and bad at remembering the eight systems in which that thing must
be registered. It is good at following a deterministic event chain and bad at
explaining frame-rate-dependent accidents. It can read anything it can `cat` and
debug anything it can name.

So the runtime should be built around these rules:

1. **The agent authors intent in a small, typed, fail-closed language.** Content
   uses approved nouns, bounded parameters, and discrete addresses. It does not
   author raw transforms, shaders, arbitrary scripts, or final pixels.
2. **Approved primitives expand into a sealed capability basis.** A door is not
   eight hand-maintained subsystem registrations. It is one primitive whose
   capabilities declare the obligations every subsystem must honor.
3. **Capabilities own contracts; projection compilers own consequences.** A
   capability contains no renderer, navigation, networking, or persistence
   implementation. Each projection compiler translates the same semantic fact
   into its subsystem's versioned artifact.
4. **One Canonical World IR is the shared semantic truth.** Projection compilers
   consume it. Runtime subsystems consume their own derived projection formats.
   No subsystem reparses source or invents a second meaning.
5. **State stays in small namespace-local machines.** Products exist
   mathematically but are never flattened into an unreadable mega-table.
   Machines interact only through declared derived formulas or typed events.
6. **Effective facts are resolved from the whole current state.** Local
   transitions identify affected claims; final movement, visibility, emission,
   and other facts are composed at transaction time, then checked for
   cross-capability coherence.
7. **The runtime is deterministic and server-authoritative.** A replay is a
   proof, a bug is an event chain, and authoritative state has one clock.
8. **Compiled worlds are immutable signed evidence.** Mutable runtime state,
   command logs, migrations, and receipts are separate versioned artifacts.
9. **Every result is traceable.** The system can produce a machine-generated
   argument for why an entity blocked, moved, emitted light, persisted, or
   changed.
10. **The claim is tested by strangers.** A different model family must author
    and debug from the docs and CLI alone, and a clean machine must reproduce the
    proof.

This is broader than art consistency. It is an architecture intended to stop an
LLM from producing locally plausible pieces that fail to form a coherent game.

## 2. Goals and non-goals

### Goals

- Consistency by construction: the approved vocabulary is the style.
- Cross-system coherence by construction: one semantic edit, all consequences.
- Authoring feedback in seconds; complete proof in minutes; never the reverse.
- Deterministic, inspectable runtime behavior.
- Explainability across simulation, navigation, persistence, networking, audio,
  and rendering.
- Reproducibility from versioned manifests and immutable artifacts.
- Provenance as an enforced process boundary, not a hopeful comment.
- A command surface an agent can operate with `ls`, `cat`, `diff`, and structured
  JSON.
- Falsifiable gates before expensive systems are built.

### Non-goals

- A general-purpose engine.
- Arbitrary scene trees or transform hierarchies.
- General rigid-body physics.
- User-authored runtime scripts.
- A universal material graph or open plugin system.
- An editor that silently mutates canonical content.
- Replacing human taste.
- Supporting games whose ontology differs from the adopting game's ontology.
- Treating a cryptographic signature as proof that content is desirable or
  policy-compliant.

Call it an engine internally if that keeps everyone's hair shiny.
Architecturally it should be one game's runtime, aggressively refusing to solve
problems that game does not have.

## 3. The canonical pipeline

There are two distinct phases. Revision 1 blurred them under the phrase
“capability resolution.” Revision 2 does not.

### Compile time: prepare claims, machines, and projections

```text
Authoring source
      │
      ▼
Typed name resolution       — entity, catalog, region, machine, and relation namespaces
      │
      ▼
Primitive expansion         — approved nouns become capability claim templates
      │
      ▼
Machine compilation         — small namespace-local machines and typed transitions
      │
      ▼
Interaction compilation     — derived formulas and causal event edges
      │
      ▼
Resolver compilation        — composition laws and cross-capability invariants
      │
      ▼
Ownership/link validation   — one owner per fact; dangling and smuggled facts fail
      │
      ▼
Canonical World IR          — versioned semantic truth plus resolver plan
      │
      ├──► Simulation projection
      ├──► Navigation projection
      ├──► Rendering projection
      ├──► Audio projection
      ├──► Persistence projection
      ├──► Networking projection
      └──► Diagnostic / provenance projection
```

The compiler may precompute dependency sets, affected facts, candidate
consequences, transition-local work, and static projection structures. It may
not assume the final effective consequence of a local transition without the
rest of the current state.

### Command time: resolve effective facts

```text
validate command
      │
      ▼
apply local machine transition
      │
      ▼
emit typed causal events
      │
      ▼
settle declared interactions in fixed order
      │
      ▼
resolve effective capability facts from all active claims
      │
      ▼
apply cross-capability coherence rules
      │
      ▼
compare before / after effective facts
      │
      ▼
derive subsystem deltas
      │
      ▼
commit runtime state atomically
      │
      ▼
write receipts and publish external effects
```

The Canonical World IR is the architecture's center. The compiler produces it;
projection compilers translate it; the server executes derived runtime forms;
the renderer draws a rendering projection; the Workbench explains all of them.

**Every projection compiler consumes the Canonical World IR. Runtime subsystems
consume only their versioned projection artifacts. No subsystem independently
parses the authoring language.**

## 4. The typed lattice and the world graph

Agents do not author arbitrary floats. Spatial facts live on a common typed
lattice. Relational and temporal facts live in a graph. Stable IDs stitch them.

### Lattice

The lattice owns topology, occupancy, surfaces, regions, portals, elevation,
spatial affordances, and legal anchors.

```text
cell(12, 8, 0)
face(cell(12, 8, 0), north)
edge(cell(12, 8, 0), northeast)
region(flooded_section)
socket(guard_04, right_hand)
path(main_patrol_route)
room(old_chapel)
```

Authored placement uses cells, faces, edges, sockets, regions, discrete
orientation enums, elevation steps, approved footprints, and rational subcell
anchors where a primitive explicitly permits them.

### Graph

The graph owns identity, ownership, state machines, encounters, factions,
quests, triggers, dependencies, inventory relations, and narrative relations.
“The gaoler owns this key” and “these encounters are mutually exclusive” are
graph edges. Encoding them as decorative cell properties would be tidy in the
same way that putting tax records in the cutlery drawer is tidy.

### Linker

The linker owns cross-domain bindings. Content may declare that an entity is
anchored to a face, but neither lattice nor graph stores a second homemade copy
of the resolved relationship.

The runtime may use fixed-point subcell positions and float rendering matrices.
The authoring language exposes neither directly.

Voxel or grid volume data describes source topology, not appearance. A renderer
may compile it into broad stylized meshes rather than cubes.

## 5. The fact-ownership constitution

Every fact class has one canonical owner:

```text
spatial.anchor             -> lattice
spatial.occupancy          -> lattice
spatial.boundary           -> lattice
entity.identity            -> graph
relation.ownership         -> graph
relation.faction           -> graph
state.machine              -> graph
entity.spatial_binding     -> linker-derived
navigation.connectivity    -> projection-derived
render.transform           -> projection-derived
runtime.effective_fact     -> resolver-derived
```

Content may not supply a derived fact. Projection code may not claim canonical
ownership. The compiler fails on:

- duplicate authorities;
- dangling typed IDs;
- a relation smuggled into a lattice cell;
- a raw transform posing as spatial truth;
- graph state contradicting lattice topology;
- an illegal capability combination;
- a derived fact supplied by source;
- a subsystem projection attempting to redefine semantics.

Every build emits fact-ownership receipts:

```json
{
  "fact": "entity.north_gate.spatial_binding",
  "owner": "world_linker",
  "declared_at": "rooms/gaol.estate:42",
  "resolved_to": "face(cell(5,0,0),north)",
  "consumers": ["render", "simulation", "navigation"],
  "derivation": ["primitive/iron_barred_door", "binding/face_anchor"]
}
```

Diagnostics have stable codes, source spans, rejected facts, and legal repair
classes. “Invalid scene” is not a diagnostic. It is the compiler sulking.

Each invariant has a mutation test: inject the violation and assert the stable
rejection code.

## 6. Primitives and the capability basis

Three layers change at different rates:

```text
Engine capability basis      — small, sealed, heavily tested, rarely changed
        ↓
Approved primitive catalog   — extensible through promotion
        ↓
World content                — instantiates approved nouns with bounded parameters
```

A room author writes:

```text
door north_gate {
    anchor = face(cell(5, 0, 0), north)
    archetype = iron_barred
    initial_access = locked
    credential = credential/gaoler_key
}
```

An engine author defines `primitive/iron_barred_door` once from typed machines,
claims, interactions, lifecycle policy, and presentation references.

The room author cannot forget collision, navigation, persistence, audio,
replication, or diagnostics because those consequences do not belong to room
content.

### Why capabilities

If each primitive independently emits into eight subsystems, fifty primitives
create four hundred bespoke emitters and four hundred places to forget a
consequence. With a small capability basis, each projection compiler implements
its consequence per capability family. A new primitive is a new approved bundle,
not a new switch statement in every subsystem.

### Contract versus consequence

A `Portal` capability contains no renderer, navigation, collision, persistence,
or networking code. It declares a typed semantic obligation. Projection
compilers own the subsystem-specific artifacts.

Capabilities are primarily a compile-time semantic language. A static blocker
may compile into a collision bitset, navigation update plan, and visibility
boundary. The runtime does not need to carry a philosophical `Blocks` object for
emotional support.

### Content cannot assemble arbitrary capability soup

Normal content instantiates approved primitives. Capability-bundle authoring is
an engine/catalog operation subject to promotion fixtures and review. This
prevents every room from inventing a slightly different door constitution.

## 7. Capability namespaces, algebra, and coherence

The basis is not one flat trait bag. Capabilities live in typed namespaces:

```text
topology
  Occupies
  Boundary
  Portal
  Region

affordance
  Blocks<movement_channel>
  TraversalCost<locomotion_mode>
  Occludes<sense_channel>
  Supports<load_class>
  Interactable<verb_set>
  Contains<inventory_kind>
  Damages<damage_channel>

state
  Machine<state_schema>
  Trigger<event_schema>

lifecycle
  Authority
  Persisted
  Replicated
  Lifetime

presentation
  Visual
  EmitsLight
  EmitsAudio
  EmitsEffect
```

The exact basis emerges from Gate K and Gate 1 rather than architectural fiat.
The target is a minimal orthogonal basis with one owner, one meaning, and a
known consumer set per capability.

Absence means absence. `Occludes(none)` is suspicious; omit the capability.
Negative traits tend to become three-valued logic and, eventually, a small
demon.

### Per-capability composition

Independent sources may make overlapping claims. Each capability type declares
its algebra:

```text
Blocks<channel>       -> any active claim
TraversalCost<mode>   -> declared cost composition
Occludes<sense>       -> bounded composition
EmitsAudio            -> union
Authority             -> exactly one
Persisted             -> compatible union or error
```

Some combine. Some require one owner. Some conflict and fail.

### Cross-capability coherence

Per-capability algebra alone can still produce nonsense:

```text
Blocks<ground>      = true
Traversable<ground> = cost(1)
Portal              = open
```

Simulation and navigation are not allowed to choose their favorite answer.
Where facts are mutually exclusive, the resolver emits a composite effective
fact. Gate K uses:

```text
MovementDisposition<ground> =
    Blocked { reasons }
  | Traversable { cost, reasons }
```

Any blocker wins over traversal cost; an open portal does not guarantee a valid
route; traversal requires lattice connectivity. Unresolved contradiction fails
closed with a stable diagnostic.

The larger engine will need similar coherence rules for visibility, cover,
containment, authority, and lifecycle. Each one must be explicit and tested.

## 8. Per-namespace state machines

A lockable, damageable, burnable, warded door has a product state. Flattening it
into one table creates entries such as `burning_locked_damaged_sealed`, which is
an arbitrary script written in a particularly stupid notation.

Keep machines separate:

```text
access:      locked | closed | open
integrity:   intact | damaged | destroyed
combustion:  cold | burning | spent
ward:        sealed | unsealed
```

The product exists mathematically. It is never authored or stored as one giant
state enum.

Each transition is local and explicit:

```text
access.unlock(credential) : locked -> closed
access.open               : closed -> open
access.close              : open -> closed
integrity.apply_damage    : intact -> damaged -> destroyed
combustion.ignite         : cold -> burning
ward.unseal               : sealed -> unsealed
```

A local transition does not own final subsystem deltas. Opening a door normally
removes one movement blocker. If a ward still blocks, the effective movement
fact does not change.

The compiler may precompute which claims and resolver dependencies a transition
can affect. The command-time resolver decides the final before/after facts from
the complete state.

## 9. Cross-machine interactions and atomic transactions

Machines own state. Resolvers own effective facts.

Interactions have two forms.

### Pure derived interactions

They calculate effective facts without lying about another machine's state:

```text
portal_open = access == open OR integrity == destroyed
blocks_ground = ward == sealed OR NOT portal_open
```

A destroyed closed door remains `access.closed` and `integrity.destroyed`. Its
passage is breached. That distinction matters for repair, rendering,
persistence, and explanation.

### Causal interactions

They issue typed events to another machine:

```text
combustion.on_enter(burning)
  -> integrity.apply_damage(fire, 2)

integrity.on_enter(destroyed)
  -> container.release_contents
```

No machine directly writes another machine's state. The target machine validates
and owns its transition.

Every causal edge declares trigger semantics such as `on_enter`, `on_exit`,
`on_command`, or an explicitly scheduled pulse. There is no implicit “while
true” behavior. Gate K proves `on_enter`; recurring scheduling remains an open
production question.

A command transaction:

1. validates the command;
2. applies its local transition;
3. emits typed events;
4. settles interactions in deterministic phase order;
5. resolves all affected effective facts;
6. enforces cross-capability coherence;
7. rejects ambiguity, illegal writes, or cycles;
8. commits the complete authoritative result atomically;
9. derives projection deltas and causal receipts;
10. publishes external effects after commit.

The compiler rejects interaction cycles unless a future capability explicitly
defines fixed-point behavior. The runtime retains a transition budget as a last
defense, because eventually somebody builds a torch that lights itself when
extinguished.

## 10. Deterministic, server-authoritative simulation

The intended game model is fixed-tick and server-authoritative, not peer
lockstep.

The server receives typed commands:

```text
move_toward(cell(8,4))
attack(entity/goblin_17)
open(entity/north_gate)
cast(spell/fireball, cell(12,9))
```

It validates and executes them on deterministic ticks.

Authoritative state uses:

- integer lattice coordinates;
- fixed-point subcell positions and timing where needed;
- checked arithmetic;
- stable identity and ordering;
- canonical serialization;
- deterministic system scheduling;
- deterministic pathfinding tie-breaking;
- explicit event phase order;
- isolated RNG streams.

Rendering and cosmetic particles may use floats; neither enters authoritative
state hashes.

### No global RNG stream

A harmless new ambient event must not alter next Tuesday's critical hit. Random
streams are keyed by semantic scope, including a stream ID and a local
occurrence counter. The algorithm and key schema are versioned. Gate K contains
no randomness and therefore cannot accidentally claim to have proved more.

### No authoritative client prediction for a deliberate-pulse game

Clients may preview paths and targets and interpolate between confirmed states.
They do not speculatively move actors, apply damage, or change inventory.
Commands can visibly queue for an authoritative pulse. The design has one clock
and one opinion about where the goblin is.

## 11. Symbolic source IDs and content-addressed builds

Authored source uses stable semantic names:

```text
material/wet_keep_stone
primitive/iron_barred_door
room/flooded_guard_post
actor/gaoler
credential/gaoler_key
```

The compiler resolves them into a content-addressed dependency graph. Build
manifests record source hashes, compiler/schema/catalog versions, projection
hashes, runtime compatibility, and provenance closure.

This gives caching, deduplication, reproducibility, and content negotiation
without making authors edit references that resemble the serial number from a
stolen catalytic converter.

Symbolic names are not automatically persistent identities. What survives a
rename, file move, split, or merge is an explicit open question and must be
settled before production save compatibility.

## 12. Versioned boundaries

The Canonical World IR is constitutional from the first commit:

- explicit schema version;
- canonical serialization;
- migration support;
- fixture coverage;
- compatibility receipts;
- build failure on silent incompatible change.

It is not also the save format, replay format, packet protocol, renderer package,
and live memory layout. The constitution should not regulate sewer-pipe
diameters.

```text
Authoring Source Schema
        │
        ▼
Canonical World IR
        │
        ├──► Simulation Projection Schema
        ├──► Navigation Projection Schema
        ├──► Rendering Projection Schema
        ├──► Audio Projection Schema
        ├──► Persistence Projection Schema
        └──► Diagnostic Projection Schema

Runtime State Schema
        ├──► Save Schema
        ├──► Replay / Command Log Schema
        └──► Network Protocol Schema
```

Rules:

- every persisted artifact declares schema and version;
- each boundary evolves at its own rate;
- incompatible change requires migration or recorded epoch break;
- successful deserialization does not imply semantic compatibility;
- projection compilers consume Canonical World IR;
- runtime subsystems consume projections, not source or Canonical World IR;
- migrations write new artifacts and preserve old evidence.

Gate K defines one actual v1-to-v2 movement migration. Production migration
identity, content-version compatibility, and save evolution remain later proof
obligations.

## 13. The signed-world package

“Signed world” is a build concept, not wall poetry.

A release package includes:

- Canonical World IR;
- versioned projection artifacts;
- source-map index;
- dependency graph;
- provenance closure;
- schema, compiler, catalog, and capability-basis versions;
- machine and resolver definitions needed by the runtime;
- replay-compatibility version;
- validation and invariant receipts;
- artifact hashes;
- a cryptographic signature when distribution requires one.

The runtime refuses unresolved source, unpromoted primitives, laboratory
transforms, prohibited provenance, incompatible schemas, missing projections,
failed invariants, modified artifacts, and forbidden development taints.

A valid signature alone is insufficient. A beautifully signed pile of debug
garbage remains garbage with excellent paperwork.

### Immutable package, separate state

Commands never edit the package. Runtime snapshots, command logs, causal
receipts, saves, and replays are separate versioned artifacts. Migration creates
a new package. This keeps the signed input available as evidence and makes
reproduction possible.

### Development and release binaries

Hot reload is a separate development binary, not a release flag:

```text
estate-devd
  watcher
  incremental compiler
  dev-signed package swapping
  render capture
  instrumentation
  forensic overlays

estate-runtime
  sealed package loading
  no compiler
  no watcher
  no source parser
  no package replacement
  no development RPC
  no laboratory primitives
```

CI proves the release dependency graph contains none of the development loader,
compiler, watcher, or mutation APIs.

### No raw transforms in shippable content

Not discouraged—absent. Primitive development may use a tainted laboratory.
Anything produced there is explicitly non-shippable until promoted through
fixtures and review.

## 14. Provenance and process taint

A hash proves identity, not lineage. Every source node carries provenance:

```text
provenance {
    zone = clean
    origin = project_authored
    license = project_owned
    derived_from = []
    permitted_builds = [development, release]
}
```

Reference-only material lives in a separate namespace and storage/tool boundary:

```text
provenance {
    zone = private_reference
    permitted_builds = [research]
}
```

Taint propagates through declared derivation. The release linker computes the
full closure and refuses prohibited zones.

The enforceable boundary includes separate roots, compiler namespaces, recorded
inputs, immutable provenance edges, policy checks, and manifests. It cannot
prove what passed through a human brain. Reality remains offensively analog. It
can prevent reference material from wandering into shipping assets wearing a
fake moustache.

### Governed raster

Raster is permitted as source material for portraits, icons, decals, signs,
distant scenery, and similar uses. It is not automatically a shipping asset.
The style compiler normalizes palette, values, contrast, edges, dimensions,
alpha behavior, detail frequency, masks, and semantic metadata.

Every visual input must compile into the project's visual language.

## 15. Iteration lanes

A content edit must never require compiling the engine. Feedback latency is an
architectural property.

```text
Content lane
  rooms, encounters, primitive instances, bounded material parameters
  parse -> type-check -> incremental compile -> hot-load -> capture diagnostics
  no Cargo rebuild; no shader rebuild unless a genuinely new variant exists

Catalog lane
  primitive definitions, machine templates, material families, capability bundles
  broader fixture suite; no unrelated runtime rebuild

Engine lane
  capability basis, resolver, projection compilers, renderer, network, simulation
  slower is acceptable because it is rare during content work
```

The dependency graph and cache enforce the lanes physically. The fast lane is
defined by what it excludes. The complete lane is defined by running everything.
Neither is allowed to drift into the other.

## 16. Measured budgets

Budgets live in source control and CI records regressions:

```text
validate_room_p95_ms
compile_room_p95_ms
command_transaction_p95_ms
render_contact_sheet_p95_ms
incremental_cache_size_mb
full_build_time_s
ci_runner_peak_disk_mb
release_package_size_mb
replay_throughput_ticks_per_s
```

Gate K records build time, peak disk, validation latency, command latency, and
replay throughput. Later visual gates add capture time and package size.

“Seems fast enough” is not evidence. Software has repeatedly demonstrated that
it will use any unmeasured budget as a food source.

## 17. Gate 0 — the visual target pack

Engine output cannot determine whether the engine's intended output is desirable.
That is circular. One hero image is also insufficient because it can hide the
actual camera distance, overlapping actors, UI, darkness, animation, and visual
noise.

Gate 0 produces a target pack in any convenient medium:

1. hero environment frame;
2. normal gameplay frame at actual camera distance;
3. actor scale and silhouette references;
4. combat frame with overlapping actors;
5. one restrained spell effect;
6. underground or low-light variant;
7. UI-overlay frame;
8. material and palette sheet;
9. animation or motion timing strip;
10. explicit invariants and explicit non-invariants.

Example invariants:

```text
camera
  fixed oblique; restrained perspective

materials
  broad value groups; restrained texture frequency; visible bevel response

lighting
  readable silhouettes; interactables not crushed; fixed shadow-softness family

palette
  environment subordinate to actors; effects may briefly exceed its saturation

animation
  deliberate cadence; limited idle noise
```

A human says: yes, this is the game we want to look at for hundreds of hours.
Until then, no visual primitive expansion. Twenty rooms in a bad style merely
prove the runtime can manufacture regret at industrial scale.

Temporary code used to make the pack stays quarantined. The durable artifacts
are the pack, provenance, extracted visual rules, and approval receipt. A
throwaway renderer does not graduate by squatting there long enough.

### Expected tactical-RPG representation

The current thesis expects voxel- or lattice-authored environments rendered as
stylized meshes:

- broad extruded walls;
- beveled corners;
- arches, vaults, trim, stairs, roofs, and terrain transitions;
- restrained oblique camera;
- stable world scale;
- vertex-color families and tiny approved atlases;
- low internal resolution;
- palette quantization and controlled dithering;
- fixed shadow behavior;
- discrete light rigs;
- bounded outlines, fog, and depth grading.

Characters use typed skeletal assemblies—rig, proportions, head family, outfit,
locomotion, sockets—rather than a fresh sculpt for every guard. Effects use a
bounded effect language.

### Procedural generation

Three kinds, three verdicts:

- **Deterministic derivation:** encouraged. A declared roof or water region
  produces its legal geometry, joins, movement effects, audio, and diagnostics.
- **Bounded seeded variation:** sparingly, for secondary texture under approved
  fixtures, density limits, and clearance rules.
- **Open-ended synthesis:** nowhere in shipping content. That is where oatmeal
  gets tenure.

## 18. Proof gates

### Gate K — semantic preflight

Before visual machinery, prove the renderer-free kernel in `KERNEL.md`:

- exact three-primitive fixture;
- typed entity and catalog namespaces;
- namespace-local machines;
- capability claim preparation and command-time effective resolution;
- cross-capability coherence;
- deterministic state and replay hashes;
- immutable package and separate runtime state;
- one real migration;
- useful explanations;
- cold author, cold debugger, and non-author rerun.

Passing Gate K means the semantic architecture deserves another experiment. It
does not authorize a renderer.

### Gate 0 — visual thesis

Human-approved target pack from section 17. Required before renderer work.

### Gate 1 — cross-system thesis

Prove three primitives end to end once visual and additional runtime systems
exist:

- door: visual state, collision, navigation, audio, interaction, network,
  persistence, diagnostics;
- water: rendering, shoreline, movement, navigation cost, sound, effects,
  diagnostics;
- extinguishable light: lighting, visibility where applicable, audio/animation,
  replication, persistence, forensic renders.

### Gate 2 — vocabulary thesis

A knowledgeable author creates twenty distinct rooms from one approved kit with
no renderer changes, no new capabilities, and no raw transforms. They must be
clearly different and unmistakably one game.

### Gate 3 — cold-author thesis

A different model family receives only the approved author packet and CLI and
authors room 21. Record time to first valid compile, validation cycles, errors,
human intervention, forbidden escape attempts, time to visual acceptance, and
files changed.

### Gate 4 — cold-debug thesis

A different-family model receives a seeded failing replay and forensic tools. It
must identify the true semantic cause without reading engine source.

### Gate 5 — reproducibility thesis

A clean machine or container rebuilds from the manifest, runs the replay suite,
captures pinned renders, and reaches expected hashes and perceptual thresholds.

Formal cold-agent runs follow
`docs/evaluation/COLD_AGENT_PROTOCOL.md`. Model diversity is design fuzzing, not
decision authority.

## 19. Forensics, explanation, and the visual proof ladder

The detailed forensic command surface, contact-sheet layers, and visual proof
ladder live in
[docs/thesis-forensics-and-disagreements.md](docs/thesis-forensics-and-disagreements.md#section-19-details--forensics-explanation-and-the-visual-proof-ladder).
That surface must expose semantic causality, not merely implementation state.

## 20. Resolved disagreements

The full disagreement ledger is preserved in
[docs/thesis-forensics-and-disagreements.md](docs/thesis-forensics-and-disagreements.md#section-20-ledger--resolved-disagreements).
Its resolutions remain part of thesis revision 2.

## 21. Open questions

The live question ledger is kept in
[docs/thesis-open-questions.md](docs/thesis-open-questions.md). Questions become
decisions only through code and evidence or an owner-authorized record.

## 22. Adoption criteria

This thesis applies to no game project until all of the following are true:

1. Gate K passes `KERNEL.md`, including cold-author, cold-debug, determinism,
   migration, and non-author rerun evidence.
2. Gate 0 has a human-approved target pack for the adopting game.
3. Gate 1's three primitives are proven end to end in the intended runtime.
4. The adopting project records an explicit decision in its own authority tree,
   including migration or epoch-break consequences.
5. The adopting project accepts the measured cost of the custom runtime rather
   than treating previous effort as an argument.

Until then: exploratory, non-authoritative, and not to be extended with more
ornamental architecture merely because Markdown is cheaper than a compiler.
