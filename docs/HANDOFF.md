# Handoff — state of the repository

Updated 2026-08-22 for the issue #49 Pi provider-harness qualification. No next
semantic implementation slice is currently filed.
This file orients a fresh session; it is rewritten at each slice boundary and
never accumulates history (git has that).

## Where things stand

- **Nomos is the active project identity:** contract revision 6
  and decision 0007 rename the runtime/project to Nomos while retaining The
  Signed World as the thesis name. Active crates use `nomos-*`, the binary is
  `nomos`, authoring source uses `.nomos`, schemas use `nomos.*`, and the fresh
  construction epoch began at `nomos.world_ir.construction@1` and its active
  light-aware shape is `nomos.world_ir.construction@3`. References to
  `signed-world`, `estate-*`, `.estate`, and `estate.*` below describe immutable
  prototype-era history unless explicitly called current.
  The mechanical implementation is commit `7c0ca31`; its full author proof,
  old-to-new golden relationship, and classified legacy-name audit are recorded
  under `docs/evaluation/runs/identity/2026-08-21-nomos-cutover/`. GPT-5.6 Luna
  max reran exact PR head `ec4e270` green; PR CI run `32526065565` passed. PR
  #32 merged as `6b803e1`, issue #31 closed, the repository and description were
  renamed, and post-merge CI run `32526851179` passed under
  `https://github.com/ConaryLabs/nomos`.

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
  covered: IR expansion is proved, but the active observable `nomos inspect`
  command still requires complete packages and projections.
- **Issue #4 is closed:** revision 3 merged in PR #13.
- **The whole-kernel cold roster is predeclared:** Gemini 3.7 Flash High is the
  formal cold author; DeepSeek V4 Pro is the formal cold debugger; each
  independently checks the other's output. Issue #49 changes their transport
  to Pi without changing either family or role. The plan and invalidation rules
  are in `docs/evaluation/GATE_K_COLD_AGENT_PLAN.md`.
