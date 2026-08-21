---
title: The Signed World — an LLM-native game engine thesis
status: Exploratory design thesis
authority: Non-authoritative
scope: Clean-room / vacuum architecture exercise
implementation_commitment: None
supersedes: Nothing
superseded_by: Nothing
applies_to_the_mortal_estate: No, unless separately adopted by an explicit project decision
authors: Claude Fable 5 and GPT Pro, in adversarial review, with Peter Permenter as owner and referee
date: 2026-08-21
revision: 1
---

# The Signed World

> **The agent names the thing. Namespaces own state. Capabilities define
> obligations. The resolver composes effective facts. Projection compilers own
> the consequences. The runtime executes a sealed world. The renderer owns every
> pixel. A cold stranger can rebuild and explain all of it.**

## 0. How to read this

This is a design thesis written in a vacuum: *if the primary author of a game's
content and much of its tooling is a large language model, what engine
architecture plays to that author's strengths and fences its weaknesses?* It
takes no history as given and commits nobody to anything. It was produced by two
model families arguing, with a human deciding which findings mattered; §20
records the path, because the reasoning is more durable than the diagram.

It is not a spec. Nothing here is implemented. §22 says what would have to be
true before any project adopted it.

Lessons drawn from real projects are stated as lessons, not as history.

---

## 1. Executive thesis

An LLM is good at text that compiles and bad at pixels that match. It is good at
naming a thing and bad at remembering the eight places that thing must be
registered. It is good at reasoning over a deterministic replay and bad at
reasoning over frame-rate-dependent physics. It can read anything it can `cat`
and debug anything it can name.

So the engine should be built so that:

1. **The agent authors intent in a small, typed, fail-closed language.** Nouns
   from an approved catalog, bounded parameters, discrete addresses. Never a raw
   transform, never a raw shader, never a pixel.
2. **A compiler owns every consequence.** A declared door becomes collision,
   navigation, animation, audio, persistence, replication, and diagnostics
   without the author touching any of them.
3. **One canonical, versioned world model is the shared truth**, and every
   subsystem projects from it. No subsystem reinterprets the source.
4. **The runtime is deterministic and server-authoritative**, so a replay is a
   proof and a bug is an event chain.
5. **Every visible and behavioral result is traceable to a source line**, and
   the engine can produce a machine-generated argument for why it happened.
6. **Nothing ships that the compiler did not sign**, and the signature carries
   provenance, promotion state, schema compatibility, and invariant receipts.
7. **The whole claim is tested by strangers**: a different model family authors
   and debugs from the docs alone, and a clean machine reproduces every proof.

This is not an art-consistency solution. It is an engine designed to stop an LLM
from producing locally plausible pieces that fail to form a coherent game.

## 2. Goals and non-goals

**Goals**

- Consistency by construction: the vocabulary *is* the style.
- Cross-system coherence by construction: one semantic edit, all consequences.
- Authoring latency in seconds; full proof in minutes; never the reverse.
- Explainability across every subsystem, not only pixels.
- Reproducibility from a manifest on a clean machine.
- Provenance as an enforced process boundary.

**Non-goals**

- A general-purpose engine. No arbitrary scene tree, rigid-body physics, user
  scripts, universal material graph, open plugin system, freeform transform
  hierarchies, or an editor that mutates canonical content.
- Replacing human taste. Gate 0 (§17) is a human saying yes to a target pack.
- Supporting any game other than the one whose ontology it is built around.
  Call it an engine internally; architecturally it is *one game's runtime*.

## 3. The canonical pipeline

```text
Authoring source (semantic, human-readable, symbolic IDs)
      │
      ▼
Primitive expansion          — nouns from the approved catalog expand to capability bundles
      │
      ▼
Capability resolution        — state-indexed claims composed by each capability's algebra
      │
      ▼
Fact ownership + cross-reference validation   — one owner per fact class; dangling IDs fail
      │
      ▼
State-space compilation      — per-namespace machines, interactions, transition tables
      │
      ▼
Canonical World IR           — versioned, canonically serialized, the shared truth
      │
      ├──► Simulation projection
      ├──► Navigation projection
      ├──► Rendering projection
      ├──► Audio projection
      ├──► Persistence projection
      ├──► Networking projection
      └──► Diagnostic / provenance projection
```

