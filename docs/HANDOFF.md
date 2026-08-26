# Nomos handoff

Status snapshot: 2026-08-26, at the decision 0022 authorization slice. This file
is an operational map; owner decisions and revisioned contracts remain the
authority when prose conflicts.

**Play the accepted six-area viewer:** <https://conarylabs.github.io/nomos/>

## Stop line

R1 is complete, passing, accepted, and closed as this repository's runtime
baseline. Decision 0019 expressly does **not** adopt Nomos into a game. No later
runtime epoch, capability family, Gate K retry, or game adoption is authorized.
Decision 0022 is the narrow exception: it authorizes bounded Mortal Estate
presentation-adoption evidence collection, not accepted Nomos implementation,
R2, a platform, or adoption.

There is no unfinished implementation slice to resume. At baseline commit
`22506c6b808e704153dcd8ff340fc1086226c804`, GitHub had zero open issues and
zero open pull requests, and all four main workflows were green. Issue #186 and
its decision PR are the expected temporary exception while decision 0022 lands.
After that PR closes, later evidence work must appear as separately falsifiable
issues under decision 0022; a new agent must not infer an implementation slice
from the authorization alone.

A fresh agent must not infer active work from an old remote branch. The
repository intentionally retains evidence branches and annotated tags. Check
owner decisions, then open issues and pull requests.

The exact last-main receipts at that baseline are:

| Workflow | Run | Result |
| --- | ---: | --- |
| `verify` | [32922996009](https://github.com/ConaryLabs/nomos/actions/runs/32922996009) | success |
| `gate-k-evidence` | [32922996010](https://github.com/ConaryLabs/nomos/actions/runs/32922996010) | success |
| `nomos viewer` | [32922996033](https://github.com/ConaryLabs/nomos/actions/runs/32922996033) | success |
| `executable gaol pages` | [32922996019](https://github.com/ConaryLabs/nomos/actions/runs/32922996019) | success |

The final completed sequence before this handoff was PR #180, which closed #134
and #141 by pinning load-bearing evaluation ordering to `LC_ALL=C`; PR #181,
which closed #160 with bounded and receipt-recorded browser shutdown; and PR
#185, which repaired `RUNTIME.md`'s stale revision-1 footer and established
revision 4 without changing a criterion or R1's revision-3 acceptance evidence.
PR #183 then refreshed the reinstall handoff at the exact decision 0022 input
baseline. Each implementation or contract slice had its required exact-head
non-author rerun, and all four workflows were green at the baseline above.

## Authoritative state

| Line | State | Authority |
| --- | --- | --- |
| Gate K round one | failed; criteria 17 and 18 failed | decision 0013 |
| Gate K round two | terminated incomplete; no verdict | decision 0016 |
| R1 | all five criteria passed; accepted and closed | decision 0019 |
| R1 contract | revision 4 in force | `RUNTIME.md`, decision 0021 |
| Game adoption | not authorized; thesis applies to no game | decision 0019 |
| Mortal Estate presentation evidence | bounded collection authorized; no accepted implementation or adoption | decision 0022 |
| Later epoch or new capability family | not authorized | decision 0019 consequence 2 |
| Current queue | decision issue #186 while this slice lands; later work requires new issues | GitHub issues and PRs, decision 0022 |

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

## Read in this order

1. `README.md` — short project map and status.
2. This file — operational state, setup, and stop line.
3. `docs/decisions/0019-r1-final-disposition.md` — final R1 verdict and exact
   non-claims.
4. `docs/decisions/0022-mortal-estate-presentation-adoption-evidence.md` — the
   bounded evidence authority, upstream-admission rule, and stop line.
5. `THESIS.md` — exploratory design thesis; not authority for another project.
6. `KERNEL.md` — frozen Gate K revision 7 contract and historical failed bar.
7. `RUNTIME.md` — accepted R1 revision 4 contract.
8. `docs/decisions/0021-runtime-revision-4.md` — revision 4's exact
   lifecycle-history repair.
9. `docs/decisions/0020-runtime-revision-3.md` — revision 3's exact
   comparison-count repair.
10. `docs/decisions/0013-gate-k-disposition.md` and
   `docs/decisions/0016-terminate-gate-k-round-two.md` — historical Gate K
   verdicts.
11. `docs/workspace.md` — crate graph and boundary proof.

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
`origin/main`, zero open issues, zero open PRs, and successful `verify`,
`gate-k-evidence`, `nomos viewer`, and `executable gaol pages` runs on the
handoff merge. Old remote heads may still exist; they do not override that
state.

## What can happen next

Ordinary bug maintenance against the accepted R1 baseline may start from a new
issue with falsifiable acceptance.

Decision 0022 authorizes a bounded Mortal Estate presentation-evidence program.
The adopter must first record its own authority and accept its visual target.
Nomos then requires separately falsifiable issues for the content-addressed
evaluation plan, immutable dependency point, any adopter-neutral failing
fixture, and final disposition. No accepted Nomos implementation, R2, deeper
adopter boundary, or game-adoption claim follows from decision 0022 by
implication. Disposable evidence work remains subject to the decision's
quarantine and stop line.

A new accepted capability family, dependency policy, runtime epoch, Gate K
attempt, deeper adopter boundary, or adoption into a game still requires a new
owner decision. Work toward an actual game remains exploratory until the game
satisfies its own gates and accepts measured cost.
