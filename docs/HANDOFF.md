# Nomos handoff

## Current state

Gate K is closed. Decision 0013 remains its controlling verdict: **failed**.
Decision 0016 terminates the separately governed round two incomplete, with no
round-two verdict and no remaining checker, audit, retry, or evidence-assembly
work authorized.

The kernel workspace implements the renderer-free semantic system through
SW-N. The active product direction is the owner-authorized quarantined study at
`experiments/executable-gaol/`: four independently authored areas, one
projection-only WebGL renderer, one bounded procedural look, and a connected
playable route. It is not Gate K or Gate 1 evidence.

**Play online:** <https://conarylabs.github.io/nomos/>

## Gate K record

- Round one is preserved at annotated tag `gate-k-rc1`; its exact 1–19 matrix
  is in `docs/decisions/0013-gate-k-disposition.md`.
- Round two's mechanically proven candidate is preserved at annotated tag
  `gate-k-rc2`, commit
  `53db236d397b3db0779f0d2aab23180d926e55a5`.
- The round-two Gemini author subject completed with subject-stage outcome
  `eligible-for-checker`, but no checker adjudicated it.
- The round-two DeepSeek debugger subject is preserved, unmerged, at annotated
  tag `gate-k-rc2-debug-subject-incomplete`, commit
  `55bb77bf4221c2c5600cd20bb781c0018a6d40a8`. Its non-author audit and checker
  did not complete.
- Neither round-two subject is a formal pass. Round two has no acceptance tag,
  criteria matrix, or overall verdict.
- No protocol revision 7, Gate K retry, or round three is authorized.

The large `docs/evaluation/runs/` tree and archival tags intentionally preserve
the exact packets, transcripts, receipts, binaries, rehearsals, blocked audits,
and failed attempts behind historical claims. They are evidence, not active
product work or disposable build output.

## What works

The kernel provides source parsing, stable World IR, deterministic
simulation/navigation/persistence/diagnostic projections, immutable runtime
transactions, hash-verified packages, replay and migration, and package-bound
explanations.

`nomos effective-facts <world/> --state <state.json>` adds the R1-1 read-only
projection: given a strictly verified package and a runtime state it emits
`nomos.effective_facts@1`, the composed movement disposition, cost, ordered
reasons, and effective light for every resolver subject, resolved entirely by
the existing `resolve_movement` and `resolve_light` rather than by new logic.

`nomos entity-catalog <world/>` adds the read-only catalog of what a compiled
world *contains*: for every entity it emits `nomos.entity_catalog@1`, carrying
the World IR primitive kind and `expansion.capabilities` beside the simulation
projection's binding and machines and the movement and light resolver claims
with their source spans. It classifies nothing and reads no `.nomos` source, so
a downstream compiler no longer has to infer an entity's kind from a naming
convention.

`nomos-render-plan --catalog <entity-catalog.json> --facts <dir> --runs <dir>
--world <world/> --source <presentation.json> --out <plan.json>` is the R1-2/R1-3 compiler: it
turns those two read-only projections, the per-scenario run bundles, the four
projection members' identities and digests, and the presentation source into
`nomos.rendering_plan@1` as canonical bytes. It is the first declared R1 member,
depends on `nomos-core` alone, and replaces
`experiments/executable-gaol/src/build-plan.mjs`.

`nomos-render-plan collection --plans <dir-or-plan> --out <areas.json>` is the
same binary's second mode, added by issue #152: it reads the compiled plans and
emits `nomos.area_collection@1` — the route chain, the visual grammar every area
shares, and one row per area naming the published plan file and its SHA-256. It
replaces `experiments/executable-gaol/src/build-collection.mjs`, which was the
last identity accepted code bound whose declaration lived under `experiments/`.

The executable study provides:

- Cistern Walk, Ember Vault, Ossuary Reach, and North Gaol as separately
  authored content;
- shared camera, palette, materials, assemblies, actor silhouettes, masonry,
  effects, water, lighting, and minimal UI;
- content-derived doors, movement cost, effective light, objectives, and a
  connected escape route;
- deterministic semantic/SVG evidence plus a WebGL presentation layer;
- forensic overlays and a procedural-versus-baseline comparison;
- a static public build containing no source, World IR, credentials, or
  dependency on the development machine.

Actor movement and gaoler pursuit remain presentation-only because Gate K has
no dynamic actor state. Audio, combat, networking, and the later cross-system
Gate 1 contract remain absent.

