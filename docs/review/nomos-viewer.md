---
title: The promoted viewer — R1-4 design record
status: R1-4 design record; reviewed, ruled on, and implemented on this branch
date: 2026-08-25
issue: 148
branch: r1/issue-148-nomos-viewer
accepts_against: RUNTIME.md §5 R1-4 (revision 1)
depends_on: issue #139 (R1-2 rendering-plan compiler), issue #146 (R1-3 typed presentation source)
follow_ups: issue #152 (promote the area collection), issue #153 (rendering_plan@3)
applies_to: RUNTIME.md §3, §4, §5, §6, §7; docs/workspace.md; docs/review/presentation-source.md; docs/review/executable-gaol-ownership-audit.md
---

# The promoted viewer

## Problem

The only viewer is `experiments/executable-gaol/viewer.html` plus
`src/webgl-renderer.mjs`, `src/play-state.mjs`, and `src/renderer-catalog.mjs`.
It imports Three.js from a CDN at `src/webgl-renderer.mjs:1`, so the published
page fetches a third-party origin on every load; it has no browser test at all,
only the source-level assertions in `src/webgl-viewer.test.mjs`; and under
`RUNTIME.md` §2 nothing in `experiments/` can satisfy acceptance.

This record is the design R1-4 implements: one accepted app under
`apps/nomos-viewer/`, a vendored Three.js recorded under `RUNTIME.md` §4, a
staged artifact that is scanned before it is published, and a headless Chromium
lane that actually plays the four-area route to the final escape and fails on a
single console error.

Promotion is by clean implementation (`RUNTIME.md` §2). No file moves and no
file is copied. Every promoted behaviour names the study lines it reproduces and
the test that proves it; §2 below is that table, and where the promoted
behaviour deliberately differs the row says so and gives the cause.

Nothing here is accepted until the implementation lands with its evidence and a
non-author rerun (`AGENTS.md`).

---

## 1. Module map

```text
apps/nomos-viewer/
  index.html                    document shell; one inline module; no colour literal
  src/plan.mjs                  strict decoder + loader for the two published identities
  src/catalog.mjs               the one renderer catalog
  src/play.mjs                  play state: movement, cost, interaction, pursuit, transition
  src/render.mjs                the WebGL renderer, over an injected Three.js namespace
  src/ui.mjs                    DOM binding, the pure readout, key handling
    vendor/three/three.module.min.js   three@0.185.1, verbatim, 365 552 bytes
  vendor/three/three.core.min.js     its sibling, verbatim, 385 386 bytes (finding 8)
  vendor/three/LICENSE               MIT, verbatim, 1 081 bytes
  vendor/MANIFEST.json               provenance, digests, imports, external-URL counts
  build.mjs                     stages dist/ from published artifacts; runs the scan
  test/fixtures.mjs             hand-authored plans and collections, written fresh
  test/plan.test.mjs            decoder: identity, shape, refusals, accessors
  test/catalog.test.mjs         units, sockets, closed sets, palette, prose rule
  test/play.test.mjs            the ported play rules and the winnable route
  test/render.test.mjs          scene graph against a recording Three.js stub
  test/ui.test.mjs              the pure readout and scenario selection
  test/vendor.test.mjs          recomputes both vendored digests
  test/scan.test.mjs            the dist scan, on a good dist and on planted bad ones
  smoke/smoke.mjs               the lane: launch, drive, assert, write the receipt
  smoke/chrome.mjs              binary discovery, flag sets, launch, DevTools port
  smoke/cdp.mjs                 dependency-free CDP client over global WebSocket
  smoke/server.mjs              localhost static server, node:http only
  smoke/route.mjs               the route solver over the decoded plans
  README.md                     what it is, how to build, run, and smoke it
```

| File | Responsibility | Lines (est.) |
| --- | --- | --- |
| `index.html` | Document, stylesheet, and a three-line inline module that imports `start` from `src/ui.mjs` and calls it. Every colour is `var(--nomos-<role>)`; the custom properties are written at boot from the catalog palette, so no colour value exists here. `<link rel="icon" href="data:,">` so the page issues no favicon request. | ~120 |
| `src/plan.mjs` | Binds `nomos.rendering_plan@2` and `nomos.experiment.area_collection@2`; refuses a mismatch, an unknown field, a missing field, a non-integer number, a name outside the catalog, and a broken cross-reference, each with a stable `NV####` code. Exports `loadArtifacts(base, fetchImpl)` — the only place a URL is constructed — and the typed accessors (`movementOf`, `lightOf`, `machineState`, `doorState`, `wardSealed`, `initialScenario`, `interactionsFrom`). | ~300 |
| `src/catalog.mjs` | The one renderer catalog: units, scales, camera, sockets, closed sets, entity assemblies and material families, palette, look profiles, and the prose tables keyed by closed sets. Pure data plus pure resolution functions; imports nothing. | ~280 |
| `src/play.mjs` | Play state over a decoded plan: movement keys, terrain cost, masonry, the declared-face exit, gaoler pursuit, interaction adjacency, area arrival, completion, and guidance. No DOM, no renderer, no fetch. | ~260 |
| `src/render.mjs` | `createGaolRenderer(container, three, catalog)`: floor, walls, masses, water, doors, braziers, socket-placed effects, actors, lights, look profiles, and the frame loop. Dispatches on catalog entries only — no entity id, no assembly string literal, no area name. Three.js arrives as a parameter, which is what makes the scene graph testable in node. | ~470 |
| `src/ui.mjs` | `start(document, three)` wires the DOM; `readout(collection, plan, play, scenario)` is a pure function returning every visible string, so the presentation model is node-tested and only the wiring depends on a browser. Writes the palette custom properties and the machine-readable run readout the smoke lane reads. | ~260 |
| `build.mjs` | Stages `dist/` from published artifacts and the app, then runs `scanDist` and fails closed. Exports `stage` and `scanDist` so the tests can drive both; the CLI entry is guarded by `pathToFileURL(process.argv[1]).href === import.meta.url`, not `import.meta.main`, because CI pins Node 22. | ~300 |
| `smoke/smoke.mjs` | Serves `dist/`, launches Chrome, subscribes, navigates, drives the solved route, asserts, screenshots, writes the receipt, exits 0/1, or skips with an explicit message. | ~300 |
| `smoke/chrome.mjs` | `CHROME_BIN`, then PATH names, then the Playwright cache as a last resort; the two flag sets and the retry; reads `DevToolsActivePort` from the throwaway user-data dir. | ~140 |
| `smoke/cdp.mjs` | `connect(wsUrl)` → `send(method, params)` promise map, `on(event, handler)`, and a `waitFor(predicate, timeout)`; nothing but global `WebSocket`. | ~150 |
| `smoke/server.mjs` | `node:http` static server on `127.0.0.1:0`, path-traversal refusal, a fixed extension→type table, `no-store`, and a request log for the receipt. | ~90 |
| `smoke/route.mjs` | The solver: decodes the plans, walks the route graph, and emits the key sequence plus the expected counters. Test tooling; it never ships in `dist/`. | ~170 |
| `test/*.test.mjs`, `test/fixtures.mjs` | Node's built-in test runner; fixtures are written fresh in this record's own vocabulary, never copied from `experiments/`. | ~1 000 total |

Nothing in `apps/nomos-viewer/` is over 1 000 lines, and the two largest files
(`render.mjs`, `plan.mjs`) are the two with the most declared data.

**Dependency direction inside the app.** `catalog.mjs` imports nothing.
`plan.mjs` imports `catalog.mjs` (to refuse a name outside the closed sets).
`play.mjs` imports `plan.mjs`. `render.mjs` imports `catalog.mjs` and takes the
Three.js namespace as a parameter. `ui.mjs` imports all of them and the vendored
module, and is the only importer of `vendor/three/three.module.min.js`.
`index.html` imports `ui.mjs` only. `build.mjs` and `smoke/` import `plan.mjs`
but are never staged into `dist/`.

---

## 2. Promotion table

Each row: the behaviour, the study lines it reproduces, where it lands, and the
test that proves it. **Divergence** marks a row where the promoted behaviour is
deliberately different; the cause is stated, as `RUNTIME.md` §2 requires.

