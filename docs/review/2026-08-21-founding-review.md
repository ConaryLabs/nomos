# Founding adversarial review — 2026-08-20/21

The exchange that produced THESIS.md, verbatim. Two model families (Claude
Fable 5, via Claude Code; GPT Pro, via ChatGPT) with Peter Permenter relaying
and refereeing. Preserved because the reasoning is more durable than the
diagram. Light formatting only; no content edited.

Order: Peter's question → Fable 1 → GPT 1 → Fable 2 (vacuum reset) → GPT 2 →
Fable 3 → GPT 3 → Fable 4 → GPT 4 (final).

---

## Peter — the question

> while we wait, if you were making a game engine from scratch specifically
> for LLMs to use to their strengths to create a consistent look with the
> least amount of consternation, what technologies/approaches do you think
> would be ideally suited? was just perusing a guys custom voxel engine that
> uses webgl shaders and it got me thinking

## Fable 1

Fun question, and your two dead art directions are basically the data set for the answer.

**The core insight: LLMs are good at text that compiles and bad at pixels that match.** Every time we asked a model to *produce an image*, consistency died (20 whole-scene rolls, seam grading, the repaint chain). Every time we asked it to *write a program that produces the image*, consistency was free, because the program is the style. So the engine should be designed so that **100% of the look is authored as code/data the model writes, and 0% is authored as raster the model generates.**

What that implies:

1. **Procedural/shader-defined look, not asset-defined look.** That voxel-WebGL guy is onto the right thing for the wrong reason. Voxels are great not because they're retro but because a voxel world is *pure data* — a model can write `place(x,y,z, "oak_wall")` and the renderer guarantees it looks like every other oak wall. Same for SDF raymarching, tile grammars, or a flat-shaded low-poly kit with a fixed palette. The renderer is the art director; content is just nouns in the right slots.

2. **A tiny, closed vocabulary with a compiler that rejects anything outside it.** A palette of 32 colors, a material table, 6 light rigs, N named tiles. The model can't drift if the compiler won't accept drift. Fail-closed is the consistency mechanism, not taste.

3. **Declarative scene description → deterministic render.** JSON/TOML scene files, content-addressed, goldens diffed byte-for-byte. The model edits text, the test says "pixel-identical to golden or you explain why." That's a loop an LLM actually wins at; "does this look right" is a loop it loses.

4. **Shaders as the style layer.** Outline, dither, palette-quantize, fog, day/night grading — all post-process, all code, all version-controlled. Style changes are a one-file diff, never a re-render of 150 screens.

5. **Web stack or Godot, and I'd actually lean web for *this* engine.** WebGL/WebGPU + TypeScript: everything is text, hot-reloads, screenshots via headless Chromium in CI, no binary scene formats, no class-cache `--import` dance, and the model has seen a billion lines of it. For a shipped tactical RPG Godot still wins on packaging/input/platforms — but for an engine *built around LLM authoring*, web.

6. **Everything inspectable by file tools.** If the model can `cat` the state, it can reason about it. Scene graph, materials, render settings — plain files. Every pixel traceable to a line of data.

7. **Grammar before assets.** Where you *do* need bespoke art, the model writes the generator (an L-system, a tile ruleset, a parametric building), not the output. Wave-function-collapse with a hand-made tileset is the canonical example — a human makes 40 tiles once, the model writes adjacency rules forever.

So: **voxels or flat-shaded kit + fixed palette + post-process style stack + declarative scenes + golden-image CI + text-only authoring.**

The voxel guy's engine is a nice toy; the part worth stealing is "the world is a data structure and the look is a function of it." That's the whole thesis.

## GPT 1

Yeah. Fable is about **80% right**, and the missing 20% is precisely where a clever architecture turns into a six-month custom-engine hostage situation.

The central thesis is excellent:

> **The world is data, and the renderer is the art director.**

Where I differ is that I would build an **LLM-native visual compiler layered over a conventional runtime**. The renderer technology matters much less than the authoring surface, validation, and feedback loop exposed to the agent.

### What Fable got completely right

