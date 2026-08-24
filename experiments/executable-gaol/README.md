# Executable gaol experiment

This is the deliberately quarantined answer to issue #101: one room, one
camera, one small rendering plan, and visible semantic state. It is not Gate K
or Gate 1 evidence and it does not change the accepted workspace.

**Play online:** <https://conarylabs.github.io/nomos/>

Run:

```sh
experiments/executable-gaol/gaol capture
```

That command compiles `gaol.nomos`, executes four real Nomos command scripts,
derives `nomos.experiment.rendering_plan@1` only from subsystem projections and
runtime state, emits deterministic SVG frames, and rasterizes a PNG contact
sheet with `rsvg-convert` when available.

To use the interactive state and forensic-overlay controls:

```sh
experiments/executable-gaol/gaol serve
```

Open the printed local URL. The renderer receives only `rendering-plan.json`;
it does not parse `.nomos`, World IR, or compiler receipts. The second door is
ordinary content in `gaol.nomos` and requires no renderer change.

Use WASD or the arrow keys to cross the room. Walk beside `north_gate`, press
`E` to ignite it, press `E` again to unseal it, and cross the resulting opening.
After unsealing, walk beside `brazier_02` and press `E` to extinguish its bounded
amber light pool.
Those interaction edges are derived from consecutive, state-hash-bound Nomos
command logs rather than interpreted by the browser. Shallow water consumes the
projected movement cost of `3`; stone costs `1`. The north edge opens only at a
door whose selected Nomos runtime state resolves to `traversable`. Keys 1–4
switch the real runtime scenarios, `R` resets the presentation actor, and the
viewer interpolates movement without placing fractional positions into Nomos
authoritative state.

The internet build is the same static viewer staged without a running process:

```sh
experiments/executable-gaol/gaol site
```

The GitHub Pages workflow publishes that directory. No dev-machine port,
repository checkout, `.nomos` source, World IR, or credential enters the public
artifact.

Known limits: this is stylized deterministic SVG rather than a GPU renderer;
actor position is presentation-only because Gate K has no dynamic actor state;
audio, combat, networking, and persistence beyond the existing Gate K state are
absent. Its job is to make the semantic bridge and the room playable quickly
enough to learn from.
