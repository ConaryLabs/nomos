# Observed-scene presentation gap

This is a quarantined, adopter-neutral experiment under decision 0022 and issue
#189. It is evidence about the closed R1 presentation boundary, not an accepted
schema, implementation proposal, contract change, or R2 authorization.

`fixture.json` describes one already-resolved observer frame using only a
bounded integer logical crop, overlapping semantic terrain layers, actor facts,
and one exact enabled action. It contains no adopter identity or payload, raw
transform, final pixel, palette, image, shader, legality rule, pathing rule,
visibility rule, clock, persistence fact, or gameplay inference.

The executable probes ask the current `nomos.presentation_source@2` decoder to
accept four otherwise valid source documents carrying the generic facts. The
expected result is a structured refusal in every case:

- `actor-role`: the third protected interactive actor cannot enter the closed
  `player | pursuer` pair;
- `actor-facts`: life state, hostility, and protection are not accepted actor
  inputs;
- `terrain-layers`: the source cannot receive overlapping calm-ground, route,
  and structure-footprint roles; and
- `observed-actions`: the source cannot receive an already-resolved enabled
  action.

Run from the repository root:

```text
experiments/observed-scene-gap/verify
```

The wrapper first rebuilds and verifies the standing six-area evidence, then
runs one byte-exact positive control and the four refusal probes against the
real R1 rendering-plan compiler. Generated
probe inputs and `result.json` live only under `target/observed-scene-gap/`.
The committed receipt is checked after that run with:

```text
node experiments/observed-scene-gap/verify-record.mjs
```

## Outcome classification

1. **Already representable honestly: no.** The strict accepted decoder refuses
   each missing field or role with `RP0202`.
2. **Adopter-owned mapping only: no.** Mapping the third actor to `player` or
   `pursuer` changes its meaning; mapping an overlapping terrain role to a
   masonry mass changes its meaning; and omitting the supplied enabled action
   would require the presenter either to lose it or recompute gameplay
   legality. Those are semantic lies, not presentation mappings.
3. **Reusable Nomos capability: candidate.** A generic observation boundary
   could carry already-resolved terrain roles, independent actor facts, and
   observed action availability without owning their gameplay derivation. This
   experiment does not design or implement that boundary. Owner classification
   and a separate owner decision are required before any accepted work.

There is no presenter implementation here, so no presenter resolves a gameplay
fact. Any later proposal must consume the fixture's booleans, state labels,
positions, and action availability as supplied and may derive only pixels and
other presentation consequences.