**Consistency should be enforced, not requested.** "Please make another room in the same style" is how you end up with fifteen cousins who vaguely resemble each other at a funeral. A proper system defines legal camera configurations, world scale, geometry vocabulary, palette families, material properties, light rigs, shadow treatment, fog behavior, outline rules, animation cadence, clutter density, allowed asset combinations. The agent selects from those systems. The renderer becomes a visual constitution. Amendments require ceremony.

**Semantic scenes beat arbitrary scene graphs.** The model should author something resembling:

```text
room "flooded_guard_post" {
    shape = rectangle(12, 8)
    floor = "worn_keep_stone"
    walls = "damp_masonry"
    ceiling = "low_vault"
    exits { north = door("iron_barred"); east = passage(width = 2) }
    lighting = "cold_underground"
    water_depth = 0.18
    dressing { kit = "abandoned_guard_post"; density = 0.30; seed = 18442 }
}
```

It should not author `place mesh_318 at (12.471, -0.022, 7.839)`. Raw transforms are an escape hatch for special cases. They should feel faintly embarrassing.

### Where I think Fable overcorrected

**"Zero raster" is unnecessary.** Raster itself didn't betray you. **Ungoverned raster production** did. Portraits, UI illustrations, decals, signs, spell icons, clouds, distant scenery are all reasonable raster assets. The important part is that every asset passes through a style compiler that can remap it to approved colors, enforce value ranges, normalize contrast, apply a common edge treatment, resize to legal dimensions, check alpha behavior, generate masks or normals, reject inappropriate detail frequency, attach semantic metadata. Generated imagery should be treated as **source material**. The governing rule: **Every visual input must compile into the project's visual language.**

**Raw shaders should not be the normal authoring interface.** Models can write shaders. They can also produce a perfectly valid shader that looks like someone smeared Vaseline across a haunted microwave. Everyday content work should use a **material and effect DSL** that compiles into shader parameters. Same distinction as CSS variables versus every page author implementing a browser compositor because the button needs to be blue.

**Byte-for-byte screenshot goldens are too brittle.** Byte-identical pixels across machines are fantasy bureaucrat bait. Visual testing should have layers: structural assertions; image statistics; perceptual screenshot comparisons on a fixed runner; multimodal review over beauty, material-ID, entity-ID, depth, normals, lighting-only, navigation, and collision buffers. Goldens are useful receipts. They are not taste.

**A closed vocabulary needs controlled expansion.** Thirty-two colors and twelve wall pieces eventually produce a world resembling one very long airport bathroom. The vocabulary should be **small but parameterized** (construction period, wealth level, moisture, damage, local mineral tint, ornament family, repair history), and new primitives go through a promotion process: prototype → fixture sheet → validate under all light rigs → compare against references → approve → stable ID → catalog.

### The representation I would choose

**Semantic voxels or a grid-based volume model internally, rendered as stylized meshes rather than cubes.** Discrete coordinates, predictable adjacency, easy room construction, trivial collision foundations, understandable diffs, deterministic meshing, excellent LLM authorability — without Minecraft chic. **Voxel authoring, mesh rendering.**

### The most important feature: explainable pixels

```text
estate explain-pixel scene/flooded_guard_post 612 344
→ { "entity": "north_wall_04", "source": "rooms/flooded_guard_post.estate:14",
    "primitive": "masonry_wall", "material": "wet_keep_stone",
    "light_rig": "cold_underground", "decorator": "dampness_pass", "seed": 18442 }
```

Today, when a generated image contains an ugly wall, the reason is unknowable. In this system, the wall is ugly for a nameable reason, fixable as code or data. No séance required.

### The agent feedback loop

A small command surface — `validate`, `compile`, `render --preset`, `inspect`, `diff --against approved`, `explain-pixel`, `propose-promotion` — every command returning structured JSON plus artifact paths, every render producing a contact sheet.

### Where procedural generation belongs

Execution and secondary variation, not creative decisions. Wave-function collapse is useful for filling controlled gaps. Letting it design important maps usually produces technically valid oatmeal.

### Web, Godot, or a custom Rust renderer

