# Nomos handoff

Status snapshot: 2026-08-27, at the R2-2 implementation stop line. This file is
an operational map; owner decisions and revisioned contracts remain the
authority when prose conflicts.

**Play the accepted six-area viewer:** <https://conarylabs.github.io/nomos/>

## Stop line

R1 is complete, passing, accepted, and closed as this repository's runtime
baseline. Decision 0019 expressly does **not** adopt Nomos into a game. Decision
0022's bounded Mortal Estate evidence then admitted one immutable R1 dependency
point, recorded one representative adopter frame, and reduced the observed gap
to an adopter-neutral fixture classified `reusable missing Nomos capability`.

Decision 0023 opens one narrow R2 observed-scene presentation epoch. The
separately reviewed root `R2.md` revision 1 is owner-authorized. R2-1's strict
carrier and compiler landed through PR #196 at
`cc47a7235f92d0ed460c7db5d178448b12fdba02`, tree
`2bce614d2df94464c20042cdf059a7b22ec39c09`. R2-2's isolated offline consumer
and independent second scene land through PR #198. Those implementation
targets do not themselves admit R2. The next target is a new, separately
falsifiable final-evidence-and-disposition issue; no platform choice or
game-adoption claim is authorized.

A fresh agent must not infer active work from an old remote branch. The
repository intentionally retains evidence branches and annotated tags. Check
owner decisions, then open issues and pull requests.

The exact last-main receipts at decision 0023's input baseline are:

