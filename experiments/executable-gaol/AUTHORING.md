# Area authoring packet

Create one readable 9×6 gaol area using only the vocabulary already exercised
by the executable viewer. The renderer, camera, palette, materials, silhouettes,
and effects are fixed; area authors edit content and presentation anchors only.

Required files:

- `world.nomos`: exactly two `primitive/iron_barred_door` entities on the north
  face, one `primitive/shallow_water_region`, and one
  `primitive/extinguishable_light`.
- `area.json`: stable area identity, one `exit_via` objective targeting the
  primary gate, pursuit light, player and gaoler cell anchors, one
  `visual/cyan_crescent` presentation anchor, and bounded architecture data.
  Connected runs additionally declare whether this is the start area and where
  its primary gate leads.
- `scenarios/*.commands`: five ordered scenarios. Each of scenarios 2–4 adds
  exactly one command to the preceding script so browser interactions can be
  derived from real input and resulting state hashes.

Bounds may not exceed `x=0..8`, `y=0..5`, `z=0`. Architecture uses only the
declared `visual/beveled_masonry` style and axis-aligned rectangular masses no
higher than 4 cells. The primary gate must be reachable from the player anchor,
water should create a legible route choice, and the brazier must light a useful
landmark. Change composition and traversal, not the bounded look. Do not edit
either renderer implementation for an area.

Run `experiments/executable-gaol/gaol verify` to compile every area, project its
entity catalog and per-scenario effective facts, compile its rendering plan with
`nomos-render-plan`, compare that plan against the committed one, check the
shared visual grammar, and capture the cross-area sheet.

Nothing an area declares reaches the plan by convention. An entity's kind comes
from its `primitive/...` declaration by way of `nomos entity-catalog`, and its
movement disposition, cost, reasons, and light come from `nomos effective-facts`
— so renaming an entity or a machine cannot change how it is drawn, and a
primitive the compiler has no kind for is refused rather than drawn as a
marker.