**For the actual game: Godot plus a custom content compiler.** The LLM never authors `.tscn` directly. Godot provides the boring, expensive things. Godot's true `--headless` mode disables rendering, so visual tests need an offscreen or virtual-display arrangement.

**For a genuine custom engine: Rust + `wgpu`, with a browser Workbench.** I would not make the browser the architectural center of the shipped game merely because models have seen lots of TypeScript. That is optimizing the wrong boundary.

### The first experiment

One constrained kit: one dungeon construction family, ~20 props, three light rigs, one water and one fog treatment, one fixed camera, one body rig, a small equipment set, one post-process stack. Then twenty rooms with distinct purposes. The test: **Can the system produce twenty clearly different places that unmistakably belong to the same game, without editing renderer code or inventing new visual vocabulary?** Until that works, building a complete custom engine is premium-grade procrastination wearing a graphics-programming hat.

**Excellent governing idea. Too absolutist about raster. Too enthusiastic about browser primacy. Too trusting of pixel goldens.**

## Peter — reset

> hold. i don't want you counting what we've already done in this project. i
> mean clean break, best approach in a vacuum.

## Fable 2

Vacuum, no sunk costs, no rulings.

**GPT is right about the representation, and I'd go further than it did.** Voxel-authored, mesh-rendered is the correct core. But I wouldn't stop at voxels — I'd make the *entire* world a **typed semantic grid with a compiler**, and voxels are just the geometry layer of it. Every layer the agent touches is discrete, addressable, diffable. One coordinate system for everything. The agent never thinks in floats.

**Where GPT is still too conservative: it's building a visual compiler *next to* a conventional engine. In a vacuum I'd make the compiler the engine.** The renderer is ~10% of a game engine; the other 90% is exactly where "bolt a compiler onto Godot" starts fighting the host. Godot wants scene trees and signals; an LLM wants a single typed data model and a CLI.

- **Rust + `wgpu` runtime, one crate.** Deterministic, compiles WASM for the browser Workbench and native for shipping. One renderer, zero drift.
- **Lockstep simulation.** Fixed tick, integer math where it matters, seeded RNG, replayable from a log. A test is a log + expected state hash.
- **Everything is a typed file in a content-addressed store.** No binary resources. The model's whole world is `ls`, `cat`, `diff`, and a CLI.

**The agent-facing surface is the product; the renderer is plumbing.** A small DSL with a fail-closed compiler; forensic renders with `explain-pixel` as table stakes; a tiered proof ladder.

**Where I'd overrule GPT:**
- **"Raw transforms should feel embarrassing" → make them impossible.** An escape hatch an LLM can reach is an escape hatch it will reach at 2 a.m. to make a test pass.
- **Procedural gets *less* than it proposed.** Roof geometry, water edges, foliage are taste decisions.
- **Raster as "source material" — yes, but the style compiler must be the only path in, and it should quantize aggressively.**

**The representation I'd pick for a tactical RPG:** fixed orthographic/oblique camera, voxel-grid world, mesh-rendered with heavy stylization, vertex colors + tiny atlas, socketed rigs, small shared animation library. Pixel-native 2D is a worse fit for *this* engine — it's raster by definition, so the best authoring affordance is forfeited for the one layer that matters most.

**The first experiment is GPT's, verbatim.** If that takes more than a few weeks on the custom stack, the custom stack was the wrong call and Godot-plus-compiler is the fallback.

Wall sentence: *the agent writes the world, the engine owns every pixel, and nothing reaches the screen that the compiler didn't sign.*

## GPT 2

Okay. Fable moved me. In a genuine vacuum, starting with Rust + `wgpu` is more coherent than teaching Godot to tolerate a foreign government inside its borders. But I'd change the central claim:

> **The compiler is not the engine. The semantic IR is the engine.**

The compiler creates it. The server executes it. The renderer draws it. The Workbench explains it. Save files, replays, networking, tests, and diagnostics all speak it. Otherwise "compiler as engine" eventually becomes a compiler containing a renderer, a simulation, an audio system, a network stack, and a small municipal government.

### The real synthesis: an affordance compiler

