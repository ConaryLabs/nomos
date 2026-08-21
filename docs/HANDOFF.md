# Handoff — state of the repository

Updated 2026-08-21 for owner-authorized contract revision 4. This file orients
a fresh session; it is rewritten at each slice boundary and never accumulates
history (git has that).

## Where things stand

- **Contract revision 3 is owner-authorized:** decision 0003 pins the canonical
  escape profile, ASCII identifier and field-name grammar, isolated `xtask`,
  and the honest evidence boundary for semantic schema ownership.
- **Contract revision 4 is merged (#16):** issue #15 and decision 0004 correct
  the incomplete SW-C linker snapshot from `estate.world_ir@1` to separately
  versioned `estate.world_ir.construction@1`. The merge commit is `2603a4e`;
  the full-byte golden guard, Opus 5 review history, exact-head DeepSeek reruns,
  PR CI, and post-merge CI run `32507602324` are green. Issue #15 is closed.
- **SW-B is merged (#3):** six kernel crates plus isolated `xtask`, Rust 1.97.1,
  zero third-party crates, deterministic core primitives, immutable package
  mechanics, boundary enforcement, and green CI on main.
- **SW-C is implemented by PR #6:** source-language decision 0002, the
  exact `fixtures/gaol.estate`, source AST and Canonical World IR construction
  schemas, typed lattice bindings, parser, distinct typed symbol tables, the
  sealed three-primitive expansion catalog, ownership linker/receipts, and
  mutation tests. The implementation commit is `be5576d`.
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
- **Issue #4 is closed:** revision 3 merged in PR #13.
- **The whole-kernel cold roster is predeclared:** Gemini 3.7 Flash High through
  `agy` is the formal cold author; DeepSeek V4 Pro through direct Reasonix is the
  formal cold debugger; each independently checks the other's output. The plan
  and invalidation rules are in `docs/evaluation/GATE_K_COLD_AGENT_PLAN.md`.
- **The `agy` lane is currently broken (#17):** three print-mode prompts,
  including a `pwd` preflight, were ignored in favor of a canned model greeting.
  Those attempts have zero evidentiary value. Gemini may not perform the formal
  cold-author role until #17 proves a working invocation and preflight.
- **CI uses `actions/checkout@v7` (#11):** PR and post-merge verification passed
  without the Node 20 compatibility annotation.

## What is next

Pause for the owner-requested holistic **GPT Pro architecture review** of the
tree through contract revision 4. Fix or file every resulting finding before
starting the next implementation slice. This review is an architecture
checkpoint, not a formal Gate K cold-author or cold-debug run.

After that disposition, implement **SW-D issue #14 — namespace-machine
transitions, typed interactions, deterministic phase order, and atomic
transaction preparation**. The schemas now contain machine and claim templates;
SW-D must add executable transition and interaction semantics without moving
command-time truth into the compiler.

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

The SW-C and revision-4 author/non-author reruns passed all four commands.
Revision 4 PR and post-merge CI are green. Still unproven: Linux aarch64
release, ten runs per target, the complete `estate` command surface, complete
package projections, runtime semantics, migration/replay, and formal cold-agent
gates. The contract also requires a final explicit schema-ownership
source-review receipt after the Gate K schema set stabilizes; that final receipt
does not exist yet.

## Remaining evidence points

Linux aarch64 release and the ten-runs-per-target matrix remain evidence gaps,
not contract-wording questions. The whole-kernel roster and invalidation rules
are resolved; the formal runs remain unperformed.
