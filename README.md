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
- **Gate K status:** `gate-k-rc1` freezes exact candidate `d8a0b85`. Both formal
  cold-agent attempts are complete and owner-disposed `fail` after otherwise
  correct semantic work because a subject or checker requested a forbidden
  outside-workspace path. Draft PR #80 repairs fail-closed final assembly with
  hash-bound structured command adjudication. Nine non-author audits found
  binding defects in successive revisions; the current repair rejects
  duplicate-key or reordered transcripts, validates the complete qualification
  envelope, binds final writable packet bytes, and admits only the four frozen
  formal task receipts. The current repair also requires receipt-backed attempt
  closure, exact public JSON schemas, pre-sanitization stream validation, and
  path-and-hash runtime identities, full-record close authentication, and exact
  legacy-receipt admission. It awaits exact-head non-author proof. Gate K is not
  accepted or green
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

## Read in this order

1. [THESIS.md](THESIS.md) — the design, boundaries, proof gates, resolved
   disagreements, and adoption criteria.
2. [KERNEL.md](KERNEL.md) — the revisioned acceptance contract for Gate K, the
   renderer-free executable semantic kernel.
3. [docs/decisions/0012-cold-agent-evidence-authentication.md](docs/decisions/0012-cold-agent-evidence-authentication.md)
   and [docs/decisions/0011-cold-agent-attempt-ledger.md](docs/decisions/0011-cold-agent-attempt-ledger.md)
   — the owner-authorized protocol revisions 4 and 5 for prospective attempt
   reservation and complete evaluation-envelope authentication.
3. [docs/decisions/0009-transition-explanation-input-boundary.md](docs/decisions/0009-transition-explanation-input-boundary.md)
   — the owner-authorized revision-6 to revision-7 repair requiring a verified
   world for transition explanations and separate tick-7 run evidence.
4. [docs/decisions/0010-cold-agent-token-budget.md](docs/decisions/0010-cold-agent-token-budget.md)
   — the owner-authorized cold-agent protocol revision 3 removal of resource
   ceilings while preserving complete usage and command accounting.
5. [docs/decisions/0008-cold-agent-nomos-cli-identity.md](docs/decisions/0008-cold-agent-nomos-cli-identity.md)
   — the owner-authorized cold-agent protocol revision 2 correction from the
   prototype `estate` CLI name to active `nomos`; no tool scope or rubric changes.
6. [docs/decisions/0007-adopt-nomos-identity.md](docs/decisions/0007-adopt-nomos-identity.md)
   — the owner-authorized revision-5 to revision-6 identity cutover: Nomos is
   the project/runtime, The Signed World remains the thesis, and active schemas
   begin a fresh pre-Gate epoch.
7. [docs/decisions/0006-package-evidence-boundary.md](docs/decisions/0006-package-evidence-boundary.md)
   — the owner-authorized revision-4 to revision-5 repair sealing package
   receipts, publication, exact manifest decoding, and filesystem entry types.
8. [docs/decisions/0003-contract-profile-closure.md](docs/decisions/0003-contract-profile-closure.md)
   — the owner-authorized revision-2 to revision-3 closure of the canonical
   profile and workspace-evidence contract gaps.
9. [docs/decisions/0004-world-ir-construction-lineage.md](docs/decisions/0004-world-ir-construction-lineage.md)
   — the owner-authorized revision-3 to revision-4 repair separating incomplete
   construction snapshots from the stable World IR migration line.
10. [docs/decisions/0005-gate-k-dependency-policy.md](docs/decisions/0005-gate-k-dependency-policy.md)
   — the owner-authorized temporary zero-third-party-dependency policy for Gate
   K; it does not amend contract revision 4 or bind later gates.
11. [docs/decisions/0001-contract-repair.md](docs/decisions/0001-contract-repair.md)
   — the owner-authorized revision-1 to revision-2 contract repair.
12. [docs/evaluation/COLD_AGENT_PROTOCOL.md](docs/evaluation/COLD_AGENT_PROTOCOL.md)
   — the reproducible cold-author, cold-debug, and cold-review procedure.
13. [docs/evaluation/GATE_K_COLD_AGENT_PLAN.md](docs/evaluation/GATE_K_COLD_AGENT_PLAN.md)
   — the owner-authorized whole-kernel subject roster and eligibility checks.
14. [docs/workspace.md](docs/workspace.md) — the crate map, how to run the
   proof, and the decisions the first implementation slice had to make.
15. [docs/authoring.md](docs/authoring.md) — source schema version 1 and the
   approved Gate K authoring vocabulary.
16. [docs/compiler.md](docs/compiler.md) — parser/linker stages, schema
   ownership, proof coverage, and limits.
17. [docs/transitions.md](docs/transitions.md) — compiled command/event
   semantics, causal ordering, and immutable runtime preparation.
18. [docs/movement.md](docs/movement.md) — compiled claim composition,
   shared simulation/navigation movement semantics, and SW-E evidence.
19. [docs/packages.md](docs/packages.md) — atomic package publication, exact
   manifest/member verification, and the revision-5 evidence boundary.
20. [docs/migration.md](docs/migration.md) — stable-v1 to stable-v2 movement
   migration, normalized runtime proof, and digest mapping.
21. [docs/provenance.md](docs/provenance.md) — typed fact identities, resolved
    values, causal inputs, and the boundary between semantics and display text.
22. [docs/runtime.md](docs/runtime.md) — compiler-owned light union, immutable
    runtime snapshots, state hashes, atomic commit, and typed causal receipts.
23. [docs/explanations.md](docs/explanations.md) — package-bound entity and
    transition explanations, stable selection failures, and SW-N evidence.
24. [docs/review/2026-08-21-founding-review.md](docs/review/2026-08-21-founding-review.md)
   — the condensed primary record of the founding adversarial review, written
   in the originating session, with its provenance limits stated.
25. [docs/review/2026-08-21-founding-review-synthesis.md](docs/review/2026-08-21-founding-review-synthesis.md)
    — the contract-revision-2 edited synthesis of that review.

## Layout

```text
README.md          status and reading order
THESIS.md          the design thesis
KERNEL.md          the Gate K acceptance contract
docs/              decisions, evaluation protocols, reviews, workspace notes
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