In a conventional engine, a door is a mesh, a transform, a collider, a nav obstacle, an animation, a script, audio events, replicated state, saved state, an interaction prompt — eight opportunities for an agent to forget something and produce a door that looks convincing but can be walked through, cannot be opened, screams continuously, or respawns locked every restart. In this engine the agent writes:

```text
door north_gate {
    anchor = face(cell(5, 0, 0), north)
    archetype = iron_barred
    initial_state = closed
    lock = key(gaoler_key)
    persistence = world
}
```

and the door primitive compiler emits every consequence: visual, collision, navigation, animation, interaction, audio, simulation, network, persistence, diagnostics. **The agent declares an affordance. The compiler owns every system-level consequence.** A semantic primitive is simultaneously visual, physical, behavioral, networked, persistent, and inspectable.

### The world should be a lattice **and** a graph

Spatial facts belong on a common lattice (`cell`, `face`, `edge`, `region`, `socket`, `path`, `room`). Relational and temporal facts belong in a graph. "The gaoler owns this key" as a material cell would be conceptual tax fraud. Stable entity IDs connect the two. The runtime may use fixed-point for moving actors and floats for rendering; the authoring language never exposes either except wrapped in bounded semantic types.

### No raw transforms in shipping content

Here I mostly concede. Not "discouraged." Gone. Authored placement uses cells, faces, edges, sockets, regions, discrete orientation enums, elevation steps, approved footprints, rational subcell anchors where absolutely necessary. There still needs to be an unsafe primitive-development laboratory; anything authored through it receives a build taint — `NON_SHIPPABLE` — and cannot enter a release package.

### Fable's procedural restriction is too broad

Three different things are called "procedural": **deterministic derivation** (a declared gabled roof or water region produces its only legal geometry — exactly what the engine should do); **bounded seeded variation** (approved rules choose among legal alternatives — fine for secondary texture); **open-ended synthesis** ("fill this room with whatever looks ruined" — absolutely not; that is where oatmeal gets tenure).

### Fixed-tick deterministic simulation: yes. Client lockstep: no.

Server-authoritative, fixed-tick. Typed commands, validated and executed at deterministic ticks. Integer grid coordinates, fixed-point subcell positions, stable entity ordering, canonical serialization, deterministic scheduling and tie-breaking. Avoid a single global seeded RNG — a harmless new random call in ambient sound should not alter next Tuesday's critical hit; use keyed streams from `(world_seed, tick, system_id, entity_id, event_sequence)`. A replay is a manifest, an initial snapshot, an ordered command log, and periodic expected hashes.

### Content-addressed builds, human-readable sources

**Symbolic IDs in source. Content addresses in compiled packages.** Don't make agents edit references that resemble the serial number from a stolen catalytic converter.

### One repository, absolutely not one crate

Fable's "one crate" is where I throw a shoe. One Rust workspace with hard dependency boundaries: `core`, `schema`, `compiler`, `sim`, `render`, `audio`, `net`, `cli`, `workbench`, and the game crate. The server should not compile or link `wgpu`.

### Do not build a general engine

Deliberately omit general rigid-body physics, an arbitrary scene tree, user-authored runtime scripts, a universal material graph, an editor that mutates canonical content, an open plugin system. Architecturally, it should be **the Mortal Estate runtime**, built around one game's ontology.

### Forensics should extend beyond pixels

`explain-entity`, `why-blocked`, `why-visible`, `trace-event`, `explain-material`, `explain-save`, `explain-replication`. Conventional debugging tools expose implementation state. This engine exposes **semantic causality**.

### I'd strengthen the first experiment

Three cross-system primitives — a door, water, a light source — each proven to alter every subsystem from one declaration. Then: twenty rooms from one kit, no renderer or catalog edits, no arbitrary transforms, one semantic edit produces all cross-system changes, replay to identical hashes, everything traceable, browser and native renderers consume the same package.

Revised wall sentence: **The agent declares intent. The compiler owns the consequences. The runtime executes a signed world. The renderer owns every pixel.**

## Fable 3

GPT moved, and moved well. I concede more than I hold this round — and I've got one structural addition it's missing.

**Conceded outright:** IR at the center; lattice + graph; workspace not crate (shoe deserved); server-authoritative fixed tick, keyed RNG, symbolic IDs / content addresses, game-specific runtime.

