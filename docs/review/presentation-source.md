---
title: Typed presentation source — R1-3 design record
status: R1-3 design record; reviewed, ruled on, and implemented on this branch
date: 2026-08-25
issue: 146
branch: r1/issue-146-presentation-source
accepts_against: RUNTIME.md §5 R1-3 (revision 1)
registers: docs/evaluation/R1_SCHEMA_OWNERSHIP.md (nomos.presentation_source@1, nomos.rendering_plan@2)
depends_on: issue #139 (R1-2 rendering-plan compiler), issue #144 (retire the private encoder)
applies_to: RUNTIME.md §3, §5, §6; docs/review/executable-gaol-ownership-audit.md; docs/review/rendering-plan-compiler.md
---

# Typed presentation source

## Problem

`experiments/executable-gaol/areas/*/area.json` is an unversioned second
content language. `docs/review/executable-gaol-ownership-audit.md` counts what
that costs: 69 field paths, 9 double authorities, 26 convention-derived facts,
and 26 raw floating-point values. R1-2 put a strict Rust decoder in front of it
(`crates/nomos-render-plan/src/area.rs`) but did not redesign it, so the shape
is still the JavaScript's — camelCase, dotted object keys, decimal transforms —
and the plan it emits therefore needs a private canonical encoder
(`src/doc.rs`, issue #144) that `nomos_core::CanonicalValue` cannot replace.

This record is the design R1-3 implements: one versioned source schema, one
plan version, one owner per field, and no decimal anywhere on the path.

Nothing here is accepted until the implementation lands with its evidence and a
non-author rerun (`AGENTS.md`).

---

## 1. `nomos.presentation_source@1`

One file per area, `experiments/executable-gaol/areas/<area-id>/presentation.json`.
Hand-authored, pretty-printed JSON — it is not canonical bytes, because a human
writes it — read by `crates/nomos-render-plan/src/source.rs`, which is its
declaring owner and the register's Owner file.

### 1.1 Reading discipline

- **Strict.** Duplicate object keys are refused, not resolved by last-write-wins.
  Unknown fields are refused. Nesting is bounded at 64. This is
  `crates/nomos-render-plan/src/json.rs`'s existing behaviour, kept.
- **Versioned.** The file's first fact is `"schema": "nomos.presentation_source@1"`.
  A different name or version is refused with `RP0104`, message
  ``expected schema `nomos.presentation_source@1`, found `<found>` ``. The
  spelling is the bare `name@version` string, matching `nomos entity-catalog`,
  `nomos effective-facts`, and the plan's own `schema` field. Issue #145 owns
  choosing one spelling across R1 stdout documents; this slice changes no
  spelling and narrows no reader.
- **Integer-only.** The reader has no decimal variant at all. Any number lexeme
  carrying `.`, `e`, `E`, or a leading `+` is refused with `RP0205` before it is
  ever interpreted, anywhere in the file, at any depth, in a field the schema
  knows or does not know. `crates/nomos-render-plan/src/decimal.rs` is deleted;
  `Json::Number(Decimal)` becomes `Json::Integer(i64)`. Refusal is structural,
  the same way `CanonicalValue`'s missing float variant is structural.
- **Fail closed.** Every violated constraint below is `RP0202` with a message
  naming what was expected and what was found. Every violated identifier
  grammar is `RP0206`. No default is substituted for an absent field.

### 1.2 Identifier grammars

Grammars apply to *values*. Field names are all `[a-z][a-z0-9_]*`, which is
`nomos_core::FieldName`'s grammar exactly, so the whole document is expressible
in `CanonicalValue` without widening anything.

| Grammar | Regular expression | Length | Used by |
| --- | --- | --- | --- |
| `AreaId` | `[a-z][a-z0-9]*(-[a-z0-9]+)*` | 1–64 bytes | `area.id`, `route.exit.to_area` |
| `EntityId` | `[a-z][a-z0-9_]*` | 1–64 bytes | `route.exit.gate`, `pursuit.light`, `actors[].id`, `effects[].id`, `effects[].anchor.entity`, `architecture.masses[].id` |
| `AssemblyName` | `[a-z][a-z0-9_]*(/[a-z][a-z0-9_]*)+` | 1–96 bytes | `actors[].assembly`, `effects[].assembly`, `architecture.style.assembly` |
| `FamilyName` | `[a-z][a-z0-9_]*` | 1–64 bytes | `architecture.style.material_family`, `architecture.style.trim_family` |
| `SocketName` | `[a-z][a-z0-9_]*` | 1–32 bytes | `effects[].anchor.socket` |
| `Label` | any UTF-8 with no byte below `0x20` | 1–64 characters | `area.label` |

`AreaId` is deliberately the only grammar admitting `-`: an area id is also a
directory name and a collection key. `EntityId` is deliberately identical to
`FieldName`, so an entity id can key a canonical collection without widening.

### 1.3 Fields

Owner categories are the audit's: **World IR**, **runtime state**, **kernel
projection**, **presentation source**, **renderer catalog**, **area/gameplay
graph**, **test fixture**, **tooling only**.

| Field | Type | Constraints | Owner |
| --- | --- | --- | --- |
| `schema` | string | exactly `nomos.presentation_source@1`; `RP0104` otherwise | tooling only |
| `area` | object | exactly `{id, label, start}` | — |
| `area.id` | string | `AreaId`; must equal the containing directory name | area/gameplay graph |
| `area.label` | string | `Label` | presentation source |
| `area.start` | boolean | declared `true`/`false`; no truthiness. Exactly one area in the collection may be `true` (`build-collection.mjs`) | area/gameplay graph |
| `route` | object | exactly `{exit}` for the start area, exactly `{exit, entry}` for every other area | — |
| `route.exit.gate` | string | `EntityId`; must name a compiled entity whose `nomos.entity_catalog@1` kind is `door` | area/gameplay graph |
| `route.exit.to_area` | string or null | `AreaId`, or `null` for the route's terminal area; a non-null value must name a declared area that itself declares an `entry` (`build-collection.mjs`) | area/gameplay graph |
| `route.entry` | object | exactly `{x, y, z}` — **this area's own arrival cell**; present **iff** `area.start` is `false` | area/gameplay graph |
| `route.entry.x` | integer | `0 ≤ x < bounds.width` | area/gameplay graph |
| `route.entry.y` | integer | `0 ≤ y < bounds.height` | area/gameplay graph |
| `route.entry.z` | integer | `z == 0`; the cell must not lie inside one of this area's own `masses` | area/gameplay graph |
| `pursuit` | object | exactly `{light}` | — |
| `pursuit.light` | string | `EntityId`; must name a compiled entity whose catalog kind is `light` | area/gameplay graph |
| `architecture` | object | exactly `{bounds, wall_height_steps, style, masses}` | — |
| `architecture.bounds.width` | integer | `1 ≤ width ≤ 9` | presentation source — deliberate deviation from the audit's World IR, §6 finding 2 |
| `architecture.bounds.height` | integer | `1 ≤ height ≤ 6` | presentation source — deliberate deviation from the audit's World IR, §6 finding 2 |
| `architecture.wall_height_steps` | integer | `1 ≤ steps ≤ 50`, in `vertical_step` units of 1/10 lattice cell | presentation source |
| `architecture.style.assembly` | string | `AssemblyName`; member of the catalog's `ARCHITECTURE_ASSEMBLIES` | renderer catalog defines, source selects |
| `architecture.style.material_family` | string | `FamilyName`; member of `MATERIAL_FAMILIES` | renderer catalog defines, source selects |
| `architecture.style.trim_family` | string | `FamilyName`; member of `TRIM_FAMILIES` | renderer catalog defines, source selects |
| `architecture.masses` | array | 0–8 entries, ordered as authored; ids unique within the area | presentation source |
| `architecture.masses[].id` | string | `EntityId` | presentation source |
| `architecture.masses[].min` | object | exactly `{x, y}`; `0 ≤ x < max.x ≤ bounds.width`, `0 ≤ y < max.y ≤ bounds.height` | presentation source |
| `architecture.masses[].max` | object | exactly `{x, y}`; same rectangle rule | presentation source |
| `architecture.masses[].height_steps` | integer | `1 ≤ steps ≤ 40`, in `vertical_step` units | presentation source |
| `actors` | array | exactly two entries; ids exactly `{player, gaoler}` — §4.3 item 7, whose remainder is deferred to R1-5 | — |
| `actors[].id` | string | `EntityId` | presentation source |
| `actors[].assembly` | string | `AssemblyName`; member of `ACTOR_ASSEMBLIES` | renderer catalog defines, source selects |
| `actors[].cell` | object | exactly `{x, y, z}`; `0 ≤ x < bounds.width`, `0 ≤ y < bounds.height`, `z == 0`; not inside a declared mass | presentation source |
| `effects` | array | 0–8 entries, ordered as authored; ids unique within the area | — |
| `effects[].id` | string | `EntityId` | presentation source |
| `effects[].assembly` | string | `AssemblyName`; member of `EFFECT_ASSEMBLIES` | renderer catalog defines, source selects |
| `effects[].anchor` | object | exactly `{entity, socket}`; **no coordinate of any kind** | — |
| `effects[].anchor.entity` | string | `EntityId`; must name a compiled entity | presentation source |
| `effects[].anchor.socket` | string | `SocketName`; must be a socket the anchor entity's kind declares (§3.3) | presentation source |

**Removed from the source entirely**, per the issue's Scope: `primaryGate`,
`objective`, `forensicScenario`, `camera`, `palette`, `lookProfile`,
`uiAnchors`, `deterministic`, `presentationAnchor`, `actors[].anchor.kind`.
`objective` is derived, not authored: the compiler emits
`{"kind": "exit_via", "gate": route.exit.gate}`, which is what collapses the
`primaryGate` / `objective.target` / `exit.gate` triple to one authored string.

**`route.entry` is this area's own arrival cell, not the destination's.** Owner
ruling 3, replacing the `exit.entry` of `area.json`: the exiting area used to
author a cell in the *destination* area, which is cross-area authority, and
`build-collection.mjs:44-51` was the only thing checking it — against a global
9×6 rather than against the destination's own bounds and masses. Flipped, each
area owns the one cell a player arrives on, validates it against its own
`bounds` and its own `masses` inside `source.rs`, and the start area declares
none because nothing arrives there. `build-collection.mjs` keeps one cross-area
check that is now purely referential: every non-null `to_area` names a declared
area, and that area declares an `entry`.

**Assembly and family names: the renderer catalog defines, the source selects.**
Owner ruling 1, and the same definition/selection split as socket names.
`architecture.style.{assembly, material_family, trim_family}`,
`actors[].assembly`, and `effects[].assembly` stay in content; the *closed sets*
they draw from are renderer-catalog data. `source.rs` checks the grammar and
`verify.mjs` checks membership in the renderer's closed sets, exactly as for
sockets, so a name that is well-formed but not in the catalog fails the build
rather than a frame. Audit §1 rows 14, 15, 16, 21, and 25 are **resolved** under
that split, not deferred: each fact has one owner, and it is the renderer
catalog that owns what the legal values are. R1-4 turns the JavaScript closed
sets into a Rust-side catalog.

### 1.4 North Gaol, converted

`experiments/executable-gaol/areas/north-gaol/presentation.json`:

```json
{
  "schema": "nomos.presentation_source@1",
  "area": {
    "id": "north-gaol",
    "label": "North Gaol",
    "start": false
  },
  "route": {
    "exit": { "gate": "north_gate", "to_area": null },
    "entry": { "x": 2, "y": 4, "z": 0 }
  },
  "pursuit": {
    "light": "brazier_02"
  },
  "architecture": {
    "bounds": { "width": 9, "height": 6 },
    "wall_height_steps": 45,
    "style": {
      "assembly": "visual/beveled_masonry",
      "material_family": "stone_bounded",
      "trim_family": "broad_mortar"
    },
    "masses": []
  },
  "actors": [
    { "id": "player", "assembly": "visual/player_silhouette", "cell": { "x": 2, "y": 4, "z": 0 } },
    { "id": "gaoler", "assembly": "visual/gaoler_silhouette", "cell": { "x": 5, "y": 3, "z": 0 } }
  ],
  "effects": [
    {
      "id": "ward_crescent",
      "assembly": "visual/cyan_crescent",
      "anchor": { "entity": "north_gate", "socket": "ward" }
    }
  ]
}
```

North Gaol is not the start area, so it declares the cell a player arrives on —
`{2, 4, 0}`, the cell Ossuary Reach's `exit.entry` used to name from the outside
— and its `to_area` is `null` because it is the route's terminal. Cistern Walk,
the start area, is the mirror image: it declares no `entry` at all, because
nothing arrives there.

```json
  "route": {
    "exit": { "gate": "sluice_gate", "to_area": "ember-vault" }
  },
```

and its one mass is
`{ "id": "channel_buttress", "min": { "x": 4, "y": 0 }, "max": { "x": 5, "y": 1 }, "height_steps": 26 }`.

The four `entry` cells after the flip: Cistern Walk none (start), Ember Vault
`{7, 5, 0}`, Ossuary Reach `{1, 5, 0}`, North Gaol `{2, 4, 0}` — the same three
cells `area.json` authored, each now declared by the area it belongs to.

The whole corpus after conversion contains **no `.` in any number**: the ten
former decimals become `wall_height_steps` 45 / 45 / 50 / 48 and `height_steps`
26 / 32 / 32 / 7 / 7 / 24, and the twelve `presentationAnchor` components are
gone.

---

## 2. `nomos.rendering_plan@2`

Declared by `crates/nomos-render-plan/src/plan.rs`, which stays the Owner file.
`@1`'s register row is replaced by `@2`; the history is here rather than in the
register.

### 2.1 Shape

Thirteen top-level fields, all snake_case. Canonical bytes sort keys, so the
order below is reading order, not byte order.

| Field | Shape | Source of truth |
| --- | --- | --- |
| `schema` | `"nomos.rendering_plan@2"` | this file |
| `area` | `{id, label, start}` | presentation source |
| `objective` | `{kind: "exit_via", gate}` | derived from `route.exit.gate` |
| `route` | `{to_area, entry?}` — `entry` is this area's own arrival cell | presentation source |
| `pursuit` | `{light}` | presentation source |
| `projection_schemas` | `[{name, version}]`, four members in declared order | the world package's four projection members |
| `projection_digests` | `[{file, digest}]`, same four members, same order | SHA-256 over the members' raw bytes |
| `architecture` | `{bounds:{width,height}, wall_height_steps, style:{assembly,material_family,trim_family}, masses:[{id,min,max,height_steps}]}` | presentation source |
| `entities` | `[{id, kind, visual_assembly, material_family, anchor, machine_namespaces, provenance}]` | `nomos.entity_catalog@1` |
| `actors` | `[{id, assembly, cell}]` | presentation source |
| `effects` | `[{id, assembly, anchor:{entity, socket}}]` | presentation source |
| `scenarios` | `[{id, label, tick, state_hash, machine_states, movement, effective_light}]` | run bundles + `nomos.effective_facts@1` |
| `interactions` | `[{id, from_scenario, to_scenario, target_entity, action, input_state_hash, resulting_state_hash}]` | derived from the committed command logs |

### 2.2 What changed from `@1`, and why

| # | `@1` | `@2` | Why |
| --- | --- | --- | --- |
| 1 | `deterministic: true` | removed | Hardcoded, never read downstream; audit §1 row 29 calls it a dead declared-intent flag. |
| 2 | `camera: {identity, projection, width, height, tileWidth, tileHeight}` | removed | Renderer-catalog constants re-typed into every content artifact (audit §2 item 1, §4's last row). They move into `render-core.mjs` (§3.4). |
| 3 | `palette: "gaol_bounded_01"` | removed | Never dereferenced by any consumer; each renderer already holds its own table (audit §2 item 9). |
| 4 | `uiAnchors: [...]` | removed | Fully dead: the four strings appear nowhere else in the tree (audit §1 row 43). |
| 5 | `presentation.primaryGate`, `presentation.objective.{kind,target}`, `presentation.exit.gate` | one `objective: {kind, gate}` | Three fields forced equal collapse to one (audit §2 item 5). `route` keeps only where the gate leads. |
| 6 | `presentation.exit.{toArea,entry}` | `route.{to_area,entry}` | snake_case; the `presentation` wrapper had no members left worth wrapping. `entry` also changes meaning under owner ruling 3: it is now the area's *own* arrival cell rather than a cell the previous area named inside it. |
| 7 | `presentation.pursuitLight` | `pursuit.light` | snake_case; grouped with the other pursuit facts R1-5 will add. |
| 8 | `presentation.forensicScenario` | removed | Test-fixture constant, identical in all four areas; moves to capture tooling (§3.5). |
| 9 | `architecture.wallHeight: 4.5` | `architecture.wall_height_steps: 45` | Integer tenths of a cell; removes 4 of the 26 floats and fixes the unit in the schema instead of in two renderers. |
| 10 | `architecture.masses[].height: 2.6` | `architecture.masses[].height_steps: 26` | Same, 6 more floats. |
| 11 | `architecture.style.materialFamily`, `.trimFamily` | `material_family`, `trim_family` | snake_case. |
| 12 | `entities[].visualAssembly`, `.materialFamily`, `.machineNamespaces` | `visual_assembly`, `material_family`, `machine_namespaces` | snake_case. |
| 13 | `actors[].anchor.{kind,cell}` | `actors[].cell` | `kind` was a single-valued enum in the whole corpus (audit §1 row 22). |
| 14 | `effects[].anchorEntity` + `effects[].presentationAnchor:{x,y,z}` | `effects[].anchor:{entity, socket}` | The audit's proposed repair for all twelve `presentationAnchor` components; removes the last 12 floats from content. |
| 15 | `projectionDigests: {"simulation.json": "…", …}` | `projection_digests: [{file, digest}]` | The object was keyed by dotted file names, illegal for `FieldName`. §2.3. |
| 16 | `scenarios[].stateHash` | `scenarios[].state_hash` | snake_case; also the spelling `nomos.effective_facts@1` already uses. |
| 17 | `scenarios[].machineStates: {"north_gate.ward": "sealed", …}` | `scenarios[].machine_states: [{namespace, state}]` | Dotted keys, illegal for `FieldName`. §2.3. |
| 18 | `scenarios[].movement: {"north_gate": {…}}` | `scenarios[].movement: [{entity, disposition, cost, reasons}]` | Entity-keyed object → the kernel's own stable-ID array idiom. §2.3. |
| 19 | `scenarios[].effectiveLight: {"brazier_02": true}` | `scenarios[].effective_light: [{entity, emitting}]` | Same. |
| 20 | `interactions[].fromScenario`, `.toScenario`, `.targetEntity`, `.inputStateHash`, `.resultingStateHash` | snake_case equivalents | snake_case. |

### 2.3 Every value is expressible in `CanonicalValue`

`nomos_core::CanonicalValue` has seven variants — `Null`, `Bool`, `Int`, `Uint`,
`Text`, `Array`, `Object` — with `FieldName` = `[a-z][a-z0-9_]*` and no
floating-point variant (`crates/nomos-core/src/canonical.rs:39-118`). `@2` fits
inside it with no widening:

- **Field names.** Every literal above is snake_case ASCII. There is no
  remaining object whose keys come from data.
- **The two dotted-key objects.** `projectionDigests` was keyed by projection
  file name (`simulation.json`) and `scenarios[].machineStates` by machine
  namespace (`north_gate.ward`); both are illegal `FieldName`s. Each becomes an
  array of declared-field pairs:
  `projection_digests: [{"file": "simulation.json", "digest": "…"}]` and
  `machine_states: [{"namespace": "north_gate.ward", "state": "sealed"}]`. The
  dotted identifiers are now *values*, which are unconstrained UTF-8 strings.
  `machine_states` is exactly how the kernel already spells the same collection
  in `final-state.json` (`state.machines[]` = `{namespace, state}`), so this is
  adopting an existing kernel convention rather than inventing one.
- **The two entity-keyed objects.** `movement` and `effectiveLight` were keyed
  by entity id. Entity ids *are* legal `FieldName`s, so these could have stayed
  objects; they become `[{entity, …}]` arrays anyway because that is exactly how
  `nomos.effective_facts@1` spells `ground_movement` and `light_emission`, and
  because `KERNEL.md` §7's rule for entity collections is a stable-ID-ordered
  array. `nomos_core::canonical::keyed_array` is the constructor, so ordering
  and duplicate-id refusal come from the kernel rather than from this crate.
- **Ordering.** `machine_states`, `movement`, and `effective_light` are ordered
  by `keyed_array` (ascending stable id) — the same order the `@1` objects had,
  because canonical object keys were byte-sorted too. `projection_digests` is a
  plain `Array` in the declared `PROJECTION_FILES` order, so it stays row-for-row
  aligned with `projection_schemas`, which is already a declared-order array.
- **Numbers.** `bounds.width/height`, `wall_height_steps`, `height_steps`, mass
  and cell coordinates, `tick`, and movement `cost` are unsigned integers.
  `cost` on a blocked subject stays the JSON `null` that `RUNTIME.md` §5 R1-1
  names as the one normalization — `CanonicalValue::Null`. Nothing in the
  document is negative and nothing is fractional.
- **Consequence.** `crates/nomos-render-plan/src/doc.rs` and its `PlanField`,
  `PlanValue`, and decimal variant are deleted; `plan.rs` builds a
  `CanonicalValue` and calls `to_canonical_bytes`. `tests/canonical_profile.rs`
  is replaced by a round-trip test: compile a plan, strip the single trailing
  `LF` the writer appends, feed the bytes to
  `nomos_core::canonical::read::parse_canonical`, and assert the reparsed value
  re-encodes to the same bytes. That closes issue #144's three boxes.

### 2.4 What the plan does *not* change

`schema` stays the bare `name@version` **string**, as `@1` spelled it. Issue
#145 owns the string-versus-`{name, version}` decision across R1 stdout
documents; this slice keeps every spelling as it is and leaves `bind_schema`
accepting both forms. Nothing here narrows a reader.

---

## 3. Renderer catalog additions

The catalog is renderer-owned data that content may name but not define. It
lands in a new module, `experiments/executable-gaol/src/renderer-catalog.mjs`,
imported by both renderers, `play-state.mjs`, and `viewer.html`, plus per-renderer
constants that only one renderer can own.

### 3.1 `vertical_step = 1/10 cell`

One declared constant, `VERTICAL_STEPS_PER_CELL = 10`. A height in the plan is
an integer step count; a renderer computes `cellsOf(steps) = steps /
VERTICAL_STEPS_PER_CELL` to get lattice cells and then applies its own
cells-to-screen scale.

**The conversion is a division, and that is load-bearing.** IEEE-754 division is
correctly rounded, so `n / 10` is the nearest double to the real n/10 — which is
the same double the decimal literal it replaces denoted. Multiplying by the
nearest double to `0.1` is a *different operation* with a different answer for
three of the ten values:

```text
45 / 10 === 4.5   50 / 10 === 5     48 / 10 === 4.8   26 / 10 === 2.6
32 / 10 === 3.2    7 / 10 === 0.7   24 / 10 === 2.4

48 * 0.1 === 4.800000000000001    7 * 0.1 === 0.7000000000000001
24 * 0.1 === 2.4000000000000004
```

Implementation note, recorded because it is the one defect this slice found in
its own work: the first version wrote `steps * (1 / 10)`, and every Ossuary
Reach frame moved — that being the only area whose values are 48, 7, and 24.
Fixed to the division this section specifies. `area-collection.test.mjs` now
pins all ten values rather than asserting the property, so the same mistake
fails a test instead of a digest comparison.

### 3.2 The `0.72` WebGL scale, declared

`webgl-renderer.mjs:189` and `:207` apply an undeclared `* 0.72` to wall and
mass heights (audit §2 items 2 and 3). It becomes a named catalog constant in
`webgl-renderer.mjs`:

```js
// Lattice cells of elevation to WebGL world units. Horizontal cells are 1.0
// world unit; vertical cells are shorter, so a 4.5-cell wall reads at the
// height the door and brazier assemblies were modelled against.
const VERTICAL_SCALE = 0.72;
```

`render-core.mjs`'s equivalent — the `- z * 38` inside `iso()` — becomes
`CELL_HEIGHT_PIXELS = 38` beside the camera constants. The audit's complaint was
never the number; it was that the same content field meant two different things
in two renderers with only one of the two scales written down. After this
change the content field means one thing (`steps` of 1/10 cell) and each
renderer declares its own conversion out of cells.

### 3.3 The socket table

```js
// Sockets are named attachment points on a visual assembly. A socket's offset
// is measured from the origin corner of the anchor entity's lattice cell, in
// vertical_step units (tenths of a cell) on all three axes, so the catalog
// holds integers only and each renderer applies its own cells-to-screen scale.
export const SOCKETS = Object.freeze({
  "visual/iron_barred_door": Object.freeze({
    ward: Object.freeze({ x: 5, y: 0, z: 17 }),
  }),
});
```

**The `ward` socket on `visual/iron_barred_door` is `{x: 5, y: 0, z: 17}`** —
half a cell along the door's own axis, on the wall plane, 1.7 cells above the
floor. Why that offset:

- `x = 5` (half a cell) is where both renderers already place the door: the SVG
  draws its glyph at `iso(cell.x + .5, cell.y)` (`render-core.mjs:136`) and the
  WebGL renderer at `cellPosition(cell)`, which centres on the cell
  (`webgl-renderer.mjs:92-96,234`). The socket sits on the door's centre line in
  both.
- `y = 0` puts the socket on the cell's north edge — the face the door is bound
  to. Every door in the corpus is `anchor.direction: "north"`, and this is the
  first use the tree makes of that declared field (audit §3 item 22).
- `z = 17` is chosen so the socket lands on the ward mark each renderer already
  draws. In WebGL the ward ring sits at local `y = 1.22`
  (`webgl-renderer.mjs:259`); `17 * 0.1 * 0.72 = 1.224`, so the crescent lands on
  the ring to within 0.004 world units. In the SVG the socket resolves to the
  door glyph's own screen column, `1.7 * 38 = 64.6` px above the glyph anchor,
  which places the crescent's body across the head of the gate's arch (the inner
  opening's apex is 87 px above the anchor and the ward diamond's top vertex 76
  px above it) with its bounding box centred within 10 px of the ward glow's
  centre. The crescent reads as a sigil set into the sealed gate in both
  renderers, and is drawn only while `ward` is `sealed`, exactly where the ward
  diamond and ring are drawn.

Ownership split, stated once so it cannot drift:

- the **socket name** is a value of `nomos.presentation_source@1`, owned by the
  schema in `source.rs`, which declares the closed set per entity kind
  (`Door => ["ward"]`, others `[]`) and refuses anything else with `RP0202`;
- the **socket offset** is renderer-catalog data, owned by `SOCKETS` above, and
  appears in no content file and in no plan;
- `verify.mjs` asserts that every `effects[].anchor.socket` in each compiled plan
  resolves in `SOCKETS`, so a name legal to the compiler but unknown to the
  renderer fails the build rather than a frame.

The WebGL crescent changes from a torus lying flat on the floor
(`rotation.x = -π/2`, `position.y = .09`) to an upright arc at the socket, facing
the way the ward ring faces. Honouring all three components in both renderers is
the point; a renderer that silently dropped the socket's `z` would be audit §2
item 2 again in a new field.

A non-north door would need the catalog to rotate the socket by the entity's
declared `anchor.direction`. Every door in the corpus is north-facing, so the
table is a fixed offset today; direction-aware socket resolution belongs with
the promoted renderer's catalog and is **deferred to R1-4**.

### 3.4 Camera constants in `render-core.mjs`

```js
export const camera = Object.freeze({
  identity: "gaol_oblique_01",
  projection: "fixed_oblique",
  width: 1200,
  height: 540,
  tileWidth: 96,
  tileHeight: 50,
});
const ORIGIN = Object.freeze({ x: 470, y: 125 });
const CELL_HEIGHT_PIXELS = 38;
```

`capture.mjs` and `capture-collection.mjs` import `camera` instead of reading
`plan.camera`. The values are byte-identical to the ones the plan carried, so
the frames and both contact sheets keep their geometry.

`webgl-renderer.mjs` keeps its own camera and names its four previously bare
floats: `ORTHO_HALF_HEIGHT = 3.7` and
`CAMERA_OFFSET = { x: 0.86, y: 0.92, z: 1.08 }` (`webgl-renderer.mjs:415,447`).
These were noted in the audit §4 prose but excluded from the 26; declaring them
costs nothing and closes the note.

### 3.5 Other catalog and tooling constants

| Constant | Home | Replaces |
| --- | --- | --- |
| `LOOK_PROFILE_IDS = ["baseline", "procedural"]` | `renderer-catalog.mjs` | `collection.lookProfile.id`, and `viewer.html:196-201`'s bare `"procedural"`/`"baseline"` literals |
| `ARCHITECTURE_ASSEMBLIES`, `MATERIAL_FAMILIES`, `TRIM_FAMILIES`, `ACTOR_ASSEMBLIES`, `EFFECT_ASSEMBLIES` | `renderer-catalog.mjs` | the closed sets audit §1 rows 14, 15, 16, 21, 25 name; content selects from them, `verify.mjs` checks membership (owner ruling 1) |
| `machineState(scenario, entity, machine)` | `renderer-catalog.mjs` | the two independent lookups with their own fallbacks at `render-core.mjs:67` and `webgl-renderer.mjs:89-90`; throws when the namespace is absent |
| `doorState(scenario, entity)` → `{access, integrity, ward}` | `renderer-catalog.mjs` | the four `"sealed"`, two `"intact"`, and two `"locked"` literal fallbacks |
| `wardSealed(scenario, entity)` | `renderer-catalog.mjs` | the four independent `=== "sealed"` re-derivations |
| `isHunting(plan, scenario)` | `renderer-catalog.mjs` | `play-state.mjs:139` and `viewer.html:118`, which computed the same condition with opposite operators |
| `FORENSIC_SCENARIO = "03-breached-unsealed"` | `capture.mjs` | `presentation.forensicScenario`, four copy-pasted identical content fields |

---

## 4. Audit disposition

### 4.1 The 69 rows of §1, with their owner after R1-3

"Where it lives" is the accepted home; **removed** means the fact exists nowhere
after this slice.

| # | Audit §1 field | Where it lives after R1-3 | Owner | Reason |
| --- | --- | --- | --- | --- |
| 1 | `id` | `presentation.json` `area.id` | area/gameplay graph | Structural key the collection and other areas' `to_area` reference. |
| 2 | `label` | `presentation.json` `area.label` | presentation source | The only authored prose in the model. |
| 3 | `start` | `presentation.json` `area.start` | area/gameplay graph | Declared boolean; the collection still requires exactly one. |
| 4 | `primaryGate` | removed | area/gameplay graph | Collapsed into `route.exit.gate`; the plan derives `objective.gate`. |
| 5 | `objective.kind` | plan `objective.kind`, emitted by the compiler | area/gameplay graph | A single-valued constant in the whole corpus; the compiler declares it, content no longer repeats it. |
| 6 | `objective.target` | removed | area/gameplay graph | Was forced equal to `primaryGate`; carries no independent information. |
| 7 | `pursuitLight` | `presentation.json` `pursuit.light` | area/gameplay graph | Names the light whose extinction wakes the gaoler; now checked to be a compiled `light`. |
| 8 | `forensicScenario` | `capture.mjs` `FORENSIC_SCENARIO` | test fixture | Identical in all four areas and read only by capture tooling. |
| 9 | `exit.gate` | `presentation.json` `route.exit.gate` | area/gameplay graph | The one surviving spelling of the triple. |
| 10 | `exit.toArea` | `presentation.json` `route.exit.to_area` | area/gameplay graph | Graph edge target, cross-checked against declared area ids. |
| 11 | `exit.entry` | `presentation.json` `route.entry` | area/gameplay graph | Owner ruling 3: each area declares its own arrival cell, validated against its own bounds and masses; the start area declares none. Cross-area authority removed. |
| 12 | `architecture.bounds.width`/`.height` | `presentation.json` `architecture.bounds` | presentation source (deliberate deviation) | Audit proposes World IR; `nomos.source@1` has no lattice-extent syntax, so it is unreachable in R1. Owner-approved deviation with the remedy recorded — §6 finding 2. |
| 13 | `architecture.wallHeight` | `presentation.json` `architecture.wall_height_steps` | presentation source | Integer tenths of a cell; the unit is now in the schema, not in two renderers. |
| 14 | `architecture.style.assembly` | `presentation.json`, selected from `ARCHITECTURE_ASSEMBLIES` | renderer catalog defines the closed set; presentation source selects from it | Owner ruling 1: same definition/selection split as socket names. `source.rs` checks the grammar, `verify.mjs` checks membership. Resolved. |
| 15 | `architecture.style.materialFamily` | `presentation.json`, selected from `MATERIAL_FAMILIES` | renderer catalog defines the closed set; presentation source selects from it | Same. |
| 16 | `architecture.style.trimFamily` | `presentation.json`, selected from `TRIM_FAMILIES` | renderer catalog defines the closed set; presentation source selects from it | Same. |
| 17 | `architecture.masses[].id` | `presentation.json` | presentation source | Presentation-only collision mass; unique per area. |
| 18 | `architecture.masses[].min`/`.max` | `presentation.json` | presentation source | Integer lattice rectangle, validated against `bounds`. |
| 19 | `architecture.masses[].height` | `presentation.json` `height_steps` | presentation source | Integer tenths of a cell. |
| 20 | `actors[].id` | `presentation.json` `actors[].id` | presentation source | Still required to be `player` and `gaoler`; §4.3 item 21 defers the role to R1-5. |
| 21 | `actors[].assembly` | `presentation.json`, selected from `ACTOR_ASSEMBLIES` | renderer catalog defines the closed set; presentation source selects from it | Owner ruling 1. Resolved. |
| 22 | `actors[].anchor.kind` | removed | presentation source | Single-valued enum; the field is now literally `cell`. |
| 23 | `actors[].anchor.cell` | `presentation.json` `actors[].cell` | presentation source | Sole authority: `play-state.mjs`'s duplicate defaults are deleted. |
| 24 | `effects[].id` | `presentation.json` | presentation source | Stable effect identity. |
| 25 | `effects[].assembly` | `presentation.json`, selected from `EFFECT_ASSEMBLIES` | renderer catalog defines the closed set; presentation source selects from it | Owner ruling 1. Resolved. |
| 26 | `effects[].anchorEntity` | `presentation.json` `effects[].anchor.entity` | presentation source | Binds the effect to a compiled entity, unchanged in meaning. |
| 27 | `effects[].presentationAnchor` | removed | presentation source | Replaced by `anchor.socket`; the audit's proposed repair. |
| 28 | `schema` (plan) | plan `schema`, from `plan.rs` | tooling only | Declared by the emitting code; now `nomos.rendering_plan@2`. |
| 29 | `deterministic` (plan) | removed | tooling only | Dead flag, read by nothing. |
| 30 | `projectionSchemas[].name`/`.version` | plan `projection_schemas` | kernel projection | Copied verbatim from the four projection members. |
| 31 | `projectionDigests` | plan `projection_digests` as `[{file, digest}]` | test fixture | Integrity evidence; the dotted keys become values. |
| 32 | `camera.identity` | `render-core.mjs` `camera.identity` | renderer catalog | Leaves content and the plan; the renderer that projects owns it. |
| 33 | `camera.projection` | `render-core.mjs` `camera.projection` | renderer catalog | Same. |
| 34 | `camera.width`/`.height`/`.tileWidth`/`.tileHeight` | `render-core.mjs` `camera` | renderer catalog | Same; four of the 26 floats. |
| 35 | `palette` | removed | renderer catalog | Never dereferenced; each renderer's own table stands. |
| 36 | `entities[].id` | plan `entities[].id` | kernel projection | Passthrough from `nomos.entity_catalog@1`. |
| 37 | `entities[].kind` | plan `entities[].kind` | World IR (via kernel projection) | Resolved at R1-2: the catalog's declared `primitive`, cross-checked against `capabilities`. |
| 38 | `entities[].visual_assembly` | plan, from `EntityKind::visual_assembly` | renderer catalog | Still in the compiler; §4.3 items 5–6, deferred to R1-4. |
| 39 | `entities[].material_family` | plan, from `EntityKind::material_family` | renderer catalog | Same. |
| 40 | `entities[].anchor.*` | plan `entities[].anchor` | World IR | Passthrough of the projection's binding; `direction` is now read (§3.3, §4.3 item 22). |
| 41 | `entities[].machine_namespaces` | plan | kernel projection | Passthrough; still forensic-only. |
| 42 | `entities[].provenance[]` | plan | test fixture | Byte-range citations, forensic-only, unchanged. |
| 43 | `uiAnchors` | removed | renderer catalog | Fully dead in every consumer. |
| 44 | `scenarios[].id` | plan | test fixture | The scenario-capture directory name. |
| 45 | `scenarios[].label` | plan, derived in `plan.rs` | presentation source | Still convention-derived; §4.3 item 14, deferred to R1-5. Its authority is Rust, not JavaScript. |
| 46 | `scenarios[].tick` | plan | runtime state | From the effective-fact document. |
| 47 | `scenarios[].state_hash` | plan | runtime state | Same document, so it cannot disagree with the dispositions. |
| 48 | `scenarios[].machine_states` | plan, `[{namespace, state}]` | runtime state | Raw snapshot; the four re-derivations above it collapse into one accessor. |
| 49 | `scenarios[].movement[]` | plan, `[{entity, disposition, cost, reasons}]` | kernel projection | Resolved at R1-2; copied from `nomos.effective_facts@1`. |
| 50 | `scenarios[].effective_light[]` | plan, `[{entity, emitting}]` | kernel projection | Same. |
| 51 | `interactions[].id` | plan | test fixture | Synthetic; §4.3 item 15, deferred to R1-5. |
| 52 | `interactions[].from_scenario`/`.to_scenario`/`.target_entity`/`.action` | plan | test fixture | Same derivation, same deferral. |
| 53 | `interactions[].input_state_hash`/`.resulting_state_hash` | plan | runtime state | Direct command-log facts. |
| 54 | `collection.schema` | `build-collection.mjs` | tooling only | Now `nomos.experiment.area_collection@2`; its shape changes here. |
| 55 | `collection.deterministic` | removed | tooling only | Dead flag, matching plan change #1. |
| 56 | `lookProfile.id` | removed | renderer catalog | Look ids exist only in `LOOK_PROFILE_IDS` and the two profiles. |
| 57 | `lookProfile.digest` | collection `visual_grammar.digest` | test fixture | The one consumed field; renamed with its container. |
| 58 | `lookProfile.grammar.renderingPlanSchema` | collection `visual_grammar.rendering_plan_schema` | tooling only | Cross-area equality check. |
| 59 | `lookProfile.grammar.projectionSchemas` | collection `visual_grammar.projection_schemas` | kernel projection | Same. |
| 60 | `lookProfile.grammar.camera` | removed | renderer catalog | The plan no longer carries a camera to copy. |
| 61 | `lookProfile.grammar.palette` | removed | renderer catalog | Same. |
| 62 | `lookProfile.grammar.architectureStyle` | collection `visual_grammar.architecture_style` | renderer catalog | Still checked equal across areas; membership in the catalog's closed sets is checked per area by `verify.mjs` (owner ruling 1). |
| 63 | `lookProfile.grammar.entityAssemblies` | collection `visual_grammar.entity_assemblies` | renderer catalog | Still derived from the plan while rows 38–39 stay in the compiler. |
| 64 | `lookProfile.grammar.actorAssemblies` | collection `visual_grammar.actor_assemblies` | renderer catalog | Same, from content-declared actor assemblies. |
| 65 | `lookProfile.grammar.effectAssemblies` | collection `visual_grammar.effect_assemblies` | renderer catalog | Same, from content-declared effect assemblies. |
| 66 | `lookProfile.grammar.uiAnchors` | removed | renderer catalog | Second copy of a dead field. |
| 67 | `startArea` | collection `start_area` | area/gameplay graph | Derived with a cardinality check from `area.start`. |
| 68 | `route[].fromArea/.gate/.toArea/.entry` | collection `route[]`, snake_case | area/gameplay graph | Walked from each plan's `objective.gate` and `route`; mechanically derived. |
| 69 | `areas[].id/.label/.plan` | collection `areas[]` | area/gameplay graph | Unchanged derivation from each plan's `area`. |

Every row has exactly one owner, and no row's only authority is JavaScript:
rows 1–27 are decoded and validated by `source.rs`, rows 28–53 are emitted by
`plan.rs`, rows 32–34 and 56 are declared renderer constants, and rows 54–69 are
computed by `build-collection.mjs` from the first two groups. Rows 14, 15, 16,
21, and 25 hold under the definition/selection split of owner ruling 1: the
renderer catalog defines the closed set, the source selects a member, and both
halves are checked — grammar in Rust, membership in `verify.mjs`. Row 12 is the
one recorded deviation from the audit's proposed owner (§6 finding 2).

### 4.2 §2 "Double authorities" — 9 rows

| # | Double authority | Disposition | Reason |
| --- | --- | --- | --- |
| 1 | Camera identity and geometry, declared in the plan and again in the collection | **resolved** | Camera leaves both artifacts; `render-core.mjs` declares the six SVG constants and `webgl-renderer.mjs` its own named ortho constants — one owner each, zero content copies. |
| 2 | Wall-height scale meaning two different things in two renderers | **resolved** | Content declares `wall_height_steps` in a schema-fixed unit; each renderer declares its own cells-to-screen scale (`CELL_HEIGHT_PIXELS`, `VERTICAL_SCALE`) instead of applying an undeclared one. |
| 3 | Masonry mass height scale, same pattern | **resolved** | Same mechanism, via `height_steps`. |
| 4 | Actor start cell authored in content and defaulted again in `play-state.mjs:18-19` | **resolved** | The two hardcoded fallbacks are deleted; `createPlayState` throws when the plan declares no such actor. |
| 5 | `primaryGate` / `objective.target` / `exit.gate` forced equal | **resolved** | One authored field, `route.exit.gate`; the compiler derives `objective`. |
| 6 | Ward / integrity / access re-derived in four places with four private fallbacks | **resolved** | One `machineState` accessor and one `doorState`/`wardSealed` pair; all eight literal fallbacks deleted, absent namespace throws. Each renderer still *compares* the state strings to choose geometry — that is consumption of a single authority, not a second derivation, and the two renderers draw different shapes. |
| 7 | Gaoler hunting computed for gameplay and again, with the opposite operator, for the HUD | **resolved** | One `isHunting(plan, scenario)`; `play-state.mjs` and `viewer.html` both call it. |
| 8 | Four disjoint look-profile identifier schemes | **resolved** | `lookProfile.id` leaves the collection; `LOOK_PROFILE_IDS` and the two profile objects are the only look ids, and the viewer's toggle uses those keys. |
| 9 | `palette` string plus two unrelated hardcoded colour tables | **resolved** | The string leaves the plan and the collection. Two renderers keep two tables because there are two renderers; R1-4 promotes one viewer, after which one table remains. |

### 4.3 §3 "Derived by convention" — 26 rows

| # | Convention | Disposition | Reason |
| --- | --- | --- | --- |
| 1 | door via `machine.endsWith(".access")` | **resolved (R1-2)** | Catalog `primitive`; nothing left for R1-3. |
| 2 | light via light-resolver membership | **resolved (R1-2)** | Same table. |
| 3 | water via a `traversal_cost_ground` claim | **resolved (R1-2)** | Same table. |
| 4 | silent `unknown` / `visual/marker` fallback | **resolved (R1-2)** | `RP0201` refuses a primitive carrying another kind's full capability signature. |
| 5 | kind → `visualAssembly` table inside the compiler | **deferred → R1-4** | It is renderer-catalog data and R1-4 creates the accepted renderer that can own it; R1-3 may not move an accepted plan's field into `experiments/`, which `RUNTIME.md` §2 makes non-authoritative. |
| 6 | kind → `materialFamily` table, defaulting to `"stone"` | **deferred → R1-4** | Same reason; the two tables move together. |
| 7 | `{kind, target}` key set, literal `exit_via`, literal actor ids | **split: resolved / deferred → R1-5** | The key set and `exit_via` are resolved — `objective` leaves content and is derived. The required literal ids `player` and `gaoler` are deferred to R1-5, which the issue's Scope names as the slice that makes actors authoritative runtime state. |
| 8 | magic `width ≤ 9`, `height ≤ 6`, `0 < wallHeight ≤ 5` in the compiler | **resolved** | They become declared constraints of `nomos.presentation_source@1` (§1.3), stated in `AUTHORING.md`, and enforced with `RP0202`. A documented schema constraint is not a convention. |
| 9 | magic `0 < height ≤ 4` mass bound | **resolved** | Same, as `1 ≤ height_steps ≤ 40`. |
| 10 | `activationIsActive`, a second activation evaluator | **resolved (R1-2)** | Deleted with `build-plan.mjs`. |
| 11 | `"01-baseline"` special-cased as a permitted rejection | **resolved (R1-2)** | `runs.rs` keeps only the condition that carries meaning. |
| 12 | movement recomputed in JavaScript | **resolved (R1-2)** | Copied from `nomos.effective_facts@1`. |
| 13 | `effectiveLight` recomputed in JavaScript | **resolved (R1-2)** | Same. |
| 14 | `scenario.label` regex-stripped from a directory name | **deferred → R1-5** | A scenario names a run, not an area; R1-3's source describes an area and the issue's Scope declares no `scenarios` section. R1-5 makes runtime state carry a stable ordered collection, which is where a run's declared label attaches. |
| 15 | `interactions[]` reconstructed by diffing command logs, `O(n²)` | **deferred → R1-5** | R1-5's declared command batches and successor ordering replace the reconstruction; `runs.rs` already records this owner. |
| 16 | the machine-state lookup implemented twice with private fallbacks | **resolved** | One `machineState`, fail closed. |
| 17 | `kind` and `effect.assembly` re-checked by literal string in three files | **resolved** | The `effect.assembly === "visual/cyan_crescent"` filters are deleted: effects are drawn through the catalog's assembly entry and placed by socket. `kind` comparisons remain as each renderer's dispatch on a closed enum the compiler resolved once — consumption of one authority, not re-derivation — and `play-state.mjs`'s three copies collapse into the shared accessors. |
| 18 | ward `=== "sealed"` in four places | **resolved** | One `wardSealed`. |
| 19 | integrity `=== "destroyed"` in two places | **resolved** | One `doorState` lookup; the two comparisons are two renderers' drawing decisions. |
| 20 | access `=== "open"`/`"locked"` in three places | **resolved** | Same. |
| 21 | `actor.id === "player"` as the only role signal | **deferred → R1-5** | The issue's Scope defers actors to R1-5 as runtime state; a declared role belongs with the authoritative actor collection. |
| 22 | north-wall door derived from `anchor.cell.y === 0` while `anchor.direction` is never read | **resolved** | `buildWalls` reads `entity.anchor.direction === "north"`, and the socket table is anchored to the declared face. |
| 23 | `plan.scenarios[0]` treated as the default by array position | **deferred → R1-5** | The default is a property of the declared ordered scenario collection R1-5 introduces; picking one in R1-3 would be a second convention. |
| 24 | the HUD re-deriving the pursuit condition | **resolved** | One `isHunting`. |
| 25 | number keys mapped to `states.children[n-1]` DOM order | **resolved** | The handler indexes `plan.scenarios[n-1]` and selects by id, so DOM order is irrelevant. |
| 26 | `displayName()` title-casing identifiers into all on-screen prose | **deferred → R1-4** | The accepted viewer must not invent prose from identifiers; R1-4 decides whether that is an authored display-string table or a per-entity `label`. Adding authored prose now would put strings into content that no accepted consumer reads, since today's only consumer is the quarantined viewer. |

18 resolved (items 1–4, 8–13, 16–20, 22, 24, 25), 1 split (item 7), 7 deferred
(items 5, 6, 14, 15, 21, 23, 26). Three go to R1-4 (5, 6, 26) and five to R1-5
(7's remainder, 14, 15, 21, 23).

### 4.4 §4 "Raw floating-point presentation values" — 26 values

All 26 are resolved; none is deferred.

| # | Value | File:line today | Becomes | Reason |
| --- | --- | --- | --- | --- |
| 1 | `wallHeight` `4.5` | `north-gaol/area.json:15` | `wall_height_steps: 45` | Integer tenths of a cell; `45/10 === 4.5`. |
| 2 | `wallHeight` `4.5` | `cistern-walk/area.json:16` | `wall_height_steps: 45` | Same. |
| 3 | `wallHeight` `5` | `ember-vault/area.json:16` | `wall_height_steps: 50` | Integer-valued but still an untyped decimal field; now typed. |
| 4 | `wallHeight` `4.8` | `ossuary-reach/area.json:16` | `wall_height_steps: 48` | `48/10 === 4.8`. |
| 5 | mass `height` `2.6` (`channel_buttress`) | `cistern-walk/area.json:27` | `height_steps: 26` | `26/10 === 2.6`. |
| 6 | mass `height` `3.2` (`west_vault_pier`) | `ember-vault/area.json:27` | `height_steps: 32` | `32/10 === 3.2`. |
| 7 | mass `height` `3.2` (`east_vault_pier`) | `ember-vault/area.json:33` | `height_steps: 32` | Same. |
| 8 | mass `height` `0.7` (`west_tomb_bank`) | `ossuary-reach/area.json:27` | `height_steps: 7` | `7/10 === 0.7`. |
| 9 | mass `height` `0.7` (`east_tomb_bank`) | `ossuary-reach/area.json:33` | `height_steps: 7` | Same. |
| 10 | mass `height` `2.4` (`reliquary_pier`) | `ossuary-reach/area.json:39` | `height_steps: 24` | `24/10 === 2.4`. |
| 11–13 | `ward_crescent.presentationAnchor` `x 3.6`, `y 3.4`, `z 0` | `north-gaol/area.json:40` | `anchor: {entity: "north_gate", socket: "ward"}` | Named socket on the anchor entity; the audit's own proposed repair. |
| 14–16 | `sluice_ward_crescent` `x 4.9`, `y 3.8`, `z 0` | `cistern-walk/area.json:48` | `anchor: {entity: "sluice_gate", socket: "ward"}` | Same. |
| 17–19 | `vault_ward_crescent` `x 4`, `y 3.8`, `z 0` | `ember-vault/area.json:54` | `anchor: {entity: "vault_gate", socket: "ward"}` | Same. |
| 20–22 | `bone_ward_crescent` `x 6.1`, `y 1.2`, `z 0` | `ossuary-reach/area.json:60` | `anchor: {entity: "bone_gate", socket: "ward"}` | Same. |
| 23 | `camera.width` `1200` | `build-plan.mjs:177` and 5 artifacts | `render-core.mjs` `camera.width` | Renderer-catalog constant owned once by the renderer that projects with it. |
| 24 | `camera.height` `540` | same | `render-core.mjs` `camera.height` | Same. |
| 25 | `camera.tileWidth` `96` | same | `render-core.mjs` `camera.tileWidth` | Same. |
| 26 | `camera.tileHeight` `50` | same | `render-core.mjs` `camera.tileHeight` | Same. |

The renderer-internal floats the audit's §4 prose notes but excludes from the 26
— `0.72`, `3.7`, `.86`, `.92`, `1.08` — become named constants too (§3.2, §3.4),
so nothing on the path is an unnamed magic number.

**Totals across §4.2–§4.4: 61 rows — 53 resolved (9 + 18 + 26), 1 split (§4.3
item 7, resolved in part), 7 deferred. Every deferral names R1-4 or R1-5; none
is open-ended.**

---

## 5. Consumers, and the visual delta

### 5.1 Edits by file

**Rust — `crates/nomos-render-plan/`**

| File | Edit |
| --- | --- |
| `src/area.rs` → `src/source.rs` | Rewritten as the `nomos.presentation_source@1` decoder: identity binding, the §1.3 constraints, the identifier grammars, the socket vocabulary, typed `PresentationSource` instead of `Json` passthrough. Declares `SchemaId::new("nomos.presentation_source", 1)`. |
| `src/doc.rs` | **Deleted** (issue #144). |
| `src/decimal.rs` | **Deleted**; the source is integer-only. |
| `src/json.rs` | `Json::Number(Decimal)` → `Json::Integer(i64)`; any `.`/`e`/`E`/leading-`+` lexeme refused with `RP0205`. |
| `src/plan.rs` | Assembles a `CanonicalValue`; `rendering_plan_schema()` → version 2; the `look` module deleted; `objective`/`route`/`pursuit` emitted; `keyed_array` for the three stable-ID collections. |
| `src/lib.rs` | Module list and re-exports follow. |
| `src/error.rs` | `RP0205` re-documented as the integer-profile refusal; `RP0206` re-purposed as the identifier-grammar refusal; codes and prefixes otherwise unchanged. |
| `src/bin/nomos-render-plan.rs` | `--area` → `--source`; usage string. |
| `tests/canonical_profile.rs` | Replaced by `tests/canonical_round_trip.rs`: `parse_canonical` round-trips the emitted plan and re-encodes byte-identically. |
| `tests/source.rs` (new) | Version-mismatch refusal; float-literal refusal at every depth and in unknown fields; exponent refusal; each identifier grammar; each bounded invariant; the socket vocabulary. |
| `tests/common/mod.rs` | Fixture writes `presentation.json`. |
| `tests/inputs.rs`, `classification.rs`, `kernel_divergences.rs`, `normalization.rs`, `schema_binding.rs` | Field renames; `--source`; the `no_code_path_holds_a_floating_point_type` grep kept. |

**Content**

| File | Edit |
| --- | --- |
| `areas/*/presentation.json` (×4) | New. |
| `areas/*/area.json` (×4) | Deleted. |
| `areas/*/rendering-plan.example.json` (×4) | Regenerated under `@2`. |
| `area-collection.example.json` | Regenerated: `nomos.experiment.area_collection@2`, snake_case, `visual_grammar` without `id`/`camera`/`palette`/`ui_anchors`, no `deterministic`. |
| `AUTHORING.md` | Rewritten for the typed source: the schema's constraints stated as the authoring rules, sockets instead of anchors, steps instead of decimals. |
| `README.md`, `CAPTURE.md` | Pipeline description and the new digests. |

**JavaScript**

| File | Edit |
| --- | --- |
| `src/renderer-catalog.mjs` | New: `VERTICAL_STEPS_PER_CELL`, `cellsOf`, `SOCKETS`, `socketPosition`, `LOOK_PROFILE_IDS`, the five closed sets, `machineState`, `doorState`, `wardSealed`, `isHunting`. |
| `src/render-core.mjs` | Camera constants and `CELL_HEIGHT_PIXELS` declared and exported; `cellsOf(steps)`; socket-resolved crescent; fallbacks removed; `plan.objective.gate`; array lookups for `movement`/`effective_light`/`machine_states`. |
| `src/webgl-renderer.mjs` | `VERTICAL_SCALE`, `ORTHO_HALF_HEIGHT`, `CAMERA_OFFSET` declared; socket-resolved upright crescent; `anchor.direction === "north"`; fallbacks removed; renamed fields. |
| `src/play-state.mjs` | Actor fallbacks deleted; `plan.objective.gate`, `plan.pursuit.light`; `isHunting`; array lookups; `enterArea` places the player at the destination plan's own `route.entry`. |
| `viewer.html` | `isHunting`; look ids from `LOOK_PROFILE_IDS`; number keys index `plan.scenarios`; `collection.visual_grammar.digest`; `connection.to_area`. |
| `src/build-collection.mjs` | Collection `@2`; grammar without camera/palette/ui anchors; route walked from `plan.objective.gate` and `plan.route.to_area`, with each hop's `entry` read from the destination plan; every non-null `to_area` must name a declared area that declares an `entry`; snake_case output. |
| `src/verify.mjs` | `@2`; renamed fields; membership assertions against the renderer catalog's closed sets — sockets in `SOCKETS`, and assemblies and families in `ARCHITECTURE_ASSEMBLIES`/`MATERIAL_FAMILIES`/`TRIM_FAMILIES`/`ACTOR_ASSEMBLIES`/`EFFECT_ASSEMBLIES` (owner ruling 1). |
| `src/capture.mjs` | `FORENSIC_SCENARIO`; camera imported from `render-core.mjs`. |
| `src/capture-collection.mjs` | Camera imported from `render-core.mjs`. |
| `src/area-collection.test.mjs`, `src/play-state.test.mjs`, `src/webgl-viewer.test.mjs` | Renamed fields; the `steps/10` exactness assertion; no `presentationAnchor` anywhere; no bare look-id literals. |
| `gaol` | `--source "$area_dir/presentation.json"`; stages `renderer-catalog.mjs`. |
| `compare-rendering-plan.sh` | `--source`. |

**Documents**

`RUNTIME.md` §3 (declared R1 members: the crate now owns two identities),
`docs/workspace.md`, `docs/HANDOFF.md`, `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`
(`@1` row replaced by `@2`; `nomos.presentation_source@1` row added;
`schema_identities_r1` becomes 4), and this record.

### 5.2 The expected visual delta

**The crescent glyph, and nothing else.** The four `visual/cyan_crescent`
effects move from hand-placed floats to the gate's `ward` socket:

| Area | Old anchor | Old SVG point | New SVG point (gate cell → socket) |
| --- | --- | --- | --- |
| north-gaol | `(3.6, 3.4, 0)` | `(479.6, 300.0)` | `north_gate (5,0)` → `(734.0, 197.9)` |
| cistern-walk | `(4.9, 3.8, 0)` | `(522.8, 342.5)` | `sluice_gate (2,0)` → `(590.0, 122.9)` |
| ember-vault | `(4.0, 3.8, 0)` | `(479.6, 320.0)` | `vault_gate (4,0)` → `(686.0, 172.9)` |
| ossuary-reach | `(6.1, 1.2, 0)` | `(705.2, 307.5)` | `bone_gate (6,0)` → `(782.0, 222.9)` |

All four land clear of both HUD panels and inside the 1200×540 frame.

The crescent is drawn only while the primary gate's ward is `sealed`, which in
every area is scenarios `01-baseline` and `02-breached-warded` only. So per
area: frames `01` and `02` change, frames `03`, `04`, `05` are byte-identical,
`contact-sheet.svg` changes (it composes frames 1–4), and `forensic.svg` — whose
scenario is `03-breached-unsealed`, ward unsealed, no crescent — changes by
**one string only**, `nomos.rendering_plan@1` → `nomos.rendering_plan@2` in the
overlay's own provenance line (`render-core.mjs:197`). The cross-area
`frames/contact-sheet.svg` composes scenarios `[0]` and `[2]` per area, so it
changes in its four `01-baseline` panels, and `frames/contact-sheet.png` follows.

Expected: 30 SVG/PNG artifacts, **12 unchanged** (three scenario frames × four
areas) and **18 changed** (8 scenario frames, 4 per-area contact sheets, 4
forensic overlays, the cross-area sheet, and its PNG).

### 5.3 How it is proved

1. **Digests before and after.** `experiments/executable-gaol/gaol capture` at
   the branch's merge base and at its head, `sha256` over every artifact under
   `target/executable-gaol`, both tables recorded in this record and in the PR
   body. The 12 unchanged frames must match byte for byte.
2. **Crescent-only substitution, for the 13 crescent-bearing artifacts.** The
   crescent occupies exactly three SVG elements — one `<path>` and two
   `<circle>`s (`render-core.mjs:188-189`) — all generated from a single point
   `p`. For each changed frame the proof renders that three-element block at the
   old point and at the new one and asserts:

   ```python
   assert new_bytes.count(crescent_block(p_new)) == expected_instances
   reverted = new_bytes.replace(crescent_block(p_new), crescent_block(p_old))
   assert sha256(reverted).hexdigest() == digest_before
   ```

   A pass means every other byte of the frame is unchanged — a stronger
   statement than a visual diff, and the same technique
   `docs/review/rendering-plan-compiler.md` used for the forensic overlay.
3. **Forensic overlays.** The same substitution with
   `nomos.rendering_plan@2` → `nomos.rendering_plan@1` reproduces each old
   `forensic.svg` digest exactly; those four files contain no crescent.
4. **A described diff.** For one frame per area, the `diff` of the SVG text is
   included in the PR body; it is expected to be exactly three changed elements
   and no other line.
5. **Owner sees before/after.** The regenerated `contact-sheet.png` is committed
   and the PR body carries the before/after pair.
6. **Everything else green.** `gaol verify` (`cmp` against the regenerated
   fixtures, the three node suites, four `EXECUTABLE_GAOL_VERIFY PASS`
   receipts), `gaol site`, `docs/evaluation/r1-schema-ownership.sh`, and
   `RUNTIME.md` §6's four kernel commands, all recorded verbatim, then rerun by
   a non-author.

---

## 6. Findings against the issue, and the owner's rulings

Three findings were raised in phase 1. All three are decided; the rulings are
applied above and are what ships.

### Finding 1 — five content fields whose audited owner is the renderer catalog — RULED

`architecture.style.{assembly, material_family, trim_family}`,
`actors[].assembly`, and `effects[].assembly` hold one value each across the
whole corpus, and `build-collection.mjs:36-38` already enforced cross-area
equality: four content files were re-declaring constants. The audit assigns all
five to *renderer catalog* (§1 rows 14, 15, 16, 21, 25); the issue's Scope keeps
them in content.

**Ruling: keep them in content, under the definition/selection split.** The
renderer catalog defines the closed set a field may draw from; the presentation
source selects one member of it. That is exactly the split already adopted for
socket names (§3.3), and it gives each fact one owner: the catalog owns *what
is legal*, content owns *which one this area uses*. `source.rs` checks the
grammar; `verify.mjs` checks membership in `ARCHITECTURE_ASSEMBLIES`, `MATERIAL_FAMILIES`,
`TRIM_FAMILIES`, `ACTOR_ASSEMBLIES`, and `EFFECT_ASSEMBLIES`, so a well-formed
name outside the catalog fails the build rather than a frame. The five rows are
**resolved**, not deferred. R1-4 turns the JavaScript closed sets into a
Rust-side catalog when it promotes the viewer.

### Finding 2 — the audit's proposed owner for `architecture.bounds` is unreachable in R1 — APPROVED

Audit §1 row 12 assigns `bounds.width`/`.height` to *World IR*, citing
`THESIS.md:280` (`spatial.boundary -> lattice`), and notes the value "is never
cross-checked against the compiled `world.nomos` extent". The extent it would be
checked against does not exist: `nomos.source@1` has no lattice-boundary syntax
— all four `experiments/executable-gaol/areas/*/world.nomos` declare only
`schema`, `catalog`, and `entity` blocks, and
`grep -c 'lattice\|extent\|bounds\|boundary'` returns `0` for each of the four.
Giving World IR the extent needs new source grammar plus a World IR field plus a
projection field, touching `nomos.source@1` and `nomos.world_ir@2` — two of the
twenty frozen Gate K identities.

**Ruling: approved as written.** `bounds` stays a presentation-source field in
R1, recorded in the owner column as a **deliberate deviation** from the audit's
proposal rather than as agreement with it. The remedy is named for a later owner
decision and is not R1-4 or R1-5 work: declare a `boundary` in `.nomos`, carry
it into World IR, publish it on `nomos.entity_catalog@1`, and cross-check
`bounds` against it in `source.rs` with `RP0202` on disagreement.

### Finding 3 — the audit's internal section cross-references are off by one

`docs/review/executable-gaol-ownership-audit.md` heads its double-authority list
"## 2. Double authorities" but refers to it as "§3 (Double authorities)" in
items 18 and 19 of the next section, and calls its summary "§6" when it is §5.
Nothing in the audit's content is wrong; only three pointers are. This record
therefore maps the audit by heading name — "Double authorities" (9), "Derived by
convention" (26), "Raw floating-point presentation values" (26) — so no
mis-numbered pointer can mis-route a row. The audit itself is owner-reviewed
evidence at its commit and is left untouched.

### Correction 3 — `route.entry` changes hands — RULED

Not a phase-1 finding; an owner correction to the design. `area.json` had the
*exiting* area author `exit.entry`, a cell inside the *destination* area. That
is cross-area authority — an area declaring a fact about a room it does not own
— and the only validation was `build-collection.mjs:44-51` checking it against a
global 9×6 rather than against the destination's own bounds and masses.

**Ruling: flip it.** `route.exit` carries `{gate, to_area}` only. Every area
with `area.start == false` declares its own `route.entry`, the one cell a player
arrives on, validated by `source.rs` against *that area's* `bounds` and refused
if it lies inside *that area's* `masses`. The start area declares no `entry`,
because nothing arrives there. `to_area` is non-null exactly when the area is
not the route's terminal. `play-state.mjs` places the player at the destination
area's own `route.entry` on transition, and `build-collection.mjs` keeps one
purely referential cross-area check: every non-null `to_area` names a declared
area, and that area declares an `entry`.

The three cells are unchanged in value, only in owner: Ember Vault `{7, 5, 0}`,
Ossuary Reach `{1, 5, 0}`, North Gaol `{2, 4, 0}`, Cistern Walk none.

### Nothing in the issue is impossible

The four settled decisions all hold:

- **Tenths-of-a-cell integer heights** reproduce today's doubles exactly, not
  approximately, for all ten values (§3.1) — verified arithmetically, not
  assumed.
- **Socket-attached effects** remove all twelve `presentationAnchor` components
  with a single offset that lands on the ward mark in both renderers (§3.3).
- **What leaves content** leaves cleanly; every removed field is either dead
  (`uiAnchors`, `deterministic`, `palette`), duplicated (`camera`,
  `lookProfile`, the gate triple), or tooling's (`forensicScenario`).
- **Plan `@2` via `CanonicalValue`** is expressible with no widening at all, so
  `doc.rs`, `PlanField`, and `decimal.rs` are deleted rather than shrunk (§2.3).

The one mechanical care point: the compiler appends a single `LF` after the
canonical bytes, so the round-trip test must strip it before calling
`parse_canonical` — the same allowance `read.rs` already makes for
`nomos effective-facts`.
