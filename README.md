# Nomos

**Nomos — a semantic game runtime designed for AI authors.**

This repository contains the executable semantic kernel, **The Signed World**
architectural thesis, and a quarantined executable gaol used to test whether
AI-authored areas can share one coherent visual grammar.

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
  permit self-waiver. Those attempts remain failed, no acceptance tag exists,
  and Gate K is not green. Decision 0015 authorized a separately governed round
  two; its candidate and two formal subjects completed, but neither independent
  checker ran. Decision 0016 terminates round two incomplete with no verdict,
  preserves the unmerged debugger record at annotated tag
  `gate-k-rc2-debug-subject-incomplete`, and authorizes no retry, protocol
  revision 7, or round three. No Gate K work remains active
- **Executable-study status:** the static target pack remains preserved under
  `experiments/gate-0-gaol-target-pack/`. A separate quarantined study under
  `experiments/executable-gaol/` now connects four independently authored areas
  through one projection-only WebGL renderer and one bounded procedural look.
  It is playable online and explicitly authorized to continue by decision
  0016, but remains non-authoritative and satisfies neither Gate K nor Gate 1
- **Post-Gate-K epoch status:** decision 0017 is **owner-authorized** under
  issue #124 and the **R1 epoch is open**: an explicit epoch break, not a Gate K
  pass, with promotion by clean implementation only. Its contract document
  `RUNTIME.md` is now owner-authorized under issue #128 and governs what R1
  accepts. **R1-1, the kernel effective-facts projection (`nomos
  effective-facts`, PR #130), is the first accepted R1 slice**: it meets every
  `RUNTIME.md` §5 R1-1 criterion, and its identity `nomos.effective_facts@1` is
  the first row of the R1 register `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`
- **Contract revision:** 7, owner-authorized in decision 0009
- **Cold-agent protocol revision:** 6, owner-authorized in decision 0015;
  revision-6 packet, boundary, record, adjudication, and finalization tooling is
  implemented under issue #88 and merged in PR #89 at `7744610`. Fresh
  cross-family author/checker and
  debugger/checker rehearsals passed at exact tooling commit `cbfa3f7`, and the
  complete proof plus zero-finding non-author audit passed at `da19239`. Issue
  #90 / PR #91 froze exact candidate `53db236`, whose verify run `32689876814`
  and gate-k-evidence run `32689876846` passed before the annotated
  `gate-k-rc2` tag was created. Revision 6 and the incomplete round-two records
  are now historical evidence; decision 0016 closes the evaluation without
  further checker or audit launches
- **R1 contract status:** [RUNTIME.md](RUNTIME.md) revision 1 is
  **owner-authorized and in force** as of 2026-08-25 under issue #128. It is the
  R1 epoch's own acceptance contract under decision 0017, reconciled with the
  merged ownership audit (#125, PR #129); its section 3 permits read-only R1
  surface inside the kernel crates under stated conditions. R1-1, the kernel
  effective-facts projection prototyped on PR #130, is the first slice under
  acceptance. `KERNEL.md` revision 7 stays frozen and unamended
- **Scope:** greenfield / vacuum architecture exercise
- **Effect on other projects:** none unless separately adopted by an explicit
  decision in that project's own authority records
- **License:** MIT

**Play the quarantined executable gaol:**
<https://conarylabs.github.io/nomos/>

The repository does not claim the thesis is correct. The kernel gives it a
cheap, falsifiable semantic boundary; the executable study tests a narrower
visual proposition without changing the accepted runtime or its formal gate
status.

That falsification happened at the round-one cold-agent boundary. Nomos remains
a useful semantic experiment. The decision-0014 static visual study produced a
compelling owner-approved target from its ordinary gameplay frame and
supporting tests. The executable study now asks whether separately authored
rooms can remain coherent through shared projections, visual assemblies, and
renderer-owned grammar. Decision 0016 terminates the remaining Gate K ceremony
and explicitly permits this quarantined work to continue. It is still not a
Gate K waiver or an accepted renderer architecture.

## Read in this order

First read
[decision 0016](docs/decisions/0016-terminate-gate-k-round-two.md), which
terminates round two, preserves its incomplete record, and names the active
visual direction. The historical proof sequence follows:

1. [docs/decisions/0015-gate-k-round-two.md](docs/decisions/0015-gate-k-round-two.md)
   — the prospective round-two protocol, safeguards, and operating order.
2. [docs/decisions/0013-gate-k-disposition.md](docs/decisions/0013-gate-k-disposition.md)
   — the final owner verdict, exact 1–19 matrix, and project consequence.
3. [docs/decisions/0014-quarantined-gaol-visual-target-experiment.md](docs/decisions/0014-quarantined-gaol-visual-target-experiment.md)
   — the narrow authorization for a non-authoritative static gaol target pack.
4. [experiments/gate-0-gaol-target-pack/TARGET.md](experiments/gate-0-gaol-target-pack/TARGET.md)
   — the static visual invariants, frame intent, provenance, risks, and
   compelling owner disposition for issue #83.
5. [experiments/executable-gaol/README.md](experiments/executable-gaol/README.md)
   — the four-area public visual/playability study, authoring boundary, controls,
   and known limits.
6. [THESIS.md](THESIS.md) — the design, boundaries, proof gates, resolved
   disagreements, and adoption criteria.
7. [KERNEL.md](KERNEL.md) — the revisioned acceptance contract for Gate K, the
   renderer-free executable semantic kernel.
8. [docs/evaluation/GATE_K_FINAL_EVIDENCE_INDEX.json](docs/evaluation/GATE_K_FINAL_EVIDENCE_INDEX.json)
   — the content-addressed final evidence index.
9. [docs/decisions/0012-cold-agent-evidence-authentication.md](docs/decisions/0012-cold-agent-evidence-authentication.md)
   and [docs/decisions/0011-cold-agent-attempt-ledger.md](docs/decisions/0011-cold-agent-attempt-ledger.md)
   — the owner-authorized protocol revisions 4 and 5 for prospective attempt
   reservation and complete evaluation-envelope authentication.
10. [docs/decisions/0009-transition-explanation-input-boundary.md](docs/decisions/0009-transition-explanation-input-boundary.md)
   — the owner-authorized revision-6 to revision-7 repair requiring a verified
   world for transition explanations and separate tick-7 run evidence.
11. [docs/decisions/0010-cold-agent-token-budget.md](docs/decisions/0010-cold-agent-token-budget.md)
   — the owner-authorized cold-agent protocol revision 3 removal of resource
   ceilings while preserving complete usage and command accounting.
12. [docs/decisions/0008-cold-agent-nomos-cli-identity.md](docs/decisions/0008-cold-agent-nomos-cli-identity.md)
   — the owner-authorized cold-agent protocol revision 2 correction from the
   prototype `estate` CLI name to active `nomos`; no tool scope or rubric changes.
13. [docs/decisions/0007-adopt-nomos-identity.md](docs/decisions/0007-adopt-nomos-identity.md)
   — the owner-authorized revision-5 to revision-6 identity cutover: Nomos is
   the project/runtime, The Signed World remains the thesis, and active schemas
   begin a fresh pre-Gate epoch.
14. [docs/decisions/0006-package-evidence-boundary.md](docs/decisions/0006-package-evidence-boundary.md)
   — the owner-authorized revision-4 to revision-5 repair sealing package
   receipts, publication, exact manifest decoding, and filesystem entry types.
15. [docs/decisions/0003-contract-profile-closure.md](docs/decisions/0003-contract-profile-closure.md)
   — the owner-authorized revision-2 to revision-3 closure of the canonical
   profile and workspace-evidence contract gaps.
16. [docs/decisions/0004-world-ir-construction-lineage.md](docs/decisions/0004-world-ir-construction-lineage.md)
   — the owner-authorized revision-3 to revision-4 repair separating incomplete
   construction snapshots from the stable World IR migration line.
17. [docs/decisions/0005-gate-k-dependency-policy.md](docs/decisions/0005-gate-k-dependency-policy.md)
   — the owner-authorized temporary zero-third-party-dependency policy for Gate
   K; it does not amend contract revision 4 or bind later gates.
18. [docs/decisions/0001-contract-repair.md](docs/decisions/0001-contract-repair.md)
   — the owner-authorized revision-1 to revision-2 contract repair.
19. [docs/evaluation/COLD_AGENT_PROTOCOL.md](docs/evaluation/COLD_AGENT_PROTOCOL.md)
   — the reproducible cold-author, cold-debug, and cold-review procedure.
20. [docs/evaluation/GATE_K_COLD_AGENT_PLAN.md](docs/evaluation/GATE_K_COLD_AGENT_PLAN.md)
   — the owner-authorized whole-kernel subject roster and eligibility checks.
21. [docs/workspace.md](docs/workspace.md) — the crate map, how to run the
   proof, and the decisions the first implementation slice had to make.
22. [docs/authoring.md](docs/authoring.md) — source schema version 1 and the
   approved Gate K authoring vocabulary.
23. [docs/compiler.md](docs/compiler.md) — parser/linker stages, schema
   ownership, proof coverage, and limits.
24. [docs/transitions.md](docs/transitions.md) — compiled command/event
   semantics, causal ordering, and immutable runtime preparation.
25. [docs/movement.md](docs/movement.md) — compiled claim composition,
   shared simulation/navigation movement semantics, and SW-E evidence.
26. [docs/packages.md](docs/packages.md) — atomic package publication, exact
   manifest/member verification, and the revision-5 evidence boundary.
27. [docs/migration.md](docs/migration.md) — stable-v1 to stable-v2 movement
   migration, normalized runtime proof, and digest mapping.
28. [docs/provenance.md](docs/provenance.md) — typed fact identities, resolved
    values, causal inputs, and the boundary between semantics and display text.
29. [docs/runtime.md](docs/runtime.md) — compiler-owned light union, immutable
    runtime snapshots, state hashes, atomic commit, and typed causal receipts.
30. [docs/explanations.md](docs/explanations.md) — package-bound entity and
    transition explanations, stable selection failures, and SW-N evidence.
31. [docs/review/2026-08-21-founding-review.md](docs/review/2026-08-21-founding-review.md)
   — the condensed primary record of the founding adversarial review, written
   in the originating session, with its provenance limits stated.
32. [docs/review/2026-08-21-founding-review-synthesis.md](docs/review/2026-08-21-founding-review-synthesis.md)
    — the contract-revision-2 edited synthesis of that review.

## Layout

```text
README.md          status and reading order
THESIS.md          the design thesis
KERNEL.md          the Gate K acceptance contract
docs/              decisions, evaluation protocols, reviews, workspace notes
experiments/       quarantined target-pack and executable-gaol studies
fixtures/          exact Gate K authoring source, command, and replay fixtures
crates/            the six Gate K kernel crates named in KERNEL.md section 10
xtask/             workspace tooling; `cargo xtask boundary`
.github/workflows/ the verification lane
```

The kernel crates have no third-party dependencies: `Cargo.lock` holds seven
local entries and nothing else, so the workspace builds and tests offline.

The repository deliberately retains a large `docs/evaluation/runs/` archive.
It contains immutable formal/rehearsal packets, transcripts, receipts, and
exact binaries used by the evidence records; it is historical evidence, not an
accidental build cache, and should not be pruned as routine cleanup.

## Start here if you are new to the tree

[docs/HANDOFF.md](docs/HANDOFF.md) — current state, what is next, how to prove it.

## Gate order

- **Gate K:** prove the semantic kernel without graphics.
- **Gate 0:** obtain a human-approved visual target pack.
- **Gate 1 and later:** prove cross-system primitives, vocabulary, cold authors,
  cold debugging, and clean rebuilds.

Gate K is closed as failed and round two is terminated incomplete. The
executable gaol is an owner-authorized quarantined feasibility study, not a
passed gate. Visual progress does not rewrite the Gate K verdict.
