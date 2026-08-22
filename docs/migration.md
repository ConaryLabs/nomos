---
title: Stable World IR v1 to v2 migration
status: SW-M implementation and evidence reference
date: 2026-08-22
applies_to: KERNEL.md sections 6 and 8; acceptance 12
---

# Stable World IR v1 to v2 migration

SW-M implements the one Gate K migration: stable ground movement v1 to v2. It
is a closed conversion, not a general compatibility framework. New compilation
emits only `nomos.world_ir@2`; `validate` and `compile` consume source and report
or publish v2. Semantic package commands `inspect`, `run`, `command`, and
`replay` refuse a stable-v1 package with `EK0414` until it is migrated.

## Exact command and immutable boundary

```text
nomos migrate <v1-world/> --to 2 --out <new-v2-world/>
```

The reader first opens the complete v1 package through the ordinary manifest,
member-hash, canonical-byte, regular-file, and exact-member-set boundary. It
then strictly decodes stable v1, checks compiler/catalog versions, validates its
semantics, regenerates all four legacy projections and the schema registry,
and verifies compiler receipt hashes and passes. Only then does it convert the
movement rows and regenerate every compiler-owned active member.

The output uses the existing verified sibling staging directory and one
same-filesystem rename. Existing output, output equal to or below input, and a
symlinked alias into input fail before mutation. Only target `2` is supported;
`EK0415` rejects another target and `EK0416` rejects overlapping input/output.

## Representation change

Each v1 subject encoded two coupled fields:

```text
blocked_ground: boolean
traversal_cost_ground: positive integer or null
```

Each v2 subject instead encodes exactly one tagged
`movement_disposition_ground`:

```text
blocked { reasons: nonempty sorted unique claim references }
traversable { cost: positive integer, reasons: sorted unique claim references }
```

The v2 decoder rejects unknown fields or variants, zero cost, empty blocked
reasons, duplicates, unstable ordering, incomplete subject coverage, and
dangling or cross-capability claim references: blocked reasons must name
ground-blocker claims and traversable reasons must name ground-cost claims for
that subject. Construction IR, source bytes, the resolver plan, and all
projection schemas are unchanged.

## Frozen provenance and digest mapping

`fixtures/gaol-v1.world/` is the exact accepted stable-v1 compiled package from
SW-K merge commit `c0e77a6a42426503413f8797b36e88ed6cceda52`; fixture commit
`2822dda` copies it byte-for-byte. Its compiler receipt still binds the unchanged
`fixtures/gaol.nomos` SHA-256
`a69582e7400921cb0fed84fde16469c21081363af4ebaba93411e59ae3ca4725` and
provenance. The migration does not edit that directory. Its manifest records
the exact legacy member hashes:

| Legacy member | SHA-256 |
| --- | --- |
| `compiler-receipts.json` | `364d14ac12382c23c540c610241a30201105cfc3c13fb1c9f7c43420ad02d8d4` |
| `diagnostics.json` | `da208fbc883dc9aa64748dd44a9c940956e1842a5be101156cd88c1204fd31e4` |
| `navigation.json` | `e22e585399cb902bebbd3cdb00dfbd7497344d51c76454aa14404de831c93f71` |
| `persistence.json` | `40a0622cfb1ef13af32328208bc5618a6508e111cf4e375a3d00d20571d0c540` |
| `schemas.json` | `9f0f9effc184c684e454786dde8447378381e47c0ce0f19d257eb421f86d0af3` |
| `simulation.json` | `0cdb454be9876dccdc05369e91fb9bdea017973d77f7803d23a921a969a1ef3c` |
| `world-ir.json` | `555017cf5e13a33b4bb5b18bae14b7577fd1fc38abf89b1f6f475874600fa493` |

| Evidence | Stable v1 | Stable v2 | Cause |
| --- | --- | --- | --- |
| World IR schema | `nomos.world_ir@1` | `nomos.world_ir@2` | tagged movement disposition and retained typed reasons |
| World IR SHA-256 | `555017cf5e13a33b4bb5b18bae14b7577fd1fc38abf89b1f6f475874600fa493` | `c55f09fe400465c637c75cbef0ea3a0ef06bd4a585c0ead78b1a50a63d790b7c` | stable movement representation only |
| Package manifest digest | `f1af0cc92ea44fd09ba93815bb99cc6c24517b56888f39be33a9d47b1299bab7` | `42af352bdbf0a3c0642d4f86f6d74384351c44fc40bbe3e3134829dcf715d17a` | v2 IR, registry, receipts, and regenerated member hashes |
| Fresh active compile package | n/a | `fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440` | compile receipts record the compile pass path; migrated output records the migration pass path |
| Runtime state schema | `nomos.runtime_state@1` | `nomos.runtime_state@2` | explicit normalization boundary; v1 bytes are incompatible |
| Persisted state schema | `nomos.persisted_runtime_state@1` | `nomos.persisted_runtime_state@2` | binds the active runtime-v2 snapshot |
| `fixtures/gaol.replay` SHA-256 | `ea68dd27fc33d32e7bba3095c46b8b964c2aba099357806938c12222c36370d2` | `859a066ba74966648ccf16389fbb1a132b94cbf850d47efa37a63dbf78c8d912` | binds the active package and runtime-v2 hash chain |