| # | Promoted behaviour | Study lines reproduced | Home | Test |
| --- | --- | --- | --- | --- |
| 1 | Plan identity is bound before any field is read. **Divergence:** the study parsed the plan with `JSON.parse` and checked `schema` out of band in a build script only. | `viewer.html:68-72`; `src/verify.mjs:20` | `plan.mjs` | `plan_binds_its_identity_and_refuses_a_mismatch` |
| 2 | Collection identity bound the same way | `src/build-collection.mjs:88`; `viewer.html:68` | `plan.mjs` | `collection_binds_its_identity_and_its_route` |
| 3 | Unknown field, missing field, and non-integer number refused | none — the study had no reader | `plan.mjs` | `plan_refuses_an_unknown_field`, `plan_refuses_a_fractional_number` |
| 4 | `VERTICAL_STEPS_PER_CELL = 10`; `cellsOf(steps) = steps / 10`, a division | `src/renderer-catalog.mjs:44,47` | `catalog.mjs` | `steps_convert_by_division` — pins all ten corpus values, including `48/10`, `7/10`, `24/10` |
| 5 | The `ward` socket offset `{5, 0, 17}` in tenths | `src/renderer-catalog.mjs:91-95` | `catalog.mjs` | `the_ward_socket_offset_is_five_zero_seventeen` |
| 6 | Socket resolution. **Divergence:** resolved through the entity's declared `anchor.direction` rather than as a fixed north offset — the R1-3 §3.3 deferral. | `src/renderer-catalog.mjs:101-116` | `catalog.mjs` | `a_socket_resolves_by_the_declared_direction` — the north case equals the study's value exactly |
| 7 | `VERTICAL_SCALE = 0.72` | `src/webgl-renderer.mjs:18` | `catalog.mjs` | `the_catalog_declares_the_vertical_scale` |
| 8 | Camera: half-height `3.7`, offset `{.86,.92,1.08}`, target height `0.5`, near `0.1`, far `80` | `src/webgl-renderer.mjs:23-25,419-423,482-488` | `catalog.mjs` | `camera_constants_match_the_study` |
| 9 | The five closed sets content selects from | `src/renderer-catalog.mjs:53-60` | `catalog.mjs` | `a_name_outside_a_closed_set_is_refused` |
| 10 | Entity kind → assembly and material family. **Divergence:** the catalog owns what each assembly *means* and refuses `unknown`/`visual/marker`; see §3.2. | `crates/nomos-render-plan/src/catalog.rs:89-107` | `catalog.mjs` + `plan.mjs` | `the_catalog_knows_every_assembly_the_compiler_can_emit`, `an_unclassified_entity_is_refused` |
| 11 | One palette. **Divergence:** the two study tables unify into one, and the UI reads it too; §3.5. | `src/render-core.mjs:31-37`; `src/webgl-renderer.mjs:27-41` | `catalog.mjs` | `one_palette_serves_the_scene_and_the_ui`, `no_colour_literal_outside_the_catalog` |
| 12 | Two look profiles, `baseline` and `procedural`, with their six controls | `src/webgl-renderer.mjs:43-66`; `src/renderer-catalog.mjs:67` | `catalog.mjs` | `two_look_profiles_and_no_bare_look_literal` |
| 13 | Machine-state lookup with no fallback; `doorState`; `wardSealed` | `src/renderer-catalog.mjs:126-142` | `plan.mjs` | `an_absent_machine_namespace_is_refused` |
| 14 | `movementOf` / `lightOf` over the `@2` stable-ID arrays | `src/renderer-catalog.mjs:144-148` | `plan.mjs` | `movement_and_light_are_indexed_by_entity` |
| 15 | `isHunting` — one pursuit condition | `src/renderer-catalog.mjs:155`; `src/play-state.mjs:162` | `play.mjs` | `the_gaoler_hunts_only_when_the_pursuit_light_is_out` |
| 16 | The initial scenario. **Divergence:** the unique lowest `tick` instead of `scenarios[0]` by array position; resolves audit §3 item 23 here rather than deferring it. | `viewer.html:93,141,155` | `plan.mjs` | `the_initial_scenario_is_the_unique_lowest_tick`, `a_tie_on_tick_is_refused` |
| 17 | Movement key map | `src/play-state.mjs:3-8` | `play.mjs` | `movement_keys_map_to_lattice_deltas` |
| 18 | Actor start cells from the plan, with no fallback | `src/play-state.mjs:15-19,27-41` | `play.mjs` | `a_plan_without_an_actor_is_refused` |
| 19 | Water traversal cost from the projection | `src/play-state.mjs:78-86` | `play.mjs` | `water_uses_the_projected_traversal_cost` (reproduces `src/play-state.test.mjs:17-26`) |
| 20 | Masonry mass blocking, half-open rectangle | `src/play-state.mjs:88-91` | `play.mjs` | `a_mass_blocks_the_cells_it_covers` |
| 21 | Move: bounds, mass, water cost, message and tone | `src/play-state.mjs:93-156` | `play.mjs` | `the_baseline_gate_refuses_an_exit`, `the_unchanged_second_door_remains_blocked` (reproduce `src/play-state.test.mjs:28-49`) |
| 22 | Exit through a door. **Divergence:** the exit test is "the attempted move leaves the lattice through a door on the player's own cell whose declared `anchor.direction` is the direction of travel", replacing the `target.y < 0` special case. | `src/play-state.mjs:99-130` | `play.mjs` | `an_exit_uses_the_doors_declared_direction` — all four faces, on a fixture area |
| 23 | Gaoler pursuit: every second successful move, axis-major step, capture on contact | `src/play-state.mjs:158-183` | `play.mjs` | `the_dark_gaoler_advances_every_second_successful_move`, `the_dark_gaoler_can_catch_and_stop_the_player` (reproduce `src/play-state.test.mjs:120-150`) |
| 24 | Interaction adjacency, Manhattan ≤ 1, hash-bound edges | `src/play-state.mjs:185-195,230-256` | `play.mjs` | `nearby_interactions_follow_verified_state_hashes`, `interaction_range_does_not_invent_remote_actions`, `the_brazier_interaction_follows_the_verified_extinguish_receipt` (reproduce `src/play-state.test.mjs:51-71,110-118`) |
| 25 | Arrival at the destination's own `route.entry`, preserving cumulative counters | `src/play-state.mjs:47-59` | `play.mjs` | `arrival_uses_the_destinations_own_entry_cell` |
| 26 | Completion and its summary | `src/play-state.mjs:61-73` | `play.mjs` | `completion_reports_cumulative_run_state` (reproduces `src/play-state.test.mjs:95-108`) |
| 27 | Guidance. **Divergence:** no identifier is re-cased into prose; `displayName` does not survive. §3.7. | `src/play-state.mjs:21,197-228` | `play.mjs`, `ui.mjs` | `no_identifier_is_re_cased_into_prose` |
| 28 | The full winnable extinguish-and-escape route | `src/play-state.test.mjs:152-174` | test | `the_extinguish_and_escape_route_remains_winnable` |
| 29 | Floor, walls (doors read from `anchor.direction`), masses, water | `src/webgl-renderer.mjs:200-254` | `render.mjs` | `the_scene_graph_matches_the_plan` (stub Three.js: one floor box per cell, one wall segment per non-door column) |
| 30 | Door assembly: frame, bars or wreckage, ward ring, diamond, glow light | `src/webgl-renderer.mjs:256-297` | `render.mjs` | `a_destroyed_door_draws_wreckage_and_a_sealed_ward_draws_its_ring` |
| 31 | Brazier assembly and its light | `src/webgl-renderer.mjs:299-323` | `render.mjs` | `an_extinguished_brazier_has_no_flame_and_no_light` |
| 32 | Socket-placed effects, drawn only while the ward is sealed | `src/webgl-renderer.mjs:325-348` | `render.mjs` | `the_crescent_sits_at_the_resolved_socket` |
| 33 | Actor silhouettes and outlines | `src/webgl-renderer.mjs:350-391` | `render.mjs` | `actors_are_placed_at_their_cells` |
| 34 | Materials, look switching, rebuild-on-identity-change, the frame loop | `src/webgl-renderer.mjs:393-541` | `render.mjs` | `switching_look_profiles_rebuilds_the_world` |
| 35 | HUD, area buttons, scenario buttons, arrival banner, completion card | `viewer.html:99-163,185-208` | `ui.mjs` | `the_readout_reports_area_progress_and_cumulative_counters` |
| 36 | Number keys select by scenario identity, not DOM order | `viewer.html:227-237` | `ui.mjs` | `number_keys_select_by_scenario_identity` |
| 37 | Animation. **Divergence:** authoritative state advances synchronously on the key event and the tween is presentation-only between authoritative endpoints; the study gated input and the area transition on the animation's completion callback (`viewer.html:164-184,245-254`). Cause: `RUNTIME.md` §5 R1-5 forbids interpolation inside authoritative state, and a lane that depends on `requestAnimationFrame` firing in headless Chrome is a lane that hangs. | `viewer.html:164-184` | `ui.mjs` | `state_advances_without_waiting_for_a_frame` |
| 38 | Static staging | `gaol:68-85` (`stage_site`) | `build.mjs` | `build_stages_only_published_artifacts`, `building_twice_is_byte_identical` |
| 39 | Localhost static serving | `src/serve.mjs:1-18` | `smoke/server.mjs` | exercised by the smoke lane; unit-tested for path-traversal refusal |
| 40 | Three.js. **Divergence:** vendored in-tree instead of imported from `cdn.jsdelivr.net`. | `src/webgl-renderer.mjs:1`; asserted at `src/webgl-viewer.test.mjs:15-20` | `vendor/` | `vendor_digests_match_the_manifest`, `no_external_origin_survives_the_scan` |

Rows 29–34 are the ones the study could only assert by grepping its own source
(`src/webgl-viewer.test.mjs:1-67`). Passing the Three.js namespace into
`render.mjs` turns them into real assertions about the scene graph, which is why
the injection is in the design rather than a convenience.

---

## 3. The catalog

`src/catalog.mjs` is renderer-owned data that content may name but not define.
It is the accepted home of everything `experiments/executable-gaol/src/renderer-catalog.mjs`
holds, everything `src/webgl-renderer.mjs` held privately, and the three
deferrals `docs/review/presentation-source.md` §4.3 routed to R1-4.

### 3.1 Units and scales

| Constant | Value | Meaning |
| --- | --- | --- |
| `VERTICAL_STEPS_PER_CELL` | `10` | One vertical step is a tenth of a lattice cell. |
| `cellsOf(steps)` | `steps / VERTICAL_STEPS_PER_CELL` | A **division**. `48/10 === 4.8`, `7/10 === 0.7`, `24/10 === 2.4`; multiplying by the nearest double to `0.1` gives a different answer for exactly those three, which moved every Ossuary Reach frame the first time R1-3 wrote it that way. The test pins all ten corpus values. |
| `CELL_WORLD_UNITS` | `1.0` | One lattice cell horizontally is one world unit. |
| `VERTICAL_SCALE` | `0.72` | Lattice cells of elevation to world units. |