The Canonical World IR contains the spatial lattice, the relationship graph,
stable identities, resolved capabilities, state machines and transition tables,
source mappings, provenance, and compiler/schema versions.

**Every subsystem consumes the IR. None independently parses the source.**
Letting the renderer and the simulator each read the room language would
produce two religions within a dozen commits.

The IR is the engine's center. The compiler is one producer of it. The server
executes it, the renderer draws it, the Workbench explains it.

## 4. The typed lattice and the world graph

Agents never author floats. Spatial facts live on a common **lattice**;
relational and temporal facts live in a **graph**; stable entity IDs stitch
them.

**Lattice** — topology, occupancy, surfaces, regions, portals, elevation,
spatial affordances. Addresses are discrete and typed:

```text
cell(12, 8, 0)
face(cell(12, 8, 0), north)
edge(cell(12, 8, 0), northeast)
region(flooded_section)
socket(guard_04, right_hand)
path(main_patrol_route)
room(old_chapel)
```

**Graph** — identity, ownership, state machines, encounters, quests, factions,
triggers, dependencies, narrative relations. "The gaoler owns this key" and
"these two encounters are mutually exclusive" are graph edges; encoding them as
cell properties is conceptual tax fraud.

The runtime may use fixed-point coordinates for moving actors and float
matrices for rendering. The authoring language exposes neither except wrapped
in bounded semantic types (`elevation = step(1)`, `facing = southwest`,
`anchor = cell(4, 7, 0).center`).

Voxels — or any grid-based volume — are the *source topology* of the lattice,
not the appearance. See §17.

## 5. The fact-ownership constitution

Two authorities drift unless one pass owns every cross-reference. Every fact
class has exactly one declared canonical owner:

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
```

Content may declare an entity's lattice anchor through the sanctioned binding
syntax; the **linker** owns the resolved relationship. Neither side stores a
second copy.

The compiler fails on: duplicate authorities for one fact; dangling entity IDs;
a relation encoded as a cell property; an authored transform posing as spatial
truth; graph state contradicting lattice topology; an illegal capability
combination; a derived fact supplied by content.

Every build emits a **fact-ownership receipt** per fact:

```json
{
  "fact": "entity.north_gate.spatial_binding",
  "owner": "world_linker",
  "declared_at": "rooms/gaol.estate:42",
  "resolved_to": "face(cell(5,0,0),north)",
  "consumers": ["render", "sim", "nav"],
  "derivation": ["primitive:iron_barred_door", "binding:face_anchor"]
}
```

Diagnostics are structured, with stable codes, source spans, and legal repairs.
"Invalid scene" is not a diagnostic; it is the compiler sulking.

```json
{
  "code": "E_WORLD_041",
  "message": "Spatial occupancy is owned by the lattice",
  "source": "rooms/gaol.estate:48:9",
  "illegal_fact": "entity.north_gate.graph.position",
  "legal_repairs": [
    "Bind the entity to a cell, face, edge, region, or socket",
    "Remove the graph position field"
  ]
}
```

Every invariant has a mutation test: inject the violation, assert the code.

## 6. Primitives and the capability basis

Three layers, with sharply different change rates:

```text
Engine capability basis      — tiny, closed, sealed, heavily tested, rarely changed
        ↓
Approved primitive catalog   — extensible through the promotion process
        ↓
