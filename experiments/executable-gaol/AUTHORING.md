# Area authoring packet

Create one readable 9×6 gaol area using only the vocabulary already exercised
by the executable viewer. The renderer, camera, palette, materials,
silhouettes, and effects are fixed; an area author writes content and nothing
else. No renderer file is edited to add an area.

Required files:

- `world.nomos`: exactly two `primitive/iron_barred_door` entities on the north
  face, one `primitive/shallow_water_region`, and one
  `primitive/extinguishable_light`.
- `presentation.json`: the typed presentation source, described below.
- `scenarios/*.commands`: five ordered scenarios. Each of scenarios 2–4 adds
  exactly one command to the preceding script so browser interactions can be
  derived from real input and resulting state hashes.

## `presentation.json`

Schema `nomos.presentation_source@1`, declared and decoded by
`crates/nomos-render-plan/src/source.rs`. Every rule below is enforced by that
decoder with a stable `RP####` diagnostic — none of it is convention, and none
of it is a check some consumer happens to perform.

```json
{
  "schema": "nomos.presentation_source@1",
  "area": { "id": "north-gaol", "label": "North Gaol", "start": false },
  "route": {
    "exit": { "gate": "north_gate", "to_area": null },
    "entry": { "x": 2, "y": 4, "z": 0 }
  },
  "pursuit": { "light": "brazier_02" },
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

**Numbers are integers.** There is no decimal anywhere in the file, at any
depth, in any field — including one the schema does not know. A lexeme carrying
`.`, `e`, `E`, or a leading `+` is refused as `RP0205` when the file is read,
before any field is interpreted.

**Heights are vertical steps.** One step is a tenth of a lattice cell, so a
wall of four and a half cells is `"wall_height_steps": 45`. Walls are `1..=50`
steps and masonry masses `1..=40`. The renderer divides by ten.

**Positions are lattice cells.** `0 ≤ x < bounds.width`, `0 ≤ y < bounds.height`,
`z == 0`. A mass is a positive rectangle, `min` inclusive and `max` exclusive,
inside the bounds. An actor may not start inside a mass, and neither may the
arrival cell.

**Effects attach by socket, never by coordinate.** `anchor` is exactly
`{ entity, socket }`. The entity must be compiled, and the socket must be one
its kind declares — today that is `ward` on a door, and nothing on anything
else. Where the socket *is* is the renderer's business, in
`src/renderer-catalog.mjs`; content only names it.

**The route is owned one area at a time.** `route.exit.gate` is the single
authored spelling of the gate this area leaves by; the compiler derives the
`exit_via` objective from it, so there is no `primaryGate` and no
`objective.target` to keep in agreement. `route.entry` is *this* area's own
arrival cell — the cell a player lands on when another area's gate leads here —
validated against this area's own bounds and masses. The start area declares no
`entry`, because nothing arrives there; every other area declares one.
`to_area` is `null` exactly at the route's terminal.

**Names come from closed sets.** The decoder checks that an id, assembly, or
family name is well formed: an area id is `[a-z][a-z0-9]*(-[a-z0-9]+)*`, every
other identifier is `[a-z][a-z0-9_]*`, and an assembly is two or more such
segments joined by `/`. Which *values* are legal is the renderer's catalog, in
`src/renderer-catalog.mjs`, and `src/verify.mjs` checks each compiled plan
against it. Change composition and traversal, not the bounded look.

**Fields are exact.** An unknown field is refused, not ignored, so a typo
cannot silently disable a fact.

## Proving it

```sh
experiments/executable-gaol/gaol verify
```

That compiles every area, projects its entity catalog and per-scenario
effective facts, compiles its rendering plan with `nomos-render-plan`, compares
that plan against the committed one, checks the shared visual grammar, and
captures the cross-area sheet.

Nothing an area declares reaches the plan by convention. An entity's kind comes
from its `primitive/...` declaration by way of `nomos entity-catalog`, and its
movement disposition, cost, reasons, and light come from `nomos
effective-facts` — so renaming an entity or a machine cannot change how it is
drawn, and a primitive the compiler has no kind for is refused rather than
drawn as a marker.
