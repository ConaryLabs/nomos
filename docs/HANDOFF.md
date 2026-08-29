# Nomos handoff

Status snapshot: 2026-08-29, during the issue #199 revision-4 contract-and-proof
repair. This file is an operational map; owner decisions and revisioned
contracts remain the authority when prose conflicts.

**Play the accepted six-area viewer:** <https://conarylabs.github.io/nomos/>

## Stop line

R1 is complete, passing, accepted, and closed as this repository's runtime
baseline. Decision 0019 expressly does **not** adopt Nomos into a game. Decision
0022's bounded Mortal Estate evidence then admitted one immutable R1 dependency
point, recorded one representative adopter frame, and reduced the observed gap
to an adopter-neutral fixture classified `reusable missing Nomos capability`.

Nomos and The Mortal Estate operate as sister projects with separate
repositories and authority trees. The Mortal Estate is attempting to consume
Nomos as a prospective adopter. Generally useful discoveries and fixes from
that use should flow back through separately authorized, adopter-neutral Nomos
work. Game-specific content, mechanics, integration, adopter mapping,
acceptance, and the final adoption decision remain in The Mortal Estate. This
working relationship is not itself a game-adoption verdict and issue #199 does
not authorize Mortal Estate integration.

Decision 0023 opens one narrow R2 observed-scene presentation epoch. R2-1's
strict carrier and compiler landed through PR #196 at
`cc47a7235f92d0ed460c7db5d178448b12fdba02`, tree
`2bce614d2df94464c20042cdf059a7b22ec39c09`. R2-2's isolated offline consumer
and independent second scene landed through PR #198 at main commit
`6cbce64cb867aef24faf227e62bdfc585bbcbd5d`, tree
`6dada35f44e178f0d6cafc5ac2b5c94ab3fd0522`. Those implementation targets do
not themselves admit R2.

Issue #199 is now the active final-evidence-and-disposition slice. Decision
0024 established R2 revision 2 by repairing the impossible terminal-evidence
order. Decision 0025 established R2 revision 3 by replacing the falsified
recursive disk observer with bounded accounting on a dedicated, fully allocated
8 GiB XFS image. Decision 0026 establishes current `R2.md` revision 4 under the
owner's exact disposition `repair the contract and rerun affected evidence`.
It retains the exact maximum-scene workload, process and publication method,
raw samples, arithmetic, environment, and deterministic-output checks, but
corrects the unsupported 50 ms median and 100 ms p95 classification from
acceptance ceilings to required recorded observations. Branch
`feat/issue-199-r2-final` contains the historical revision-3 formal-red records
and the revision-4 contract-and-proof repair. R2 still has no passing formal
author proof, exact-head non-author proof, owner visual verdict, owner R2
verdict, or merge authorization.

The most recent formal attempt was revision-4 candidate
`00987913615b266f2fa792edb37db7b9304da439`, tree
`abe227ef4e33d23578a535427a3b999a2cdcfb61`. Its fresh detached source was
`/data/dev/src/nomos-r4-k-candidate.akDvtZ`; its fresh work directory was
`/data/dev/src/nomos-r4-k-xfs-run.h5H2w9`. The inner proof passed all 33 ordered
commands and independently assembled and verified its receipt. The compile
observation was a median of `116912362 / 2` ns (`58,456,181` ns) and p95
`64,134,818` ns; all 100 outputs were the same 111,604 bytes. The 18.54 s build,
1,544 MiB peak disk use, `97,894,050` ns maximum sampler gap, 805,600-byte
distribution, both browser lanes, and process-closure checks all passed.

The outer wrapper nevertheless remained formally red and emitted no
`wrapper-receipt.json`; receipt assembly exited one with `image stat evidence
differs from the image file` while the supervisor exited zero. The `-K` repair
worked: exact formatter argv was
`/usr/sbin/mkfs.xfs -f -K -l internal /dev/loop1`, with no discard output. The
pre-format image was exactly 8 GiB logical and allocated. After ordinary
unmount, detach, and proof of no association, it remained exactly 8 GiB logical
and had `8,589,938,688` allocated bytes, 4,096 more than the pre-format
snapshot. Read-only diagnosis found the full 8 GiB data map and 98 extents; the
additional host-XFS block is consistent with extent-map metadata, not a hole.
The contract requires allocation **at least** 8 GiB and never requires the two
checkpoints to be equal. The sole refusal was an extra-contractual comparison
between a pre-format evidence file and the post-teardown inode.

The repair in this candidate records separately named pre-format and
post-teardown stat evidence, validates each exact logical size, 512-byte
`st_blocks` arithmetic, and minimum allocation, binds facts to the matching
checkpoint, and compares the live inode only to the post-teardown snapshot. It
changes no contract, workload, compiler, viewer, or acceptance ceiling.

