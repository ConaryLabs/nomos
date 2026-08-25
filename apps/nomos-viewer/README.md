# The promoted viewer

Accepted R1 code under `RUNTIME.md` §5 R1-4. It renders the four
`nomos.rendering_plan@2` artifacts the Rust compiler emits, plays them, and
fetches nothing: the renderer is vendored in-tree and every URL the page
constructs is a relative path the artifacts declared.

`docs/review/nomos-viewer.md` is the design record. Its §2 names, for every
promoted behaviour, the lines of `experiments/executable-gaol/` it reproduces
and the test that proves it — the study is the specification and the comparison
target, never a source of truth, and no file here was moved or copied from it.

## Layout

```text
index.html      the page; no colour value of its own, and an empty data icon
src/plan.mjs    the strict decoder, and the only module that builds a URL
src/catalog.mjs the renderer catalog: units, scale, camera, sockets, palette
src/play.mjs    movement, cost, interaction, pursuit, arrival, completion
src/render.mjs  the WebGL renderer, over an injected Three.js namespace
src/ui.mjs      the DOM binding, and the pure readout it paints
vendor/three/   three@0.185.1, verbatim, with its licence and a digest manifest
build.mjs       stages dist/ from published artifacts, then scans it
smoke/          the headless Chromium lane; test tooling, never staged
test/           node's test runner; no dependency, no install step
```

## Running it

```sh
experiments/executable-gaol/gaol capture
node apps/nomos-viewer/build.mjs --from target/executable-gaol --out apps/nomos-viewer/dist
node apps/nomos-viewer/smoke/smoke.mjs --dist apps/nomos-viewer/dist --out target/nomos-viewer-smoke
node --test apps/nomos-viewer/test/*.test.mjs
```

The smoke lane uses `CHROME_BIN` if it is set, then `google-chrome` or
`chromium` on `PATH`, and falls back to a Playwright cache only if one is
already present. With none of those it skips with an explicit message; in CI it
is required, and `--require-chrome` makes an absent browser a failure.

There is no `npm install`, no lockfile, and no bundler. Node's built-ins and one
vendored file are the whole dependency set.

## What it consumes, and what owns it

| Artifact | Identity | Emitted by |
| --- | --- | --- |
| `areas/<area-id>.json` | `nomos.rendering_plan@2` | `crates/nomos-render-plan`, accepted |
| `areas.json` | `nomos.experiment.area_collection@2` | `experiments/executable-gaol/src/build-collection.mjs`, **quarantined tooling** |

The second row is recorded rather than hidden: the four plans are accepted
output and the file that stitches them into a route is not. The owner ruled that
acceptable for R1-4 and issue #152 carries promoting the collection into
`nomos-render-plan`, which is what will retire the row.

Both identities are bound before any field is read, and a mismatch, an unknown
field, a missing field, a fractional number at any depth, a name outside a
closed catalog set, or a cross-reference that does not resolve is refused with
an `NV####` code that the page renders as visible text.

## What it does not do

It performs no runtime network fetch, holds no area, entity, or actor
identifier, invents no prose from an identifier, and needs no edit to accept new
content: adding an area edits nothing here, which
`docs/review/nomos-viewer.md` §9 proves by adding one.
