# SUBJECT.md — non-formal cold-author rehearsal

Shape: author. Classification: rehearsal (`plan.json` `formalAttempt: false`).
Candidate commit: `c1b9f355fa32f8ba749b62aa8d15bd05e9c62808`.
Operator intervention: none requested, none received.

## What I changed

Exactly one additive edit to `/workspace/workspace/gaol.nomos`: a new entity
block appended after `brazier_02`.

```text
entity watch_lamp primitive/extinguishable_light
  anchor cell 4 0 1
end
```

Nothing else in the file was touched. `diff /workspace/input/gaol.nomos
/workspace/workspace/gaol.nomos` reports only the added blank line plus the
four added lines (`15a16,19`).

Source SHA-256 before: `a69582e7400921cb0fed84fde16469c21081363af4ebaba93411e59ae3ca4725`
Source SHA-256 after:  `6e3b29965d59bc788563be3dc9985bc403299f062e258f71ad87d69aeaf8f372`

## Why this satisfies the brief

- **Approved primitive.** `primitive/extinguishable_light` is one of the three
  approved kinds in `reference/authoring.md`. No new kind was introduced; the
  file still names exactly three kinds (`iron_barred_door`,
  `shallow_water_region`, `extinguishable_light`) across four instances.
- **Distinct entity ID.** `watch_lamp` is a single identifier segment matching
  `[a-z][a-z0-9_]*` and collides with no existing entity, so the entity symbol
  table stays duplicate-free.
- **Legal cell anchor.** `authoring.md` lists a cell anchor as the sole required
  source field for `extinguishable_light`. `anchor cell 4 0 1` supplies exactly
  that field with three signed 32-bit decimals and no leading `+`. No extra or
  duplicate field is present, so the per-primitive shape check in compile stage 5
  passes.
- **Preserved content.** The `catalog credential/gaoler_key` declaration and all
  three original entities are byte-identical. No credential field was added:
  `extinguishable_light` does not accept one, and adding it would fail.
- **No door.** Instance count for `primitive/iron_barred_door` remains 1. The
  second-door case is reserved for the formal task and was deliberately not used.

## Why the compiler accepted it

`nomos validate` and `nomos compile` both exit 0 and report
`"status":"completed"` with `nomos.world_ir@2`. Confirmed by `inspect` and
`explain-entity`:

- `watch_lamp` expands to capabilities `machine, interactable, emits_light,
  authority, persisted`, exactly matching `brazier_02`'s expansion — the sealed
  three-kind catalog drove the expansion, not authored content.
- It receives its own namespace `watch_lamp.emission` (`lit | extinguished`,
  initial `lit`) with the `extinguish` command transition. This is namespace-local
  and independent of `brazier_02.emission`, so the two lights do not alias.
- Its claim `watch_lamp.emission#emits_light` is distinct from
  `brazier_02.emission#emits_light`, so the compiler-owned `EmitsLight = union`
  validation sees two positive, non-duplicate resolver subjects.
- Typed ownership receipts resolve for `entity_identity` (owner `graph`),
  `entity_spatial_anchor` (owner `lattice`), and `entity_spatial_binding`
  (owner `world_linker`, resolved to `cell {x:4, y:0, z:1}`), each with a
  correct `gaol.nomos` source span. The new light reaches the `simulation`,
  `persistence`, and `diagnostics` consumers, so it is not parsed and ignored.
- I authored no fact the compiler owns — no `lattice_relation`, `transform`,
  `derived`, or `fact_owner` line — so stage 4 had nothing to reject.

Determinism: recompiling the same bytes to a second output directory produced a
byte-identical package and the same manifest digest
`0ca6bd71e61c158786feb711dca102eb24c005f45a80cb2579f71709d6f2d8a6`. That scratch
directory was deleted.

## Reproduction

```bash
cd /workspace/workspace
diff /workspace/input/gaol.nomos gaol.nomos     # expect only 15a16,19
/workspace/bin/nomos validate gaol.nomos        # exit 0, status completed
/workspace/bin/nomos compile gaol.nomos --out gaol.world
/workspace/bin/nomos inspect gaol.world
/workspace/bin/nomos explain-entity gaol.world watch_lamp
```

`compile` requires a non-existent `--out` path; remove `gaol.world` first when
re-running. Expected compile manifest digest:
`0ca6bd71e61c158786feb711dca102eb24c005f45a80cb2579f71709d6f2d8a6`.

## Boundary notes

All reads, writes, and scratch stayed inside `/workspace`; the only writes were
inside `/workspace/workspace`. Packet references and the binary are unmodified —
every path in `packet-manifest.json` still hashes to its recorded SHA-256. I did
not access `/tmp`, `/dev`, `/home`, `/etc`, `/workspace/..`, or the network, and
used no suppression fallback. No sandbox errors were encountered.