**The addition — the affordance compiler needs a capability layer or it eats itself.** If every primitive independently emits into eight subsystems, the catalog becomes the municipal government GPT warned about: fifty primitives × eight subsystems = four hundred hand-written emitters. The fix is one level down: a **small closed set of capabilities** — `Occludes`, `Blocks`, `Portal`, `Stateful(machine)`, `Persisted(scope)`, `Replicated(fields)`, `Interactable(verbs)`, `Emits(audio|light)`, `Traversable(cost)` — and each subsystem emits *per capability*, never per primitive. A door is `Portal + Stateful(open|closed|locked) + Persisted(world) + Interactable(open,close,unlock) + Emits(iron_door)`. New primitive = new bundle, zero new emitters. That's the difference between an engine and a very long switch statement.

**Two pushbacks:** the lattice/graph seam needs a single owner per fact class, enforced by a check with a mutant receipt; and for a deliberate-tempo tactical RPG, client prediction is zero, not "narrow" — the moment you predict, you've built a second clock.

**Criterion 8:** *a second, cold agent given only the docs and the CLI authors room #21 and passes the same gates.* The engine's thesis is that an LLM is the primary author; the test must include an author that wasn't in the room when the engine was built.

**Lessons that survive a clean break:** the proof loop's latency is an architectural property; provenance is tainted by process, not just payload; art directions die on taste, not tooling, so the taste gate comes before the twenty rooms; budgets are measured, not assumed; nothing is done until someone other than the author re-runs the proof.

Revised: *The agent declares intent. Capabilities own the consequences. The runtime executes a signed world. The renderer owns every pixel. A stranger can reproduce all of it from the docs.*

## GPT 3

Yeah. Fable won the capability-layer point decisively. That is the missing load-bearing abstraction. One correction to the wording:

> **Capabilities own contracts and obligations. Subsystem projection compilers own the consequences.**

A `Portal` capability should not contain rendering code, navigation code, collision code, persistence code, and networking code. It declares a typed fact. The render, simulation, navigation, persistence, and network compilers each understand that fact and project their own artifacts.

### The canonical pipeline

Semantic source → primitive expansion → capability resolution → fact ownership + cross-reference validation → state-space compilation → **Canonical World IR** → simulation / navigation / rendering / audio / persistence / networking / diagnostic projections. Every subsystem consumes that model. None independently reinterprets the source — letting the renderer and simulator parse the room DSL separately would create two religions within about eleven commits.

### Sealed capabilities, extensible primitives

Engine capability basis (tiny, closed, heavily tested) → approved primitive catalog (expandable through promotion) → world content (instantiates approved primitives). An ordinary room author cannot accidentally omit persistence, navigation, or collision because those decisions belong to the primitive definition. **The capability basis is sealed. The primitive vocabulary is extensible. Content uses nouns.**

### The capability layer cannot be a flat bag

A flat component soup permits technically type-correct gibberish such as a replicated audio emitter that owns a key and occludes swimming. Divide into typed namespaces: `topology`, `affordance`, `state`, `lifecycle`, `presentation`. The exact basis should emerge from the vertical slice. Absence should mean absence: `Occludes(none)` is suspicious. Negative traits turn into three-valued logic and, eventually, a small demon.

### State-indexed capabilities are the real trick

A door does not simply "have" `Blocks`. It blocks in some states and not others. The compiler resolves capabilities against canonical state, makes transitions explicit, and precomputes the subsystem deltas for every legal transition. The runtime executes a compiled transition table, not an arbitrary door script.

### The lattice/graph seam gets a fact-ownership constitution

Every fact class has one declared canonical owner; the linker owns resolved bindings; the compiler fails on duplicate authorities, dangling IDs, relations in cells, transforms posing as truth, graph contradicting lattice, illegal capability combinations, derived facts supplied by content. The build produces a fact-ownership receipt. Diagnostics are structured with stable codes, source spans, and legal repairs. "Invalid scene" is not a diagnostic. It is the compiler sulking.

### Zero authoritative client prediction

