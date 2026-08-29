# Nomos

**Nomos is a semantic game runtime designed for AI authors. The Signed World is
the architectural thesis this repository tests.**

> The agent proposes the world. Nomos supplies the law.

> The agent names the thing. Namespaces own state. Capabilities define
> obligations. The resolver composes effective facts. Projection compilers own
> the consequences. The runtime executes a sealed world. The renderer owns every
> pixel. A cold stranger can rebuild and explain all of it.

New machine or new agent? Read [the current handoff](docs/HANDOFF.md) before
choosing work. It contains the fresh-box prerequisites, proof order, exact stop
line, and operational gotchas.

**Play the six-area viewer:** <https://conarylabs.github.io/nomos/>

## Status

| Line | Current disposition |
| --- | --- |
| The Signed World thesis | exploratory; applies to no game project |
| Gate K round one | failed under decision 0013 |
| Gate K round two | terminated incomplete under decision 0016 |
| R1 runtime epoch | all five criteria passed; accepted and closed under decision 0019 |
| R1 contract | `RUNTIME.md` revision 4, in force under decision 0021 |
| Game adoption | not authorized |
| Mortal Estate evidence | sister-project prospective adopter; bounded evidence produced one admitted dependency point, representative frame, and classified adopter-neutral capability gap; no adoption |
| R2 observed-scene epoch | revision 4 authorized under decision 0026; R2-1 and R2-2 landed; the revision-3 formal red remains historical and a fresh revision-4 author proof is pending |

R1 is the accepted runtime baseline for this repository. That result does not
rewrite Gate K, pass a later gate, approve production art, or authorize Nomos
for another project. Decision 0019 is explicit: **accept R1; do not adopt Nomos
into a game.**

Decision 0022 authorized one bounded presentation-adoption evidence program for
The Mortal Estate. That program admitted an immutable R1 dependency point,
recorded a representative adopter frame, and reduced its presentation gap to an
adopter-neutral fixture classified `reusable missing Nomos capability`.
Nomos and The Mortal Estate are sister projects: the game is attempting to
consume Nomos, and generally useful discoveries from that work should return as
adopter-neutral Nomos fixes. Their repositories, authority, adopter mapping,
acceptance, and final adoption decisions remain separate. This relationship
does not authorize Mortal Estate integration in issue #199.

Decision 0023 opens one narrow R2 observed-scene presentation epoch from that
evidence. Decisions 0024 and 0025 repaired the finalization order and falsified
recursive-disk-observer method. Decision 0026 then established the current
`R2.md` revision 4 by retaining the exact maximum-scene compile workload and
measurement as required evidence while correcting its ungrounded numeric
latency classification from acceptance ceiling to recorded observation. R2-1's
strict carrier and compiler landed at
`cc47a7235f92d0ed460c7db5d178448b12fdba02`; R2-2's isolated offline consumer
and hash-frozen independent second-scene evidence landed through PR #198.
Issue #199 carries the revision-4 contract-and-proof repair. Its fresh complete
candidate-native XFS author proof is next. The retained revision-3 attempt stays
formal red under the contract it ran against and cannot satisfy revision 4. R2
is not admitted until one revision-4 combined candidate passes the complete
author proof, exact-head non-author rerun, owner visual judgment, required CI,
and owner disposition. No platform choice or game adoption is authorized.

The implementation includes:

- six dependency-free semantic kernel crates through SW-N: strict authoring,
  stable World IR, compiled projections and transitions, immutable runtime
  transactions, verified packages, replay, migration, and explanations;
- read-only effective-facts and entity-catalog projections;
- the R1 `nomos-render-plan` compiler and typed presentation boundary;
- the R1 `nomos-play` native/wasm runtime for actors, movement, pursuit,
  receipts, sessions, and replay; and
- `apps/nomos-viewer`, an accepted offline viewer with vendored Three.js,
  strict decoders, scanned artifacts, native/browser session identity, and a
  bounded headless-Chromium proof; plus
- the unadmitted R2 `nomos-observed-scene` carrier/compiler and isolated
  `apps/nomos-observed-viewer`, with strict render-only decoding and two-scene
  offline browser evidence awaiting the R2 final disposition.

The proof corpus connects six independently authored areas from the
quarantined executable-gaol study through one route. Two were cold-authored
from the packet alone. The study remains non-authoritative even though accepted
R1 code consumes its published artifacts as a specification and comparison
target.

Audio, networking, replication, combat, production scaling, and the adopting
game's Gate 0/Gate 1 evidence do not exist here.

## Proof

The pinned toolchain is Rust 1.98.0 with `rustfmt`, `clippy`, and
`wasm32-unknown-unknown`. The workspace has no third-party Cargo dependency;
Three.js is vendored with its license and digest. No `npm install` is needed.

Run the accepted workspace proof from a clean checkout:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

