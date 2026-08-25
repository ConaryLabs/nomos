---
title: Presentation-boundary ownership audit for area.json and RenderingPlan@1
status: Audit only; no code changed; owner disposition pending
date: 2026-08-25
issue: 125
scope: experiments/executable-gaol (quarantined, non-authoritative)
---

# Presentation-boundary ownership audit for area.json and RenderingPlan@1

This is an audit of `experiments/executable-gaol`, produced against issue #125.
It assigns every field in the four `area.json` files, `area-collection.example.json`,
and `nomos.experiment.rendering_plan@1` to exactly one proper owner, lists every
current double authority, lists every classification derived by convention or
magic string, and lists every raw floating-point presentation value. It does not
recommend a design. `experiments/executable-gaol` is quarantined per
`AGENTS.md` ("Quarantined experiments are allowed... non-authoritative, cannot
satisfy acceptance"); nothing here changes the accepted Gate K workspace.

Owner categories used below, per the issue's Scope: **World IR**, **runtime
state**, **kernel projection**, **presentation source**, **renderer catalog**,
**area/gameplay graph**, **test fixture**, **tooling only**.

Files audited:

- `experiments/executable-gaol/areas/{cistern-walk,ember-vault,north-gaol,ossuary-reach}/area.json`
- `experiments/executable-gaol/area-collection.example.json`
- `experiments/executable-gaol/areas/{cistern-walk,ember-vault,north-gaol,ossuary-reach}/rendering-plan.example.json`
- `experiments/executable-gaol/src/build-plan.mjs`
- `experiments/executable-gaol/src/build-collection.mjs`
- `experiments/executable-gaol/src/play-state.mjs`
- `experiments/executable-gaol/src/render-core.mjs`
- `experiments/executable-gaol/src/webgl-renderer.mjs`
- `experiments/executable-gaol/viewer.html`

`north-gaol` is used as the line-citation representative for fields whose shape
is identical across all four areas; values that differ per area are cited for
all four.

## 1. Field ownership

| Field | Source file | Current authority | Proper owner | Notes |
|---|---|---|---|---|
| `id` | `areas/north-gaol/area.json:2` (all 4); `rendering-plan.example.json:5` (`area.id`, `build-plan.mjs:174` passthrough) | Author-typed in `area.json`; copied unchanged into the plan | area/gameplay graph | Structural key other areas' `exit.toArea` and the collection's `route`/`areas[]` reference by string. |
| `label` | `areas/north-gaol/area.json:3`; `rendering-plan.example.json:6` (`build-plan.mjs:174`) | Author-typed | presentation source | Only display string authored directly; every other on-screen name is derived from an id (`play-state.mjs:12`, see §4 item 26). |
| `start` | `areas/north-gaol/area.json:4`; `rendering-plan.example.json:7` (`build-plan.mjs:174`) | Author-typed bool | area/gameplay graph | `build-collection.mjs:55-57` requires exactly one `true` across all areas; correctly single-owner. |
| `primaryGate` | `areas/north-gaol/area.json:5`; `rendering-plan.example.json:232` (`presentation.primaryGate`, `build-plan.mjs:184`) | Author-typed; validated to be a compiled entity (`build-plan.mjs:56`) | area/gameplay graph | Forced textually identical to `objective.target` and `exit.gate` — see §3.5. |
| `objective.kind` | `areas/north-gaol/area.json:6`; `rendering-plan.example.json:234` | Author-typed, but only `"exit_via"` is legal (`build-plan.mjs:60`) | area/gameplay graph | Single legal value in the whole corpus; effectively a constant, not real content. |
| `objective.target` | `areas/north-gaol/area.json:6`; `rendering-plan.example.json:235` | Author-typed; validated `=== area.primaryGate` (`build-plan.mjs:63`) | area/gameplay graph | Redundant with `primaryGate`/`exit.gate` — see §3.5. |
| `pursuitLight` | `areas/north-gaol/area.json:7`; `rendering-plan.example.json:237` (`presentation.pursuitLight`) | Author-typed; validated to be a compiled entity (`build-plan.mjs:64`) | area/gameplay graph | Drives presentation-only gaoler AI (`play-state.mjs:137-159`); no kernel pursuit system exists. |
| `forensicScenario` | `areas/north-gaol/area.json:8`; `rendering-plan.example.json:238` | Author-typed | test fixture | Value `"03-breached-unsealed"` is identical in all 4 `area.json` files — a copy-pasted constant, not per-area content. Consumed only by `experiments/executable-gaol/src/capture.mjs:16` (outside audited scope). |
| `exit.gate` | `areas/north-gaol/area.json:10`; `rendering-plan.example.json:240` | Author-typed; validated `=== area.primaryGate` (`build-plan.mjs:65`) | area/gameplay graph | Third field forced equal to `primaryGate` — see §3.5. |
| `exit.toArea` | `areas/north-gaol/area.json:11` (`null`); `cistern-walk/area.json:11` (`"ember-vault"`); `rendering-plan.example.json:241` | Author-typed; cross-checked against the collection's known area ids (`build-collection.mjs:42-43`) | area/gameplay graph | Graph edge target; correctly single-owner (checked, not duplicated). |
| `exit.entry` | `cistern-walk/area.json:12` (`{x:7,y:5,z:0}`); absent when `toArea` is `null` (`north-gaol/area.json:9-12`) | Author-typed; validated against the target area's `bounds`/`masses` (`build-collection.mjs:44-51`) | area/gameplay graph | Landing cell used by `play-state.mjs:32-42` (`enterArea`) to place the player; integer lattice coordinate, not a raw float. |
| `architecture.bounds.width` / `.height` | `areas/north-gaol/area.json:14`; `rendering-plan.example.json:44-46` (`build-plan.mjs:179` passthrough) | Author-typed; validated against hardcoded `9`/`6` (`build-plan.mjs:73`) | World IR | Per `THESIS.md:280` (`spatial.boundary -> lattice`), room extent is a lattice fact. Here it is never cross-checked against the compiled `world.nomos` extent at all — purely a JS-side magic-number check. |
| `architecture.wallHeight` | `areas/{north-gaol:15, cistern-walk:16, ember-vault:16, ossuary-reach:16}/area.json`; `rendering-plan.example.json:47` | Author-typed float; bounded `(0,5]` only in `build-plan.mjs:76-78` | presentation source | Double authority on unit/scale — see §3.2. |
| `architecture.style.assembly` | `areas/north-gaol/area.json:17`; `rendering-plan.example.json:49`; `area-collection.example.json:37` (`grammar.architectureStyle.assembly`) | Author-typed, identical in all 4 areas | renderer catalog | AUTHORING.md mandates one fixed value; effectively renderer-owned data authored 4+1 redundant times. `build-collection.mjs:36-38` does enforce cross-area equality — one of the few checked invariants in the corpus. |
| `architecture.style.materialFamily` | same as above, `area.json:18` | same | renderer catalog | Same pattern as `assembly`. |
| `architecture.style.trimFamily` | same as above, `area.json:19` | same | renderer catalog | Same pattern as `assembly`. |
| `architecture.masses[].id` | `cistern-walk/area.json:24`, `ember-vault/area.json:24,30`, `ossuary-reach/area.json:24,30,36` (`north-gaol` has none: `area.json:21`) | Author-typed | presentation source | README: masonry-mass collision is "presentation-only because Gate K has no dynamic actor or architecture state." |
| `architecture.masses[].min` / `.max` | same locations | Author-typed integer cell rectangle; validated against `bounds` (`build-plan.mjs:80`) | presentation source | Integer lattice rectangle, not a raw float. |
| `architecture.masses[].height` | `cistern-walk/area.json:27`; `ember-vault/area.json:27,33`; `ossuary-reach/area.json:27,33,39` | Author-typed float; bounded `(0,4]` (`build-plan.mjs:81`) | presentation source | Double authority on unit/scale — see §3.3. |
| `actors[].id` | `areas/north-gaol/area.json:25,30` (all 4) | Author-typed; must literally include `"player"` and `"gaoler"` (`build-plan.mjs:66-68`) | presentation source | Magic-ID gameplay role — see §4 item 21. No Gate K actor/entity backs this. |
| `actors[].assembly` | `areas/north-gaol/area.json:26,31` | Author-typed, always one of two fixed strings | renderer catalog | Constant vocabulary; effectively renderer data. |
| `actors[].anchor.kind` | `areas/north-gaol/area.json:27,32` (`"cell"`) | Author-typed | presentation source | Always `"cell"` across the entire corpus — a single-valued enum, no other kind ever appears. |
| `actors[].anchor.cell` | `areas/north-gaol/area.json:27,32`; `rendering-plan.example.json:199-203,211-215` (`build-plan.mjs:181` passthrough) | Author-typed integer cell | presentation source | Double authority with `play-state.mjs:18-19` hardcoded fallback defaults — see §3.4. |
| `effects[].id` | `areas/north-gaol/area.json:37` | Author-typed | presentation source | |
| `effects[].assembly` | `areas/north-gaol/area.json:38` (`"visual/cyan_crescent"`) | Author-typed, single value in the entire corpus | renderer catalog | Matched by exact-string comparison in two renderers — see §4 item 17. |
| `effects[].anchorEntity` | `areas/north-gaol/area.json:39`; validated to reference a compiled entity (`build-plan.mjs:69-71`) | Author-typed | presentation source | Binds decorative effect visibility to a semantic entity's `ward` state. |
| `effects[].presentationAnchor` | `areas/{north-gaol:40, cistern-walk:48, ember-vault:54, ossuary-reach:60}/area.json` | Author-typed raw float `{x,y,z}` | presentation source | Raw transform — see §5. Field name itself concedes it is presentation, but per `THESIS.md:733-736` shippable content should not carry raw transforms at all; this experiment is explicitly exempt as a "tainted laboratory." |
| `schema` (plan) | `rendering-plan.example.json:2` (`build-plan.mjs:172`) | Hardcoded literal in `build-plan.mjs:172` | tooling only | Never read by `render-core.mjs`, `webgl-renderer.mjs`, or `viewer.html`. |
| `deterministic` (plan) | `rendering-plan.example.json:3` (`build-plan.mjs:173`) | Hardcoded `true` in `build-plan.mjs:173` | tooling only | Never read downstream; dead declared-intent flag. |
| `projectionSchemas[].name` / `.version` | `rendering-plan.example.json:9-25` (`build-plan.mjs:175`, sourced from `simulation.schema` etc.) | Kernel-projection-sourced | kernel projection | Declared but unconsumed beyond the collection's grammar-equality check (`build-collection.mjs:21,36-38`). |
| `projectionDigests` | `rendering-plan.example.json:27-32` (`build-plan.mjs:164-169`) | Computed by hashing the raw projection JSON bytes | test fixture | Never read by any renderer or `viewer.html`; pure integrity evidence. |
| `camera.identity` | `rendering-plan.example.json:34` (`build-plan.mjs:177`); `area-collection.example.json:28` | Hardcoded literal `"gaol_oblique_01"` in `build-plan.mjs:177` | renderer catalog | Double authority — see §3.1. Read by neither renderer. |
| `camera.projection` | `rendering-plan.example.json:35`; `area-collection.example.json:29` | Hardcoded literal `"fixed_oblique"` | renderer catalog | Read by neither renderer. |
| `camera.width` / `.height` / `.tileWidth` / `.tileHeight` | `rendering-plan.example.json:36-39`; `area-collection.example.json:30-33` | Hardcoded literals in `build-plan.mjs:177` | renderer catalog | Double authority — see §3.1. Only `render-core.mjs:53-56` reads these; `webgl-renderer.mjs` ignores them entirely. |
| `palette` | `rendering-plan.example.json:41` (`build-plan.mjs:178`); `area-collection.example.json:35` | Hardcoded literal `"gaol_bounded_01"` | renderer catalog | Never dereferenced — see §3.9. Both renderers hardcode their own independent color tables regardless of this string's value. |
| `entities[].id` | `rendering-plan.example.json:57,75,107,150` (`build-plan.mjs:40`, from `simulation.json` `entity.id`) | Kernel-projection-sourced | kernel projection | Correctly single-owner; direct passthrough. |
| `entities[].kind` | `rendering-plan.example.json:58,76,108,151` (`build-plan.mjs:24-29,32`) | **`build-plan.mjs` (derived)** — heuristic classification, see §4 items 1-4 | World IR (should be a compiled primitive-type tag) | Headline finding of issue #125; classified from `.endsWith(".access")`, light-resolver membership, and navigation-claim capability, not read from any declared type. |
| `entities[].visualAssembly` | `rendering-plan.example.json:59,77,109,152` (`build-plan.mjs:33-38`) | **`build-plan.mjs` (derived)** from a hardcoded `kind → assembly` table | renderer catalog | Correct target category, wrong location: the catalog lives inside the content compiler, not the renderer. |
| `entities[].materialFamily` | `rendering-plan.example.json:60,78,110,153` (`build-plan.mjs:43`) | **`build-plan.mjs` (derived)**, defaults unknown kinds to `"stone"` | renderer catalog | Same misplacement as `visualAssembly`. |
| `entities[].anchor.kind` / `.cell` / `.direction` / `.min` / `.max` | `rendering-plan.example.json:61-68` (cell), `79-91` (region), `111-119` (face) (`build-plan.mjs:44`, from `simulation.json` `entity.binding`) | Kernel-projection-sourced (World-IR-derived) | World IR | Correctly single-owner passthrough — one of the few clean examples in the corpus. `anchor.direction` is declared but never read (see §4 item 22). |
| `entities[].machineNamespaces` | `rendering-plan.example.json:69-71,120-125` (`build-plan.mjs:45`, from `entity.machines`) | Kernel-projection-sourced | kernel projection | Never consumed by `render-core.mjs`, `webgl-renderer.mjs`, `play-state.mjs`, or `viewer.html` — dead outside forensic value. |
| `entities[].provenance[]` (`claim`, `source.path/line/column/byte_start/byte_end`) | `rendering-plan.example.json:93-104,126-147` (`build-plan.mjs:46-49`, from navigation claims) | Kernel-projection-sourced | test fixture | Byte-range citations into `world.nomos`; never read by any consumer, forensic-only. |
| `uiAnchors` | `rendering-plan.example.json:244-249` (`build-plan.mjs:190`); `area-collection.example.json:65-70` | Hardcoded literal array in `build-plan.mjs:190` | renderer catalog | Fully dead: the strings `"vitals"`/`"abilities"`/`"gate_state"`/`"water_cost"` appear nowhere else in the codebase — no matching DOM ids in `viewer.html`, no reads in either renderer. |
| `scenarios[].id` | `rendering-plan.example.json:252` (`build-plan.mjs:97-100,132`, from scenario-capture directory names) | Directory-name-sourced | test fixture | |
| `scenarios[].label` | `rendering-plan.example.json:253` (**`build-plan.mjs:133` (derived)**, regex-stripped from `id`) | **`build-plan.mjs` (derived)** | presentation source | Consumed as UI text (`viewer.html:106`, `render-core.mjs:70,193`) despite being derived from a test-fixture directory-naming convention — see §4 item 14. |
| `scenarios[].tick` | `rendering-plan.example.json:254` (`build-plan.mjs:134`, from `finalState.state.tick`) | Runtime-state-sourced | runtime state | Correctly single-owner. |
| `scenarios[].stateHash` | `rendering-plan.example.json:255` (`build-plan.mjs:135`, from `finalState.state_hash`) | Runtime-state-sourced | runtime state | Correctly single-owner; used to bind interactions. |
| `scenarios[].machineStates` | `rendering-plan.example.json:256-265` (`build-plan.mjs:108-110`, from `finalState.state.machines`) | Runtime-state-sourced | runtime state | Correctly single-owner as a raw snapshot; every higher-level fact (ward/access/integrity booleans) is then re-derived from this map independently in up to 4 places — see §3.6, §4 items 18-20. |
| `scenarios[].movement[entity].disposition/.cost/.reasons` | `rendering-plan.example.json:267-291` (**`build-plan.mjs:111-121` (derived)**) | **`build-plan.mjs` (derived)** — reimplements the navigation resolver | kernel projection | Should be an already-resolved kernel-projection fact (`runtime.effective_fact -> resolver-derived`, `THESIS.md:283`); instead recomputed by a hand-rolled activation interpreter (`build-plan.mjs:86-95`). |
| `scenarios[].effectiveLight` | `rendering-plan.example.json:292-294` (**`build-plan.mjs:123-128` (derived)**) | **`build-plan.mjs` (derived)** — reimplements the light resolver | kernel projection | Same pattern as `movement`. |
| `interactions[].id` | `rendering-plan.example.json:469` (**`build-plan.mjs:154` (derived)**, string template) | **`build-plan.mjs` (derived)** | test fixture | Synthetic id, not a stable declared identifier. |
| `interactions[].fromScenario/.toScenario/.targetEntity/.action` | `rendering-plan.example.json:470-473` (**`build-plan.mjs:144-161` (derived)**, reconstructed by diffing command-log row prefixes) | **`build-plan.mjs` (derived)** | test fixture | Heuristic reconstruction of "what real command produced this transition," not a declared edge. |
| `interactions[].inputStateHash/.resultingStateHash` | `rendering-plan.example.json:474-475` (`build-plan.mjs:159-160`, from command-log rows) | Runtime-state-sourced | runtime state | Correctly single-owner; the one part of `interactions[]` that is a direct fact rather than a reconstruction. |
| `collection.schema` | `area-collection.example.json:2` (`build-collection.mjs:71`) | Hardcoded literal | tooling only | |
| `collection.deterministic` | `area-collection.example.json:3` (`build-collection.mjs:72`) | Hardcoded `true` | tooling only | Never read downstream. |
| `lookProfile.id` | `area-collection.example.json:5` (`build-collection.mjs:74`, hardcoded `"gaol_bounded_01"`) | Hardcoded literal | renderer catalog | Double authority — see §3.8. |
| `lookProfile.digest` | `area-collection.example.json:6` (`build-collection.mjs:75`, sha256 of the grammar) | Computed | test fixture | The one `lookProfile.*` field actually consumed — displayed verbatim as HUD trivia (`viewer.html:184`), not used for any behavior. |
| `lookProfile.grammar.renderingPlanSchema` | `area-collection.example.json:8` (`build-collection.mjs:20`) | Copied from `plans[0].schema` | tooling only | Unconsumed beyond the equality check at `build-collection.mjs:36-38`. |
| `lookProfile.grammar.projectionSchemas` | `area-collection.example.json:9-26` | Copied from `plans[0].projectionSchemas` | kernel projection | Second copy of the same data; unconsumed. |
| `lookProfile.grammar.camera` | `area-collection.example.json:27-34` | Copied from `plans[0].camera` | renderer catalog | Third copy of the same 6 camera literals (1 build-tool literal + 4 plans + this) — see §3.1 and §4 item (camera duplication). |
| `lookProfile.grammar.palette` | `area-collection.example.json:35` | Copied from `plans[0].palette` | renderer catalog | Second copy; same non-consumption as `plan.palette` — see §3.9. |
| `lookProfile.grammar.architectureStyle` | `area-collection.example.json:36-40` | Copied from `plans[0].architecture.style` | renderer catalog | Second copy; equality is checked (`build-collection.mjs:36-38`), unlike most other duplicates. |
| `lookProfile.grammar.entityAssemblies` | `area-collection.example.json:41-57` (`build-collection.mjs:25`, computed as `[kind, visualAssembly, materialFamily]` triples) | Computed from `plan.entities` | renderer catalog | Derived summary of already-derived `entities[].kind/visualAssembly/materialFamily`; equality-checked across areas. |
| `lookProfile.grammar.actorAssemblies` | `area-collection.example.json:58-61` (`build-collection.mjs:26`) | Computed from `plan.actors` | renderer catalog | Equality-checked. |
| `lookProfile.grammar.effectAssemblies` | `area-collection.example.json:62-64` (`build-collection.mjs:27`) | Computed from `plan.effects` | renderer catalog | Equality-checked. |
| `lookProfile.grammar.uiAnchors` | `area-collection.example.json:65-70` (`build-collection.mjs:28`) | Copied from `plan.uiAnchors` | renderer catalog | Second copy of a fully dead field. |
| `startArea` | `area-collection.example.json:73` (`build-collection.mjs:55-57`) | Computed: the one area with `start === true` | area/gameplay graph | Correct single derivation with a cardinality check. |
| `route[].fromArea/.gate/.toArea/.entry` | `area-collection.example.json:74-109` (`build-collection.mjs:58-67`) | Computed by walking each area's `presentation.exit` from `startArea` | area/gameplay graph | Mechanically derived, not independently authored — cannot drift from the per-area `exit` fields it is built from, though it does encode the same edge fact a second time in a second file. |
| `areas[].id/.label/.plan` | `area-collection.example.json:111-132` (`build-collection.mjs:80-84`) | Computed from each plan's `area.id`/`area.label` plus a constructed path string | area/gameplay graph | `plan` is a hand-built relative-path string (`\`areas/${plan.area.id}.json\``), not validated against any real filesystem manifest at build time. |

## 2. Double authorities

1. **Camera identity and geometry.** Declared once per plan (`build-plan.mjs:177`; e.g. `experiments/executable-gaol/areas/north-gaol/rendering-plan.example.json:33-40`) and again in the collection (`area-collection.example.json:27-34`). `render-core.mjs:53-56` reads `plan.camera.width/height/tileWidth/tileHeight` for its isometric projection. `webgl-renderer.mjs` never reads `plan.camera` at all — it builds an entirely separate `THREE.OrthographicCamera` with its own frustum and position math at `webgl-renderer.mjs:386,410-420,447-448`. `camera.identity` and `camera.projection` are read by neither renderer.
2. **Wall height scale.** `architecture.wallHeight` (e.g. `north-gaol/area.json:15` = `4.5`) is passed through unchanged (`build-plan.mjs:179`) and then interpreted two different ways: `render-core.mjs:59` uses it as a raw unit (`const wallHeight = plan.architecture.wallHeight;`); `webgl-renderer.mjs:189` applies an undeclared `* 0.72` scale (`const wallHeight = plan.architecture.wallHeight * 0.72;`). The same field means two different things depending on which renderer reads it.
3. **Masonry mass height scale.** Same pattern as (2): `render-core.mjs:129-131` uses `mass.height` raw; `webgl-renderer.mjs:207` uses `mass.height * 0.72`.
4. **Actor start cell.** `areas/north-gaol/area.json:27,32` authors `player` at `{x:2,y:4,z:0}` and `gaoler` at `{x:5,y:3,z:0}`; these pass through unchanged into the plan (`build-plan.mjs:181`). `play-state.mjs:8-10,18-19` (`createPlayState`) hardcodes fallback defaults of exactly `{x:2,y:4,z:0}` and `{x:5,y:3,z:0}` for use when a plan lacks a matching actor. Currently masked by `build-plan.mjs:66-68` (every area is required to declare both actors), but the fallback is a second, area-specific authority baked into generic runtime code — it silently encodes North Gaol's coordinates as "the" defaults.
5. **`primaryGate` / `objective.target` / `exit.gate` triple redundancy.** `build-plan.mjs:63` and `build-plan.mjs:65` force `area.objective.target === area.primaryGate` and `area.exit.gate === area.primaryGate`. Three independently-authored string fields in every `area.json` (e.g. `north-gaol/area.json:5,6,10`) are required to hold the identical value; two of the three carry zero independent information.
6. **Ward-sealed / integrity-destroyed / access-open state.** No field in the plan carries a resolved "is this door's ward currently blocking" boolean. It is re-derived from the raw `scenarios[].machineStates["<id>.ward"]` string independently in `render-core.mjs:151` (`if (ward === "sealed")`) and `render-core.mjs:186` (`!== "sealed"`), and in `webgl-renderer.mjs:256` and `webgl-renderer.mjs:306` (`stateOf(...) !== "sealed"`) — four independent re-derivations, each choosing its own literal `"sealed"` fallback default. `integrity === "destroyed"` (`render-core.mjs:144`, `webgl-renderer.mjs:245`) and `access === "open"/"locked"` (`render-core.mjs:147,155`, `webgl-renderer.mjs:251`) follow the identical pattern.
7. **Gaoler hunting/dormant state.** `play-state.mjs:139` (`advanceGaoler`) uses `scenario.effectiveLight[pursuitLight] !== false` to decide whether the gaoler actually advances and can catch the player — this is the authoritative gameplay fact. `viewer.html:118` independently computes the same condition for HUD display only, phrased as `... effectiveLight[pursuitLight] === false ? "hunting" : "dormant"`. The two boolean expressions are logical mirrors today, but nothing ties them together; an id typo in one would silently desync the displayed pursuit state from the real one.
8. **Look-profile identity.** `area-collection.example.json:5` (`build-collection.mjs:74`) declares `lookProfile.id = "gaol_bounded_01"`. `webgl-renderer.mjs:19-42` defines its own, completely disjoint catalog: `lookProfiles.baseline.id = "gaol_baseline_01"` and `lookProfiles.procedural.id = "gaol_procedural_01"`. `viewer.html:196-201` drives the actual toggle with a third, unrelated pair of bare strings, `"procedural"`/`"baseline"`, passed straight to `gpu.setLookProfile(...)`. Four different identifier schemes for "which visual look is active," none referencing any of the others.
9. **Palette identity.** `plan.palette` / `lookProfile.grammar.palette` is the literal string `"gaol_bounded_01"` (`build-plan.mjs:178`), never dereferenced anywhere. `render-core.mjs:1-7` defines its own hardcoded `palette` object; `webgl-renderer.mjs:3-17` defines its own, differently-named hardcoded `colors` object. Three unconnected color definitions exist for what the schema implies should be one palette.

## 3. Derived by convention

Every classification-by-convention, magic ID, magic assembly string, and duplicated resolver found in `build-plan.mjs`, `play-state.mjs`, `webgl-renderer.mjs`, `render-core.mjs`, and `viewer.html`:

1. `build-plan.mjs:25` — door classification via `machine.endsWith(".access")`.
2. `build-plan.mjs:26` — light classification via membership in `lightEntities` (persistence `light_resolver` subjects).
3. `build-plan.mjs:27` — water classification via presence of a `traversal_cost_ground` navigation claim.
4. `build-plan.mjs:28` — silent `"unknown"` / `"visual/marker"` fallback when none of the above three heuristics match.
5. `build-plan.mjs:33-38` — `kind → visualAssembly` lookup table hardcoded inside the content compiler (a renderer catalog living in the wrong layer).
6. `build-plan.mjs:43` — `kind → materialFamily` lookup table, silently defaulting unknown kinds to `"stone"`.
7. `build-plan.mjs:56-71` — structural invariants enforced as ad hoc JS checks instead of schema: `objective` must have exactly the keys `{kind, target}` (line 57); `objective.kind` must literally equal `"exit_via"` (line 60); `actors` must contain the literal ids `"player"` and `"gaoler"` (lines 66-68).
8. `build-plan.mjs:73-78` — bounded-lattice magic numbers (`width ≤ 9`, `height ≤ 6`, `0 < wallHeight ≤ 5`) hardcoded in the compiler; the `wallHeight` bound is not documented anywhere else in the repository, including `AUTHORING.md`.
9. `build-plan.mjs:81` — masonry mass height bound `0 < height ≤ 4`, matching `AUTHORING.md`'s prose ("no higher than 4 cells") but enforced only here.
10. `build-plan.mjs:86-95` — `activationIsActive` reimplements a `state_equals`/`not`/`any`/`all` activation-expression resolver that a kernel projection compiler should already have evaluated.
11. `build-plan.mjs:106` — `expectedBaselineRejection` special-cases the literal scenario directory name `"01-baseline"` plus a hardcoded `status === "rejected"` / `committed_command_count === 0` pair.
12. `build-plan.mjs:111-121` — `movement` (disposition/cost/reasons) recomputed in JS from raw navigation claims rather than consumed pre-resolved.
13. `build-plan.mjs:123-128` — `effectiveLight` recomputed in JS from raw light-resolver claims, same pattern as (12).
14. `build-plan.mjs:133` — `scenario.label` derived from the scenario directory name via `replace(/^\d+-/, "").replaceAll("-", " ")`.
15. `build-plan.mjs:144-162` — `interactions[]` reconstructed by diffing command-log row-count/prefix/hash chains between every pair of scenarios (`O(n²)` over scenario count) rather than reading a declared "previous scenario" pointer; `interactions[].id` is built by string template at line 154.
16. `render-core.mjs:67` and `webgl-renderer.mjs:89-90` — the identical one-line machine-state lookup (`` scenario.machineStates[`${entity}.${name}`] ?? fallback ``) is implemented independently twice, each choosing its own literal fallback defaults.
17. `entity.kind === "door"/"water"/"light"` and `effect.assembly === "visual/cyan_crescent"` are re-checked by literal string/array equality independently in three consumer files after `build-plan.mjs` already resolved `kind` once: `render-core.mjs:99,135,161,184`; `webgl-renderer.mjs:191,216,304,440-441`; `play-state.mjs:62,83,121,127-128`.
18. "Is this door's ward sealed" is re-derived via `=== "sealed"` / `!== "sealed"` string comparison in four places — `render-core.mjs:151,186`; `webgl-renderer.mjs:256,306` — see also §3 (Double authorities) item 6.
19. "Is this door's integrity destroyed" is re-derived via `=== "destroyed"` in `render-core.mjs:144` and `webgl-renderer.mjs:245`.
20. "Is this door open/locked" is re-derived via `=== "open"`/`=== "locked"` in `render-core.mjs:147,155` and `webgl-renderer.mjs:251`.
21. `render-core.mjs:175` and `webgl-renderer.mjs:330` — `actor.id === "player"` is the only signal that distinguishes the player's silhouette from the gaoler's; there is no declared "role" field.
22. `webgl-renderer.mjs:191` — "is this door on the north wall" is derived from `anchor.cell.y === 0`, even though the same `anchor` object already carries an authoritative `anchor.direction` field (always `"north"`, 8 of 8 doors in the corpus — `rendering-plan.example.json:117,160` and equivalents in the other 3 areas) that no file in the audited set ever reads.
23. `viewer.html:92,140,154` — `plan.scenarios[0]` is treated as "the default scenario" purely by array position; that position is itself an accident of `build-plan.mjs:97-100` sorting scenario-capture directory names alphabetically, which only puts `"01-baseline"` first because of its numeric filename prefix.
24. `viewer.html:118` — gaoler "hunting"/"dormant" HUD text re-derives the pursuit condition `play-state.mjs:139` already computes for real gameplay, phrased with the opposite comparison operator — see also §3 (Double authorities) item 7.
25. `viewer.html:221` — number keys 1-9 are mapped to `states.children[number - 1]`, assuming DOM button order always matches `plan.scenarios` array order.
26. `play-state.mjs:12` — `displayName()` derives every on-screen label for entities, actors, and interaction actions by title-casing the snake_case id. `area.label` is the only entity in this content model with an authored display string; every other visible name (door ids, actor ids, action verbs) is convention-derived from an identifier that was never intended as prose.

## 4. Raw floating-point presentation values

Per `THESIS.md:733-736` ("No raw transforms in shippable content... Not
discouraged — absent. Primitive development may use a tainted laboratory.
Anything produced there is explicitly non-shippable until promoted through
fixtures and review") and the fact-ownership table at `THESIS.md:280-291`
(`spatial.boundary -> lattice`, `render.transform -> projection-derived`),
every float below is exactly the kind of raw transform the target system
forbids in shippable content. `experiments/executable-gaol` is explicitly the
tainted laboratory the passage allows. Integer lattice cell coordinates
(`actors[].anchor.cell`, `exit.entry`, mass `min`/`max`) are excluded here —
they already address the lattice discretely; the values below do not.

| Field | File:line | Value | Typed replacement shape |
|---|---|---|---|
| `architecture.wallHeight` | `areas/north-gaol/area.json:15` | `4.5` | Discrete step count on a fixed vertical lattice unit (e.g. `9 half-courses`). |
| `architecture.wallHeight` | `areas/cistern-walk/area.json:16` | `4.5` | Same. |
| `architecture.wallHeight` | `areas/ember-vault/area.json:16` | `5` | Same (integer-valued but still an untyped raw float field). |
| `architecture.wallHeight` | `areas/ossuary-reach/area.json:16` | `4.8` | Same. |
| `architecture.masses[].height` (`channel_buttress`) | `areas/cistern-walk/area.json:27` | `2.6` | Discrete step count, same lattice unit as wall height. |
| `architecture.masses[].height` (`west_vault_pier`) | `areas/ember-vault/area.json:27` | `3.2` | Same. |
| `architecture.masses[].height` (`east_vault_pier`) | `areas/ember-vault/area.json:33` | `3.2` | Same. |
| `architecture.masses[].height` (`west_tomb_bank`) | `areas/ossuary-reach/area.json:27` | `0.7` | Same. |
| `architecture.masses[].height` (`east_tomb_bank`) | `areas/ossuary-reach/area.json:33` | `0.7` | Same. |
| `architecture.masses[].height` (`reliquary_pier`) | `areas/ossuary-reach/area.json:39` | `2.4` | Same. |
| `effects[].presentationAnchor` (`ward_crescent`) | `areas/north-gaol/area.json:40` | `{x:3.6, y:3.4, z:0}` | Named socket on the anchor entity (e.g. `north_gate.ward_socket`), not a free-floating world coordinate. |
| `effects[].presentationAnchor` (`sluice_ward_crescent`) | `areas/cistern-walk/area.json:48` | `{x:4.9, y:3.8, z:0}` | Same. |
| `effects[].presentationAnchor` (`vault_ward_crescent`) | `areas/ember-vault/area.json:54` | `{x:4, y:3.8, z:0}` | Same. |
| `effects[].presentationAnchor` (`bone_ward_crescent`) | `areas/ossuary-reach/area.json:60` | `{x:6.1, y:1.2, z:0}` | Same. |
| `camera.width` / `.height` / `.tileWidth` / `.tileHeight` | `build-plan.mjs:177`, duplicated verbatim in all 4 `rendering-plan.example.json` files (e.g. `north-gaol:36-39`) and in `area-collection.example.json:30-33` | `1200 / 540 / 96 / 50` | Fixed-point renderer-catalog constants owned once by the renderer, not re-typed into every content artifact. |

The renderer implementations compound this: `webgl-renderer.mjs:189,207`
apply an undeclared `* 0.72` scale to `wallHeight`/mass `height` that exists
nowhere in the content schema (see §3 Double authorities, items 2-3), and
`webgl-renderer.mjs:386,415,447` hardcode additional untyped camera floats
(orthographic half-height `3.7`, position multipliers `.86`/`.92`/`1.08`)
that have no content-side counterpart at all. These renderer-internal floats
are noted for completeness but are not counted in §6, which counts only
floats present in area content or plans, per the issue's scope.

## 5. Summary counts

- **Fields in the ownership table:** 69 distinct field paths across the four
  `area.json` files (union), `area-collection.example.json`, and
  `nomos.experiment.rendering_plan@1` (some rows group tightly-coupled
  sub-fields, e.g. a mass's `min`/`max` rectangle, or the four `camera`
  dimension scalars, as one row).
- **Fields with a clear single owner today:** 55 of 69. (69 minus the 14 rows
  directly implicated in a double authority in §2: `primaryGate`,
  `objective.target`, `exit.gate` (item 5); `architecture.wallHeight` (item
  2); `architecture.masses[].height` (item 3); `actors[].anchor.cell` (item
  4); `camera.identity`, `camera.projection`, `camera.width/.height/
  .tileWidth/.tileHeight` (item 1); `scenarios[].machineStates` (item 6, the
  raw data four consumers each re-derive a boolean from independently);
  `scenarios[].effectiveLight` (item 7); `lookProfile.id` (item 8); `palette`
  and `lookProfile.grammar.palette` (item 9). This count is about authority
  conflicts, not usage — several of the 55 "clean" fields are simply
  unconsumed dead data, e.g. `uiAnchors`, `entities[].provenance[]`,
  `entities[].machineNamespaces`, `projectionDigests`.)
- **Double authorities:** 9 (§2).
- **Convention-derived facts:** 26 (§3).
- **Raw floating-point presentation values:** 26 — 4 `wallHeight` + 6 mass
  `height` + 12 `presentationAnchor` coordinate components (`x`,`y`,`z` × 4
  areas) + 4 shared camera-dimension constants (§4).
