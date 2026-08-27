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
| Mortal Estate evidence | bounded evidence produced one admitted dependency point, representative frame, and classified capability gap |
| R2 observed-scene epoch | revision 1 authorized; R2-1 implementation must begin from its own issue |

R1 is the accepted runtime baseline for this repository. That result does not
rewrite Gate K, pass a later gate, approve production art, or authorize Nomos
for another project. Decision 0019 is explicit: **accept R1; do not adopt Nomos
into a game.**

Decision 0022 authorized one bounded presentation-adoption evidence program for
The Mortal Estate. That program admitted an immutable R1 dependency point,
recorded a representative adopter frame, and reduced its presentation gap to an
adopter-neutral fixture classified `reusable missing Nomos capability`.

Decision 0023 opens one narrow R2 observed-scene presentation epoch from that
evidence. The separately reviewed root contract `R2.md` revision 1 is now
owner-authorized. Its first target is R2-1, the strict carrier and compiler,
which must begin from its own falsifiable issue. No R2 implementation is yet
admitted, and no platform choice or game adoption is authorized.

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
  bounded headless-Chromium proof.

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

## Read in this order

1. [docs/HANDOFF.md](docs/HANDOFF.md) — current state, fresh-box setup, stop
   line, and next authorized action.
2. [decision 0019](docs/decisions/0019-r1-final-disposition.md) — final R1
   verdict, evidence boundary, and no-adoption disposition.
3. [decision 0022](docs/decisions/0022-mortal-estate-presentation-adoption-evidence.md)
   — bounded Mortal Estate evidence authority and upstream-admission stop line.
4. [decision 0023](docs/decisions/0023-observed-scene-presentation-epoch.md)
   — narrow R2 authority, semantic boundary, target order, and stop line.
5. [THESIS.md](THESIS.md) — the exploratory architecture and adoption bars.
6. [KERNEL.md](KERNEL.md) — frozen Gate K revision 7 contract.
7. [RUNTIME.md](RUNTIME.md) — accepted R1 revision 4 contract.
8. [decision 0021](docs/decisions/0021-runtime-revision-4.md) — revision 4's
   lifecycle-history repair after R1 closed.
9. [decision 0020](docs/decisions/0020-runtime-revision-3.md) — revision 3's
   exact comparison-count repair.
10. [decision 0013](docs/decisions/0013-gate-k-disposition.md) and
   [decision 0016](docs/decisions/0016-terminate-gate-k-round-two.md) — the
   historical Gate K dispositions.
11. [docs/workspace.md](docs/workspace.md) — crate map and dependency boundary.

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
crates/            six kernel crates plus two declared R1 crates
apps/nomos-viewer/ accepted offline viewer and browser harness
experiments/       quarantined studies; never authority for accepted code
xtask/             dependency-boundary checker
.github/workflows/ verification, viewer, evidence, and Pages lanes
```

## Starting new work

Open issues, open pull requests, and owner decisions identify active work. Old
remote branches do not. Ordinary maintenance begins with a falsifiable issue
and follows [AGENTS.md](AGENTS.md). A new capability family, dependency policy,
runtime epoch, Gate K attempt, or game adoption requires a new owner decision.

Decision 0023 is the narrow exception for the observed-scene R2 epoch. The next
authorized action is the separately falsifiable R2-1 issue required by the
owner-authorized `R2.md` revision 1. Each implementation target still requires
its own issue and exact proof.

Nomos is authority only for this repository. Nothing here becomes authority for
another project without that project's own explicit decision.