### 3.2 Closed sets, entity assemblies, and the two compiler tables

The five sets content selects from, unchanged in membership from
`src/renderer-catalog.mjs:53-60`:

```js
ARCHITECTURE_ASSEMBLIES = ["visual/beveled_masonry"]
MATERIAL_FAMILIES       = ["stone_bounded"]
TRIM_FAMILIES           = ["broad_mortar"]
ACTOR_ASSEMBLIES        = ["visual/gaoler_silhouette", "visual/player_silhouette"]
EFFECT_ASSEMBLIES       = ["visual/cyan_crescent"]
```

And the entity vocabulary, which is where audit §3 items 5 and 6 land:

| Kind | `visual_assembly` | `material_family` | Sockets | Builder |
| --- | --- | --- | --- | --- |
| `door` | `visual/iron_barred_door` | `iron_oxidized` | `ward` | `buildDoor` |
| `water` | `visual/shallow_water` | `water_cold` | — | `buildWater` |
| `light` | `visual/brazier` | `iron_brazier` | — | `buildBrazier` |

**How the two deferred tables resolve.** The audit called
`build-plan.mjs:33-38` and `:43` "a renderer catalog living in the wrong layer";
R1-2 moved them into `crates/nomos-render-plan/src/catalog.rs:89-107`, whose own
doc comment (`catalog.rs:50-63`) says they do not belong there either. R1-4
resolves them the way the owner already ruled the five content fields in
`docs/review/presentation-source.md` §6 finding 1 — a **definition/selection
split**:

- the **catalog defines** what an assembly name and a material family *mean*:
  which geometry builder draws it, which material parameters it takes, which
  sockets it declares. That is here, in the accepted renderer, and nowhere else.
- the **compiler selects** one assembly and one family per entity kind, exactly
  as an area's `presentation.json` selects one architecture assembly. Its table
  stays a selection from a set this catalog defines.
- `plan.mjs` refuses any `visual_assembly` or `material_family` the catalog does
  not declare, with `NV0301`, so a name legal to the compiler but unknown here
  fails the decode rather than a frame — the same fail-closed shape
  `experiments/executable-gaol/src/verify.mjs:46-60` gives the other five sets.
- `test/catalog.test.mjs` reads `crates/nomos-render-plan/src/catalog.rs`,
  extracts the two `match` tables, and asserts they are exactly the three rows
  above. Neither side can drift without a red test. This is a coherence
  assertion, not a dependency: no code path imports Rust.

`EntityKind::Unknown` is refused rather than drawn. `catalog.rs:276-312` shows
`classify` returning `Unknown` for a primitive the compiler has no kind for,
which would reach a plan as `kind: "unknown"`, `visual_assembly: "visual/marker"`,
`material_family: "stone"`. The study drew a marker; audit §3 item 4 records
that silent fallback as a defect. The accepted viewer refuses it with `NV0301`
and says which entity and which primitive. The compiler must tolerate an
unclassified primitive in order to report it; the viewer must not pretend to
draw one. That asymmetry is deliberate and tested.

§10 finding 3 records what this does *not* do: the mapping "kind → assembly"
still lives in Rust, because removing it means the plan stops carrying
`entities[].visual_assembly` and `material_family`, which is a
`nomos.rendering_plan@3` and is outside this issue's Scope. Issue #153 carries
that move.

### 3.3 Sockets, resolved by the declared face

```js
SOCKETS = {
  "visual/iron_barred_door": { ward: { x: 5, y: 0, z: 17 } },
};
```

The offset is unchanged from `src/renderer-catalog.mjs:91-95`: half a cell along
the door's own axis, on the wall plane its `anchor.direction` names, 1.7 cells
up, where both study renderers already draw the ward mark
(`(17/10) * 0.72 = 1.224` against the WebGL ward ring at `y = 1.22`).

What changes is the frame it is expressed in. `docs/review/presentation-source.md`
§3.3 deferred direction-aware resolution to R1-4: "A non-north door would need
the catalog to rotate the socket by the entity's declared `anchor.direction`."
The offset is now read in the entity's **local face frame** — `x` runs along the
face, `y` runs inward from the face, `z` is up — and `resolveSocket(entity, name)`
maps it into lattice space by the declared direction:

| `anchor.direction` | Lattice position, in cells, for offset `(x, y, z)` tenths |
| --- | --- |
| `north` | `(cell.x + x/10, cell.y + y/10, z/10)` |
| `south` | `(cell.x + x/10, cell.y + 1 − y/10, z/10)` |
| `west` | `(cell.x + y/10, cell.y + x/10, z/10)` |
| `east` | `(cell.x + 1 − y/10, cell.y + x/10, z/10)` |

For `north`, `{5, 0, 17}` gives `(cell.x + 0.5, cell.y, 1.7)` — byte-identical to
what `socketPosition` computes today, which is the equivalence the test asserts.
The other three rows are proved on a fixture area with one door per face; all
eight corpus doors are north-facing, so no committed frame moves.

Resolution fails closed: a socket the catalog has no offset for, an entity with
no `anchor.cell`, or a direction outside the four is `NV0301`, naming the
entity, the assembly, and the socket.

### 3.4 Camera

| Constant | Value | From |
| --- | --- | --- |
| `ORTHO_HALF_HEIGHT` | `3.7` | `src/webgl-renderer.mjs:23` |
| `CAMERA_OFFSET` | `{ x: 0.86, y: 0.92, z: 1.08 }` | `src/webgl-renderer.mjs:24` |
| `CAMERA_TARGET_HEIGHT` | `0.5` | `src/webgl-renderer.mjs:25` |
| `CAMERA_NEAR` / `CAMERA_FAR` | `0.1` / `80` | `src/webgl-renderer.mjs:422` |
| `MAX_PIXEL_RATIO` | `2` | `src/webgl-renderer.mjs:407` |
| `SHADOW_MAP_SIZE` | `2048` (moon), `512` (brazier) | `src/webgl-renderer.mjs:429,318` |
| `SHADOW_FRUSTUM` | `±8` | `src/webgl-renderer.mjs:430-433` |

The SVG camera (`src/render-core.mjs:15-22`, `ORIGIN`, `CELL_HEIGHT_PIXELS`)
does **not** come along. It belongs to the study's evidence renderer, which
stays quarantined; the accepted tree projects one way.

### 3.5 One palette

The audit's ninth double authority is "a `palette` string plus two unrelated
hardcoded colour tables", and `docs/review/presentation-source.md` §4.2 item 9
left it as "two renderers keep two tables because there are two renderers; R1-4
promotes one viewer, after which one table remains." One viewer, one table —
and the table serves the WebGL scene *and* the page chrome, because the third
uncounted colour source was `viewer.html`'s stylesheet.

Unification rule, stated once so it cannot drift: **where a role exists in both
study tables, the WebGL value wins**, because the WebGL renderer is the one that
survives; roles that exist only in the SVG table or only in the stylesheet keep
their own value; every role appears exactly once. The palette is authored as
integers (what Three.js takes) and rendered to `#rrggbb` for CSS by one helper.

| Role | Value | From |
| --- | --- | --- |
| `void` | `0x090e13` | `webgl-renderer.mjs:28` (SVG had `#10161d`) |
| `fog` | `0x111b24` | `webgl-renderer.mjs:29` (SVG had `#1c2832`) |
| `stone_0` | `0x202b34` | both tables agree |
| `stone_1` | `0x2d3a43` | `webgl-renderer.mjs:31` (SVG had `#2c3942`) |
| `stone_2` | `0x3d4b52` | `webgl-renderer.mjs:32` (SVG had `#3c4a51`) |
| `edge` | `0x536168` | `render-core.mjs:33`, and the same value as the procedural stone accent at `webgl-renderer.mjs:61` — two roles that were already one number |
| `mortar` | `0x111920` | `webgl-renderer.mjs:33` |
| `iron` | `0x111a20` | `webgl-renderer.mjs:34` |
| `rust` | `0x70412f` | `webgl-renderer.mjs:35` |
| `water` | `0x244857` | `webgl-renderer.mjs:36` |
| `water_deep` | `0x173744` | `webgl-renderer.mjs:166` |
| `water_light` | `0x4d8290` | `webgl-renderer.mjs:167` |
| `water_high` | `0x70909a` | `render-core.mjs:34` |
| `cyan` | `0x83eeea` | `webgl-renderer.mjs:37` (SVG had `#8ee6e3`) |
| `cyan_dim` | `0x4d9f9f` | `render-core.mjs:35` |
| `cyan_bright` | `0xbfffff` | `viewer.html:15` |
| `amber` | `0xffa544` | `webgl-renderer.mjs:38` |
| `amber_dim` | `0x87552e` | `render-core.mjs:35` |
| `player` | `0x347b7d` | `webgl-renderer.mjs:39` |
| `gaoler` | `0x8c5638` | `webgl-renderer.mjs:40` |
| `skin` | `0x96735b` | `webgl-renderer.mjs:400` |
| `sky` | `0x8aa8b8` | `webgl-renderer.mjs:424` |
| `ground` | `0x131c24` | `webgl-renderer.mjs:424` |
| `moon` | `0xabc8d7` | `webgl-renderer.mjs:426` |
| `grid` | `0x30434a` | `webgl-renderer.mjs:506` |
| `surface` | `0x0a1016` | `viewer.html:9` header/footer |
| `surface_raised` | `0x101b22` | `viewer.html:31` |
| `surface_sunk` | `0x091117` | `viewer.html:20` |
| `surface_button` | `0x202d35` | `viewer.html:14` |
| `border` | `0x33424a` | `viewer.html:9` |
| `border_strong` | `0x4c6068` | `viewer.html:14` |
| `text` | `0xd7ddd9` | both tables agree |
| `text_muted` | `0x8e9b9c` | `render-core.mjs:36` |
| `text_dim` | `0x6f8288` | `viewer.html:21` |
| `prompt` | `0x9baaad` | `viewer.html:23` |
| `danger` | `0xd47158` | `render-core.mjs:36` |