World content                — instantiates approved primitives; uses nouns
```

A content author writes:

```text
door north_gate {
    anchor = face(cell(5, 0, 0), north)
    archetype = iron_barred
    initial_state = locked
    key = gaoler_key
}
```

An engine author defined `iron_barred` once, from capabilities:

```text
primitive iron_barred_door {
    topology { portal(axis = normal) }
    machines { access = lockable_door; integrity = breakable_iron }

    affordances {
        blocks(movement.ground)                 when access != open and integrity != destroyed
        occludes(vision.normal, amount = 0.80)  when access != open and integrity != destroyed
        traversable(movement.ground, cost = 1)  when access == open or integrity == destroyed
        interactable {
            unlock(key) -> access.unlock
            open        -> access.open
            close       -> access.close
        }
    }

    lifecycle {
        persisted(scope = world, fields = [access, integrity])
        replicated(fields = [access, integrity])
        authority = server
    }

    presentation {
        visual = iron_barred_door
        transition_audio = iron_door
    }
}
```

The primitive passes the promotion suite once. Content agents use it safely
forever, and cannot omit persistence, navigation, or collision, because those
decisions are not theirs to make.

**Why a capability layer at all.** If every primitive independently emitted into
eight subsystems, fifty primitives would mean four hundred hand-written
emitters, each a place to forget something. With ~a dozen capabilities, each
subsystem emits *per capability*, never per primitive. A new primitive is a new
bundle and zero new emitters. Cross-system correctness becomes a property of a
handful of capabilities that can be tested exhaustively.

**Capabilities own contracts and obligations. Projection compilers own the
consequences.** A `Portal` capability contains no rendering, navigation,
collision, persistence, or networking code. It declares a typed fact; each
projection compiler understands that fact and emits its own artifact.

Capabilities are primarily a **compile-time semantic language**. They need not
become one-for-one runtime components; a static `Blocks<ground>` claim may
compile into a collision bitset, a navigation edge state, and a visibility
boundary, with no philosophical `Blocks` object carried at runtime for emotional
support.

## 7. Capability namespaces and composition laws

The basis is not a flat bag. A flat bag permits type-correct gibberish — a
replicated audio emitter that owns a key and occludes swimming. Capabilities are
divided into typed namespaces:

```text
topology      Occupies · Boundary · Portal · Region
affordance    Blocks<movement_channel> · Traversable<locomotion_mode> · Occludes<sense_channel>
              Supports<load_class> · Interactable<verb_set> · Contains<inventory_kind> · Damages<damage_channel>
state         Machine<state_schema> · Trigger<event_schema>
lifecycle     Authority · Persisted · Replicated · Lifetime
presentation  Visual · EmitsLight · EmitsAudio · EmitsEffect
```

The exact basis emerges from the vertical slice (§18), not from fiat. The goal
is a minimal orthogonal basis in which every capability has one owner, one
meaning, and a known consumer set.

**Absence means absence.** `Occludes(none)` is suspicious; omit the capability.
Negative traits become three-valued logic and, eventually, a small demon.

**Composition laws.** Independent machines (§8) will emit overlapping claims —
`access.closed` claims `Blocks<ground>`, a magical seal also claims
`Blocks<ground>`, `integrity.destroyed` means the doorway is breached. Each
capability type therefore declares how claims combine:

```text
Blocks<channel>       composition = any_active_claim
Traversable<mode>     composition = minimum_legal_cost
Occludes<sense>       composition = bounded_sum
EmitsAudio            composition = union
Authority             composition = exactly_one
Persisted             composition = compatible_union_or_error
```

Some combine. Some require exactly one owner. Some conflict and fail
compilation. Without declared algebra, escaping state explosion only produces
"which trait wins" soup.

## 8. Per-namespace state machines

A door that is also damageable and burnable is a product state. Flattening the
product into one table produces an arbitrary script written in a very stupid
notation, and nobody can read why `burning_locked_damaged → open` does what it
does. So:

```text
access:      locked · closed · open
integrity:   intact · damaged · destroyed
combustion:  cold · burning · spent
```

The combined state exists mathematically. It is never flattened or authored as
one table. Each machine is small, and each transition is explicit:

```text
unlock(key) : locked -> closed
open        : closed -> open
close       : open   -> closed
```

The compiler precomputes subsystem deltas per legal transition:

```text
closed -> open
  simulation:   disable blocker
  navigation:   activate portal
  rendering:    play approved opening animation
  audio:        emit iron_door.open
  persistence:  state = open
  network:      replicate state transition
  diagnostics:  record cause, actor, source primitive, tick
