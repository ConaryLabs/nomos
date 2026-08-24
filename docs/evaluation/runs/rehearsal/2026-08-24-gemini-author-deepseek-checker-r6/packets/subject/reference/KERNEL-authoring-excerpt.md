## 1. Exact base fixture

One source file describes exactly three world primitive instances in the base
fixture:

```text
Room contents:
  north_gate       primitive/iron_barred_door
  flooded_section  primitive/shallow_water_region
  brazier_02       primitive/extinguishable_light

Catalog values:
  credential/gaoler_key
```

These are three primitive **kinds** and three instances in the base fixture.
The formal cold-author evaluation operates on an isolated copy and may add one
second `primitive/iron_barred_door` instance. That produces four instances while
preserving the same three approved primitive kinds; it does not expand the
catalog or Gate K's semantic scope.

`credential/gaoler_key` is a catalog credential value, not a world entity and
not a fourth primitive. References to catalog values are resolved by their own
symbol table and cannot satisfy entity references.

### `north_gate`

The door is lockable, breakable, warded, and burnable. Its namespace-local
machines are independent:

```text
access:       locked | closed | open
integrity:    intact | damaged | destroyed
ward:         sealed | unsealed
combustion:   cold | burning | spent
```

Initial state:

```text
access       = locked
integrity    = intact
ward         = sealed
combustion   = cold
credential   = credential/gaoler_key
```

The ward supplies the second independent blocking claim used by the composition
test; no magical-seal primitive exists.

Required derived behavior:

```text
portal_open = access == open OR integrity == destroyed

movement_blockers =
  access_or_integrity_blocker when NOT portal_open
  ward_blocker                when ward == sealed
```

Opening the door while the ward remains sealed changes `access` but does not
change the effective ground movement disposition: the passage remains blocked
and the explanation names the ward as the surviving reason.

Required causal interaction:

```text
combustion.on_enter(burning)
  -> integrity.apply_damage(channel = fire, amount = 2)
```

For Gate K this interaction fires exactly once on entry into `burning`. Recurring
pulse or scheduler semantics are outside Gate K.

### `flooded_section`

The water region is static in Gate K. It has a lattice region binding and
contributes a ground traversal cost of `3` to otherwise traversable cells. It
must appear in simulation, navigation, persistence metadata where applicable,
and diagnostics. It may not be parsed and then ignored.

### `brazier_02`

The light has one namespace-local machine:

```text
emission: lit | extinguished
```

Initial state is `lit`. `extinguish` transitions to `extinguished`, removes the
effective light-emission fact, updates persistence and diagnostics projections,
and produces a causal receipt.

