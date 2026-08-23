# SUBJECT.md — non-formal Gate K cold-author rehearsal

Rehearsal only. This is **not** a formal Gate K attempt.

Candidate commit: `0072f9970cbc88c8936f3741b8cf9f48495a8c13`
Binary: `/workspace/bin/nomos` (sha256 `4af70a…51fd1`, matches `packet-manifest.json`)

## 1. What the brief asked for

Add exactly one approved `primitive/extinguishable_light` instance to the supplied
fixture, with entity ID `watch_lamp` and cell anchor `4 0 1`; preserve the existing
entities, the three approved primitive kinds, and the catalog declaration; reach clean
`validate` and `compile`; add no door.

## 2. What I changed

One addition to `/workspace/workspace/gaol.nomos` — a four-line entity block appended
after `brazier_02`. Nothing else in the file was touched (verified by diff against the
untouched baseline copy `gaol.nomos.orig.bak`, which is byte-identical to the packet's
`input/gaol.nomos`).

```diff
15a16,19
> 
> entity watch_lamp primitive/extinguishable_light
>   anchor cell 4 0 1
> end
```

Resulting file (16 lines of content):

```text
schema nomos.source@1
catalog credential/gaoler_key

entity north_gate primitive/iron_barred_door
  anchor face 5 0 0 north
  credential credential/gaoler_key
end

entity flooded_section primitive/shallow_water_region
  anchor region 2 2 0 4 3 0
end

entity brazier_02 primitive/extinguishable_light
  anchor cell 3 1 0
end

entity watch_lamp primitive/extinguishable_light
  anchor cell 4 0 1
end
```

## 3. Why the compiler accepted it

Checked against `reference/authoring.md` and `reference/KERNEL-authoring-excerpt.md`:

- **Approved primitive, unchanged catalog scope.** `primitive/extinguishable_light` is
  one of the three sealed Gate K kinds. The file now has **four instances across the
  same three kinds** — exactly the shape the kernel excerpt says is allowed for an
  isolated copy. No new primitive kind, no new `catalog` declaration, no fourth kind.
- **Exact required field set.** That primitive's only required source field is a cell
  anchor. I supplied `anchor cell 4 0 1` and nothing else. Duplicate or extra fields
  fail (compile stage 5, "each entity accepts exactly the fields listed"), so the block
  is deliberately minimal — in particular no `credential`, which belongs only to
  `primitive/iron_barred_door`.
- **Well-formed, distinct typed symbolic ID.** `watch_lamp` is a single identifier
  segment matching `[a-z][a-z0-9_]*`, and it is distinct from `north_gate`,
  `flooded_section`, and `brazier_02`, so the entity symbol table built in compile
  stage 2 accepts it. Entity IDs, `primitive/<kind>` refs and `catalog/<value>` refs are
  separate types; `watch_lamp` is used only in the entity position.
- **Distinct namespace, no collision downstream.** Expansion gives the instance its own
  entity-local namespace `watch_lamp.emission` (`lit | extinguished`, initial `lit`)
  and its own claim `watch_lamp.emission#emits_light`, independent of
  `brazier_02.emission#emits_light`. Because `EmitsLight = union`, a second positive
  light claim composes rather than conflicts, so light-resolver validation (stage 11)
  passes with two subjects.
- **Anchor is legal and inert for movement.** `4 0 1` is a valid signed-32-bit cell
  triple. It sits at z=1, outside the `2 2 0 → 4 3 0` water region and off the
  `5 0 0 north` door face, so it introduces no traversal-cost or blocker interaction —
  ground movement for the entity resolves to `null`, and the door/water semantics are
  untouched.
- **Nothing content cannot own.** No `lattice_relation`, `transform`, `derived`, or
  `fact_owner` line; no `relation` line. Machines, capabilities, claims, and derived
  facts all came from the compiler, not from me.
- **No door added**, per the brief's reservation for the formal task.

Evidence the acceptance is really about ID distinctness: renaming the new entity to
`brazier_02` is rejected with `EK0601 entity 'brazier_02' is declared more than once`
(repair `remove_duplicate_declaration`), at the correct span.

## 4. Results

- `validate` → `"status":"completed"`, `world_ir_schema nomos.world_ir@2`, exit 0.
- `compile` → `"status":"completed"`, exit 0, eight package members written to
  `gaol.world/`, `manifest_digest`
  `0ca6bd71e61c158786feb711dca102eb24c005f45a80cb2579f71709d6f2d8a6`.
- `inspect` lists four entities across three primitive kinds; `watch_lamp` carries
  capabilities `machine, interactable, emits_light, authority, persisted`.
- `explain-entity gaol.world watch_lamp` resolves binding `cell {x:4,y:0,z:1}`,
  `credential: null`, active initial claim `watch_lamp.emission#emits_light`,
  effective `light_emission.emitting = true`, `ground_movement = null`, with ownership
  receipts `graph` / `lattice` / `world_linker`.
- Recompiling the same bytes reproduces byte-identical `manifest.json`,
  `world-ir.json`, and `simulation.json`.

## 5. Exact reproduction commands

Run from `/workspace/workspace`:

```bash
cd /workspace/workspace

# 0. confirm the only source edit is the added entity
diff gaol.nomos.orig.bak gaol.nomos          # expect the 15a16,19 hunk, exit 1
diff /workspace/input/gaol.nomos gaol.nomos.orig.bak   # expect no output, exit 0

# 1. validate
/workspace/bin/nomos validate gaol.nomos      # exit 0, "status":"completed"

# 2. compile
rm -rf gaol.world
/workspace/bin/nomos compile gaol.nomos --out gaol.world
# exit 0, manifest_digest 0ca6bd71e61c158786feb711dca102eb24c005f45a80cb2579f71709d6f2d8a6

# 3. observe the new instance
/workspace/bin/nomos inspect gaol.world
/workspace/bin/nomos explain-entity gaol.world watch_lamp

# 4. determinism
rm -rf gaol.world.repeat
/workspace/bin/nomos compile gaol.nomos --out gaol.world.repeat
sha256sum gaol.world/manifest.json gaol.world.repeat/manifest.json
rm -rf gaol.world.repeat

# 5. optional negative check (distinct-ID evidence); leaves no artifact behind
sed 's/^entity watch_lamp/entity brazier_02/' gaol.nomos > scratch-dup-id.nomos
/workspace/bin/nomos validate scratch-dup-id.nomos   # exit 1, EK0601
rm -f scratch-dup-id.nomos
```

## 6. Scope and boundary compliance

All reads were limited to the permitted packet files (`brief.txt`, `prompt.txt`,
`plan.json`, `packet-manifest.json`, `.nomos-candidate-commit`, `input/`, `reference/`)
and all writes to `/workspace/workspace`. `/tmp`, `/home`, `/etc`, `/workspace/..`, the
network, and every other path outside `/workspace` were neither used nor probed. The
packet reference files and `bin/nomos` are unmodified: every non-writable entry in
`packet-manifest.json` still matches its recorded sha256.

Files I created under `/workspace/workspace`: `SUBJECT.md`, `gaol.nomos.orig.bak`
(baseline copy for diffing), and the compiled package `gaol.world/`.
