---
title: R1 canonical-schema ownership register
status: R1 register; revision 1
date: 2026-08-25
issue: 133
authority: RUNTIME.md §3
---

# R1 canonical-schema ownership register

This is the live canonical-schema register for the R1 epoch. It records every
persisted or contractual schema identity declared in `crates/*/src` **after**
the Gate K freeze commit `eb86f25f5084a5da83cdd4f26e42e68089367a11`.

The twenty Gate K identities remain owned exactly as recorded in
`docs/evaluation/SCHEMA_OWNERSHIP.md` at `eb86f25`, and are not repeated here.
That receipt is final historical evidence at its freeze commit; this register is
the additive R1 continuation of it. `docs/evaluation/r1-schema-ownership.sh`
re-asserts the twenty Gate K identities and their owner-source assertions on
every run, so an identity is owned if and only if it appears in exactly one of
the two documents.

Authority is `RUNTIME.md` §3, under which kernel crates may gain read-only R1
surface — so a new identity may be declared inside a kernel crate — provided no
Gate K command, artifact, hash, or diagnostic changes. Schema ownership stays
exact: one canonical identity, one owner crate, one owner file.

## Inventory

Columns are those of the Gate K receipt, plus an explicit **Owner file** column
so that `r1-schema-ownership.sh` can match a declaration site mechanically
rather than by prose. The identity and the owner file are read from this table
verbatim; both are wrapped in backticks and the owner file is repository-
relative.

| Canonical identity | Owner | Owner file | Authoritative type set | Encoder | Strict reader / verifier | Persisted boundary | Primary consumers | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `nomos.effective_facts@1` | `nomos-sim` | `crates/nomos-sim/src/effective_facts.rs` | the `effective_facts()` document, embedding the `ResolvedMovementFacts` and `ResolvedLightFacts` renderings it composes | `nomos_sim::effective_facts` composes the `CanonicalValue`; canonical entity-sorted bytes written to stdout by `nomos effective-facts` | none in-tree: derived read-only output, never re-read by the kernel and with no strict package reader; its first consumer binds identity and version | none: stdout only, never written into a run bundle or package, and outside the state-hash domain because it is derived | R1-2 Rust rendering-plan compilation; today `experiments/executable-gaol/compare-effective-facts.sh` | active R1-1 |

One R1 identity has entered the accepted tree. `nomos.effective_facts@1` is the
read-only effective-fact projection accepted as R1-1 under `RUNTIME.md` §5
(issue #126, PR #130): given a strictly verified world package and a runtime
state it composes, for every resolver subject, the effective movement
disposition, cost, ordered reason claim IDs, and effective light. It resolves
nothing itself — `nomos_sim::resolve_movement` and `nomos_sim::resolve_light`
do that — and it is derived output, so it is persisted nowhere and enters no
hash domain.

## How a row is added

A row is added in the same change that adds the identity to `crates/*/src`,
naming the owner crate and the exact owner file that declares it, so that
`docs/evaluation/r1-schema-ownership.sh` passes on that change's head and the
identity is owned from the moment it exists.