- **The `agy` print path is repaired (#17):** official client 1.1.17 was
  restored after the host reinstall. The old argument order passed `--model` as
  the seven-byte print prompt; a prompt-first invocation now proves the exact
  Gemini 3.7 Flash High model and a completed `pwd` tool event in the target
  worktree. The fail-closed harness and exact receipt are under
  `docs/evaluation/`. The three 2026-08-21 attempts retain zero evidentiary
  value.
- **The `agy` formal route is ineligible on 1.1.17 (#45):** a fresh project and
  conversation with sandboxing, slash commands disabled, and a custom
  three-tool main agent still reported 57 tools and no machine-readable project,
  context-source, or memory disclosure. The fail-closed guard therefore exits
  `1` with `AGY_FORMAL_BOUNDARY BLOCKED`. The exact receipt is under
  `docs/evaluation/runs/tooling/2026-08-22-agy-formal-boundary-falsification/`.
  No Gemini formal attempt may launch until a future client/route passes that
  guard or the owner approves a new plan. This is tooling falsification, not a
  formal run, roster substitution, or protocol change.
- **Pi qualifies all three non-Codex lanes (#49):** Pi 0.84.2 now drives
  `antigravity/gemini-3.7-flash` at high,
  `deepseek/deepseek-v4-pro` at max, and supplemental
  `anthropic/claude-opus-5` at max through one common fail-closed launcher.
  The Gemini lane explicitly loads the separately pinned `pi-antigravity`
  0.4.0 provider; package discovery remains off. All three authenticated
  author probes passed with fresh ephemeral sessions, the exact provider/model,
  only the repository-owned isolated `bash`, no model-requested tool call, and
  no credential leakage. Bubblewrap exposes only the target checkout as
  read-write, clears the child environment, and unshares the network. The
  offline matrix rejects every issue #49 negative case. Sanitized receipts are
  under
  `docs/evaluation/runs/tooling/2026-08-22-pi-provider-qualification/`.
  Exact-head non-author reruns are required before the slice is green.
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
- **Typed forensic provenance is merged (#24/#34):** construction IR advances
  to `nomos.world_ir.construction@2`; its exact fixture golden is
  `1a977d4f5f5bcbb11e1ae701e6cc1f3d06688707ee98c04a143d10454aeb126a`,
  while Nomos construction-v1 remains preserved as historical evidence. Fact
  identity, resolved values, projection consumers, derivation producers/passes,
  and causal inputs are typed. The IR rejects dangling roots and edges, values
  that contradict actual entity records, unknown or fact-incompatible passes,
  non-canonical owners, empty/duplicate derivations, and missing typed inputs.
  Canonical ordering no longer depends on human-readable `Display` output.
  Luna max rejected exact heads `9061f68` and `1426451` with real semantic
  findings; after repair it reviewed and reran final head `7ea49d6` green on
  Rust 1.98.0. PR CI run `32530844878` passed, PR #34 merged as `1f04fce`, issue
  #24 closed, and post-merge CI run `32531098740` passed. Structured canonical
  data and readable explanations are separate outputs; the actual
  `nomos explain-*` CLI remains later work.
- **SW-F is merged (#35/#38):** construction IR advances to
  `nomos.world_ir.construction@3` and simulation to
  `nomos.projection.simulation@3`; persistence and diagnostics begin at `@1`.
  The three light consumers receive one byte-identical compiler-owned union
  plan. Runtime commands resolve light after complete local/causal settlement,
  commit a new immutable `nomos.runtime_state@1` snapshot, hash its exact
  canonical envelope with SHA-256, and emit typed `nomos.causal_receipt@1`
  evidence. Rejections emit no commit evidence. GPT-5.6 Luna max found no
  semantic or test defects and reran exact head `e1b20845` green with a clean
  tree before and after. PR CI run `32533855605` passed; PR #38 merged as
  `b5ea3b2`, issue #35 closed, and post-merge CI run `32535564970` passed.
- **SW-G is merged (#40/#42):** stable `nomos.world_ir@1` is a distinct
  authoritative artifact promoted from preserved construction evidence. All
  four public projection compilers consume stable IR. The compiler assembles
  the exact seven semantic members, `nomos-core` alone emits `manifest.json`,
  and the semantic opener verifies schemas, ownership, compiler receipts,
  initialization material, and cross-projection agreement. Reopened package
  bytes reproduce the exact initial `nomos.runtime_state@1` snapshot without
  source or construction IR. Stable-IR fixture digest
  `555017cf5e13a33b4bb5b18bae14b7577fd1fc38abf89b1f6f475874600fa493`
  and complete-package manifest digest
  `f1af0cc92ea44fd09ba93815bb99cc6c24517b56888f39be33a9d47b1299bab7`
  are frozen. The implementation head is `f200b4e`; its author proof and
  focused release SW-G suite passed. GPT-5.6 Luna max independently audited and
  reran the exact head with no findings and a clean tree before and after. PR
  CI run `32537565881` passed; PR #42 merged as `0863750`, issue #40 closed,
  and post-merge CI run `32537995524` passed.

## What is next

No next semantic implementation slice is currently filed; do not infer one
from this handoff. The remaining post-SW-G work includes the filesystem
CLI surface, run-directory artifacts, replay, the required stable v1-to-v2
movement migration, direct v2 runtime loading/refusal evidence, the Linux
aarch64 and ten-run matrix, measured Gate K budgets, final schema-ownership
review, and the formal cold-agent gates.

Issue #47 is the next scoped maintenance item: the contractual cold-agent
protocol still names the prototype-era `estate` CLI in its active tool policy.
It requires an owner-authorized protocol revision that changes only that
identity to `nomos`; issue #45 deliberately does not amend it.

The old `agy` Gemini route remains falsified evidence. Issue #49 qualifies Pi as
the replacement transport without spending a formal attempt or changing the
roster. Neither the historical `agy` result nor the Pi qualification probe is
formal Gate K evidence.

## How to prove the current branch

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
docs/evaluation/test-agy-print-preflight.sh
docs/evaluation/test-agy-formal-boundary-preflight.sh
docs/evaluation/test-pi-cold-agent-preflight.sh
```

The authenticated formal-boundary command is an expected negative proof on
1.1.17: `docs/evaluation/agy-formal-boundary-preflight.sh` must exit `1` and
print `AGY_FORMAL_BOUNDARY BLOCKED`. A zero exit would require inspection and
owner disposition before any formal launch.

SW-G's author proof, Luna max exact-head rerun, PR CI, and post-merge CI passed
all four commands; the focused release SW-G test also passed. SW-F, SW-D, and
issue #24 have the same author/non-author/CI chain as recorded above. Earlier
SW-C, revision-4, and Rust-1.98 maintenance reruns also passed.
Still unproven: Linux aarch64 release, ten runs per target, the complete
`nomos` command surface, migration/replay, measured Gate K budgets, and formal
cold-agent gates. The contract also requires a final explicit schema-ownership
source-review receipt after the Gate K schema set stabilizes; that final
receipt does not exist yet.

## Remaining evidence points

Linux aarch64 release and the ten-runs-per-target matrix remain evidence gaps,
not contract-wording questions. The whole-kernel roster and invalidation rules
are resolved; the formal runs remain unperformed.
