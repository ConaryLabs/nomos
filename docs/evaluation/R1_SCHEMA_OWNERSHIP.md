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

No R1 identity has entered the accepted tree yet; the register is empty by
design.

## How a row is added

A row is added in the same change that adds the identity to `crates/*/src`,
naming the owner crate and the exact owner file that declares it, so that
`docs/evaluation/r1-schema-ownership.sh` passes on that change's head and the
identity is owned from the moment it exists.
