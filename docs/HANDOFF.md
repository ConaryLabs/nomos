# Nomos handoff

## Current state

Gate K is closed. Decision 0013 remains its controlling verdict: **failed**.
Decision 0016 terminates the separately governed round two incomplete, with no
round-two verdict and no remaining checker, audit, retry, or evidence-assembly
work authorized.

The kernel workspace implements the renderer-free semantic system through
SW-N. The active product direction is the owner-authorized quarantined study at
`experiments/executable-gaol/`: four independently authored areas, one
projection-only WebGL renderer, one bounded procedural look, and a connected
playable route. It is not Gate K or Gate 1 evidence.

**Play online:** <https://conarylabs.github.io/nomos/>

## Gate K record

- Round one is preserved at annotated tag `gate-k-rc1`; its exact 1–19 matrix
  is in `docs/decisions/0013-gate-k-disposition.md`.
- Round two's mechanically proven candidate is preserved at annotated tag
  `gate-k-rc2`, commit
  `53db236d397b3db0779f0d2aab23180d926e55a5`.
- The round-two Gemini author subject completed with subject-stage outcome
  `eligible-for-checker`, but no checker adjudicated it.
- The round-two DeepSeek debugger subject is preserved, unmerged, at annotated
  tag `gate-k-rc2-debug-subject-incomplete`, commit
  `55bb77bf4221c2c5600cd20bb781c0018a6d40a8`. Its non-author audit and checker
  did not complete.
- Neither round-two subject is a formal pass. Round two has no acceptance tag,
  criteria matrix, or overall verdict.
- No protocol revision 7, Gate K retry, or round three is authorized.

The large `docs/evaluation/runs/` tree and archival tags intentionally preserve
the exact packets, transcripts, receipts, binaries, rehearsals, blocked audits,
and failed attempts behind historical claims. They are evidence, not active
product work or disposable build output.

## What works

The kernel provides source parsing, stable World IR, deterministic
simulation/navigation/persistence/diagnostic projections, immutable runtime
transactions, hash-verified packages, replay and migration, and package-bound
explanations.

The executable study provides:

- Cistern Walk, Ember Vault, Ossuary Reach, and North Gaol as separately
  authored content;
- shared camera, palette, materials, assemblies, actor silhouettes, masonry,
  effects, water, lighting, and minimal UI;
- content-derived doors, movement cost, effective light, objectives, and a
  connected escape route;
- deterministic semantic/SVG evidence plus a WebGL presentation layer;
- forensic overlays and a procedural-versus-baseline comparison;
- a static public build containing no source, World IR, credentials, or
  dependency on the development machine.

Actor movement and gaoler pursuit remain presentation-only because Gate K has
no dynamic actor state. Audio, combat, networking, and the later cross-system
Gate 1 contract remain absent.

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

## What is next

Improve the shared look and presentation, then prove that additional
content-authored areas fit it without renderer-specific edits. Keep those
iterations inside the quarantined executable study until concrete evidence
justifies a separate decision to promote a boundary into a post-Gate-K runtime
epoch.

That separate decision is now owner-authorized:
`docs/decisions/0017-post-gate-k-runtime-epoch.md` opens the R1 epoch under
issue #124. Its contract document `RUNTIME.md` is pending under issue #128 and
nothing is accepted into R1 until it exists; issue #125 audits
presentation-boundary ownership and issue #126 sizes the kernel effective-facts
projection that is R1's first target.

Simulation-boundary expansion remains deferred while the visual grammar is
being established. Do not reopen Gate K evaluation work or add proof machinery
unless the owner explicitly reverses decision 0016.
