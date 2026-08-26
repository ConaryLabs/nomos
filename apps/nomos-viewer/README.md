# The promoted viewer

Accepted R1 code under `RUNTIME.md` §5 R1-4 and §5 R1-5. It renders the six
`nomos.rendering_plan@3` artifacts the Rust compiler emits and fetches nothing:
the renderer is vendored in-tree and every URL the page constructs is a relative
path the artifacts declared.

It does not *play* them. R1-5 moved every rule about the run into
`crates/nomos-play`, compiled to `wasm32-unknown-unknown` and loaded by
`src/runtime.mjs`; this app turns a key into a `nomos.play_command@1` document,
hands it over, and paints the `nomos.presentation_state@1` that comes back. What
the player experiences is decided in Rust, over the kernel's own transactions,
and the smoke lane proves it by replaying the browser's session natively.

`docs/review/nomos-viewer.md` and `docs/review/nomos-play.md` are the design
records. Its §2 names, for every
promoted behaviour, the lines of `experiments/executable-gaol/` it reproduces
and the test that proves it — the study is the specification and the comparison
target, never a source of truth, and no file here was moved or copied from it.

## Layout

```text
index.html      the page; no colour value of its own, and an empty data icon
src/plan.mjs    the strict decoder, and the only module that builds a URL
src/catalog.mjs the renderer catalog: units, scale, camera, sockets, palette
src/play.mjs    the adapter: the key table, the command documents, the prose
src/runtime.mjs the loader for the authoritative runtime; no dependency
src/render.mjs  the WebGL renderer, over an injected Three.js namespace
src/ui.mjs      the DOM binding, and the pure readout it paints
vendor/three/   three@0.185.1, verbatim, with its licence and a digest manifest
build.mjs       stages dist/ from published artifacts, then scans it
smoke/          the headless Chromium lane; test tooling, never staged
test/           node's test runner; no dependency, no install step
```

## Running it

```bash
experiments/executable-gaol/gaol verify
crates/nomos-play/build-wasm.sh
cargo build --locked -p nomos-play
node apps/nomos-viewer/build.mjs \
  --from target/executable-gaol \
  --wasm target/wasm32-unknown-unknown/wasm/nomos_play.wasm \
  --out apps/nomos-viewer/dist \
  --receipt target/nomos-viewer-build/receipt.json
node --test apps/nomos-viewer/test/*.test.mjs
node apps/nomos-viewer/smoke/smoke.mjs \
  --dist apps/nomos-viewer/dist \
  --out target/nomos-viewer-smoke \
  --require-chrome
```

The first command builds and verifies the six-area artifact the viewer consumes.
The wasm build prints the runtime's size and SHA-256; the native build produces
the binary the smoke lane shells to replay what the browser did.
`test/runtime.test.mjs` drives the staged binary under Node and skips with a
message when `dist/` has not been built, so a failure has somewhere smaller to
be found than a headless run.

The smoke lane uses `CHROME_BIN` if it is set, then `google-chrome` or
`chromium` on `PATH`, and falls back to a Playwright cache only if one is
already present. With none of those it skips with an explicit message; in CI it
is required, and `--require-chrome` makes an absent browser a failure.

There is no `npm install`, no lockfile, and no bundler. Node's built-ins and the
two vendored Three.js modules are the whole executable dependency set.

## What it consumes, and what owns it

| Artifact | Identity | Emitted by |
| --- | --- | --- |
| `areas/<area-id>.json` | `nomos.rendering_plan@3` | `crates/nomos-render-plan`, accepted |
| `areas.json` | `nomos.area_collection@2` | `crates/nomos-render-plan`, accepted |
| `areas/<area-id>.simulation.json` | `nomos.projection.simulation@3` | `crates/nomos-compiler`, accepted |
| `nomos_play.wasm` | — | `crates/nomos-play`, accepted |

Every row is accepted output. The third is the executable semantics the runtime
needs to commit a kernel transaction at all; the plan beside it publishes that
projection's SHA-256, and the runtime refuses any bytes whose digest the plan did
not publish. The fourth is not a document and declares no identity: it is the
authoritative runtime itself, staged with its digest in the build receipt and
checked as a WebAssembly module rather than scanned as text, because a 400 KB
binary read as UTF-8 can match a credential shape by coincidence.

The collection row was
`nomos.experiment.area_collection@2`, declared by
`experiments/executable-gaol/src/build-collection.mjs` — quarantined tooling —
which the design record raised as finding 2 and issue #152 closed by promoting
the route graph into the compiler. This app refuses the retired identity by name.

The collection names each area's plan by file and by SHA-256.
`apps/nomos-viewer/build.mjs` checks that digest against the bytes it stages, so
the collection decides which bytes are an area's plan; the page itself does not
hash, because `crypto.subtle` is absent outside a secure context and a plan that
failed to publish is a build failure rather than a runtime one.

Both identities are bound before any field is read, and a mismatch, an unknown
field, a missing field, a fractional number at any depth, a name outside a
closed catalog set, or a cross-reference that does not resolve is refused with
an `NV####` code that the page renders as visible text.

## What it does not do

It performs no runtime network fetch, holds no area, entity, or actor
identifier, invents no prose from an identifier, and needs no edit to accept new
content: adding an area edits nothing here, which
`docs/review/nomos-viewer.md` §9 proves by adding one.