Preserve both second-run paths above and the earlier first-red source
`/data/dev/src/nomos-r4-candidate.wbrd6Z` and work
`/data/dev/src/nomos-r4-xfs-run.1djISn` as historical formal-red evidence.
Cleanup for both runs removed `/dev/loop1`; the loop inventories were
byte-identical and the unrelated `/dev/loop0` was untouched. Neither inner pass
may be resumed, promoted, or spliced. After the two-checkpoint repair, tests,
provenance, and red record are frozen in a clean candidate, the next formal
action is one entirely fresh complete candidate-native XFS author proof using
new standalone source and work paths. Issue #199 deliverable 5 still forbids
product edits.

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
| R2 observed-scene presentation epoch | revision 4 in force; R2-1 and R2-2 landed; epoch not admitted | `R2.md`, decisions 0023–0026 |
| R2 final evidence | revision-3 attempts and two revision-4 wrapper attempts remain historical formal red; no passing revision-4 author proof | issue #199, decision 0026, branch `feat/issue-199-r2-final` |
| Current queue | finish and freeze the two-checkpoint image-stat verifier repair, then run one fresh complete candidate-native XFS author proof | issue #199 and `R2.md` sections 9, 11, and 13 |

## Current local verification

Before the retained second revision-4 formal attempt, exact-head candidate
`0098791` passed:

- `cargo fmt --all -- --check`, workspace Clippy with warnings denied, all
  locked workspace tests, and `cargo xtask boundary`;
- schema ownership, the 100-row source-provenance register and 10 plants,
  adopter neutrality and 5 plants;
- 39 complete-proof refusal plants and the XFS shell validation suite;
- 132 Node tests across the R2 viewer, receipts, process closure, XFS evidence,
  accounting, and scene signatures; and
- ShellCheck on every changed shell file with only the repository's existing
  dynamic-source-path and source-only function-analysis exemptions.

The source-provenance register SHA-256 at that exact head is
`7fb00336e5f3b35f7a69c75a5f4d72c635dbbc98e0ddd2962e8412e1e5367c6b`.
Focused receipt, evidence, XFS shell-validation, syntax, and ShellCheck tests
pass for the subsequent two-checkpoint repair, but those dirty-tree checks are
not an author proof. Freeze the repair, rerun the entire portable preflight from
its clean committed tip, and do not claim a revision-4 author pass until a
fresh complete wrapper finishes and its outer receipt independently verifies.

The latest retained formal-red work is
`/data/dev/src/nomos-r4-k-xfs-run.h5H2w9`, with standalone source
`/data/dev/src/nomos-r4-k-candidate.akDvtZ`. Its fully allocated backing image,
exported passing inner evidence, and logs remain red evidence but are no longer
attached or mounted. The preceding discard-red R4 paths are also retained. All
decisive digests are recorded in the revision-4 author receipt. Earlier red
paths and diagnostics remain recorded in the revision-3 receipt; do not
relabel or reuse any of them.

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
6. `docs/decisions/0026-r2-compile-latency-observation.md` — current R2
   revision 4, the compile-observation classification repair, retained
   revision-3 red evidence, and exact owner disposition.
7. `docs/decisions/0025-r2-filesystem-accounting.md` — R2 revision 3,
   the falsified recursive-observer record, and the XFS replacement method.
8. `docs/decisions/0024-r2-final-proof-finalization-order.md` — R2 revision 2's
   dependency-correct terminal evidence order.
9. `R2.md` — owner-authorized revision 4 acceptance contract.
10. `THESIS.md` — exploratory design thesis; not authority for another project.
11. `KERNEL.md` — frozen Gate K revision 7 contract and historical failed bar.
12. `RUNTIME.md` — accepted R1 revision 4 contract.
13. `docs/decisions/0021-runtime-revision-4.md` — revision 4's exact
   lifecycle-history repair.
14. `docs/decisions/0020-runtime-revision-3.md` — revision 3's exact
   comparison-count repair.
15. `docs/decisions/0013-gate-k-disposition.md` and
   `docs/decisions/0016-terminate-gate-k-round-two.md` — historical Gate K
   verdicts.
16. `docs/workspace.md` — crate graph and boundary proof.

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

The revision-4 complete proof additionally requires Linux with loop and XFS
support, at least 8 GiB available for one fully allocated backing image,
passwordless non-interactive `sudo` for the invoking user, and the exact host
tools checked by the wrapper. On Ubuntu 24.04 the non-base packages are
`bubblewrap`, `e2fsprogs`, `psmisc`, and `xfsprogs`; the proof uses `bwrap`,
`filefrag`, `fuser`, `losetup`, `mkfs.xfs`, `xfs_info`, and `xfs_quota`. Install
all tools before entering the wrapper because the proof itself is offline.

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

The portable R2 preflight is:

