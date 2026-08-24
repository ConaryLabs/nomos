# Executable gaol experiment

This is the deliberately quarantined answer to issue #101: one room, one
camera, one small rendering plan, and visible semantic state. It is not Gate K
or Gate 1 evidence and it does not change the accepted workspace.

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

Known limits: this is stylized deterministic SVG rather than a GPU renderer;
actors do not move; audio, animation, networking, and persistence beyond the
existing Gate K state are absent. Its job is to make the semantic bridge and
the room visible quickly enough to learn from.
