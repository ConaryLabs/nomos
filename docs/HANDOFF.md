# Handoff — state of the repository

Updated 2026-08-21 at the close of issue #1. This file orients a fresh session;
it is rewritten at each slice boundary and never accumulates history (git has
that).

## Where things stand

- **Contract revision 2** is merged (#2): `THESIS.md`, `KERNEL.md` (Gate K),
  `docs/decisions/0001-contract-repair.md`,
  `docs/evaluation/COLD_AGENT_PROTOCOL.md`.
- **SW-B is merged (#3):** the §10 workspace (six kernel crates + isolated
  `xtask`), toolchain pinned to 1.97.1, lockfile with zero third-party crates,
  `estate-core` (typed stable IDs, canonical bytes, in-crate SHA-256, checked
  arithmetic, structured diagnostics, immutable `WorldPackage`),
  `cargo xtask boundary`, CI green on main. See `docs/workspace.md`.
- **Issue #1 is closed.** Its remaining slices are tracked as their own issues.
- **Issue #4** records four contract underspecifications SW-B resolved in code
  that KERNEL.md should pin through the amendment process.

## What is next

**SW-C — source schema, parser, name resolution, ownership linker.** Defined
by KERNEL.md §1 (the exact base fixture), §2 (compile-time phase), §4, §9
(ownership mutations that must fail closed), and acceptance items 1–3, 11.
`estate-schema` and `estate-compiler` are empty shells waiting for it.

Then, in order: SW-D (namespace machines, interactions, transaction), SW-E
(effective-fact resolution, `MovementDisposition`, projections), SW-F (runtime
state, replay, migration v1→v2, `explain-*`), and the cold-author / cold-debug
evaluations under `docs/evaluation/COLD_AGENT_PROTOCOL.md` — which require a
model family other than the ones that built the kernel.

## How to work here

Read `AGENTS.md`. Run the proof before claiming anything:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

Unproven from §7 and stated as such in CI: Linux aarch64 release, and the
ten-runs-per-target determinism count.

## Open owner points (from #2, standing as proposals until amended)

SHA-256 + the canonical JSON profile; the Linux x86_64/aarch64 matrix; the
cold-agent default budgets and zero-hint rule.