| Workflow | Run | Result |
| --- | ---: | --- |
| `verify` | [33052279259](https://github.com/ConaryLabs/nomos/actions/runs/33052279259) | success |
| `gate-k-evidence` | [33052279261](https://github.com/ConaryLabs/nomos/actions/runs/33052279261) | success |
| `nomos viewer` | [33052279276](https://github.com/ConaryLabs/nomos/actions/runs/33052279276) | success |
| `executable gaol pages` | [32922996019](https://github.com/ConaryLabs/nomos/actions/runs/32922996019) | last applicable run succeeded at `22506c6` |

The final completed sequence before this handoff was PR #180, which closed #134
and #141 by pinning load-bearing evaluation ordering to `LC_ALL=C`; PR #181,
which closed #160 with bounded and receipt-recorded browser shutdown; and PR
#185, which repaired `RUNTIME.md`'s stale revision-1 footer and established
revision 4 without changing a criterion or R1's revision-3 acceptance evidence.
PR #183 then refreshed the reinstall handoff at the exact decision 0022 input
baseline. Each implementation or contract slice had its required exact-head
non-author rerun, and every applicable workflow was green at its bound
baseline.

Decision 0022 then landed through PR #187. Issue #188 admitted its exact
dependency point after a Luna max rerun. TME issue #5 and PR #6 recorded and
independently reproduced the representative observer frame. Nomos issue #189
and PR #190 reduced the gap to a quarantined generic fixture, received a Luna
max cold attack, and closed on the owner's exact classification `reusable
missing Nomos capability`. Decision 0023 is the resulting epoch boundary.
R2-1 then landed through PR #196. R2-2's packet-frozen second scene was authored
by an independent Luna Max agent without repository or adopter access; its
source, compiled plan, signatures, browser receipt, and pixels entered unchanged.

## Authoritative state

| Line | State | Authority |
| --- | --- | --- |
| Gate K round one | failed; criteria 17 and 18 failed | decision 0013 |
| Gate K round two | terminated incomplete; no verdict | decision 0016 |
| R1 | all five criteria passed; accepted and closed | decision 0019 |
| R1 contract | revision 4 in force | `RUNTIME.md`, decision 0021 |
| Game adoption | not authorized; thesis applies to no game | decision 0019 |
| Mortal Estate presentation evidence | bounded prerequisite evidence complete; no adoption | decisions 0022 and 0023 evidence |
| R2 observed-scene presentation epoch | R2-1 and R2-2 implementation complete; epoch not yet admitted | `R2.md`, PRs #196 and #198 |
| Current queue | after PR #198, a new R2 final-evidence-and-disposition issue is next | GitHub issues and PRs, `R2.md` section 11 |

The accepted R1 surface consists of:

- the six dependency-free Gate K kernel crates and their SW-N semantic surface;
- the read-only effective-facts and entity-catalog projections;
- `nomos-render-plan`, including typed presentation source and the compiled
  area collection;
- `nomos-play`, including authoritative actors, commands, movement, pursuit,
  receipts, session replay, and the wasm runtime;
- `apps/nomos-viewer`, with vendored Three.js, a scanned offline public
  artifact, native/browser session identity, and bounded browser shutdown.

That accepted path is proved against six independently authored areas from the
quarantined study: Cistern Walk, Drowned Stair, Ember Vault, Gloam Bastion,
North Gaol, and Ossuary Reach. The study remains non-authoritative; it is a
specification and comparison target, not accepted source.

The public viewer is evidence for this repository runtime, not a production-art
claim. Audio, networking, replication, combat, production scaling, an adopting
game's Gate 0 target pack, and that game's Gate 1 proof remain absent.

The unadmitted R2 implementation adds one dependency-isolated
`nomos-observed-scene` crate and `apps/nomos-observed-viewer`. It proves a
finite observed-scene carrier through two independently different scenes and a
render-only isometric browser boundary. It is evidence awaiting the R2 final
disposition, not an extension of the accepted R1 play runtime.

## Read in this order

1. `README.md` — short project map and status.
2. This file — operational state, setup, and stop line.
3. `docs/decisions/0019-r1-final-disposition.md` — final R1 verdict and exact
   non-claims.
4. `docs/decisions/0022-mortal-estate-presentation-adoption-evidence.md` — the
   bounded evidence authority, upstream-admission rule, and stop line.
5. `docs/decisions/0023-observed-scene-presentation-epoch.md` — the narrow R2
   authority, first-target order, adopter boundary, and stop line.
6. `THESIS.md` — exploratory design thesis; not authority for another project.
7. `KERNEL.md` — frozen Gate K revision 7 contract and historical failed bar.
8. `RUNTIME.md` — accepted R1 revision 4 contract.
9. `docs/decisions/0021-runtime-revision-4.md` — revision 4's exact
   lifecycle-history repair.
10. `docs/decisions/0020-runtime-revision-3.md` — revision 3's exact
   comparison-count repair.
11. `docs/decisions/0013-gate-k-disposition.md` and
   `docs/decisions/0016-terminate-gate-k-round-two.md` — historical Gate K
   verdicts.
12. `docs/workspace.md` — crate graph and boundary proof.

Read design records under `docs/review/` only for the subsystem being changed.
The large `docs/evaluation/runs/` tree is immutable historical evidence, not a
cache and not active work.

## Fresh Linux box

The repository has no package-manager bootstrap script and does not need
`npm install`. Rust dependencies are workspace-local; Three.js is vendored with
its license and digest.

Install these host tools before expecting the complete proof to run:

- Git and Bash;
- rustup (the checked-in `rust-toolchain.toml` selects Rust 1.98.0, `rustfmt`,
  `clippy`, and `wasm32-unknown-unknown`);
- Node 22 or newer, because the smoke client uses Node's global `WebSocket`;
- Google Chrome, Chromium, or a compatible headless-shell binary; and
- common GNU userland used by the proof scripts, including `jq`, `sha256sum`,
  `find`, `sort`, `diff`, `cmp`, `sed`, `grep`, `stat`, and `timeout`.

CI uses Ubuntu 24.04, Node 22, the pinned Rust toolchain, and Google Chrome.
GitHub CLI is useful for repository state but is not needed to build.

From a new checkout:

```bash
git clone https://github.com/ConaryLabs/nomos.git
cd nomos
git fetch --tags origin
rustup show
rustc --version
cargo --version
node --version
```

`rustup show` provisions the pinned components and wasm target on a connected
machine. After toolchain provisioning, the workspace and accepted artifact can
be proved offline; the recorded network-isolated receipt is
`docs/evaluation/r1-adoption-evidence.md`.

Find a browser, or set it explicitly:

```bash
command -v google-chrome || command -v chromium || command -v chromium-browser
export CHROME_BIN=/absolute/path/to/chrome-or-chrome-headless-shell
"$CHROME_BIN" --version
```

Do not copy a machine-specific Chrome path into the repository. On a minimal
Linux install a headless-shell binary can work when a full Chromium build lacks
desktop libraries.

## Verification order

Run commands from the repository root. The fast accepted-workspace proof is:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

The six-area study and promoted viewer proof must generate artifacts before the
browser consumes them:

```bash
experiments/executable-gaol/gaol verify
crates/nomos-play/build-wasm.sh
cargo build --locked -p nomos-play
node apps/nomos-viewer/build.mjs \
  --from target/executable-gaol \
  --wasm target/wasm32-unknown-unknown/wasm/nomos_play.wasm \
  --out apps/nomos-viewer/dist \
  --receipt target/nomos-viewer-build/receipt.json
node --test apps/nomos-viewer/test/*.test.mjs
node apps/nomos-viewer/smoke/smoke.mjs \
  --dist apps/nomos-viewer/dist \
  --out target/nomos-viewer-smoke \
  --require-chrome
```

The smoke lane must end with six areas, 65 moves, traversal cost 95, zero
external requests, and native replay agreement. Its receipt records bounded
CDP, Chrome-process-group, and HTTP-server shutdown. PR #181 independently ran
the full browser proof ten consecutive times; every process closed within 9 ms
of the reviewer observing PASS, against a 2-second acceptance limit.

The R2-2 local proof is:

```bash
cargo build --release --locked -p nomos-observed-scene
docs/evaluation/r2-second-scene-packet.test.sh
docs/evaluation/r2-schema-ownership.sh
docs/evaluation/r2-source-provenance.sh
docs/evaluation/r2-source-provenance.test.sh
docs/evaluation/r2-adopter-neutrality.sh
docs/evaluation/r2-adopter-neutrality.test.sh
node docs/evaluation/r2-maximum.test.mjs
node docs/evaluation/r2-scene-signature.mjs \
  fixtures/r2/scenes/scene_one.json fixtures/r2/scenes/scene_two.json
node --test apps/nomos-observed-viewer/test/*.test.mjs \
  docs/evaluation/r2-scene-signature.test.mjs
node apps/nomos-observed-viewer/build.mjs \
  --plan fixtures/r2/plans/scene_one.json \
  --plan fixtures/r2/plans/scene_two.json \
  --out target/r2-observed-dist \
  --receipt target/r2-observed-build.json
CHROME_BIN=/absolute/path/to/chrome \
  node apps/nomos-observed-viewer/smoke/smoke.mjs \
    --dist target/r2-observed-dist \
    --out target/r2-observed-smoke \
    --samples 10
```

The two committed plans must also reproduce byte-for-byte from their canonical
scene inputs. R2-2 acceptance requires 10 fresh browser profiles per scene,
zero external requests, per-scene and combined p95 at most 5 seconds, process
closure within 2 seconds, distribution size at most 2,000,000 bytes, and an
exact-head Luna Max rerun. This is not the later complete network-isolated R2
disposition proof.

The formal archived Gate K harnesses are historical and materially heavier.
Do not launch a new cold-agent attempt, checker, retry, or evidence assembly:
decision 0016 authorizes none.

## Operational gotchas

- Give every worktree its own fresh `target/`. Some evaluation and viewer
  commands intentionally refer to the literal worktree-local `target/` path;
  sharing a Cargo target across concurrent worktrees can mix candidates.
- If an external `CARGO_TARGET_DIR` is necessary, also audit commands that use
  `target/debug/nomos`, `target/release/nomos-play`, viewer wasm paths, or smoke
  defaults. Prefer the worktree-local default for the complete proof.
- The Pages workflow runs only after a push to `main`. A green PR proves the
  viewer lane, not the final Pages deployment; verify the main run after merge.
- The smoke lane skips without Chrome unless `--require-chrome` is present.
  Acceptance and CI use `--require-chrome`.
- `apps/nomos-viewer/dist`, `target/executable-gaol`, the wasm module, and the
  native `nomos-play` binary must describe the same checkout. Rebuild them in
  the order above after switching commits.
- Compiled worlds and evidence are immutable inputs. Tests and runtime commands
  write new output; they do not repair a package in place.
- `docs/evaluation/runs/` and the `gate-k-*` tags are historical evidence.
  Never prune them as reinstall cleanup.
- Byte-sensitive evaluation scripts pin `LC_ALL=C`. Preserve that pin when
  adding ordering or tree-digest logic.

## Establish current work after reinstall

After cloning and before making a branch, run:

```bash
git status --short --branch
git log -5 --oneline --decorate
git tag -l 'gate-k-*'
gh issue list --state open
gh pr list --state open
gh run list --branch main --limit 8
```

Expected steady state after this handoff merges: clean `main` matching
`origin/main`, zero open issues, zero open PRs, and successful applicable main
workflows. Pages retains its last applicable successful run when its path
filter does not select the decision change. Old remote heads may still exist;
they do not override that state.

## What can happen next

Ordinary bug maintenance against the accepted R1 baseline may start from a new
issue with falsifiable acceptance.

Decision 0023 authorizes the R2 epoch boundary and nothing past its stated
order. `R2.md` revision 1 defines exact acceptance, finite input and output
grammars, ownership, workspace boundaries, budgets, and proof. R2-1 and R2-2
are complete after PR #198. The next slice is R2 final evidence and disposition,
starting from a new falsifiable issue and stopping for the owner's explicit
`accept`, `repair and rerun`, or `stop` verdict.

A deeper adopter boundary, platform choice, Gate K attempt, R2 scope expansion,
or adoption into a game requires its own owner decision. Work toward an actual
game remains exploratory until the game satisfies its own gates and accepts
measured cost.
