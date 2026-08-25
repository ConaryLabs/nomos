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

Schema `nomos.presentation_source@2`, declared and decoded by
`crates/nomos-render-plan/src/source.rs`. Every rule below is enforced by that
decoder with a stable `RP####` diagnostic — none of it is convention, and none
of it is a check some consumer happens to perform.

```json
{
  "schema": "nomos.presentation_source@2",
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
    {
      "id": "player",
      "role": "player",
      "assembly": "visual/player_silhouette",
      "cell": { "x": 2, "y": 4, "z": 0 }
    },
    {
      "id": "gaoler",
      "role": "pursuer",
      "assembly": "visual/gaoler_silhouette",
      "cell": { "x": 5, "y": 3, "z": 0 }
    }
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

**An actor declares a role, and its identity is free.** `role` is `player` or
`pursuer`, and it is what `crates/nomos-play` reads to decide which actor a
command moves and which one the pursuit rule steps. Exactly one actor declares
`player`; at most one declares `pursuer`. `@1` required the two identities to be
spelled `player` and `gaoler` — name an actor whatever the area calls it and
nothing will notice, which is what `renaming_both_actors_changes_nothing`
proves.

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
arrival cell. In a non-start area, `route.entry` and the sole player-role
actor's `cell` are the same arrival fact and must be equal.

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
`entry`, because nothing arrives there; every other area declares one equal to
the player-role actor's starting `cell`.
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

## Connecting a new area

Adding an area to the route is content, not code. The procedure:

1. **Choose the predecessor.** Pick the existing area the new one will sit
   behind, and edit that one field: its `presentation.json` `route.exit.to_area`,
   from `null` (or another area's id) to the new area's id.
2. **Write the new area.** Create `areas/<id>/` with `world.nomos` and
   `presentation.json`, following the required-files list and the
   `presentation.json` rules above. Give it its own `route.entry` — the cell a
   player lands on arriving through the predecessor's gate — and set its own
   `route.exit.to_area` to whatever the predecessor's old destination was (`null`
   if the predecessor used to be the route's terminal).
3. **Write five scenario scripts.** Copy `areas/north-gaol/scenarios/*.commands`
   into `areas/<id>/scenarios/` and rename the entities each command names to
   the new area's own gate and light. The five scripts stay in the same order —
   baseline, ignite, unseal, extinguish, open — and each of scenarios 2–4 still
   adds exactly one command to the one before it, because that ordering is what
   lets the pipeline derive interactions between consecutive scenarios.
4. **Produce the fixtures.**
   ```sh
   experiments/executable-gaol/gaol accept
   ```
   This compiles every area — including the new one — and copies what it
   compiled over the committed examples: each area's
   `target/executable-gaol/areas/<id>/rendering-plan.json` to that area's own
   `areas/<id>/rendering-plan.example.json`, and
   `target/executable-gaol/areas.json` to `area-collection.example.json`. It
   prints every fixture it wrote.
5. **Prove it.**
   ```sh
   experiments/executable-gaol/gaol verify
   ```
   `verify` never writes a fixture; it only compares. Green here is the proof
   that the fixtures `accept` just wrote are what the pipeline actually
   produces, not what an author typed by hand.

**What a connected area legitimately changes.** The new area's own directory
under `areas/<id>/`; the predecessor's `presentation.json` (the one
`route.exit.to_area` edit) and its regenerated `rendering-plan.example.json`;
and `area-collection.example.json`, which `gaol accept` also regenerates,
because the route graph now has one more edge. Nothing under `src/`, `apps/`,
or `crates/` changes. If a change to connect an area seems to require editing
any of those, the area is not actually content — file it rather than route
around it.

**Where the vocabulary lives.** `world.nomos` is written in the source
language the compiler reads; its full vocabulary — primitives, credentials,
machines, commands — is `docs/authoring.md`, not this file. The scenario
scripts under `scenarios/*.commands` are written in the command-script
vocabulary the scenario files themselves demonstrate: `open`, `ignite`,
`unseal`, `extinguish`, and `unlock ... with ...` are the commands this corpus
uses, each one a command the compiled world already accepts or refuses on its
own terms. This file governs only `presentation.json` and the shape of the
five scenario scripts; it is not a second copy of either vocabulary.