Run the six-area artifact and browser proof in this order:

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

The browser lane requires Node 22 or newer and Chrome/Chromium; set
`CHROME_BIN` when discovery cannot find the binary. See
[docs/HANDOFF.md](docs/HANDOFF.md) for complete setup and worktree rules.

Run the R2 portable checks and two-scene browser proof with the commands
recorded in the handoff. The revision-4 final proof additionally requires the
dedicated XFS wrapper and host tools named there. It requires no npm
installation and reuses only the accepted, digest-checked Three.js bytes.

## Read in this order

1. [docs/HANDOFF.md](docs/HANDOFF.md) — current state, fresh-box setup, stop
   line, and next authorized action.
2. [decision 0019](docs/decisions/0019-r1-final-disposition.md) — final R1
   verdict, evidence boundary, and no-adoption disposition.
3. [decision 0022](docs/decisions/0022-mortal-estate-presentation-adoption-evidence.md)
   — bounded Mortal Estate evidence authority and upstream-admission stop line.
4. [decision 0023](docs/decisions/0023-observed-scene-presentation-epoch.md)
   — narrow R2 authority, semantic boundary, target order, and stop line.
5. [decision 0026](docs/decisions/0026-r2-compile-latency-observation.md) — R2
   revision 4's compile-observation classification repair, retained red
   evidence, and exact owner disposition.
6. [decision 0025](docs/decisions/0025-r2-filesystem-accounting.md) — R2
   revision 3's bounded XFS accounting repair.
7. [decision 0024](docs/decisions/0024-r2-final-proof-finalization-order.md) —
   R2 revision 2's terminal-evidence ordering repair.
8. [THESIS.md](THESIS.md) — the exploratory architecture and adoption bars.
9. [KERNEL.md](KERNEL.md) — frozen Gate K revision 7 contract.
10. [RUNTIME.md](RUNTIME.md) — accepted R1 revision 4 contract.
11. [decision 0021](docs/decisions/0021-runtime-revision-4.md) — revision 4's
   lifecycle-history repair after R1 closed.
12. [decision 0020](docs/decisions/0020-runtime-revision-3.md) — revision 3's
   exact comparison-count repair.
13. [decision 0013](docs/decisions/0013-gate-k-disposition.md) and
   [decision 0016](docs/decisions/0016-terminate-gate-k-round-two.md) — the
   historical Gate K dispositions.
14. [docs/workspace.md](docs/workspace.md) — crate map and dependency boundary.

Subsystem designs and receipts live under `docs/review/` and
`docs/evaluation/`. The large `docs/evaluation/runs/` archive and `gate-k-*`
tags are deliberate historical evidence, not build caches.

## Layout

```text
README.md          status and reading order
AGENTS.md          mandatory agent rules and change flow
THESIS.md          exploratory design thesis, revision 2
KERNEL.md          frozen Gate K contract, revision 7
RUNTIME.md         accepted R1 contract, revision 4
docs/HANDOFF.md    current operational state and fresh-box setup
docs/decisions/    owner-authorized contract and architecture decisions
docs/evaluation/   reproducible proofs and immutable run archive
docs/review/       subsystem designs, audits, and review receipts
crates/            six kernel crates, two declared R1 crates, and one isolated R2 crate
apps/nomos-viewer/ accepted offline viewer and browser harness
apps/nomos-observed-viewer/ isolated unadmitted R2 viewer and browser harness
experiments/       quarantined studies; never authority for accepted code
xtask/             dependency-boundary checker
.github/workflows/ verification, viewer, evidence, and Pages lanes
```

## Starting new work

Open issues, open pull requests, and owner decisions identify active work. Old
remote branches do not. Ordinary maintenance begins with a falsifiable issue
and follows [AGENTS.md](AGENTS.md). A new capability family, dependency policy,
runtime epoch, Gate K attempt, or game adoption requires a new owner decision.

Decision 0023 is the narrow exception for the observed-scene R2 epoch. R2-1 and
R2-2 are complete, and issue #199 is the active final-evidence-and-disposition
slice required by `R2.md` section 11. Decision 0026 records the owner's exact
disposition to repair the contract and rerun affected evidence. The latest
revision-3 XFS attempt remains formal red under revision 3; it is not relabelled
as a revision-4 pass. Issue #199 still forbids compiler edits, and no product
optimization is authorized in this slice. After the proof machinery binds
revision 4 and the frozen issue body, create fresh candidate/source/work paths
and run the complete candidate-native XFS author proof. A passing candidate
still requires the contract's exact-head non-author rerun, public-repository
hosted workflows, and the owner's visual and R2 verdicts. GitHub Actions is
available; any currently local-only branch state is a pending workflow step,
not an Actions-quota constraint. Do not start a later epoch from this branch.

Nomos is authority only for this repository. Nothing here becomes authority for
another project without that project's own explicit decision.
