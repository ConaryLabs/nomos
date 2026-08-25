# Executable gaol experiment

This is the deliberately quarantined answer to issues #101, #103, #105, #107,
#109, #110, and #113: four independently authored areas, one bounded look, one WebGL
renderer, a connected run, and visible semantic state. It is not Gate K or
Gate 1 evidence and it does not change the accepted workspace.

**Play online:** <https://conarylabs.github.io/nomos/>

Run:

```sh
experiments/executable-gaol/gaol capture
```

That diagnostic command compiles all four area sources, executes twenty real
Nomos command scripts, and then hands the result to the accepted Rust compiler
`nomos-render-plan` (issues #139 and #146), which derives four
`nomos.rendering_plan@2` artifacts from `nomos entity-catalog`, one `nomos
effective-facts` document per scenario, the run bundles, the four projection
members, and each area's `presentation.json` — and from nothing else. It then checks their exact shared visual grammar, emits
deterministic SVG frames, and rasterizes a cross-area PNG contact sheet with
`rsvg-convert` when available. SVG is retained as exact semantic/capture
evidence; it is no longer the playable presentation.

The plan compiler is the one part of this pipeline that is not quarantined:
`crates/nomos-render-plan` is a declared R1 member under `RUNTIME.md` section 3.
Its predecessor `src/build-plan.mjs` classified doors by
`machine.endsWith(".access")` and recomputed the movement and light resolvers in
JavaScript; it is deleted, and
`experiments/executable-gaol/compare-rendering-plan.sh` is the harness that
proved the replacement equal on all four areas before it was removed.

Content is typed too. `presentation.json` carries schema
`nomos.presentation_source@1`, decoded strictly by the same crate: versioned,
with every field set checked exactly, every identifier checked against a
declared grammar, and **no decimal anywhere** — heights are integer tenths of a
lattice cell and effects attach to a named socket rather than to a coordinate.
[AUTHORING.md](AUTHORING.md) is the packet; `src/renderer-catalog.mjs` is where
the renderer says what a socket, an assembly, or a family name means, which is
the half content may name but not define.

To use the interactive state and forensic-overlay controls:

```sh
experiments/executable-gaol/gaol serve
```

Open the printed local URL. The WebGL renderer receives only `areas.json`, the
four selected rendering plans, and presentation-only actor deltas; it does not
parse `.nomos`, World IR, or compiler receipts. Its pinned Three.js backend
supplies meshes, depth, shadows, fog, real point lights, and animated shader
water. North Gaol, Cistern Walk, Ember Vault, and Ossuary Reach use the same
camera, bounded palette, materials, assemblies, actor silhouettes, beveled
masonry vocabulary, effect language, and renderer — all of which the renderer
owns, so none of them appears in a content file. Their doors, water, light,
actors, wall height, masonry masses, and composition come from separate area
content.
See [AUTHORING.md](AUTHORING.md) for the intentionally small LLM authoring
packet.

The default `gaol_procedural_01` look is likewise renderer-owned: one bounded
profile controls palette roles, coarse deterministic stone/iron/cloth variation,
bevel treatment, actor silhouettes, exposure, and fog for every area. It uses no
bitmap texture or generated image asset. `Look: procedural` switches to the
untreated baseline in place, so visual iteration can compare the shared grammar
without changing or reloading area content.

Use WASD or the arrow keys to cross each room. Walk beside its primary gate,
press `E` to ignite it, press `E` again to unseal it, and cross the resulting
opening. After unsealing, walk beside the room's brazier and press `E` to
extinguish its bounded amber light pool.
Darkness wakes the gaoler: it advances by a deterministic presentation-only
rule every second successful move and catches the player on contact. Reach the
open gate before it does.
Those interaction edges are derived from consecutive, state-hash-bound Nomos
command logs rather than interpreted by the browser. Shallow water consumes the
projected movement cost of `3`; stone costs `1`. The north edge opens only at a
door whose selected Nomos runtime state resolves to `traversable`. Keys 1–5
switch the real runtime scenarios for inspection, and the viewer interpolates
movement without placing fractional positions into Nomos authoritative state.

Each area declares one bounded `exit_via` objective referencing its compiled
primary gate. The viewer derives the visible objective, nearby `E` prompt, and
open-passage guidance from that plan data and the verified interaction edges;
it contains no room-specific prompt text. Area arrivals identify route progress,
and the final escape reports cumulative areas, moves, and traversal cost.

The default run begins in Cistern Walk. Crossing its declared, traversable
sluice enters Ember Vault; crossing Ember's vault gate enters Ossuary Reach;
crossing the Bone Gate enters North Gaol; the North Gate is the final escape.
Area transitions preserve cumulative moves, water cost, and cleared-area count
while resetting area-local actors and runtime scenario selection. The area
buttons and bracket keys are forensic shortcuts that reset run progress; `R`
returns to the Cistern start.

The internet build is the same static viewer staged without a running process:

```sh
experiments/executable-gaol/gaol site
```

The GitHub Pages workflow publishes that directory. No dev-machine port,
repository checkout, `.nomos` source, World IR, or credential enters the public
artifact.

Known limits: this is a procedural WebGL visual proof rather than production
art; its pixels are not deterministic across GPU/driver combinations. Actor
positions, masonry-mass collision, and the gaoler pursuit rule are
presentation-only because Gate K has no dynamic actor or architecture state.
Audio, combat, networking, and persistence beyond the existing Gate K state are
absent. Its job is to test whether independently authored rooms can remain
visually coherent and playable through the same semantic bridge.
