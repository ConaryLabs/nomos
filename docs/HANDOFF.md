# Handoff — state of the repository

Updated 2026-08-21 while the owner-authorized Nomos identity cutover (#31) is
being implemented after package-boundary PR #30 merged.
This file orients a fresh session; it is rewritten at each slice boundary and
never accumulates history (git has that).

## Where things stand

- **Nomos is the active project identity on this branch:** contract revision 6
  and decision 0007 rename the runtime/project to Nomos while retaining The
  Signed World as the thesis name. Active crates use `nomos-*`, the binary is
  `nomos`, authoring source uses `.nomos`, schemas use `nomos.*`, and the fresh
  construction epoch begins at `nomos.world_ir.construction@1`. References to
  `signed-world`, `estate-*`, `.estate`, and `estate.*` below describe immutable
  prototype-era history unless explicitly called current.

- **Contract revision 3 is owner-authorized:** decision 0003 pins the canonical
  escape profile, ASCII identifier and field-name grammar, isolated `xtask`,
  and the honest evidence boundary for semantic schema ownership.
- **Contract revision 4 is merged (#16):** issue #15 and decision 0004 correct
  the incomplete SW-C linker snapshot from `estate.world_ir@1` to separately
  versioned `estate.world_ir.construction@1`. The merge commit is `2603a4e`;
  the full-byte golden guard, Opus 5 review history, exact-head DeepSeek reruns,
  PR CI, and post-merge CI run `32507602324` are green. Local receipts live
  under `docs/evaluation/runs/contract/` and `docs/evaluation/runs/ci/`. Issue
  #15 is closed.
- **Rust 1.98.0 is current (#19/#20):** PR #20 advanced the live toolchain pin
  and workspace MSRV, passed author proof and CI, then received a GPT-5.6 Luna
  max non-author rerun against merge commit `feacad0`. The original SW-B
  receipts correctly remain on Rust 1.97.1. The workspace still has seven local
  lockfile entries and no third-party crates.
- **The dependency policy is explicit (#23):** owner-authorized decision 0005
  keeps Gate K at zero third-party dependencies for offline reproducibility and
  a small audit surface. It is temporary through Gate K, not inherited law for
  later renderer, signing, network, asset, or platform work.
- **SW-B is merged (#3):** six kernel crates plus isolated `xtask`, deterministic
  core primitives, package foundations, boundary enforcement, and green CI on
  main. Its original evidence used Rust 1.97.1.
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
  covered: IR expansion is proved, but the active observable `nomos inspect` command
  still requires complete packages and projections.
- **Issue #4 is closed:** revision 3 merged in PR #13.
- **The whole-kernel cold roster is predeclared:** Gemini 3.7 Flash High through
  `agy` is the formal cold author; DeepSeek V4 Pro through direct Reasonix is the
  formal cold debugger; each independently checks the other's output. The plan
  and invalidation rules are in `docs/evaluation/GATE_K_COLD_AGENT_PLAN.md`.
- **The `agy` lane is currently broken (#17):** three print-mode prompts,
  including a `pwd` preflight, were ignored in favor of a canned model greeting.
  Those attempts have zero evidentiary value. Gemini may not perform the formal
  cold-author role until #17 proves a working invocation and preflight. The
  local failure record is under `docs/evaluation/runs/tooling/`.
- **CI uses `actions/checkout@v7` (#11):** PR and post-merge verification passed
  without the Node 20 compatibility annotation.
- **The GPT Pro architecture checkpoint is owner-disposed (#25):** review of
  clean `main` at `feacad0` found the project on target, endorsed SW-D's scope,
  and filed #21–#24. This was architecture fuzzing, not a formal Gate K run.
- **SW-D is merged (#14/#27):** construction IR advances to
  `estate.world_ir.construction@2` with typed transitions and one phased causal
  edge. The compiler emits `estate.projection.simulation@1`, rejects invalid
  references and cycles, and no longer claims unimplemented projection
  artifacts. `estate-sim` initializes projected machines and atomically prepares
  local-then-causal state changes without seeing source or IR. GPT-5.6 Luna max
  reviewed and reran exact PR head `ce90aa5`; PR CI run `32517611393` and
  post-merge run `32518205450` passed. Merge commit `5f5e730` is clean on main.
- **Issue #21 closed in SW-D's first isolated commit (`a3be521`):** canonical
  object fields, stable keyed arrays, package members/manifest rows,
  machine/claim identities, transitions, and interactions fail closed instead
  of retaining a final duplicate.
- **SW-E is merged (#28/#29):** construction IR advances to
  `estate.world_ir.construction@3` with explicit movement composition,
  coherence, connectivity, and resolver subjects. Simulation advances to `@2`,
  navigation begins at `@1`, and both receive one byte-identical typed resolver
  plan. `estate-sim` evaluates typed
  claim activation after complete local/causal settlement and exposes immutable
  before/after facts. The exact fixture proves two initial gate blockers, ward
  survival after opening or destruction, base cost `1` after unsealing, and
  water cost `3`. GPT-5.6 Luna max found and verified fixes for stale projection
  documentation and invalid-connectivity validation, then reran exact head
  `6dda1d0` green. PR #29 and post-merge CI run `32521857686` passed; merge
  commit `dacfaef` is clean on main. Issue #28 is closed.
- **Contract revision 5 is merged (#22/#30):** decision 0006 replaces the
  unmanifested package `receipts/` subtree with canonical hashed
  `compiler-receipts.json`; runtime causal receipts remain only in run outputs.
  It also defines same-filesystem staged publication and exact filesystem and
  manifest verification. The first GPT-5.6 Luna max pass on
  `c70b5bf` found a trailing-separator root-symlink bypass and an unstated
  path-based reader race boundary. Both are repaired: roots are lexically
  normalized before entry checks, the regression test covers both spellings,
  and revision 5 now explicitly requires a caller-owned quiescent package tree.
  The replacement Luna rerun and PR CI passed exact head `5f65978`; post-merge
  CI run `32525028043` passed merge commit `0eb50b7`. Issue #22 is closed.

## What is next

Finish **issue #31 — adopt Nomos as the project identity** under decision 0007.
Complete the active-name audit, freeze the fresh Nomos golden evidence, obtain
the exact-head Luna max rerun, merge the identity slice, rename the GitHub
repository, and verify post-merge CI under the new repository identity.

Do not pull #24 typed forensic provenance into this package-boundary repair.
Resolve #24 before stable World IR promotion or `explain-*`. Issue #17 remains
a formal cold-author tooling blocker, not a blocker for implementation.

After #31: resolve #24 at its stated boundary, then continue SW-F runtime state,
replay, migration v1→v2, package/command orchestration, explanations, the
  determinism matrix, and formal cold-agent gates under Nomos names.

## How to prove the current branch

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

SW-D's author proof, Luna max non-author rerun, PR CI, and post-merge CI passed
all four commands plus the x86_64 debug/release determinism comparison. Earlier
SW-C, revision-4, and Rust-1.98 maintenance author/non-author reruns also passed.
Still unproven: Linux aarch64 release, ten runs per target, the complete
`nomos` command surface, persistence and diagnostics projections, light
resolution, committed runtime semantics, migration/replay, and formal
cold-agent gates. The contract also requires a
final explicit schema-ownership source-review receipt after the Gate K schema
set stabilizes; that final receipt does not exist yet.

## Remaining evidence points

Linux aarch64 release and the ten-runs-per-target matrix remain evidence gaps,
not contract-wording questions. The whole-kernel roster and invalidation rules
are resolved; the formal runs remain unperformed.