```bash
cargo build --release --locked -p nomos-observed-scene
docs/evaluation/r2-second-scene-packet.test.sh
docs/evaluation/r2-schema-ownership.sh
docs/evaluation/r2-source-provenance.sh
docs/evaluation/r2-source-provenance.test.sh
docs/evaluation/r2-adopter-neutrality.sh
docs/evaluation/r2-adopter-neutrality.test.sh
docs/evaluation/r2-complete-proof.test.sh
docs/evaluation/r2-complete-proof-xfs.test.sh
node docs/evaluation/r2-maximum.test.mjs
node docs/evaluation/r2-scene-signature.mjs \
  fixtures/r2/scenes/scene_one.json fixtures/r2/scenes/scene_two.json
node --test apps/nomos-observed-viewer/test/*.test.mjs \
  docs/evaluation/r2-scene-signature.test.mjs \
  docs/evaluation/r2-complete-proof-process.test.mjs \
  docs/evaluation/r2-complete-proof-receipt.test.mjs \
  docs/evaluation/r2-complete-proof-xfs-evidence.test.mjs \
  docs/evaluation/r2-complete-proof-xfs-receipt.test.mjs \
  docs/evaluation/r2-filesystem-accounting.test.mjs \
  docs/evaluation/r2-filesystem-evidence.test.mjs
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
exact-head Luna Max rerun.

Those portable commands are not the final disposition proof. For issue #199,
first commit the candidate and rerun the preflight from a clean branch tip.
Then make new standalone paths under a host filesystem with more than 8 GiB
free. The source wrapper and source argument must come from the same detached
candidate clone:

```bash
candidate=$(git rev-parse 'HEAD^{commit}')
candidate_tree=$(git rev-parse 'HEAD^{tree}')
test -z "$(git status --porcelain=v1 --untracked-files=all)"

proof_parent=/absolute/host/path/with-more-than-8GiB-free
candidate_source=$(mktemp -d "$proof_parent/nomos-r2-candidate.XXXXXX")
candidate_work=$(mktemp -d "$proof_parent/nomos-r2-xfs-run.XXXXXX")
git clone --no-local --no-hardlinks . "$candidate_source"
git -C "$candidate_source" checkout --detach "$candidate"
test "$(git -C "$candidate_source" rev-parse 'HEAD^{tree}')" = "$candidate_tree"
test "$(git -C "$candidate_source" rev-parse --is-shallow-repository)" = false
test -z "$(git -C "$candidate_source" status --porcelain=v1 --untracked-files=all)"

browser=/absolute/path/to/chrome-or-chrome-headless-shell
(
  cd "$candidate_source"
  CHROME_BIN="$browser" \
    docs/evaluation/r2-complete-proof-xfs.sh \
      --source "$candidate_source" \
      --work "$candidate_work"
)
```

Never substitute the development checkout's wrapper into that last command.
Each attempt gets fresh source and work paths whether the preceding attempt was
red or green. A red run must be recorded as red and retained until its evidence
has been routed. Before and after every run, confirm that only the intended
proof loop appeared and that teardown removed it; this machine currently has
an unrelated Conary `/dev/loop0` that is outside Nomos and must not be touched.

A passing author run still needs a fresh exact-head Luna Max rerun, the owner
visual judgment, the required hosted workflows, and the owner's explicit R2
and merge dispositions. The repository is now public and its configured jobs
use standard public-repository runners, so the prior Actions-minutes quota is no
longer a blocker. Hosted CI remains pending because this exact candidate is
still local with no pull request or workflow run; local proof does not replace
that lane.

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

At this snapshot, local `main` and `origin/main` are still PR #198's merge
`6cbce64cb867aef24faf227e62bdfc585bbcbd5d`; issue #199 work is on
`feat/issue-199-r2-final`. Do not expect the feature branch to have an upstream
or infer a pull request from its existence. The repository is public and
Actions is available; the branch does not need to remain local because of a
quota. It has not yet been pushed and therefore has no exact-head hosted run.
Old remote heads may still exist; they do not override the owner decisions,
open issue, or this candidate state.

## What can happen next

Ordinary bug maintenance against the accepted R1 baseline may start from a new
issue with falsifiable acceptance.

Decision 0023 authorizes the R2 epoch boundary and nothing past its stated
order. `R2.md` revision 4 defines exact acceptance, finite input and output
grammars, ownership, workspace boundaries, ceilings, the required compile
observation, and proof. R2-1 and R2-2 are complete after PR #198. Issue #199 is
already the authorized final-evidence slice; do not open a replacement issue.
Decision 0026 authorizes only the contract-and-proof classification repair and
rerun. It does not authorize compiler edits or relabel the revision-3 red. Once
the repair is clean and committed, run one fresh full candidate-native XFS
author proof from new source and work paths. If it passes, obtain the required
fresh exact-head Luna Max proof, applicable hosted workflows, and owner visual
judgment, then stop for the exact owner verdicts and distinct merge disposition
required by issue #199.

A deeper adopter boundary, platform choice, Gate K attempt, R2 scope expansion,
or adoption into a game requires its own owner decision. Work toward an actual
game remains exploratory until the game satisfies its own gates and accepts
measured cost.
