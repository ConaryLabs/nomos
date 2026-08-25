# Executable gaol experiment

This is the deliberately quarantined answer to issues #101, #103, and #105:
three independently authored areas, one bounded look, one renderer, and visible
semantic state. It is not Gate K or Gate 1 evidence and it does not change the
accepted workspace.

**Play online:** <https://conarylabs.github.io/nomos/>

Run:

```sh
experiments/executable-gaol/gaol capture
```

That command compiles all three area sources, executes fifteen real Nomos
command scripts, derives three `nomos.experiment.rendering_plan@1` artifacts only from
subsystem projections and runtime state, checks their exact shared visual
grammar, emits deterministic SVG frames, and rasterizes a cross-area PNG contact
sheet with `rsvg-convert` when available.

To use the interactive state and forensic-overlay controls:

```sh
experiments/executable-gaol/gaol serve
```

Open the printed local URL. The renderer receives only `areas.json` and the three
selected rendering plans; it does not parse `.nomos`, World IR, or compiler
receipts. North Gaol, Cistern Walk, and Ember Vault use the same camera, bounded
palette, materials, assemblies, actor silhouettes, beveled masonry vocabulary,
effect language, and renderer. Their doors, water, light, actors, wall height,
masonry masses, and composition come from separate area content.
See [AUTHORING.md](AUTHORING.md) for the intentionally small LLM authoring
packet.

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

The default run begins in Cistern Walk. Crossing its declared, traversable
sluice enters Ember Vault; crossing Ember's vault gate enters North Gaol; the
North Gate is the final escape. Area transitions preserve cumulative moves,
water cost, and cleared-area count while resetting area-local actors and runtime
scenario selection. The area buttons and bracket keys are forensic shortcuts
that reset run progress; `R` returns to the Cistern start.

The internet build is the same static viewer staged without a running process:

```sh
experiments/executable-gaol/gaol site
```

The GitHub Pages workflow publishes that directory. No dev-machine port,
repository checkout, `.nomos` source, World IR, or credential enters the public
artifact.

Known limits: this is stylized deterministic SVG rather than a GPU renderer;
actor positions, masonry-mass collision, and the gaoler pursuit rule are
presentation-only because Gate K has no dynamic actor or architecture state;
audio, combat, networking, and persistence beyond the existing Gate K state are
absent. Its job is to test whether independently authored rooms can remain
visually coherent and playable through the same semantic bridge.