Thirty-six roles, one value each. Two study values are deliberately dropped
rather than renamed: the SVG renderer's `teal` (`#2f7777`) and `ochre`
(`#9a6640`) are its own actor fills, and the promoted viewer draws actors with
`player` and `gaoler`.

Two tests hold the claim: `one_palette_serves_the_scene_and_the_ui` asserts the
role set and the values, and `no_colour_literal_outside_the_catalog` scans
`index.html`, `src/ui.mjs`, and `src/render.mjs` for `#rrggbb`, `#rgb`, `0x`
colour literals, `rgb(`, and `hsl(` and requires zero. The scan test in §6
repeats it over `dist/`.

### 3.6 Look profiles

`LOOK_PROFILE_IDS = ["baseline", "procedural"]`, and the two profiles keep every
value from `src/webgl-renderer.mjs:43-66`:

| Control | `baseline` | `procedural` |
| --- | --- | --- |
| `id` | `gaol_baseline_01` | `gaol_procedural_01` |
| `fog_density` | `0.045` | `0.041` |
| `exposure` | `1.28` | `1.34` |
| `bevel` | `0` | `0.055` |
| `actor_outline` | `0` | `1.065` |
| `materials.stone` | — | `{ scale: 1.35, variation: 0.13, accent: edge, accent_mix: 0.16 }` |
| `materials.iron` | — | `{ scale: 2.1, variation: 0.08, accent: rust, accent_mix: 0.22 }` |
| `materials.cloth` | — | `{ scale: 2.8, variation: 0.09, accent: mortar, accent_mix: 0.08 }` |

`accent: 0x536168` at `webgl-renderer.mjs:61` becomes the palette's `edge`, which
is the same number the SVG table called `edge`. The procedural profile is the
default, as it is today.

### 3.7 Prose, and the end of `displayName`

Audit §3 item 26, deferred to R1-4: "`displayName()` derives every on-screen
label for entities, actors, and interaction actions by title-casing the
snake_case id… every visible name is convention-derived from an identifier that
was never intended as prose." R1-3 deferred the choice between "an authored
display-string table or a per-entity `label`".

Both of those options are unavailable to R1-4, and the record says so plainly:

- a per-entity `label` is a new field of `nomos.presentation_source@1`, which is
  a Rust schema change outside this issue's Scope;