Fable is right for this game. Commands are acknowledged; clients interpolate between confirmed states; no speculative movement, damage, or inventory. A fixed-pulse design turns latency from an embarrassing implementation detail into part of the interaction grammar.

### The proof loop needs its own architecture

**A content edit must never require compiling the engine.** Three lanes — content, catalog, engine — enforced physically by the dependency graph and cache, with budgets (`validate_room_p95_ms`, `compile_room_p95_ms`, `render_contact_sheet_p95_ms`, `full_build_time_s`, …) in source control. "Seems fast enough" has already had several opportunities to stab this project and has used all of them.

### Provenance belongs in the IR too

A hash proves identity; it says nothing about lineage. Every source node carries a provenance block with zone, origin, license, derivation, and permitted builds. Taint propagates through derivation; the release linker refuses forbidden zones. It cannot prove what once passed through a human brain, because reality remains offensively analog.

### The visual test starts with one approved room

Gate 0 visual thesis (a human says yes) → Gate 1 system thesis (door, water, light) → Gate 2 vocabulary thesis (twenty rooms) → Gate 3 cold-author thesis (a different model family authors room 21 from the docs — an "LLM-native" language that works only for the model whose fingerprints are all over it is a private dialect with a marketing department) → Gate 4 cold-debug thesis (seed a defect; a cold agent repairs it with the forensic tools) → Gate 5 reproducibility thesis (a clean machine rebuilds and matches). Nothing is green until a stranger reproduces it. Annoying, cheap, and apparently necessary because software enjoys lying.

### The signed-world package should be literal

A release package carries the IR, projection artifacts, source maps, dependency graph, provenance closure, all versions and hashes, state-machine definitions, validation receipt, and optionally a signature. The release runtime refuses unresolved source, unpromoted primitives, laboratory transforms, prohibited provenance, incompatible schemas, missing artifacts, failed invariants, modified packages.

Wall sentence: **The agent names the thing. The primitive resolves its capabilities. Capabilities define the obligations. Projection compilers own the consequences. The runtime executes a sealed world. The renderer owns every pixel. A cold stranger can reproduce and explain all of it.**

## Fable 4

Converged. Four additions, then one practical suggestion.

**1. State-indexed capabilities will explode unless machines stay per-namespace.** A door that is also damageable and burnable is a product state (3 × 4 × 2). If the transition compiler flattens products, the table becomes the new script. Rule: one machine per capability namespace, declared interactions between machines only, and the compiler rejects undeclared cross-machine effects.

**2. The IR is an external boundary on day one, even with zero users.** Version it from the first commit, ship a migration tool with it, and make "IR changed without a migration" a build failure. The one place where "no compatibility layers during prototype" does *not* apply.

**3. Gate 0 must not be engine output.** The only way to make a hero room before the engine exists is a throwaway renderer — which then becomes the engine, because it always does. So Gate 0 is a *target image* made in anything and the engine's later job is to hit it.

**4. Hot-reload is a dev-only binary, not a flag.** Separate crate, separate binary; the release runtime has no code path that loads an unsigned package.

**The cold-author gate should use a different model family — and so should design review.** Which is what this thread has been.

**Practical suggestion:** this is now a coherent thesis, and it's in a chat. Durable records over chat. Write it up as a standalone design document with both contributors' positions merged and the disagreements recorded.

Revised: *The agent declares intent. Capabilities own the consequences. The runtime executes a signed world. The renderer owns every pixel. A stranger can reproduce all of it from the docs.*

## GPT 4 — final

Yep. Converged. Fable's four additions are solid. One real correction and one final structural piece.

### 1. Per-namespace machines are right — but they need a resolver algebra

Independent machines will emit overlapping capability claims, so each capability type needs a declared composition rule — `Blocks` = any_active_claim, `Traversable` = minimum_legal_cost, `Occludes` = bounded_sum, `EmitsAudio` = union, `Authority` = exactly_one, `Persisted` = compatible_union_or_error. Otherwise we escaped state explosion only to create "which trait wins?" soup.