The migrated package and a fresh v2 compile intentionally have different
manifest digests because their compiler receipts truthfully record different
build paths. Their stable IR and all four semantic projections are
byte-identical.

## Normalized execution proof

The migration test projects the fully validated legacy type into the active
runtime-v2 boundary and executes the accepted five requests. It separately
opens and executes the migrated v2 package. Initial and final state bytes,
every state hash, resolved command row, causal receipt, movement/light fact,
projection delta, and final result meaning are identical. The final normalized
state hash is
`3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc`.

Every changed exact runtime golden follows from the runtime-state schema byte.
The accepted five-command state sequence maps as follows:

| Snapshot | Runtime v1 hash | Runtime v2 hash |
| --- | --- | --- |
| initial | `271ef5e80b4b6143e0cfbf0822f2cf1b0ec82d3fdda3b9ca1929e57a81bd624d` | `fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42` |
| unlock | `4076b938a5d03134810301257022d1124481343fb0d01bf06fa98772350022ae` | `b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b` |
| open | `9594a153dd1a65975d3737e5c7080868c14cd6d370482cee9809ac22fbb3aafb` | `0545806ca38848850f9e9ca674fa4891066ed57b72fa7a957fd66332cae7314b` |
| unseal | `1753f1f199c33add2827e105ce81af5bced8d25d7d53a918d9f797669f4aa49f` | `7843d53de4c8bb543a8ea11603d5ada260b549b18a5d5076e74e0e29b81a6e79` |
| ignite | `84f0488f1f96e75f8601223053cf8ea44444393821feb5a2882b8f5229191272` | `2459a218d9d99015aff8a7c5cee88513f61542eabd63d2871fcd3caf973ed545` |
| extinguish | `e34f62421c5ba8d0385853db5d8c6c694086c8ff0ab5896f94e2e9029b2d21cf` | `3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc` |

The corresponding receipt digests map as follows:

| Command | Runtime v1 receipt SHA-256 | Runtime v2 receipt SHA-256 |
| --- | --- | --- |
| unlock | `f03814a5501a5e63e1464aaf834622dfccff06146c5d70732478708f4b54b48e` | `137aae97ca284b1d086d36b03967df22eccdb8c0787464b56d56eebf1f0b0fad` |
| open | `c8f1bab24339dec8f4be88006a18d81b259cdba61f4fa25f9626ff914740dc55` | `02ae95579f61814b68a1735a08ab85c64a1662e81a52de61a8ca6961941fe538` |
| unseal | `6461972f898ee79984ecbca85f6e41728dcc1a5e993c257b3fde006738e35691` | `7105dd0ffdde41eef147b55115f4b617845db8f11c7ba161f2970bfd919a349f` |
| ignite | `69c03e03653e470e591ca2545d57749f91142a8eb4865394e1105b1f9266d38a` | `5a3d02664dc1ccd3fb4622117c3c3e67e4be1f422c23b586ce13999058e035f4` |
| extinguish after ignite | `d473ef4fbbf5e2d4993002aa55c5efaf3ab289492d0b50d65639106e4a5b91e6` | `0926482aa929518952992fa3734b30fc4a8dff6046c57515b698e7d22961d2e5` |
| extinguish after unseal (SW-F four-command regression) | `c7e9ca01679a6bf14c1c9c92dc78e87e27b89b5d8b32e8b294d595f974554517` | `f41a0505df7937e99622f1e63fe910c4a926d0a571c4702b34d7f749aaba6842` |

The separate SW-F four-command regression's final state maps from
`d9eed238e219747752154bfe8697d79773531df34ba66f96d5e11ee30b29affc`
to `1fa9f01245beae92a25aa96b3b74a4c8e5138ad4fbde545d3d765fc6597c1fde`.
No command, transition, movement, light, or projection meaning changed.

Replay inputs remain package-bound. A replay derived for the migrated manifest
therefore names `42af...` rather than the fresh-compile or legacy digest, while
its normalized command/state/receipt hash chain is unchanged. This distinction
is required package identity, not semantic drift.

## Evidence limits

This slice supports exactly the preserved stable-v1 package boundary and target
v2. It does not migrate construction snapshots, expose a legacy runtime path,
add a package member, promise future save compatibility, implement explanation,
or claim signing, authenticity, hostile-filesystem safety, or concurrent-writer
safety. Decision 0009 resolves the transition-explanation input boundary;
implementation remains in issue #65.
