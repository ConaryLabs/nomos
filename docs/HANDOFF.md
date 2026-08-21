# Handoff — state of the repository

Updated 2026-08-21 for SW-C draft PR #6. This file orients a fresh session; it
is rewritten at each slice boundary and never accumulates history (git has
that).

## Where things stand

- **Contract revision 2** is merged (#2): `THESIS.md`, `KERNEL.md`, decision
  0001, and the cold-agent protocol.
- **SW-B is merged (#3):** six kernel crates plus isolated `xtask`, Rust 1.97.1,
  zero third-party crates, deterministic core primitives, immutable package
  mechanics, boundary enforcement, and green CI on main.
- **SW-C is implemented in draft PR #6:** source-language decision 0002, the
  exact `fixtures/gaol.estate`, source AST and Canonical World IR schemas, typed
  lattice bindings, parser, distinct typed symbol tables, the sealed
  three-primitive expansion catalog, ownership linker/receipts, and mutation
  tests. The implementation commit is `be5576d`; check the PR for its current
  head, CI, review, and merge state.
- **Issue #5 remains open** until owner disposition and merge. Its record states
  that acceptance 3 is only partially covered: IR expansion is proved, but the
  observable `estate inspect` command still requires complete packages and
  projections.
- **Issue #4 remains open** for the four SW-B contract underspecifications.
- **Issue #7 records the cold-agent roster problem:** SW-B is Claude-authored
  and SW-C is GPT-authored, so GPT is no longer eligible as the formal cold
  author for the SW-C authoring interface. A third family is the cleanest
  whole-kernel subject unless the owner predeclares slice-specific eligibility.

## What is next

First, obtain a non-author rerun of PR #6 at its final head. Author proof and CI
do not satisfy the repository's non-author rule. Owner review and merge remain
separate dispositions.

After SW-C, implement **SW-D — namespace-machine transitions, typed
interactions, deterministic phase order, and atomic transaction preparation**.
The schemas now contain machine and claim templates; SW-D must add executable
transition and interaction semantics without moving command-time truth into the
compiler.

Then: SW-E (effective-fact resolution, `MovementDisposition`, projections),
SW-F (runtime state, replay, migration v1→v2, `explain-*`), complete command
surface/package orchestration, determinism matrix, and formal cold-agent gates.

## How to prove the current branch

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

The SW-C author rerun passed all four commands. Still unproven: Linux aarch64
release, ten runs per target, the complete `estate` command surface, complete
package projections, runtime semantics, migration/replay, and cold-agent gates.

## Open owner points

From #4 / decision 0001: SHA-256 plus the canonical JSON profile; Linux
x86_64/aarch64 matrix; cold-agent default budgets and zero-hint rule. From #7:
the eligible cold-family roster after mixed-family implementation authorship.