Machine interactions have two forms. **Pure derived** (`portal_open = access == open OR integrity == destroyed` — destroying the door does not lie by changing `access` to `open`) and **causal** (`combustion.burning -> integrity.apply_damage(fire, 2)` — no machine directly writes another's state; it emits a typed event). Every external command resolves as one deterministic state transaction: apply local transition → emit effects → evaluate interactions in fixed phase order → resolve claims → reject conflicts/cycles/ambiguous writes → commit atomically → emit deltas and a causal receipt. The compiler rejects cycles unless a capability defines fixed-point behavior; the runtime keeps a transition budget, because eventually somebody will build a cursed torch that lights itself when extinguished.

> **Machines own state. Capability resolvers own effective facts.**

### 2. Version the canonical IR immediately — but do not make everything literally speak it

The Canonical World IR should **not** also be the save format, replay format, network protocol, renderer package, and live simulation layout. Adding a renderer-only field would force a network migration. Congratulations, the constitution now regulates sewer pipe diameters. Use a chain of explicit versioned representations: authoring source schema → Canonical World IR → projection IRs; runtime state schema → save / replay / network protocol schemas. Every checked-in schema has a version; every artifact names its schema; every incompatible change is a migration or a recorded epoch break; no compatibility is implied merely because two structures happen to deserialize. Capabilities remain primarily a compile-time semantic language; the runtime need not carry a philosophical `Blocks` object around for emotional support.

### 3. Gate 0 should be a target pack, not one target image

One gorgeous hero image conceals every difficult part. The pack: hero frame, gameplay frame at actual camera distance, actor scale and silhouette references, a combat frame with overlapping actors, one spell effect, one low-light variant, one UI overlay, a material and palette sheet, a motion timing strip, explicit visual invariants — and what *doesn't* need to survive, or the renderer team spends three weeks reproducing a decorative crack nobody cared about. Any code used to make it lives in quarantine; the temporary renderer does not graduate by squatting there long enough.

### 4. Hot reload as a separate binary is exactly right

`estate-devd` vs `estate-runtime`. The release runtime should not merely disable hot reload with a flag; it should not link the machinery. A valid signature alone is insufficient — policy also verifies versions, provenance closure, promotion state, schema compatibility, invariant receipts, and the absence of development taints. Otherwise a beautifully signed pile of debug garbage remains garbage with excellent paperwork.

### Cold-model review is design fuzzing

> Model diversity is a fuzzing strategy, not a decision authority.

The review packet is blind. The reviewer hunts hidden coupling, unowned facts, impossible migrations, capability overlap, cycles, escape hatches, performance traps, unverifiable claims, and places where the design requires taste but pretends to require logic. Then the human decides. Models are excellent at attacking textual systems. They are also capable of objecting to gravity if given enough tokens.

### Yes, this should become a durable thesis now

Put it somewhere deliberately non-authoritative with painfully obvious front matter. Record the disagreements: browser-first → custom runtime; compiler-at-center → IR-at-center; primitive emitters → capability projections; one grid → lattice plus graph; lockstep → server-authoritative pulses; flat state products → namespace machines; one universal IR → versioned derived boundaries. The reasoning is more durable than the final diagram.

### The next artifact should not be a renderer

Build an **executable semantic kernel** before touching `wgpu`: one room, one door, one water region, one extinguishable light, versioned IR, namespace-local machines, capability resolution, three or four projections as JSON, deterministic command execution, state hashes, causal receipts, one migration, `explain-entity`, `explain-transition`, mutation tests. No graphics. If that kernel becomes ugly, the renderer would merely conceal the corpse beneath some lovely palette quantization.

Final wall sentence:

> **The agent names the thing. Namespaces own state. Capabilities define obligations. The resolver composes effective facts. Projection compilers own the consequences. The runtime executes a sealed world. The renderer owns every pixel. A cold stranger can rebuild and explain all of it.**

That's the design. Have Fable draft the thesis. This conversation has reached the point where more free-floating architecture would start inventing decorative complexity simply because nobody has made it compile yet.

## Peter — disposition

> i'm hesitant to let cairn's conventions and validators impact this thesis
> too much. this might be better off as a separate project that converges
> with the mortal estate/cairn at a later time?

Ruled: separate repository, MIT license, working name `signed-world`.
