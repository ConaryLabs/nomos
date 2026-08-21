# Handoff — state of the repository

Updated 2026-08-21 on the SW-E issue #28 implementation branch.
This file orients a fresh session; it is rewritten at each slice boundary and
never accumulates history (git has that).

## Where things stand

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
- **SW-E is implemented on `feature/sw-e-28`:** construction IR advances to
  `estate.world_ir.construction@3` with explicit movement composition,
  coherence, connectivity, and resolver subjects. Simulation advances to `@2`,
  navigation begins at `@1`, and both receive one byte-identical typed resolver
  plan. `estate-sim` evaluates typed
  claim activation after complete local/causal settlement and exposes immutable
  before/after facts. The exact fixture proves two initial gate blockers, ward
  survival after opening or destruction, base cost `1` after unsealing, and
  water cost `3`. The four-command author proof passes; exact-head non-author
  and CI evidence are pending.

## What is next

Finish **SW-E issue #28**: run the complete author proof, obtain the exact-head
GPT-5.6 Luna max non-author rerun, merge after CI, and record the resulting
evidence.

Do not pull #22 package hardening or #24 typed forensic provenance sideways into
SW-E. Resolve #22 before package/CLI/migration evidence and #24 before stable
World IR promotion or `explain-*`. Issue #17 remains a formal cold-author
tooling blocker, not a blocker for semantic implementation.

After SW-E: SW-F (runtime state, replay, migration v1→v2, `explain-*`), complete
command surface/package orchestration, determinism matrix, and formal cold-agent
gates.

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
`estate` command surface, persistence and diagnostics projections, light
resolution, committed runtime semantics, migration/replay, and formal
cold-agent gates. The contract also requires a
final explicit schema-ownership source-review receipt after the Gate K schema
set stabilizes; that final receipt does not exist yet.

## Remaining evidence points

Linux aarch64 release and the ten-runs-per-target matrix remain evidence gaps,
not contract-wording questions. The whole-kernel roster and invalidation rules
are resolved; the formal runs remain unperformed.
