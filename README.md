# Nomos

**Nomos — a semantic game runtime designed for AI authors.**

This repository contains the executable semantic kernel and **The Signed
World**, the exploratory architectural thesis Nomos is intended to test before
any renderer is built.

> The agent proposes the world. Nomos supplies the law.

> The agent names the thing. Namespaces own state. Capabilities define
> obligations. The resolver composes effective facts. Projection compilers own
> the consequences. The runtime executes a sealed world. The renderer owns every
> pixel. A cold stranger can rebuild and explain all of it.

## Status

- **Architecture status:** exploratory and non-authoritative
- **Implementation status:** semantic implementation is complete through SW-N
  on `main`. The implemented surface includes foundations, source and preserved
  construction IR, stable
  `nomos.world_ir@2` plus strict v1 migration, compiled transitions,
  shared movement and light resolution, all four projections, immutable runtime
  commits, complete hash-verified world packages, the filesystem `validate`,
  `compile`, `inspect`, `run`, `command`, and `replay` commands, strict persisted
  runtime and replay evidence, atomic verified run bundles, the immutable
  `migrate` command, and package-bound read-only entity and transition
  explanations
- **Gate K status:** **failed**, owner-disposed in decision 0013. Exact candidate
  `gate-k-rc1` / `d8a0b85` passed criteria 1–16 and the final different-author
  proof passed criterion 19. The one formal cold-author attempt failed criterion
  17 because its checker requested forbidden outside-workspace paths; the one
  formal cold-debug attempt failed criterion 18 for the same class of subject
  request. Both did the semantic task correctly, but the frozen rubric does not
  permit self-waiver. No retry is authorized, no acceptance tag exists, and
  Gate K is not green
- **Visual-study status:** decision 0014 authorizes one quarantined, static
  Gate 0-format study of the Gate K gaol under `experiments/`. Issue #83 has
  assembled the coherent static target pack; Peter's owner disposition is
  **visual thesis compelling**. It is non-authoritative, contains no renderer
  or executable work, and cannot satisfy Gate K or count as a formal Gate 0
  pass
- **Contract revision:** 7, owner-authorized in decision 0009
- **Cold-agent protocol revision:** 5, owner-authorized in decision 0012
- **Scope:** greenfield / vacuum architecture exercise
- **Effect on other projects:** none unless separately adopted by an explicit
  decision in that project's own authority records
- **License:** MIT

This repository deliberately records a thesis before implementation. It does not
claim the thesis is correct. The kernel exists to give it a cheap, falsifiable
way to fail before graphics, networking, audio, or a general engine grow around
it.

That falsification has now happened at the cold-agent boundary. Nomos remains a
useful semantic experiment. The decision-0014 static visual study produced a
compelling owner-approved target from its ordinary gameplay frame and supporting
tests. A fresh prospectively governed Gate K attempt is still required before
renderer architecture or adoption; no such work is authorized here.

## Read in this order

1. [docs/decisions/0013-gate-k-disposition.md](docs/decisions/0013-gate-k-disposition.md)
   — the final owner verdict, exact 1–19 matrix, and project consequence.
2. [docs/decisions/0014-quarantined-gaol-visual-target-experiment.md](docs/decisions/0014-quarantined-gaol-visual-target-experiment.md)
   — the narrow authorization for a non-authoritative static gaol target pack.
3. [experiments/gate-0-gaol-target-pack/TARGET.md](experiments/gate-0-gaol-target-pack/TARGET.md)
   — the static visual invariants, frame intent, provenance, risks, and
   compelling owner disposition for issue #83.
4. [THESIS.md](THESIS.md) — the design, boundaries, proof gates, resolved
   disagreements, and adoption criteria.
5. [KERNEL.md](KERNEL.md) — the revisioned acceptance contract for Gate K, the
   renderer-free executable semantic kernel.
6. [docs/evaluation/GATE_K_FINAL_EVIDENCE_INDEX.json](docs/evaluation/GATE_K_FINAL_EVIDENCE_INDEX.json)
   — the content-addressed final evidence index.
7. [docs/decisions/0012-cold-agent-evidence-authentication.md](docs/decisions/0012-cold-agent-evidence-authentication.md)
   and [docs/decisions/0011-cold-agent-attempt-ledger.md](docs/decisions/0011-cold-agent-attempt-ledger.md)
   — the owner-authorized protocol revisions 4 and 5 for prospective attempt
   reservation and complete evaluation-envelope authentication.
8. [docs/decisions/0009-transition-explanation-input-boundary.md](docs/decisions/0009-transition-explanation-input-boundary.md)
   — the owner-authorized revision-6 to revision-7 repair requiring a verified
   world for transition explanations and separate tick-7 run evidence.
