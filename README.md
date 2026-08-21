# signed-world

An exploratory design thesis for a game runtime whose primary content and tooling
author is a large language model, plus the executable semantic kernel intended to
test that thesis before any renderer is built.

> The agent names the thing. Namespaces own state. Capabilities define
> obligations. The resolver composes effective facts. Projection compilers own
> the consequences. The runtime executes a sealed world. The renderer owns every
> pixel. A cold stranger can rebuild and explain all of it.

## Status

- **Architecture status:** exploratory and non-authoritative
- **Implementation status:** workspace foundations (SW-B) plus source schema,
  parser, typed name resolution, primitive expansion, and ownership linker
  (SW-C); no command surface or runtime
- **Contract revision:** 3, owner-authorized in decision 0003
- **Scope:** greenfield / vacuum architecture exercise
- **Effect on other projects:** none unless separately adopted by an explicit
  decision in that project's own authority records
- **License:** MIT

This repository deliberately records a thesis before implementation. It does not
claim the thesis is correct. The kernel exists to give it a cheap, falsifiable
way to fail before graphics, networking, audio, or a general engine grow around
it.

## Read in this order

1. [THESIS.md](THESIS.md) — the design, boundaries, proof gates, resolved
   disagreements, and adoption criteria.
2. [KERNEL.md](KERNEL.md) — the revisioned acceptance contract for Gate K, the
   renderer-free executable semantic kernel.
3. [docs/decisions/0003-contract-profile-closure.md](docs/decisions/0003-contract-profile-closure.md)
   — the owner-authorized revision-2 to revision-3 closure of the canonical
   profile and workspace-evidence contract gaps.
4. [docs/decisions/0001-contract-repair.md](docs/decisions/0001-contract-repair.md)
   — the owner-authorized revision-1 to revision-2 contract repair.
5. [docs/evaluation/COLD_AGENT_PROTOCOL.md](docs/evaluation/COLD_AGENT_PROTOCOL.md)
   — the reproducible cold-author, cold-debug, and cold-review procedure.
6. [docs/evaluation/GATE_K_COLD_AGENT_PLAN.md](docs/evaluation/GATE_K_COLD_AGENT_PLAN.md)
   — the owner-authorized whole-kernel subject roster and eligibility checks.
7. [docs/workspace.md](docs/workspace.md) — the crate map, how to run the
   proof, and the decisions the first implementation slice had to make.
8. [docs/authoring.md](docs/authoring.md) — source schema version 1 and the
   approved Gate K authoring vocabulary.
9. [docs/compiler.md](docs/compiler.md) — parser/linker stages, schema
   ownership, proof coverage, and limits.
10. [docs/review/2026-08-21-founding-review.md](docs/review/2026-08-21-founding-review.md)
   — the condensed primary record of the founding adversarial review, written
   in the originating session, with its provenance limits stated.
11. [docs/review/2026-08-21-founding-review-synthesis.md](docs/review/2026-08-21-founding-review-synthesis.md)
    — the contract-revision-2 edited synthesis of that review.

## Layout

```text
README.md          status and reading order
THESIS.md          the design thesis
KERNEL.md          the Gate K acceptance contract
docs/              decisions, evaluation protocols, reviews, workspace notes
fixtures/          exact Gate K authoring source and later command/replay fixtures
crates/            the six Gate K kernel crates named in KERNEL.md section 10
xtask/             workspace tooling; `cargo xtask boundary`
.github/workflows/ the verification lane
```

The kernel crates have no third-party dependencies: `Cargo.lock` holds seven
local entries and nothing else, so the workspace builds and tests offline.

## Start here if you are new to the tree

[docs/HANDOFF.md](docs/HANDOFF.md) — current state, what is next, how to prove it.

## Gate order

- **Gate K:** prove the semantic kernel without graphics.
- **Gate 0:** obtain a human-approved visual target pack before renderer work.
- **Gate 1 and later:** prove cross-system primitives, vocabulary, cold authors,
  cold debugging, and clean rebuilds.

Passing Gate K does not authorize a renderer. Passing Gate 0 does not prove the
semantic architecture. Both must survive independently before the project earns
the right to become expensive.