```

The runtime executes a compiled transition table. It does not run a door script.

## 9. Cross-machine interactions and transaction resolution

**Machines own state. Capability resolvers own effective facts.**

Interactions have exactly two forms.

**Pure derived interactions** change effective capabilities without mutating
another machine:

```text
portal_open   = access == open OR integrity == destroyed
blocks_ground = magical_seal == active OR NOT portal_open
```

Destroying the door does not lie by setting `access = open`. The door remains
closed and destroyed; its effective topology is breached. That distinction
matters for repair, rendering, persistence, and explanation.

**Causal interactions** issue a typed command to another machine:

```text
combustion.burning   -> integrity.apply_damage(fire, 2)
integrity.destroyed  -> container.release_contents
```

No machine writes another machine's state. It emits a typed event; the target
decides whether that event produces a legal transition.

Every external command resolves as **one deterministic state transaction**:

1. Apply the machine-local transition.
2. Emit typed effects.
3. Evaluate declared interactions in a fixed phase order.
4. Resolve capability claims through their composition laws.
5. Reject conflicts, cycles, or ambiguous writes.
6. Commit the complete result atomically.
7. Emit subsystem deltas and a causal receipt.

The compiler rejects interaction cycles unless a capability explicitly defines
fixed-point behavior. The runtime carries a transition budget as a last line of
defense, because eventually somebody builds a torch that lights itself when
extinguished.

## 10. Deterministic, server-authoritative simulation

Fixed-tick, server-authoritative. Not peer lockstep.

The server receives typed commands — `move_toward(cell(8,4))`,
`attack(entity(goblin_17))`, `open(entity(north_gate))` — validates them, and
executes them at deterministic ticks. Clients render authoritative state.

Authoritative state uses integer grid coordinates, fixed-point subcell positions
and timings, stable entity ordering, canonical serialization, deterministic
system scheduling, deterministic pathfinding tie-breaking, and explicit event
ordering. Rendering and cosmetic particles use floats; neither enters a state
hash.

**No single global RNG.** A new random call in an ambient-sound system must not
alter next Tuesday's critical-hit sequence. Use keyed or counter-based streams
derived from `(world_seed, tick, system_id, entity_id, event_sequence)`; combat,
loot, encounters, decoration, and cosmetics each get their own.

A replay is `content_manifest + initial_snapshot + ordered command log +
periodic expected state hashes`, and the debugging surface is semantic:

```text
estate replay reports/door_regression.replay
estate trace --tick 1840 --entity north_gate
estate explain-state north_gate
```

**Zero authoritative client prediction** for a deliberate-tempo tactical game.
Commands are acknowledged (`Move accepted · executes on pulse 1841`); the
client interpolates between confirmed beats; path and target previews are local
UI; nothing speculative moves, damages, or changes inventory. There is no second
clock trying to be helpful. A fixed pulse turns latency from an implementation
embarrassment into part of the interaction grammar.

## 11. Symbolic IDs in source, content addresses in builds

Authored files are not hash soup. Agents need stable semantic names in
ordinary version-controlled files with canonical formatting and meaningful
diffs:

```text
material/wet_keep_stone
primitive/iron_barred_door
room/flooded_guard_post
actor/gaoler
```

The compiler resolves those into a content-addressed build graph, and a build
manifest records source hashes, compiler/schema/catalog versions, generated
artifact hashes, runtime compatibility, and the dependency graph. That gives
reproducibility, caching, deduplication, content negotiation, and forensic
provenance without making anyone edit references that resemble a stolen
catalytic converter's serial number.

## 12. Versioning: the IR and its derived boundaries

The Canonical World IR is constitutional from the first commit: explicit schema
version, canonical serialization, migration support, fixture coverage,
compatibility receipts, and a build failure on any silent schema change. This
is the one place where "no compatibility layers during prototype" does not
apply — the IR is not an internal contract.

But the IR is **not** also the save format, replay format, network protocol,
renderer package, and live simulation layout. Coupling every boundary to every
other boundary means a renderer-only field forces a network migration and a
provenance change invalidates saves. The constitution should not regulate sewer
pipe diameters.

```text
Authoring Source Schema
        │
        ▼
Canonical World IR
        │
        ├──► Simulation Projection IR
        ├──► Navigation Projection IR
        ├──► Rendering Projection IR
        ├──► Audio Projection IR
        └──► Diagnostic Projection IR

Runtime State Schema
        ├──► Save Schema
        ├──► Replay Schema
        └──► Network Protocol Schema
