# Author notes — Drowned Stair

## `gaol verify` run count

Two runs, both green. The first run followed `gaol accept` and passed
immediately with no fixes needed; the second was a bare re-run to confirm the
exit code was stable (it was, `EXIT=0` both times). No diagnostic was ever
hit against `verify` or `accept` for this area.

## Diagnostics hit

None. `gaol accept` and `gaol verify` both succeeded on the first attempt
for `drowned-stair`. The only "failure"-shaped line in either run's output
was expected and matches the pattern already present in the other four
areas: scenario `01-baseline` (`open stair_gate`) is rejected by the
compiled world with `EK0804` (`stair_gate.access.open is illegal while the
machine is locked`), because the door starts locked and scenario 1 never
unlocks it. `ossuary-reach` and `cistern-walk`'s own `01-baseline` scenarios
reject the same way, so this was treated as intended corpus shape, not a
bug to fix.

## What the packet did not spell out, but the examples make load-bearing

- **`route.entry` equals the player actor's starting `cell`.** `AUTHORING.md`
  states the validation rule for `route.entry` (inside bounds, not inside a
  mass) but never says it must coincide with the player's `actors[].cell`.
  Checking all three non-start example areas (`north-gaol`, `ember-vault`,
  `ossuary-reach`) showed the entry cell and the player's cell are identical
  in every one. I followed that convention for `drowned-stair`
  (`{ "x": 4, "y": 5, "z": 0 }` for both) rather than inventing an
  independent arrival point; nothing in `verify` appears to check this
  directly, but diverging from an unbroken 3-for-3 convention seemed like an
  unnecessary risk.
- **Scenario file numbering vs. "the preceding script."** The prose "each of
  scenarios 2–4 adds exactly one command to the preceding script" reads at
  first as "scenario N is scenario N−1 plus one line," which would make
  scenario 1 (`open`) a prefix of scenario 2. It is not, in the reference
  area: scenario 2 is `ignite`, not `open` + something. The actual chain is
  02 → 03 → 04 (each a strict one-line extension of the previous *numbered*
  file in that range), while 01 (`baseline`) and 05 (`open-dark`) stand on
  their own with independent command sequences. I only understood this by
  diffing the four `north-gaol` scenario bodies directly rather than relying
  on the prose description.
- **The second door and second gate column are pure set-dressing.** Every
  example area declares exactly two `iron_barred_door` entities (the
  required count from `AUTHORING.md`), but only one of them is ever named in
  `route.exit.gate` or in any scenario script. The second door
  (`landing_gate` here) exists solely to satisfy the "exactly two doors on
  the north face" requirement and to vary the composition; it needed no
  scenario coverage and no route reference.
- **Gate/mass/water/brazier "difference from all four existing areas" has no
  automated check.** `verify`'s `every area is a distinct composition` test
  passed for `drowned-stair`, but nothing in the packet says what counts as
  distinct or measures it — the requirement in the task brief (different
  gate columns, water shape, masonry masses, brazier position) was satisfied
  by manual comparison against the other four `presentation.json` files
  before writing this one, not by anything `gaol` itself enforces beyond
  "distinct" in some plan-level sense.

## Composition summary

- Area id `drowned-stair`, label "Drowned Stair", inserted between Ember
  Vault and Ossuary Reach.
- Doors on the north face at `x = 0` (`landing_gate`) and `x = 3`
  (`stair_gate`, the route exit) — the only area using either column; every
  other area's door-column pair (`{5,7}`, `{4,8}`, `{2,6}`, `{1,6}`) is
  disjoint from this one.
- Water region `flood_stair`, a 5×1 shallow crossing at `x:[2,7) y:[4,5)` —
  wider and shallower than any existing region (existing shapes are 2×1,
  2×2, 1×4, 2×3).
- Two masonry masses, `stairwell_shelf` (15 steps) and `landing_plinth` (20
  steps), at positions and heights that don't repeat any existing mass.
- Brazier `sconce_light` at `(5, 2)`, distinct from the other four braziers'
  positions.
- `wall_height_steps: 40`, distinct from the other four areas' `45`, `50`,
  `45`, `48`.
