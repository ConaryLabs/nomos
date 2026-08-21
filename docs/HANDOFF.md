# Handoff — state of the repository

Updated 2026-08-21 for the deferred SW-C non-author rerun. This file orients a
fresh session; it is rewritten at each slice boundary and never accumulates
history (git has that).

## Where things stand

- **Contract revision 2** is merged (#2): `THESIS.md`, `KERNEL.md`, decision
  0001, and the cold-agent protocol.
- **SW-B is merged (#3):** six kernel crates plus isolated `xtask`, Rust 1.97.1,
  zero third-party crates, deterministic core primitives, immutable package
  mechanics, boundary enforcement, and green CI on main.
- **SW-C is implemented by PR #6:** source-language decision 0002, the
  exact `fixtures/gaol.estate`, source AST and Canonical World IR schemas, typed
  lattice bindings, parser, distinct typed symbol tables, the sealed
  three-primitive expansion catalog, ownership linker/receipts, and mutation
  tests. The implementation commit is `be5576d`.
- **Non-author disposition:** Peter explicitly authorized merging PR #6 before
  its non-author rerun. DeepSeek V4 Pro then reran the complete proof through
  direct Reasonix at max effort against merge commit `4ec25e5`; all four
  commands passed with a clean tree before and after. The durable receipt is
  under `docs/evaluation/runs/gate-k/2026-08-21-deepseek-v4-pro-sw-c-rerun/`.
  SW-C now satisfies the repository's non-author rule. This was not a formal
  cold-agent run and does not upgrade whole-Gate-K status.
- **Issue #5 is disposed by PR #6.** Acceptance 3 remains only partially
  covered: IR expansion is proved, but the observable `estate inspect` command
  still requires complete packages and projections.
- **Issue #4 remains open** for the four SW-B contract underspecifications.
- **Issue #7 records the cold-agent roster problem:** SW-B is Claude-authored
  and SW-C is GPT-authored, so GPT is no longer eligible as the formal cold
  author for the SW-C authoring interface. A third family is the cleanest
  whole-kernel subject unless the owner predeclares slice-specific eligibility.

## What is next

Implement **SW-D — namespace-machine transitions, typed
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

The SW-C author and non-author reruns passed all four commands. Still unproven:
Linux aarch64 release, ten runs per target, the complete `estate` command
surface, complete package projections, runtime semantics, migration/replay, and
formal cold-agent gates.

## Open owner points

From #4 / decision 0001: SHA-256 plus the canonical JSON profile; Linux
x86_64/aarch64 matrix; cold-agent default budgets and zero-hint rule. From #7:
the eligible cold-family roster after mixed-family implementation authorship.