## How to verify

From a clean checkout, verify the kernel workspace with:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

Verify the executable study with:

```text
experiments/executable-gaol/gaol verify
```

Build and prove the promoted viewer with:

```text
node --test apps/nomos-viewer/test/*.test.mjs
node apps/nomos-viewer/build.mjs --from target/executable-gaol --out apps/nomos-viewer/dist
node apps/nomos-viewer/smoke/smoke.mjs --dist apps/nomos-viewer/dist --out target/nomos-viewer-smoke
```

The last one needs a Chrome. It uses `CHROME_BIN` if that is set, otherwise
`google-chrome` or `chromium` on `PATH`, and falls back to a Playwright cache
only if one happens to be present. With none of those it skips with an explicit
message; in CI it is required.

## What is next

Improve the shared look and presentation, then prove that additional
content-authored areas fit it without renderer-specific edits. Keep those
iterations inside the quarantined executable study until concrete evidence
justifies a separate decision to promote a boundary into a post-Gate-K runtime
epoch.

That separate decision is now owner-authorized:
`docs/decisions/0017-post-gate-k-runtime-epoch.md` opens the R1 epoch under
issue #124. Its contract document `RUNTIME.md` is owner-authorized under issue
#128 and now governs what R1 accepts. R1-1 is accepted on PR #130 (issue #126),
and its identity is the first row of `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.

R1-2, Rust rendering-plan compilation, is accepted under issue #139.
`crates/nomos-render-plan` is the first declared R1 member: it compiles
`nomos.rendering_plan@1` from `nomos.entity_catalog@1`, one
`nomos.effective_facts@1` document per scenario, the run bundles, four
projection members, and `presentation.json`, and it opens no `.nomos` source, World IR,
or compiler receipt. `experiments/executable-gaol/src/build-plan.mjs` is
deleted, and `experiments/executable-gaol/gaol` runs the Rust binary. For all
four areas the Rust plan equalled the JavaScript fixtures under one documented
normalization — `experiments/executable-gaol/compare-rendering-plan.sh` reports
`4 areas compared, 0 differences` — and every SVG frame and `contact-sheet.png`
is byte-identical across the switch except the four `forensic.svg` overlays,
which print the plan's own identity. `docs/review/rendering-plan-compiler.md` is
its design record.

**R1-3, typed presentation source, has landed** (issue #146, PR #147). Each
area's `presentation.json` carries `nomos.presentation_source@1`: versioned,
closed against unknown fields, integer-only by the type its reader parses into,
with attachment by named socket instead of by coordinate and each area owning
its own arrival cell. The plan is `nomos.rendering_plan@2`, emitted through
`nomos_core::CanonicalValue` — which retires issue #144, since the private
encoder in `src/doc.rs` and its decimal type are deleted. The ownership audit's
69 rows each have one owner and its 61 double-authority, convention-derived, and
floating-point rows are dispositioned in `docs/review/presentation-source.md`,
which is its design record. Seven rows are deferred with a named slice: three to
R1-4 and four to R1-5.

**R1-4, the promoted viewer, has landed** (issue #148, PR #151).
`apps/nomos-viewer/` is a clean implementation - no file moved or copied - with
a strict decoder for `nomos.rendering_plan@2`, a vendored `three@0.185.1`
recorded under `RUNTIME.md` section 4, a `dist/` staged from published artifacts
and scanned before it is published, and a dependency-free headless Chromium lane
that plays the four-area route to the final escape and fails on a single console
error. `docs/review/nomos-viewer.md` is its design record, and its section 2
names the study lines each promoted behaviour reproduces and the test that
proves it.

It resolved the deferrals R1-3 named. The kind-to-assembly and kind-to-material
tables became the catalog's definition of what those names mean, with the
compiler's table held to it by a test and issue #153 carrying the move out of
Rust; `displayName()` is gone, because prose now comes only from `area.label`,
the app's own strings, and tables keyed by closed sets the schema declares;
sockets resolve through the entity's declared face; and the two colour tables
are one palette that the page reads too.

Still deferred to R1-5: the literal actor ids, the scenario label derived from a
directory name, the interaction reconstruction, and a declared actor role. The
viewer picks its initial scenario by lowest authoritative tick rather than array
position, which closes the audit's remaining positional convention early.

Simulation-boundary expansion remains deferred while the visual grammar is
being established. Do not reopen Gate K evaluation work or add proof machinery
unless the owner explicitly reverses decision 0016.
