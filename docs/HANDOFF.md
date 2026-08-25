# Nomos handoff

## Current state

Nomos now has two deliberately separate bodies of work:

- The kernel workspace implements the renderer-free semantic kernel through
  SW-N. Gate K's round-one result remains **failed** under decision 0013.
- `experiments/executable-gaol/` is a quarantined visual and playability study.
  Four independently authored areas use one projection-only WebGL renderer,
  one bounded procedural look, and a connected playable route. It is not Gate
  K or Gate 1 evidence.

The public executable is <https://conarylabs.github.io/nomos/>.

The frozen round-two candidate remains annotated tag `gate-k-rc2`, commit
`53db236d397b3db0779f0d2aab23180d926e55a5`. Round two is unfinished:

- issue #93 completed the one formal Gemini author subject. Its authenticated
  transport result is `eligible-for-checker`; issue #95 has not adjudicated it;
- issue #94 and draft PR #100 preserve the one DeepSeek debugger subject. Its
  required non-author audit is unresolved, issue #96 has not adjudicated it,
  and the PR claims no pass;
- issues #95–#97 retain the independent checks, evidence assembly, and explicit
  owner disposition;
- therefore round two has no Gate K verdict and no acceptance tag.

Do not infer a formal pass from either subject's apparent semantic work or from
the executable study. The exact round-one 1–19 matrix remains in
`docs/decisions/0013-gate-k-disposition.md`; the prospective round-two rules
remain in `docs/decisions/0015-gate-k-round-two.md`.

## What works

The semantic workspace provides source parsing, stable World IR, deterministic
simulation/navigation/persistence/diagnostic projections, immutable runtime
transactions, hash-verified packages, replay and migration, and package-bound
explanations. The kernel crates remain dependency-free under the Gate K policy.

The executable study provides:

- Cistern Walk, Ember Vault, Ossuary Reach, and North Gaol as separately
  authored content;
- shared camera, palette, materials, assemblies, actor silhouettes, masonry,
  effects, water, lighting, and minimal UI;
- content-derived doors, movement cost, effective light, objectives, and a
  connected escape route;
- deterministic semantic/SVG capture evidence plus a WebGL presentation layer;
- forensic overlays and a procedural-versus-baseline look toggle;
- a static GitHub Pages build containing no source, World IR, credentials, or
  dependency on the development machine.

Actor movement and gaoler pursuit are presentation-only because Gate K has no
dynamic actor state. Audio, combat, networking, and the later cross-system Gate
1 contract remain absent. See `experiments/executable-gaol/README.md` for the
controls, authoring boundary, and complete limitations.

## Evidence boundary

The large `docs/evaluation/runs/` tree is intentional. It preserves exact
formal and rehearsal packets, transcripts, receipts, binaries, rejected audits,
and failed attempts needed to authenticate historical claims. It is not a build
cache and should not be removed during routine repository cleanup. New proof
machinery is not current product work; formal evaluation should resume only
through the already-filed issues and existing decision-0015 procedure.

Round-one candidate evidence is bound to annotated tag `gate-k-rc1`. Round-two
candidate evidence is bound to `gate-k-rc2`. Neither the executable experiment
nor this status update mutates those candidates.

## How to verify

From a clean checkout, verify the kernel workspace with:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

Verify and stage the executable study with:

```text
experiments/executable-gaol/gaol verify
experiments/executable-gaol/gaol site
```

The exact Gate K evaluation tooling additionally has its own offline tests:

```text
docs/evaluation/test-pi-cold-agent-preflight.sh
docs/evaluation/test-gate-k-eval-tooling.sh
```

## What is next

The active product direction is visual coherence and presentation: improve the
shared look and prove that new content-authored areas can fit it without
renderer-specific edits. Keep that work inside the quarantined executable study
until an explicit decision promotes a boundary into the accepted runtime.

Simulation-boundary expansion is intentionally deferred while the visual
grammar is being established. Avoid broad architecture work, new capability
families, or another evaluation-protocol revision unless a concrete executable
slice forces the question.

PR #100 remains draft because its required audit is unresolved. It should not
be merged merely to make the repository look tidy. Issues #95–#97 likewise stay
open until the owner chooses to resume formal round-two disposition work.
