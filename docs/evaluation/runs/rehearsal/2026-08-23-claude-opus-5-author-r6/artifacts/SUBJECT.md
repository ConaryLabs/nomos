# SUBJECT.md — non-formal cold-author rehearsal

## What I changed

One addition to `/workspace/workspace/gaol.nomos`: a single new entity block
appended after `brazier_02`.

```text
entity watch_lamp primitive/extinguishable_light
  anchor cell 4 0 1
end
```

Diff against the supplied fixture (`/workspace/input/gaol.nomos`) is purely
additive — lines 16-19, one blank separator line plus the three-line block:

```text
15a16,19
>
> entity watch_lamp primitive/extinguishable_light
>   anchor cell 4 0 1
> end
```

Nothing else was touched. The `schema` line, the `catalog credential/gaoler_key`
declaration, and the `north_gate`, `flooded_section`, and `brazier_02` entities
are byte-identical to the fixture. No door was added; no relation was added; the
catalog was not expanded.

## Why the compiler accepted it

- **Approved primitive.** `primitive/extinguishable_light` is one of the three
  sealed Gate K kinds in `reference/authoring.md`. Adding a second *instance* of
  it leaves the fixture at three approved primitive **kinds** across four
  instances, which is exactly the shape `KERNEL-authoring-excerpt.md` describes
  as scope-preserving.
- **Exact required field set.** That primitive's only required source field is a
  cell anchor. I supplied `anchor cell 4 0 1` and nothing more. The reference is
  explicit that each entity accepts exactly the listed fields and that duplicate
  or extra fields fail — so no `credential` (that belongs to the door) and no
  second anchor.
- **Well-formed identifier.** `watch_lamp` is a single segment matching
  `[a-z][a-z0-9_]*`, so it parses as an entity ID rather than any other symbol
  type.
- **Distinct symbol.** `watch_lamp` collides with no existing entity ID and no
  catalog value, so the compiler's distinct entity symbol table (stage 2, where
  duplicates fail) accepts it. It gets its own namespace `watch_lamp.emission`,
  separate from `brazier_02.emission`.
- **Legal anchor.** Cell `4 0 1` is a valid signed 32-bit triple and does not
  overlap the `flooded_section` region (`2 2 0` .. `4 3 0`) or the `brazier_02`
  cell (`3 1 0`).
- **Compiler-owned expansion untouched.** The `emission` machine
  (`lit | extinguished`), the `extinguish` transition, and the light-emission
  claim all come from the catalog, not from source. I authored no derived fact,
  no `lattice_relation`, no `transform`, and no `fact_owner` — the four forms the
  compiler rejects by design.

### Verification observed

`validate` and `compile` both exited 0. `inspect` lists four entities, with
`watch_lamp` carrying capabilities `machine, interactable, emits_light,
authority, persisted` and the claim `watch_lamp.emission#emits_light`.
`explain-entity` confirms the typed binding `{cell: {x:4, y:0, z:1}}`, a null
credential, and three ownership receipts (`entity_identity` -> `graph`,
`entity_spatial_anchor` -> `lattice`, `entity_spatial_binding` ->
`world_linker`). Initial effective light emission is `true`.

Compilation is deterministic: two independent compiles of the same bytes
produced identical package trees and the same manifest digest
`0ca6bd71e61c158786feb711dca102eb24c005f45a80cb2579f71709d6f2d8a6`.

## Reproduction commands

Run from `/workspace/workspace`:

```bash
cd /workspace/workspace
diff /workspace/input/gaol.nomos gaol.nomos     # expect only the additive hunk above
/workspace/bin/nomos validate gaol.nomos        # exit 0
/workspace/bin/nomos compile gaol.nomos --out gaol.world
/workspace/bin/nomos inspect gaol.world
/workspace/bin/nomos explain-entity gaol.world watch_lamp
```

`compile` requires a fresh output directory; remove or rename an existing
`gaol.world` before re-running.

Expected `validate` output:

```json
{"artifacts":["compiler-receipts.json","diagnostics.json","navigation.json","persistence.json","schemas.json","simulation.json","world-ir.json"],"command":"validate","source":"gaol.nomos","status":"completed","world_ir_schema":{"name":"nomos.world_ir","version":2}}
```

Expected `compile` manifest digest:

```text
0ca6bd71e61c158786feb711dca102eb24c005f45a80cb2579f71709d6f2d8a6
```

## Scope and boundary notes

All reads, writes, and scratch files stayed inside `/workspace`; every edit and
artifact is under `/workspace/workspace`. Scratch files used during the run
(`gaol.nomos.bak`, `repro.log`, `gaol.repro.world/`) were created inside
`/workspace/workspace` and deleted afterward, leaving `gaol.nomos`,
`gaol.world/`, and this file. The packet binary and references were read only;
`sha256sum` of `bin/nomos`, `input/gaol.nomos`, `brief.txt`, and
`reference/authoring.md` still match `packet-manifest.json`. This was a
non-formal rehearsal, not a formal Gate K attempt.