```

Each derived contract has its own version and migration rule and evolves at its
own rate. Enforced from the beginning:

- Every checked-in schema has a version.
- Every persisted artifact names its schema version.
- Every incompatible change requires a migration or an explicit, recorded epoch
  break (permitted before release; never implicit).
- No compatibility is implied merely because two structures happen to
  deserialize.

## 13. The signed-world package

"Signed world" is a build concept, not wall poetry. A release package includes:
the Canonical World IR; projection artifacts; source-map index; content
dependency graph; provenance closure; schema, compiler, primitive-catalog, and
capability-basis versions/hashes; state-machine definitions; replay
compatibility version; validation receipt; artifact hashes; and, for
distributed builds, a cryptographic signature.

The release runtime refuses: unresolved source; unpromoted primitives;
laboratory transforms; prohibited provenance; incompatible schemas; missing
projection artifacts; failed invariants; modified packages without a valid
manifest.

A valid signature alone is insufficient — the policy also verifies
capability-basis version, compiler version, provenance closure, promotion state,
schema compatibility, invariant receipts, and the absence of development taints.
A beautifully signed pile of debug garbage is garbage with excellent paperwork.

**Development vs release binaries.** Hot reload is a separate binary, not a
flag:

```text
estate-devd     file watching · incremental compiler · dev-signed packages · package swapping
                render capture · instrumentation · forensic overlays · live inspection

estate-runtime  sealed package loading · no compiler · no watcher · no source parser
                no package replacement · no development RPC · no laboratory primitives
```

Development packages carry an ephemeral development signature and explicit
taints (`hot_reloadable`, `unapproved_content`, `debug_provenance`). Release
binaries accept only release-policy manifests, and CI proves from the build
graph that the release binary links none of the development machinery.

**No raw transforms in shippable content.** Not discouraged — gone. Authored
placement uses cells, faces, edges, sockets, regions, discrete orientations,
elevation steps, approved footprints, and rational subcell anchors where
unavoidable. The compiler produces the transform. A primitive-development
laboratory exists because someone has to create primitives; anything authored
through it carries `NON_SHIPPABLE: contains unpromoted laboratory content`. It
renders, it tests, it cannot enter a release package.

## 14. Provenance and process taint

A hash proves identity; it says nothing about lineage. Every source node carries
provenance:

```text
provenance {
    zone = clean
    origin = project_authored
    license = project_owned
    derived_from = []
    permitted_builds = [development, release]
}
```

Reference-only or private material lives in a different namespace and toolchain
boundary (`zone = private_reference`, `permitted_builds = [research]`). Taint
propagates through derivation; the release linker computes the full dependency
closure and refuses forbidden zones.

The enforceable boundary: separate storage roots, separate compiler
namespaces, recorded generation inputs, immutable provenance edges, release
policy checks, manifests showing the closure. It cannot prove what once passed
through a human brain. It can prevent reference material from wandering into
shipping assets wearing a fake moustache.

**Lesson carried:** provenance is tainted by *process*, not only by payload. A
workflow that makes reference material the first step of authoring taints
everything authored under it even where the output reads clean. The clean-break
line is drawn before the first asset, not retrofitted.

**Generated raster is source material, not a shipping asset.** Portraits,
icons, decals, distant scenery — fine. Every visual input passes through a style
compiler that remaps to approved colors, enforces value ranges, normalizes
contrast, applies the common edge treatment, resizes to legal dimensions,
checks alpha, rejects inappropriate detail frequency, and attaches semantic
metadata. The governing rule is *every visual input must compile into the
project's visual language* — more useful than banning a medium.

## 15. Iteration lanes

**A content edit must never require compiling the engine.** The proof loop's
latency is an architectural property; two art directions in one real project
died before taste got a vote because full builds sat in the iteration path.

```text
Content lane   rooms, encounters, primitive instances, material parameters, rig selections
               parse → type-check → incremental compile → hot-load → capture diagnostics
               no cargo rebuild, no shader rebuild unless the variant is new
               target: validation immediate; contact sheet in seconds

Catalog lane   primitive definitions, material families, machine templates, capability bundles
               broader fixture suites; no unrelated runtime crate rebuilds

Engine lane    capabilities, projection compilers, renderer, networking, simulation
               slow is acceptable; rare during content work