9. [docs/decisions/0010-cold-agent-token-budget.md](docs/decisions/0010-cold-agent-token-budget.md)
   — the owner-authorized cold-agent protocol revision 3 removal of resource
   ceilings while preserving complete usage and command accounting.
10. [docs/decisions/0008-cold-agent-nomos-cli-identity.md](docs/decisions/0008-cold-agent-nomos-cli-identity.md)
   — the owner-authorized cold-agent protocol revision 2 correction from the
   prototype `estate` CLI name to active `nomos`; no tool scope or rubric changes.
11. [docs/decisions/0007-adopt-nomos-identity.md](docs/decisions/0007-adopt-nomos-identity.md)
   — the owner-authorized revision-5 to revision-6 identity cutover: Nomos is
   the project/runtime, The Signed World remains the thesis, and active schemas
   begin a fresh pre-Gate epoch.
12. [docs/decisions/0006-package-evidence-boundary.md](docs/decisions/0006-package-evidence-boundary.md)
   — the owner-authorized revision-4 to revision-5 repair sealing package
   receipts, publication, exact manifest decoding, and filesystem entry types.
13. [docs/decisions/0003-contract-profile-closure.md](docs/decisions/0003-contract-profile-closure.md)
   — the owner-authorized revision-2 to revision-3 closure of the canonical
   profile and workspace-evidence contract gaps.
14. [docs/decisions/0004-world-ir-construction-lineage.md](docs/decisions/0004-world-ir-construction-lineage.md)
   — the owner-authorized revision-3 to revision-4 repair separating incomplete
   construction snapshots from the stable World IR migration line.
15. [docs/decisions/0005-gate-k-dependency-policy.md](docs/decisions/0005-gate-k-dependency-policy.md)
   — the owner-authorized temporary zero-third-party-dependency policy for Gate
   K; it does not amend contract revision 4 or bind later gates.
16. [docs/decisions/0001-contract-repair.md](docs/decisions/0001-contract-repair.md)
   — the owner-authorized revision-1 to revision-2 contract repair.
17. [docs/evaluation/COLD_AGENT_PROTOCOL.md](docs/evaluation/COLD_AGENT_PROTOCOL.md)
   — the reproducible cold-author, cold-debug, and cold-review procedure.
18. [docs/evaluation/GATE_K_COLD_AGENT_PLAN.md](docs/evaluation/GATE_K_COLD_AGENT_PLAN.md)
   — the owner-authorized whole-kernel subject roster and eligibility checks.
19. [docs/workspace.md](docs/workspace.md) — the crate map, how to run the
   proof, and the decisions the first implementation slice had to make.
20. [docs/authoring.md](docs/authoring.md) — source schema version 1 and the
   approved Gate K authoring vocabulary.
21. [docs/compiler.md](docs/compiler.md) — parser/linker stages, schema
   ownership, proof coverage, and limits.
22. [docs/transitions.md](docs/transitions.md) — compiled command/event
   semantics, causal ordering, and immutable runtime preparation.
23. [docs/movement.md](docs/movement.md) — compiled claim composition,
   shared simulation/navigation movement semantics, and SW-E evidence.
24. [docs/packages.md](docs/packages.md) — atomic package publication, exact
   manifest/member verification, and the revision-5 evidence boundary.
25. [docs/migration.md](docs/migration.md) — stable-v1 to stable-v2 movement
   migration, normalized runtime proof, and digest mapping.
26. [docs/provenance.md](docs/provenance.md) — typed fact identities, resolved
    values, causal inputs, and the boundary between semantics and display text.
27. [docs/runtime.md](docs/runtime.md) — compiler-owned light union, immutable
    runtime snapshots, state hashes, atomic commit, and typed causal receipts.
28. [docs/explanations.md](docs/explanations.md) — package-bound entity and
    transition explanations, stable selection failures, and SW-N evidence.
29. [docs/review/2026-08-21-founding-review.md](docs/review/2026-08-21-founding-review.md)
   — the condensed primary record of the founding adversarial review, written
   in the originating session, with its provenance limits stated.
30. [docs/review/2026-08-21-founding-review-synthesis.md](docs/review/2026-08-21-founding-review-synthesis.md)
    — the contract-revision-2 edited synthesis of that review.

## Layout

```text
README.md          status and reading order
THESIS.md          the design thesis
KERNEL.md          the Gate K acceptance contract
docs/              decisions, evaluation protocols, reviews, workspace notes
experiments/       quarantined non-authoritative studies; issue #83 target pack
fixtures/          exact Gate K authoring source, command, and replay fixtures
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