- a display-string table **keyed by entity id** inside `apps/` would make the
  renderer carry knowledge of specific content, which `RUNTIME.md` §5 R1-4
  forbids outright ("Must not… require a renderer-specific edit to accept new
  content") and which the §9 area-addition proof would immediately falsify.

So R1-4 resolves it by removing the invention instead of relocating it. The
rule, which `no_identifier_is_re_cased_into_prose` enforces:

> Visible text comes from exactly three places: `area.label`, the only authored
> prose in the content model; the app's own UI strings, authored here; and
> catalog tables keyed by a **closed set the accepted schema declares**. An
> identifier — an entity id, an actor id, an interaction action — is never
> re-cased, title-cased, or otherwise turned into prose. It is displayed
> verbatim, in an identifier style.

In practice the HUD reads `Exit via` `north_gate` with the identifier in the
monospace identifier style, rather than "Exit via North Gate"; the interaction
prompt reads `E · ignite north_gate`. The catalog carries two small tables keyed
by closed sets — `KIND_LABELS` for `door`/`water`/`light` and
`DISPOSITION_LABELS` for `traversable`/`blocked` — because those values are
enumerated by the accepted schema, not authored by content. The test asserts
that no source file under `apps/` contains a case-changing call
(`toUpperCase`, `toLowerCase`, `replace(/\b\w/…)`) applied to a plan value, and
that the string `displayName` does not appear.

This is a visible difference from the study, recorded as promotion-table row 27.

---

## 4. The decoder contract

`src/plan.mjs` is the only module that reads an artifact, and it reads exactly
two identities.

**Bound identities**

| Artifact | `schema` value bound | Declared by |
| --- | --- | --- |
| `areas/<area-id>.json` | `nomos.rendering_plan@2` | `crates/nomos-render-plan/src/plan.rs` (accepted) |
| `areas.json` | `nomos.experiment.area_collection@2` | `experiments/executable-gaol/src/build-collection.mjs:88` (quarantined tooling — §10 finding 2, follow-up issue #152) |

Binding is the first operation: the decoder reads `schema`, compares the exact
string, and refuses before any other field is interpreted.

**Reading discipline**, mirroring `crates/nomos-render-plan/src/json.rs` so the
two ends of the pipe refuse the same things:

- **Strict.** Every object is checked against its declared field set; an unknown
  field and a missing field are both refusals. Duplicate keys cannot survive
  `JSON.parse`, so the decoder additionally refuses any object whose serialized
  key count differs from its declared set — the shape check does this by
  construction.
- **Integer-only.** Every number in the plan is checked with
  `Number.isSafeInteger`, and `cost: null` is the one permitted null, exactly as
  `RUNTIME.md` §5 R1-1 names it. A fractional number anywhere is `NV0203`. This
  is the same statement `verify.mjs:33-37` makes about the bytes, made about the
  values.
- **Fail closed.** No default is ever substituted. No `?? fallback`, no
  `?.` that swallows an absent collection.
- **Closed vocabularies.** `kind`, `disposition`, `anchor.kind`,
  `anchor.direction`, `objective.kind`, every assembly, every family, and every
  socket must be a member of the catalog's declared set.

**Cross-references checked at decode**, so that a broken plan fails on load
rather than mid-frame:

1. `objective.gate` names an entity whose kind is `door`.
2. `pursuit.light` names an entity whose kind is `light`.
3. every `effects[].anchor.entity` names a declared entity, and its
   `anchor.socket` resolves in `SOCKETS` for that entity's assembly.
4. every `interactions[].{from_scenario,to_scenario}` names a declared scenario,
   and its `input_state_hash`/`resulting_state_hash` equal those scenarios'
   `state_hash` — the check `verify.mjs:24-25` makes for the study.
5. every `interactions[].target_entity` names a declared entity.
6. every scenario carries a `movement` row for every `door` and `water` entity
   and an `effective_light` row for every `light` entity.
7. `route.entry` is present iff `area.start` is false, and lies inside `bounds`
   and outside every mass.
8. exactly one scenario has the minimum `tick`; that scenario is the initial
   one (§2 row 16).

**Collection checks**: exactly one `start_area`; every `route[].from_area` and
non-null `to_area` names a declared area; the route visits every area exactly
once and terminates; every `areas[].plan` is a relative path of the form
`areas/<area-id>.json` with no `..` and no scheme.

**Diagnostics.** A stable `NV####` space, disjoint from `nomos-core`'s frozen
`EK` space and from `nomos-render-plan`'s `RP` space by its prefix, following
the reasoning at `crates/nomos-render-plan/src/error.rs:1-12`. Every message
names what was expected and what was found, and the artifact that carried it.

| Code | Meaning |
| --- | --- |
| `NV0101` | An artifact could not be fetched or is not well-formed JSON. |
| `NV0102` | An artifact carries a schema identity or version the viewer does not accept. |
| `NV0201` | A document is missing a required field or carries an unknown one. |
| `NV0202` | A declared constraint is violated (bounds, uniqueness, the tick rule, a hash that is not scenario-bound). |
| `NV0203` | A number is not a safe integer, or a null appears outside `movement[].cost`. |
| `NV0301` | A name is outside a closed catalog set — assembly, family, socket, kind, disposition, direction. |
| `NV0401` | A cross-reference does not resolve. |

A refusal is not a console error the smoke lane trips over by accident: the app
catches it, renders the code and message into the page as visible text, and
`ui.mjs` sets `data-error` on the root element. The smoke lane asserts
`data-error` is absent, so a refusal fails the lane loudly and legibly rather
than as an unhandled exception.

**What the decoder does not do.** It never reads `.nomos` source, World IR, a
compiler receipt, or a projection member; it constructs no URL except by
joining a relative path onto the document base; and it holds no knowledge of any
area identifier.

---

## 5. The smoke lane

`apps/nomos-viewer/smoke/` — no dependency, no Playwright, no Puppeteer. Node's
`http`, `child_process`, `fs`, and the global `WebSocket`, against the Chrome
that is already on the machine.

### 5.1 Chrome discovery, and the flags

Discovery order, first hit wins:

1. `CHROME_BIN`, if set and executable;
2. `google-chrome`, `google-chrome-stable`, `chromium`, `chromium-browser` on
   `PATH` — `google-chrome` is present on the `ubuntu-24.04` runner;
3. **last resort only**, a Playwright cache at
   `~/.cache/ms-playwright/chromium-*/chrome-linux*/chrome`, highest build
   number first. The lane never requires it and never installs it; it exists so
   a developer who happens to have one gets the lane locally.

If none is found: with `--require-chrome` (what CI passes) the lane fails with
`no Chrome found: set CHROME_BIN or install google-chrome`; without it, the lane
prints `SKIP: no Chrome found (set CHROME_BIN to run the browser lane)` and
exits 0.

Flag set A, the first attempt:

```text
--headless=new
--remote-debugging-port=0
--user-data-dir=<throwaway>
--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost
--no-first-run --no-default-browser-check --no-sandbox
--disable-dev-shm-usage --disable-extensions --disable-sync
--disable-background-timer-throttling --disable-renderer-backgrounding
--disable-backgrounding-occluded-windows
--use-gl=angle --use-angle=swiftshader --enable-unsafe-swiftshader
--window-size=1280,720 --force-device-scale-factor=1 --hide-scrollbars
about:blank
```

Flag set B, retried once if the page reports no WebGL context: the same list
with `--disable-gpu` added. Recent Chrome refuses a software WebGL context
without `--enable-unsafe-swiftshader`, and `--disable-gpu` is the older route to
the same SwiftShader backend; trying A then B covers both without guessing at
the runner's Chrome version.

`--remote-debugging-port=0` plus reading `<user-data-dir>/DevToolsActivePort`
avoids racing a fixed port. The browser's `webSocketDebuggerUrl` comes from
`GET http://127.0.0.1:<port>/json/version`; the page target from
`GET /json/list`. Both are `node:http` requests to a loopback address.

### 5.2 CDP methods and events

Connected directly to the **page** target's WebSocket, so no `Target` domain and
no session multiplexing.

Enabled before navigation: `Runtime.enable`, `Log.enable`, `Page.enable`,
`Network.enable`.

| Direction | Message | Use |
| --- | --- | --- |
| → | `Browser.getVersion` | records the product and revision in the receipt |
| → | `Page.navigate` | loads `http://127.0.0.1:<port>/` |
| → | `Page.captureScreenshot` | one PNG per area |
| → | `Runtime.evaluate` | reads the run readout, the WebGL context report, and the negative control |
| → | `Input.dispatchKeyEvent` | one `keyDown` and one `keyUp` per key |
| ← | `Page.loadEventFired` | load gate |
| ← | `Runtime.exceptionThrown` | any uncaught exception or unhandled rejection — fatal |
| ← | `Runtime.consoleAPICalled` | `type: "error"` — fatal |
| ← | `Log.entryAdded` | `level: "error"` — fatal; this is where a blocked request or a failed subresource surfaces |
| ← | `Network.requestWillBeSent` | every request URL, recorded and origin-checked |
| ← | `Network.responseReceived` | status per request, recorded |
| ← | `Network.loadingFailed` | fatal outside the negative-control window |

Key events carry `code`, `key`, `windowsVirtualKeyCode`, `nativeVirtualKeyCode`,
and `text` where the app reads `event.key`:
`ArrowUp/Down/Left/Right` (38/40/37/39), `KeyE` (69, text `e`), and the digits
(49–53, text `1`–`5`).

Between keys the lane polls `Runtime.evaluate` for the run readout rather than
sleeping. The readout is a DOM contract, not a test hook bolted on: `ui.mjs`
writes `data-area`, `data-scenario`, `data-moves`, `data-cost`, `data-areas-cleared`,
`data-completed`, `data-message`, and `data-error` onto the root element, which
is the same state the HUD paints. `test/ui.test.mjs` asserts the readout matches
`readout()`, so the contract cannot silently rot.

### 5.3 The route solver

`smoke/route.mjs` derives the key sequence from the artifacts, so a content
change moves the route without anyone editing the harness. It decodes the
collection and the four plans with the app's own `plan.mjs` — the harness has no
second decoder — and then, for each area in `collection.route` order:

1. **Start cell**: the start area's `actors[player].cell`; every other area's
   own `route.entry`.
2. **Scenario**: the initial scenario, the unique lowest `tick`.
3. **Interaction chain**: while `movementOf(scenario, objective.gate).disposition`
   is not `traversable`, take the unique interaction with
   `from_scenario === scenario.id`, walk to any cell within Manhattan distance 1
   of `target_entity`'s `anchor.cell`, press `KeyE`, and advance to
   `to_scenario`. A missing or ambiguous interaction is a harness failure, not a
   retry.
4. **Water waypoint**: if the area declares a water entity, the walk to the
   first interaction target is routed through the water region's `min` cell.
   This is deliberate: without it the cheapest walk avoids water entirely, the
   cumulative cost equals the move count, and a regression in projected
   traversal cost would pass the lane unnoticed.
5. **Exit**: walk to the objective gate's `anchor.cell` and press the key for
   its declared `anchor.direction` (`north` → `ArrowUp`).
6. **Counters**: each dispatched movement key is one move; its cost is the
   projected `movement[].cost` of the water entity containing the entered cell,
   or 1; the exit move is one move at cost 1.

Walking is Dijkstra over lattice cells, cost-weighted by terrain, tie-broken by
`(cost, steps, y, x)` so the sequence is deterministic. Blocked: outside
`bounds`, or inside a mass's half-open rectangle. The gaoler never moves on this
route, because it is only hunting once the pursuit light is out and the route
never extinguishes one; the run is therefore deterministic without depending on
pursuit timing.

Solved against the four committed plans today, the lane dispatches 60 keys:

| Area | Scenario at exit | Keys | Cumulative moves | Cumulative cost |
| --- | --- | --- | --- | --- |
| cistern-walk | `03-breached-unsealed` | `↑↑↑←←←←←←↑` `E` `E` `→↑` | 12 | 16 |
| ember-vault | `03-breached-unsealed` | `←←←←←↑↑↑→↑↑` `E` `E` `→↑` | 25 | 31 |
| ossuary-reach | `03-breached-unsealed` | `↑↑→↑↑→→→↑` `E` `E` `→↑` | 36 | 48 |
| north-gaol | `03-breached-unsealed` | `↑↑↑↑→→` `E` `E` `→↑` | 44 | 60 |

Final readout: `4 areas · 44 moves · 60 traversal cost`. The harness computes
these numbers itself; the table is the prediction this record commits to, and a
disagreement between the two is a finding, not a number to edit.

### 5.4 Pass and fail

The lane fails on the first of:

1. no Chrome, under `--require-chrome`;
2. no DevTools endpoint within 20 s, or no `Page.loadEventFired` within 20 s;
3. any `Runtime.exceptionThrown`;
4. any `Runtime.consoleAPICalled` with `type: "error"`;
5. any `Log.entryAdded` with `level: "error"`;
6. any `Network.loadingFailed`, outside the negative-control window;
7. any request whose URL does not start with the server origin;
8. `data-error` present on the root element at any poll;
9. no WebGL context: `Runtime.evaluate` of the canvas' context report returns
   nothing after both flag sets;
10. a key that does not change the readout when the solver expected it to, or a
    readout that disagrees with the solver's expected counters at the end of any
    area;
11. a final state that is not `data-completed="true"`, `data-areas-cleared="4"`,
    the expected moves and cost, and the message `Escaped the gaol`;
12. fewer than four screenshots, or a screenshot that is not a PNG (magic bytes)
    or is under 1 KiB.

It passes when all twelve hold, and it writes the receipt either way — a failure
receipt is the evidence for the failure.

### 5.5 The offline receipt

`target/nomos-viewer-smoke/receipt.json`, uploaded by both workflows.

```json
{
  "receipt": "nomos-viewer-smoke/1",
  "generated_by": "apps/nomos-viewer/smoke/smoke.mjs",
  "commit": "<git rev-parse HEAD>",
  "node": "v22.x.y",
  "chrome": { "binary": "/usr/bin/google-chrome", "source": "PATH",
              "product": "HeadlessChrome/…", "revision": "…", "flag_set": "A" },
  "flags": ["--headless=new", "…"],
  "server": { "origin": "http://127.0.0.1:41573", "root": "apps/nomos-viewer/dist",
              "files": 12 },
  "webgl": { "context": "webgl2", "vendor": "…", "renderer": "…" },
  "requests": [ { "url": "http://127.0.0.1:41573/", "type": "Document", "status": 200 } ],
  "request_count": 12,
  "external_requests": [],
  "negative_control": { "probe": "https://example.invalid/nomos-viewer-probe",
                        "outcome": "TypeError: Failed to fetch",
                        "log_entries_during_probe": 1 },
  "console_errors": [], "exceptions": [], "log_errors": [],
  "route": [ { "area": "cistern-walk", "keys": 14, "moves": 12, "cost": 16,
               "screenshot": "cistern-walk.png" } ],
  "result": { "areas_cleared": 4, "moves": 44, "cost": 60,
              "message": "Escaped the gaol",
              "summary": "4 areas · 44 moves · 60 traversal cost" },
  "duration_ms": 0,
  "outcome": "pass"
}
```

Two notes on the shape. The receipt is deliberately **not** spelled
`name@version`: it is harness output, not a canonical document, and giving it a
canonical-looking identity would invite it into a register it does not belong
in. And the **negative control** is what turns "the artifact loads offline" from
an assertion into a measurement: after the route completes and the console
assertions are captured, the lane evaluates a `fetch` to
`https://example.invalid/…` in the page and requires it to reject. The window is
marked, the log entries it produces are recorded rather than fatal, and a probe
that *succeeds* fails the lane — that would mean the host-resolver rule was not
in force and the "zero external requests" result proved nothing.

### 5.6 Screenshots

One `Page.captureScreenshot` per area, taken immediately before the exit key so
the frame shows the area played rather than the arrival banner. Written to
`target/nomos-viewer-smoke/screenshots/<area-id>.png` and uploaded with
`retention-days: 90`, matching `gate-k-evidence.yml`. They are **not hashed and
not compared**: `RUNTIME.md` §9 states the study's pixels are not deterministic
across GPUs, and SwiftShader output is not a contract. Their job is that a human
can look at four pictures and see four rooms.

---

## 6. `build.mjs` and the `dist/` scan

### 6.1 Staging

```text
node apps/nomos-viewer/build.mjs --from target/executable-gaol --out apps/nomos-viewer/dist
```

`--from` is a directory of **published artifacts**: `areas.json` and
`areas/<area-id>/rendering-plan.json`, exactly what
`experiments/executable-gaol/gaol capture` writes and what
`crates/nomos-render-plan` emitted. The staged tree:

```text
dist/
  index.html
  src/{plan,catalog,play,render,ui}.mjs
    vendor/three/three.module.min.js
  vendor/three/three.core.min.js
  vendor/three/LICENSE
  areas.json
  areas/{cistern-walk,ember-vault,north-gaol,ossuary-reach}.json
```

Rules, all enforced by `build.mjs` itself so a bad build cannot reach the scan:

- the output directory is emptied first, then written; the build is idempotent
  and `building_twice_is_byte_identical` proves it;
- every plan is decoded by `plan.mjs` before it is staged, and the collection
  after, so an artifact that the viewer could not read never ships;
- the plan file names come from `collection.areas[].plan`, and a value that is
  not `areas/<area-id>.json` is refused;
- nothing else is copied. No test, no fixture, no `smoke/`, no `build.mjs`, no
  `README.md`, no `MANIFEST.json`, no `.map`;
- the vendored files are copied byte-for-byte and their sha256 re-checked
  against `vendor/MANIFEST.json` during the copy;
- the build prints the staged byte total, which is the `RUNTIME.md` §7 "public
  artifact size" measurement.

### 6.2 The scan

`scanDist(dir)` walks every staged file and refuses:

1. **External origins in a live position.** For `index.html`: any `src`, `href`,
   `action`, `srcset`, `poster`, or CSS `url(...)` value that is not relative;
   `href="data:,"` on the icon link is the single declared exception. For
   `.mjs`: any `import`/`export … from`, dynamic `import(`, `fetch(`,
   `new URL(`, `new Worker(`, `new EventSource(`, `new WebSocket(`, or
   `importScripts(` whose first argument is a string literal beginning `http://`,
   `https://`, `//`, `file:`, or `data:`. The check is on the *position*, not on
   the byte sequence, for the reason in §10 finding 1.
2. **Any `http://` or `https://` outside a comment** in the five app modules and
   `index.html`, after comment stripping. The vendored module is exempt from
   this rule and covered by rule 3 instead.
3. **The vendored module** must match its `MANIFEST.json` sha256 exactly, and
   must contain no `import`/`from`/`fetch` of any URL. `three@0.185.1`'s
   `build/three.module.min.js` contains exactly one `https://` byte sequence —
   `https://jcgt.org/published/0007/04/01/` inside a GLSL comment in a shader
   string, a paper citation — and no import of any kind. That is measured, not
   assumed: §10 finding 1 records the count and the exact string.
4. **Forbidden inputs.** `world-ir`, `world_ir`, `compiler-receipts`,
   `receipts/`, `simulation.json`, `navigation.json`, `persistence.json`,
   `diagnostics.json`, `world.nomos` *content* markers (`schema `, `catalog `,
   `entity ` at line start), and `.nomos` anywhere **except** as the value of
   `entities[].provenance[].source.path` in a staged plan, matching
   `^experiments/executable-gaol/areas/[a-z0-9-]+/world\.nomos$`. §10 finding 1
   explains why the exception exists and what it is bounded to.
5. **Credential shapes.** `-----BEGIN`, `AKIA[0-9A-Z]{16}`, `ghp_`,
   `github_pat_`, `xox[baprs]-`, `AIza[0-9A-Za-z_-]{35}`, `Bearer [A-Za-z0-9._-]{16,}`,
   and `(password|secret|api[_-]?key|token)\s*[:=]\s*["'][^"']{8,}`.
6. **Build-machine paths.** `/home/`, `/Users/`, `/root/`, `/work/`,
   `/github/workspace`, `/runner/`, `/private/var/`, `/tmp/`, and
   `[A-Za-z]:\\`. Anchored to absolute forms, so the repo-relative provenance
   path in rule 4 is untouched.
7. **Colour literals** outside the catalog, per §3.5.
8. **Shape.** Exactly the file list above; an unexpected file is a refusal.

`test/scan.test.mjs` runs the scan over a good fixture dist and over eight
planted bad ones — one per rule — and requires each to be refused with the rule
that should catch it. That is the same discipline as `xtask/src/planted.rs`.

---

## 7. The `RUNTIME.md` §4 record

The first row of "Recorded additions", drafted with exactly the eight fields §4
requires.

| Field | Value |
| --- | --- |
| **name** | `three` |
| **version** | `0.185.1` |
| **provenance** | Vendored under `apps/nomos-viewer/vendor/three/`, extracted from the npm registry tarball `https://registry.npmjs.org/three/-/three-0.185.1.tgz`. Registry `dist.integrity` `sha512-5aojFCXKwnjBRZvUnt3WFfEcvUJgkN5LlijRFN95hMy8WVkG4I0QNcJE+OuWvuJ0bOdStrbfXn0pkd6/QyiAlg==`; the same tarball as sha256 `a2143f5bf978bd3470a51024b2b6bdd581913ba8f36ff1538d433f3a95adf2df`. Two files, because the build is two files (finding 8): `three.module.min.js` sha256 `86bcee248b64f44bcfc23c331ae74619061957d59cab040171dcb6fb5900beb6`, 365 552 bytes, and its sibling `three.core.min.js` sha256 `05b2609338c76cd65daf74f3ac515bc9a5045e1b3b33edc07d8c9bd55250fa90`, 385 386 bytes. Upstream `https://github.com/mrdoob/three.js`. All of it recorded in `apps/nomos-viewer/vendor/MANIFEST.json`. |
| **license** | MIT, preserved verbatim at `apps/nomos-viewer/vendor/three/LICENSE`, sha256 `8b378ebe60e2fe500158cb0ac71cb5e8b7d92953c2abcc63a0eb90499653b5bc`, 1 081 bytes ("Copyright © 2010-2026 three.js authors"). |
| **why not local** | A WebGL2 scene graph with material and shader compilation, orthographic camera math, shadow maps, and generated geometry — extrusion with bevels, torus, cone, icosahedron, cylinder, plane. A local implementation would be a second renderer to maintain and would not be more trustworthy for being ours; `RUNTIME.md` §4 admits exactly this case outside the six kernel crates, and `AGENTS.md` states the zero-dependency rule "was never a permanent claim that later epochs should reimplement mature libraries". |
| **determinism** | Cannot affect authoritative state, hashes, or receipts. It is loaded only by `apps/nomos-viewer/`, which consumes published artifacts and writes none. No kernel crate, no R1 crate, no `xtask` target, and no step that produces a canonical artifact links or executes it; the plans and their digests are produced by `nomos-render-plan` before `build.mjs` runs. Bounded by: `cargo xtask boundary` (§10 finding 4 adds the `apps/` isolation rule `RUNTIME.md` §3 promises), the scan's file-shape rule, and the smoke lane, which hashes no GPU output — `RUNTIME.md` §9 already states the pixels are not deterministic across GPUs, and no receipt depends on them. |
| **offline proof** | The file is committed; there is no `npm install`, no lockfile to resolve, and no bundler. `node --test apps/nomos-viewer/test/vendor.test.mjs` recomputes every sha256 and byte count from the working tree, compares them to `vendor/MANIFEST.json`, and asserts that the only module specifier either file carries is the relative sibling. `build.mjs` re-checks them while copying, and the scan refuses any external origin in `dist/`. The smoke lane runs Chrome with `--host-resolver-rules="MAP * ~NOTFOUND, EXCLUDE localhost"`, records every request in the receipt, requires the external list to be empty, and proves the rule is in force with a negative-control probe that must fail. |
| **added by** | Issue #148, pull request #151. |

`RUNTIME.md` §3 gains one sentence noting `apps/nomos-viewer/` is present and
that `cargo xtask boundary` now enforces its isolation (§10 finding 4); §6's
comparison block drops `gaol site` and gains the smoke lane's local entry point
(§10 finding 6); §7's "Public artifact size" row gains the measured byte total
of `apps/nomos-viewer/dist/` with the runner that produced it.

---

## 8. Workflows, and the deletion list

### 8.1 `.github/workflows/nomos-viewer.yml` (new)

`on: push [main]`, `pull_request`, `workflow_dispatch`. One job on
`ubuntu-24.04`, `timeout-minutes: 20`, following the conventions already in the
tree: `actions/checkout@v7`, `actions/upload-artifact@v7`, `retention-days: 90`,
`if-no-files-found: error`.

```text
- actions/checkout@v7
- actions/setup-node@v5 with node-version 22       # see below
- node --test apps/nomos-viewer/test/              # decoder, catalog, play, render, ui, vendor, scan
- experiments/executable-gaol/gaol verify          # produces the published artifacts
- node apps/nomos-viewer/build.mjs --from target/executable-gaol --out apps/nomos-viewer/dist
- node apps/nomos-viewer/smoke/smoke.mjs --dist apps/nomos-viewer/dist --require-chrome
    --out target/nomos-viewer-smoke
- upload-artifact: target/nomos-viewer-smoke       # screenshots + receipt.json
```

**Why `setup-node`.** The harness uses the global `WebSocket`, which is stable
from Node 22 and absent by default in Node 20. Nothing else in the tree pins a
Node version, and the runner image's default is not something this lane should
depend on. `actions/setup-node` is a first-party GitHub action alongside the
four already used; it installs a runtime, not a package. The harness also checks
`typeof WebSocket` and fails with a legible message rather than a `ReferenceError`.

### 8.2 `.github/workflows/executable-gaol-pages.yml` (changed)

- `paths:` gains `apps/nomos-viewer/**` and `.github/workflows/nomos-viewer.yml`;
- the build job runs `gaol verify`, then `build.mjs`, then the app's node tests,
  then the smoke lane with `--require-chrome`, and only then uploads;
- `actions/upload-pages-artifact@v5` `path:` changes from
  `target/executable-gaol-site` to `apps/nomos-viewer/dist`;
- the screenshots and the receipt are uploaded alongside, so the published page
  and the evidence that it plays come from the same run;
- the workflow name and the deploy job are unchanged.

### 8.3 Deletions, and what stays

**Deleted outright**

| Path | Why |
| --- | --- |
| `experiments/executable-gaol/viewer.html` | Replaced by `apps/nomos-viewer/index.html`. |
| `experiments/executable-gaol/src/webgl-renderer.mjs` | Replaced by `src/render.mjs`; this is the file with the CDN import. |
| `experiments/executable-gaol/src/play-state.mjs` | Replaced by `src/play.mjs`. |
| `experiments/executable-gaol/src/play-state.test.mjs` | Its ten tests are reproduced by `test/play.test.mjs` (promotion rows 19–28). |
| `experiments/executable-gaol/src/webgl-viewer.test.mjs` | Its source-level assertions are replaced by real tests (rows 29–34) and the scan. |
| `experiments/executable-gaol/src/serve.mjs` | Dead once `gaol serve` goes; `smoke/server.mjs` is the accepted equivalent. **Not in the issue's list**, added here rather than left orphaned. |
| `gaol`'s `serve` and `site` cases, and `stage_site()` (`gaol:68-85,96-101,115-119`) | The viewer has its own build and its own server. |

**Kept, and why**

`areas/**` (all content), `area-collection.example.json`,
`src/build-collection.mjs` (its `areas.json` is the published collection the
viewer consumes), `src/verify.mjs`, `src/render-core.mjs` (the SVG evidence
renderer), `src/capture.mjs`, `src/capture-collection.mjs`,
`src/area-collection.test.mjs`, `compare-effective-facts.sh`,
`compare-rendering-plan.sh`, `gaol capture` and `gaol verify`, `contact-sheet.png`,
`AUTHORING.md`, `CAPTURE.md`, `README.md`.

**`src/renderer-catalog.mjs` is trimmed, not deleted** — §10 finding 5. The
issue lists it for deletion, but `src/render-core.mjs:1-8`,
`src/verify.mjs:3-12`, and `src/area-collection.test.mjs:14` all import it, and
all three are files the issue says the experiment keeps. Deleting it breaks the
SVG evidence renderer and `gaol verify`. What actually becomes unused when the
WebGL viewer leaves is two exports — `LOOK_PROFILE_IDS` and `isHunting` — and
those are removed; the rest stays as the study's own catalog. The accepted
catalog under `apps/` is written fresh regardless, so nothing is shared.

### 8.4 Documents

`RUNTIME.md` §3, §4, §6, and §7 as in §7 above and §10 finding 6.
`docs/workspace.md`'s rule table gains the `apps/` isolation rule.
`docs/HANDOFF.md` ("How to verify"
at line 101 names `gaol site`; the R1-4 paragraph at line 144 becomes a landed
paragraph). `README.md` status. `experiments/executable-gaol/README.md` (the
`gaol serve`/`gaol site` sections at lines 43-47 and 97-105, and the "Play
online" line, which now points at the promoted viewer).
`apps/nomos-viewer/README.md` is new.

---

## 9. The area-addition proof

`RUNTIME.md` §5 R1-4: "adding an area edits no file under `apps/nomos-viewer/`,
proved by the diff of the commit adding it." `RUNTIME.md` §1 criterion 5 says
the same for renderer and compiler source.

Plan:

1. Branch `scratch/issue-148-area-proof` from this branch's head. Never merged,
   no pull request; the diffstat is pasted into this PR's body.
2. Add `experiments/executable-gaol/areas/warden-stair/` — a fifth area with a
   different composition from North Gaol, not a copy of its numbers: different
   `bounds`, two masonry masses where North Gaol has none, a water region in a
   different place, two doors, one brazier, a different `wall_height_steps`, and
   its own five scenario command scripts. Files: `world.nomos`,
   `presentation.json`, `scenarios/*.commands` (5), and the generated
   `rendering-plan.example.json`.
3. Append it to the route: `areas/north-gaol/presentation.json` changes
   `route.exit.to_area` from `null` to `"warden-stair"`, and the new area
   declares its own `route.entry` and `to_area: null`.
4. Run `experiments/executable-gaol/gaol verify`, which regenerates
   `areas/north-gaol/rendering-plan.example.json` and
   `area-collection.example.json`.
5. Run `node apps/nomos-viewer/build.mjs` and the smoke lane. The lane must
   reach the final escape through **five** areas with no edit to the harness:
   the solver reads the route from the artifacts, so the new counters are
   derived, not typed.
6. Record `git diff --stat` against the branch point in the PR body.

**What the diff touched**, measured on `scratch/issue-148-area-proof` — twelve
files, 116 insertions, 4 deletions:

```text
experiments/executable-gaol/areas/warden-stair/**            (new: source,
    presentation, five command scripts, compiled plan)
experiments/executable-gaol/areas/north-gaol/presentation.json           (1 line)
experiments/executable-gaol/areas/north-gaol/rendering-plan.example.json (regenerated)
experiments/executable-gaol/area-collection.example.json                (regenerated)
experiments/executable-gaol/src/area-collection.test.mjs                 (7 lines)
```

and **nothing** under `apps/`, `crates/`, `xtask/`, or `.github/`. That is the
criterion `RUNTIME.md` §5 R1-4 states, and §1 criterion 5's "no edit to renderer
or compiler source" with it.

The twelfth file is the one this record did not predict: the study's own
collection test enumerates the corpus — it names the terminal area and iterates
a fixed list of four plans — so a fifth area is a seven-line edit to it. It is a
quarantined test fixture, neither renderer nor compiler source, and it is
recorded here rather than quietly absorbed. The viewer's own 105 tests are
untouched.

The issue's parenthetical — "the diff touches only
`experiments/executable-gaol/areas/<new>/`" — is not reachable for a *connected*
area, because `src/build-collection.mjs:64-85` requires the route to visit every
declared area, so the new area must be named by an existing one. §10 finding 7.

**What it proved.** `gaol verify` green with five `EXECUTABLE_GAOL_VERIFY`
receipts and the visual-grammar digest unchanged at `e3f338b1`; the staged
artifact one plan larger at 15 files and 904 520 bytes; and the browser lane
playing **five** areas to the final escape with no edit to the harness — 51
moves, 71 traversal cost, zero console errors — because the solver reads the
artifacts rather than a walk written down anywhere.

---

## 10. Findings against the issue

Seven findings. None makes the issue impossible. Three wanted an owner ruling;
all seven are ruled, and what ships is what the rulings say.

### Finding 1 — the acceptance grep and the scan rule both have a false positive, and both are fixable — MEASURED, ACCEPTED

Two of the issue's acceptance lines are literal string checks that the artifacts
as they exist today would fail.

**(a) `grep -r "https://" apps/`.** The acceptance says it "returns only the
license text and documentation comments". Measured against the real file:
`three@0.185.1`'s `build/three.module.min.js` contains **exactly one**
occurrence of `https://` — the string
`https://jcgt.org/published/0007/04/01/` inside a GLSL comment in a shader
source string (the citation for the octahedral-normal encoding paper). It is a
documentation comment, so the acceptance line is satisfied as written; recorded
here because "one occurrence, and it is a paper citation in a shader comment" is
a measurement, not an assumption, and a future three.js bump could change it.
`vendor/MANIFEST.json` records the count so a bump that adds a second occurrence
is a visible diff.

**(b) The scan's "fails on any `.nomos`".** Every published plan carries
repo-relative `.nomos` paths: `entities[].provenance[].source.path` is
`experiments/executable-gaol/areas/<area>/world.nomos`, five occurrences in
`north-gaol/rendering-plan.example.json` and equivalents in the other three. A
scan that fails on any `.nomos` fails on the artifact the viewer is supposed to
publish. `RUNTIME.md` §5 R1-4 says the artifact "contains no `.nomos` source" —
a citation of a source path is not source — so the contract is satisfiable; the
issue's wording is tighter than the contract. **Resolution, §6 rule 4**: the
scan refuses `.nomos` everywhere except as a `provenance[].source.path` value
matching `^experiments/executable-gaol/areas/[a-z0-9-]+/world\.nomos$`, and
separately refuses `.nomos` *content* markers and any absolute path. A planted test proves each half.

**Ruled: accepted as designed.** The scan allows the single
structurally-located repo-relative citation and refuses `.nomos` content
markers and absolute paths, each with a planted test, and
`vendor/MANIFEST.json` records the `https://` occurrence count so a Three.js
bump that adds a second one is a visible diff. Stripping the provenance block
instead was declined: the staged plan would no longer be the published one.

### Finding 2 — the accepted app binds an identity declared by quarantined tooling — RULED (a)

`areas.json` carries `nomos.experiment.area_collection@2`, declared at
`experiments/executable-gaol/src/build-collection.mjs:88`. The issue's Scope
keeps `build-collection.mjs` in the experiment on purpose ("its `areas.json` is
the published collection artifact the viewer consumes"), so an accepted app will
bind, at runtime, an identity whose only declaration is in a tree `RUNTIME.md`
§2 calls non-authoritative.

Nothing forbids it: `RUNTIME.md` §3 says the viewer "consumes published plan and
presentation artifacts only", and the register lane
(`docs/evaluation/r1-schema-ownership.sh`) enumerates identities under
`crates/*/src` only, so nothing breaks and nothing needs registering. But it is
worth naming rather than discovering later: the four `rendering-plan.json` files
are accepted output, and the file that stitches them into a route is not.

**Ruled: (a).** R1-4 binds `nomos.experiment.area_collection@2` now, and this
record and `apps/nomos-viewer/README.md` state that its declaring file is
quarantined tooling. Issue #152, "Promote the area collection into
nomos-render-plan", carries the follow-up: a registered identity emitted by the
accepted crate. Option (c) — deriving the collection inside `build.mjs` — was
declined, because it would move route-graph validation
(`build-collection.mjs:43-85`) into the app, where the viewer would own a fact
no artifact declares.

### Finding 3 — resolving the kind→assembly deferral fully needs a plan `@3`, which is out of scope — RULED (a)

`crates/nomos-render-plan/src/catalog.rs:50-63` says of its two tables: "**This
is the last place in the tree where a visual assembly name or a material family
is assigned to an entity kind outside the renderer catalog.** No later slice may
add a third such table; the correct change is to move these two out."

Moving them out means the plan stops carrying `entities[].visual_assembly` and
`entities[].material_family` and the viewer derives both from `kind` — a
`nomos.rendering_plan@3`, regenerated fixtures, a changed
`visual_grammar.entity_assemblies` in the collection, and Rust work this issue's
Scope does not describe.

§3.2 therefore resolves the deferral as far as the scope reaches: the accepted
catalog owns what the names *mean* and refuses one it does not know; the
compiler's table becomes a selection from that set, the same definition/selection
split the owner already ruled for the five content fields; and a test asserts the
two sides name exactly the same three rows, so neither can drift. That is a
genuine single-owner story, but it is not the deletion `catalog.rs` asks for, and
it does add a *set* to the renderer while leaving the *mapping* in Rust.

**Ruled: (a).** §3.2 is the R1-4 resolution — the catalog owns what the names
mean, refuses one it does not know, and
`the_catalog_knows_every_assembly_the_compiler_can_emit` parses
`crates/nomos-render-plan/src/catalog.rs` so neither side can drift. Issue #153,
"Move kind→assembly assignment out of the Rust plan compiler
(rendering_plan@3)", carries the move itself. Widening R1-4 was declined: it
would put a schema change, four regenerated fixtures, and a collection-grammar
change inside the slice that is also introducing the first `apps/` member and
the first browser lane.

### Finding 4 — `RUNTIME.md` §3 promises a boundary-checker rule this issue does not mention — CONFIRMED

§3, last paragraph: "Viewer isolation is not enforced yet — no `apps/` member
exists — and joins the checker with R1-4."

The issue's Scope says nothing about `cargo xtask boundary`. `apps/nomos-viewer/`
is JavaScript, so it never appears in `cargo metadata` and the existing
membership rule cannot see it either way. The smallest rule that makes the
promise true, and that the checker's existing data supports:

> **`apps/` isolation.** No workspace member's `manifest_path` lies under
> `<workspace root>/apps/`. A crate placed there would be a workspace member the
> kernel graph could reach, and `RUNTIME.md` §3 forbids a kernel crate depending
> on `apps/`.

**Confirmed.** It ships in this slice: the rule in `xtask/src/boundary.rs`, one
planted-violation test in `xtask/src/planted.rs`, the module doc, a row in
`docs/workspace.md`'s rule table, and `RUNTIME.md` §3's sentence changed from
"joins the checker with R1-4" to a statement that it now holds.

### Finding 5 — the deletion list is not self-consistent — RESOLVED IN THIS RECORD

The issue deletes `experiments/executable-gaol/src/renderer-catalog.mjs` and in
the same paragraph keeps "the SVG evidence renderer, and `build-collection.mjs`"
plus `gaol verify`. But `src/render-core.mjs:1-8` imports six symbols from it,
`src/verify.mjs:3-12` imports eight, and `src/area-collection.test.mjs:14`
imports from it too. Deleting it breaks the retained SVG renderer and `gaol
verify` in the same commit.

Resolution, §8.3: trim rather than delete. `LOOK_PROFILE_IDS` and `isHunting`
are the only exports whose consumers all leave, and they go; the units, the
socket table, the closed sets, and the plan accessors stay because three
retained files use them. `gaol verify` stays green, and the accepted catalog is
still written fresh, so no line is shared between the trees. Recorded as a
correction to the issue's list rather than a silent deviation.

### Finding 6 — deleting `gaol site` removes a command `RUNTIME.md` §6 names — RULED

`RUNTIME.md` §6 lists, as the comparison target:

```text
experiments/executable-gaol/gaol verify
experiments/executable-gaol/gaol site
```

The issue deletes the `site` subcommand. After that, a document under owner
authority names a command that does not exist. §6 also anticipates the
replacement in its next paragraph — "Once R1-4 exists, its headless Chromium
smoke lane runs in CI on every change and locally through the same entry point"
— and the preamble scopes the comparison block to "while R1-2 and R1-3 are
open", both of which are now landed. So the premise has expired.

**Ruled: make the minimal edit.** §6's comparison block is scoped to "while
R1-2 and R1-3 are open", both of which are landed, so this is the expiry the
section anticipated rather than a reinterpretation, and it is not a §8 repair.
The `gaol site` line is dropped and `node apps/nomos-viewer/smoke/smoke.mjs` is
added as the R1-4 lane's local entry point; `gaol verify` stays. The pull
request quotes the before and after under a "Contract text touched" heading so
the owner sees the changed contract line beside the other pending ones.

### Finding 7 — the area-addition parenthetical is unreachable; the contract criterion is not — RESOLVED IN THIS RECORD

The issue asks to prove "the diff touches only
`experiments/executable-gaol/areas/<new>/`". A fifth area that is actually
reachable must be named by an existing area's `route.exit.to_area`, because
`src/build-collection.mjs:64-85` requires exactly one start area and a route
that visits every declared area; and `gaol verify` `cmp`s the regenerated
`area-collection.example.json`. So the minimum honest diff is the new directory,
one line in the predecessor's `presentation.json`, and two regenerated fixtures
— all under `experiments/executable-gaol/`, none under `apps/`.

`RUNTIME.md` §5 R1-4's own criterion — "adding an area edits no file under
`apps/nomos-viewer/`" — is met exactly, and §1 criterion 5's "no edit to
renderer or compiler source" is met exactly. §9 states the precise claim the
proof will make.

### Finding 8 — the file the issue names is half of a two-file build — FIXED IN THIS SLICE

Raised in phase 2, by the test rather than by a browser.
`three@0.185.1`'s `build/three.module.min.js` is **not** self-contained: it
carries `from"./three.core.min.js"` twice — one import and one re-export — so
vendoring only the file the issue names would have published a page whose first
module request 404s. `build/three.core.min.js` is 385 386 bytes, imports
nothing, and contains one URL of its own: `http://www.w3.org/1999/xhtml`, the
XML namespace identifier passed to `document.createElementNS`, which is a name
and not a fetch target.

Both files are vendored, both are digest-pinned in `MANIFEST.json`, both are
staged by `build.mjs`, and `the_vendored_modules_import_only_their_own_siblings`
asserts that every specifier either file carries is relative and resolves to a
vendored file. The constraint is unchanged — the specifier between them is a
same-origin relative URL — and the acceptance grep still returns only the two
documentation strings above. Recorded because the issue names one file and the
tree now holds two.

### Nothing in the issue is impossible

Each settled decision holds, and the ones that were open are measured rather
than assumed:

- **The vendored file exists at the exact digests the record names**, and the
  registry `dist.integrity` for `three@0.185.1` was verified against the tarball
  bytes before this record was written.
- **A dependency-free CDP client is enough.** Every capability the lane needs —
  navigation, key input, evaluation, screenshots, exceptions, console, log, and
  network — is a page-target domain, so no `Target` multiplexing and no library
  is required. The only version constraint is a Node with a global `WebSocket`,
  which §8.1 pins.
- **The route is derivable from the artifacts.** §5.3's solver, run against the
  four committed plans, produces a 60-key sequence reaching the final escape
  with cumulative counters `44 moves / 60 cost`, with no area identifier and no
  cell coordinate written into the harness.
- **WebGL initialises headlessly** through SwiftShader, with the two flag sets
  in §5.1 covering both the `--enable-unsafe-swiftshader` and the `--disable-gpu`
  routes to it.
- **The offline claim is measurable**, not merely assertable: the negative
  control in §5.5 fails the lane if the host-resolver rule is not actually in
  force.