```

The dependency graph and cache enforce the lanes physically. The fast lane is
defined by what it *excludes*; the complete lane by running everything. Neither
drifts toward the other. A long-running daemon holds the runtime, GPU
resources, and catalog indexes in memory while packages hot-reload.

## 16. Measured budgets

"Seems fast enough" has had several opportunities to stab a real project and
used all of them. Budgets live in source control and CI records regressions:

```text
validate_room_p95_ms
compile_room_p95_ms
render_contact_sheet_p95_ms
incremental_cache_size_mb
full_build_time_s
ci_runner_peak_disk_mb
release_package_size_mb
replay_throughput_ticks_per_s
```

Disk, build time, and capture time are measured, not assumed — a single lean
Rust test build can leave 6 GiB where a default one leaves 21, and a CI runner
does not care which you expected.

## 17. Gate 0 — the visual target pack

Engine output cannot determine whether the engine's intended output is
desirable; that is circular. And the throwaway renderer built to make a hero
image becomes the engine, because it always does.

So Gate 0 produces a **visual target pack** in any convenient medium — painted
mockup, Blender scene, image-generation composite, hand-edited concept,
temporary commercial renderer, cardboard and spite — containing:

1. A hero environment frame.
2. A normal gameplay frame at actual camera distance.
3. Actor scale and silhouette references.
4. A combat frame with several overlapping actors.
5. One restrained spell effect.
6. One underground or low-light variant.
7. One UI-overlay frame.
8. A small material and palette sheet.
9. An animation or motion timing strip.
10. Explicit visual invariants — and explicit non-invariants.

```text
camera:     fixed oblique; no dramatic perspective distortion
materials:  broad value groups; restrained texture frequency; visible bevel response
lighting:   readable silhouettes; no crushed interactables; fixed shadow-softness family
palette:    environment subordinate to actors; effects may briefly exceed environment saturation
animation:  deliberate cadence; no excessive idle motion
```

Recording what does *not* need to survive matters: without it, the renderer
spends three weeks reproducing a decorative crack nobody cared about.

A human says: *yes, this is the game we want to look at for several hundred
hours.* Until then, no procedural expansion — twenty rooms in a bad style proves
only that the engine can manufacture regret at industrial scale. Any code used
to create Gate 0 lives in a quarantined prototype area or separate repository;
the durable artifacts are the pack, its provenance, the extracted rules, and the
approval receipt.

**The representation this thesis expects to pass Gate 0 for a tactical RPG:**
voxel-authored, mesh-rendered. Cells and surfaces compile into broad extruded
walls, bevelled corners, arches, vaults, trim, columns, stairs, roof sections,
terrain transitions, damaged variants. The visual stack enforces a restrained
oblique camera, stable world scale, vertex-color families, tiny approved
atlases, low internal resolution, palette quantization, controlled dithering,
fixed shadow behavior, discrete light rigs, fog and depth grading, bounded
outlines, approved silhouette ranges. Characters are typed skeletal assemblies
(rig, proportions, head family, outfit, locomotion set, sockets), not sculpted.
Spells and particles use a bounded effect DSL. Semantic nouns and bounded
parameters in; renderer-controlled pixels out.

**Procedural generation, three kinds, three verdicts.** *Deterministic
derivation* (a declared gabled roof or water region produces its only legal
geometry, joins, shoreline, movement cost, audio, overlays) — encouraged;
asking the agent to author every roof cell creates more inconsistency, not
less. *Bounded seeded variation* (approved rules choose among legal, fixture-
rendered alternatives under density and clearance limits) — sparingly, for
secondary texture. *Open-ended synthesis* ("fill this with whatever looks
ruined") — nowhere in shipping content. That is where oatmeal gets tenure.

## 18. The proof gates

**Gate 0 — visual thesis.** The target pack (§17), human-approved.

**Gate 1 — system thesis.** Three cross-system primitives, proven end to end:

- A **door**: opening alters mesh and animation, collision, navigation, audio,
  interaction state, network state, persistence.
- **Water**: a region declaration alters rendering, shoreline geometry,
  movement, navigation cost, sound, applicable effects, diagnostics.
- An **extinguishable light**: placing or extinguishing it alters renderer
  lighting, visibility rules where gameplay uses them, audio/animation where
  applicable, replicated and saved state, forensic renders.

**Gate 2 — vocabulary thesis.** A knowledgeable author creates twenty distinct
rooms (guard station, flooded passage, shrine, barracks, storage, collapsed
hallway, prison, ritual chamber, kitchen, abandoned checkpoint…) from one
approved kit: one construction family, ~20 props, three light rigs, one water
and one fog treatment, one camera, one body rig, a small equipment set, one post
stack. No renderer changes, no new capabilities, no arbitrary transforms.
Pass/fail: twenty clearly different places that unmistakably belong to one game.

**Gate 3 — cold-author thesis.** A new agent, from a *different model family*
than any that helped design the system, receives only the authoring docs, the
content schema, the primitive catalog, example fixtures, and the CLI, plus a
room brief, and authors room 21. Measure: time to first valid compile; validation
cycles; compiler errors; human interventions; attempted forbidden escape
hatches; time to accepted visual result; whether anything outside the room
package changed. An "LLM-native" language that works only for the model whose
fingerprints are on it is a private dialect with a marketing department.

**Gate 4 — cold-debug thesis.** Seed a defect (a door that blocks navigation
after opening; a light whose saved state differs from replicated state; a water
region with the wrong traversal cost). A cold agent gets the failing replay and
the forensic tools and must locate and repair the real cause without engine
source access. That proves the explanation system works rather than producing
diagnostic wallpaper.

**Gate 5 — reproducibility thesis.** A clean machine or container rebuilds the
package from the manifest, runs the replay suite, captures the pinned renders,
and reaches the expected state hashes and perceptual thresholds. Nothing is
green until a stranger reproduces it.

Acceptance across the gates:

1. Twenty visually distinct rooms from one kit.
2. No renderer or catalog edits while authoring them.
3. No arbitrary transforms in content.
4. One semantic edit produces every required cross-system change.
5. The scenario replays to identical authoritative state hashes.
6. Every visible and behavioral result is traceable to source.
7. Browser and native renderers consume the same compiled package.
8. A cold author and a cold debugger succeed from the docs alone.

## 19. Forensics, explainability, and the visual proof ladder

`explain-pixel` is table stakes; the full engine answers "why" across every
subsystem:

```text
estate explain-pixel scene.guard_post 612 344
estate explain-entity north_gate
estate explain-transition north_gate --tick 4
estate why-blocked guard_04 --destination cell(8, 4)
estate why-visible thief_02 --to guard_04
estate trace-event combat_18842
estate explain-material wet_keep_stone
estate explain-save north_gate
estate explain-replication goblin_17
```

A result cites source file and line, semantic primitive, compiler passes
involved, generated IDs, active state, rules that fired, dependencies, tick and
event sequence. The agent never merely learns that something failed; it receives
a machine-generated argument for why. Conventional tools expose implementation
state; this exposes semantic causality.

Every render ships a **contact sheet**: beauty, neutral-lit, silhouette,
material-ID, entity-ID, depth, normals, light-only, navigation, collision,
annotated warnings. A multimodal reviewer sees the room and the forensic report
and can connect "that corner looks wrong" to an entity, a rule, and a source
line.

**Visual testing is a ladder, not a pixel diff.** Byte-identical screenshots
across machines are fantasy bureaucrat bait; browser, OS, font, and GPU
differences move pixels.

1. Structural assertions — permitted materials only, correct projection and
   camera, no missing assets, no illegal scales, valid navigation, no collision
   holes, legal light count, deterministic placement.
2. Image statistics — palette compliance, luminance distribution, contrast
   around interactables, edge density, occupied screen area, repetition
   frequency.
3. Perceptual comparison — tolerant diff, masked dynamic regions, a pinned
   software-rendering runner, approved reference fixtures. Byte-identical only
   on the pinned runner.
4. Multimodal review over the contact sheet.

Goldens are receipts. They are not taste.

**Cold-model review is design fuzzing, not decision authority.** The review
packet is blind — no transcript, no intended conclusions, no hints at disputed
points; only the design, requirements, constraints, and rubric. The reviewer
hunts hidden coupling, unowned facts, impossible migrations, capability
overlap, machine cycles, authoring escape hatches, performance traps,
unverifiable claims, and places where the design requires taste but pretends to
require logic. A human decides which findings matter. Models are excellent at
attacking textual systems and will also object to gravity given enough tokens.

## 20. Rejected alternatives and resolved disagreements

The path matters more than the diagram. In order of resolution:

| Position A | Position B | Resolution | Why |
| --- | --- | --- | --- |
| Browser/WebGL + TypeScript as the engine center | Godot + custom content compiler | **Rust + `wgpu` custom runtime; browser for the Workbench only** | Both initial positions optimized the wrong boundary. The agent should know the authoring language, not the backend; the renderer is plumbing. A greenfield LLM-first runtime is more coherent than teaching Godot to tolerate a foreign government. |
| "Zero raster; everything is code" | Governed raster is fine | **Governed raster** | Raster did not cause the failures; ungoverned raster production did. The style compiler is the fence. |
| Byte-identical screenshot goldens | Tiered visual proof ladder | **Ladder** | Pixels move across machines; structure, statistics, perceptual diff on a pinned runner, then multimodal review. |
| "The compiler is the engine" | The normalized IR is the engine's center | **IR at the center** | Otherwise the compiler grows a renderer, a simulation, a network stack, and a small municipal government. |
| One grid layer for everything | Typed lattice + world graph | **Lattice + graph, one owner per fact class** | Ownership and quest state as voxels is conceptual tax fraud; two authorities without a constitution drift by Thursday. |
| Per-primitive emitters into every subsystem | A sealed capability basis | **Capabilities** | N primitives × M subsystems is a very long switch statement; ~12 capabilities × M subsystems is testable. The load-bearing abstraction. |
| Capabilities own consequences | Capabilities own contracts; projection compilers own consequences | **The latter** | Keeps the capability layer from becoming the same municipal government with a cleaner badge. |
| Flat capability bag | Typed namespaces with composition laws | **Namespaces + algebra** | A flat bag permits type-correct gibberish; overlapping claims need declared composition or become "which trait wins" soup. |
| One machine per entity (product states) | One machine per namespace, declared interactions | **Per-namespace machines** | Flattened products are a script in a stupid notation. Pure derived vs causal interactions; no machine writes another's state. |
| "Raw transforms should feel embarrassing" | No raw transforms in shippable content; tainted laboratory | **Gone from content; laboratory tainted** | An escape hatch an LLM can reach, it will reach at 2 a.m. |
| Procedural handles secondary variation broadly | Three kinds: derivation, bounded variation, open synthesis | **Derivation encouraged; variation sparingly; synthesis never** | Mostly terminology; deterministic derivation is semantic compilation, not oatmeal. |
| Lockstep deterministic simulation | Server-authoritative fixed tick | **Server-authoritative** | Persistent online game; clients render confirmed state. |
| Narrow client prediction | Zero authoritative prediction | **Zero** | Deliberate tempo; a prediction is a second clock with its own opinion about the goblin. |
| One universal IR as save/replay/net format | Versioned derived boundaries off the IR | **Derived boundaries** | The constitution should not regulate sewer pipe diameters. |
| One crate | One workspace, hard-bounded crates | **Workspace** | The server must never link `wgpu`; crate boundaries make the ownership rule physical. |
| Gate 0 is engine output | Gate 0 is one target image | Gate 0 is a **target pack** | Engine output is circular; one hero image hides the hard parts; a temporary renderer becomes the engine. |
| Hot reload behind a flag | Separate dev binary | **Separate binary, proven unlinked** | A flag is a code path; the release runtime must not contain the loader. |

## 21. Open questions

- The exact minimal orthogonal capability basis. Emerges from Gate 1; §7 is a
  candidate, not a decree.
- The composition-law catalog: which capabilities combine, which require
  exactly one owner, which conflict. Decided per capability with a rejection
  test each.
- Fixed-point precision and subcell resolution for moving actors.
- Whether the rendering projection is voxel-mesh (§17) for characters' local
  environment interactions (cover, climbing) or purely lattice-derived.
- Incremental compilation granularity: per room, per package, per primitive?
- The cold-author model roster and how often it rotates.
- Naming. The repository is `signed-world` for now; the wall sentence owns the
  idea, not the name.

## 22. Adoption criteria

This thesis applies to no project until all of the following are true:

1. The executable semantic kernel (see `KERNEL.md`) passes its acceptance
   criteria, including a cold author and a cold debugger.
2. Gate 0 has a human-approved target pack for the adopting game.
3. Gate 1's three primitives are proven end to end.
4. The adopting project records the adoption as an explicit decision, with the
   migration or epoch break it implies, in its own authority documents.

Until then: exploratory, non-authoritative, and — per its own rule — not to be
extended with further free-floating architecture until something compiles.
