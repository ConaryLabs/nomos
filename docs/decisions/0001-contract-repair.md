---
title: Contract revision 2 — pre-code reconciliation
status: Owner-authorized for implementation; effective when merged
number: 0001
date: 2026-08-21
issue: 1
supersedes_contract_revision: 1
establishes_contract_revision: 2
owner: Peter Permenter
implementing_reviewer: GPT-5.6 Pro
---

# Contract revision 2 — pre-code reconciliation

## Decision

Repair the founding thesis and executable-kernel contract before the first Rust
implementation. Contract revision 2 replaces ambiguous or contradictory wording
in revision 1 while preserving the design thesis, proof-first intent, and
non-authoritative status.

This is contract repair, not an implementation success waiver. No kernel code or
acceptance evidence exists yet. The revised contract becomes effective on merge.

## Owner disposition

Peter Permenter authorized implementation of SW-A after reviewing issue #1 and
instructed the implementing reviewer to proceed. Merge remains the owner's final
acceptance action.

## Why a revision was required

Revision 1 was coherent as a thesis but not fully executable as a contract. It
would have forced the first implementation agent to choose silently among
conflicting interpretations, including:

- whether a key and magical seal were undeclared fourth objects;
- whether capability resolution happened at compile time or command time;
- whether final subsystem deltas could be precomputed from local transitions;
- how simulation and navigation should handle simultaneous block/traverse facts;
- whether a command mutated the signed world package;
- what exactly entered deterministic hashes;
- whether runtime subsystems consumed Canonical World IR or derived projections;
- whether Gate 0 preceded the renderer-free kernel;
- how cold-agent claims could be reproduced;
- whether the founding review was actually verbatim;
- whether “clean-room” accurately described a discussion that used prior lessons;
- whether the contract could repair its own contradictions;
- what the `~1,000 lines` criterion measured.

An architecture designed to reject silent interpretation cannot begin by
requiring it.

## Amendment procedure established by this decision

Future contract correction is permitted only when it repairs ambiguity,
contradiction, impossibility, or a falsified assumption. Each correction records:

1. prior wording or behavior;
2. replacement wording or behavior;
3. reason;
4. effect on existing evidence;
5. owner disposition;
6. new contract revision.

Weakening a criterion merely because an implementation failed it is prohibited.
A failed proof remains failed unless the owner explicitly changes the thesis and
accepts the resulting loss of comparability.

## Amendments

### A1. Canonical base fixture

**Prior:** one door, water region, and light were declared, while examples also
used `gaoler_key` and a magical seal without defining whether they were entities
or primitives. The later cold-author criterion also added a second door without
stating whether that expanded the primitive catalog.

**Replacement:** the base fixture contains exactly three world primitive
instances across three primitive kinds: `north_gate`, `flooded_section`, and
`brazier_02`. `credential/gaoler_key` is a catalog value in a separate typed
symbol table. The second blocking claim is the door-local `ward` machine. The
formal cold-author run uses an isolated fixture copy and may add a second door
instance of the already-approved door kind; it does not add a fourth primitive
kind or expand Gate K's semantic scope.

**Reason:** remove dangling references, the implicit magical-seal primitive, and
the instance-versus-kind ambiguity in the cold-author proof.

**Evidence effect:** none; no implementation evidence existed.

### A2. Water and light proof obligations

**Prior:** most acceptance criteria exercised only the door.

**Replacement:** water must prove lattice binding, traversal cost, projection
agreement, and explanation. The light must prove a local state transition,
effective emission change, persistence/diagnostic updates, and causal receipt.

**Reason:** three declared primitive kinds must be proof subjects rather than
decor.

**Evidence effect:** none.

### A3. Two resolution phases

**Prior:** “capability resolution” appeared both before machine compilation and
after runtime transitions.

**Replacement:** compile time prepares claim templates, machines, interaction
edges, composition laws, coherence rules, and a resolver plan. Command time
resolves effective facts from the full current state.

**Reason:** separate static preparation from dynamic truth.

**Evidence effect:** none.

### A4. Transition-local precomputation

**Prior:** subsystem deltas were described as precomputed per transition.

**Replacement:** the compiler may precompute affected facts and dependency sets.
Final deltas are derived after command-time effective-fact resolution.

**Reason:** opening a door may leave movement blocked when a ward remains active.

**Evidence effect:** none.

### A5. Cross-capability coherence

**Prior:** per-capability algebra did not prevent simultaneous blocking and
traversal answers.

**Replacement:** mutually exclusive movement semantics resolve to one composite
`MovementDisposition<channel>` with source-mapped reasons. Unresolved
contradiction fails closed.

**Reason:** simulation and navigation cannot independently choose which fact wins.

**Evidence effect:** none.

### A6. Immutable package and separate state

**Prior:** CLI examples could be read as mutating `build/gaol.world`.

**Replacement:** compiled packages are immutable directory artifacts. Commands,
runs, replays, and migrations write separate versioned outputs. Migration never
overwrites its input.

**Reason:** the input package must remain reproducible evidence.

**Evidence effect:** none.

### A7. Determinism scope

**Prior:** “same hashes across ten runs and two machines” left encoding, hash
domain, ordering, overflow, and environment unspecified.

**Replacement:** Gate K defines a canonical UTF-8 JSON profile, SHA-256 state
hash, included/excluded fields, unambiguous object/collection ordering, checked
arithmetic, pinned Rust/dependency requirements, an initial target matrix, and
RNG stream isolation.

**Reason:** determinism claims require an exact byte and execution contract.

**Evidence effect:** none.

### A8. IR and projection ownership

**Prior:** the thesis said every subsystem consumed Canonical World IR while also
defining derived projection schemas.

**Replacement:** projection compilers consume Canonical World IR. Runtime
subsystems consume only their versioned projection artifacts. No runtime
subsystem reparses authoring source.

**Reason:** preserve one semantic authority without coupling every runtime crate
to the constitutional schema.

**Evidence effect:** none.

### A9. Independent versioned boundaries

**Prior:** the IR was constitutional, but implementation obligations for derived
formats were not explicit.

**Replacement:** source, Canonical World IR, each projection, runtime state,
replay/log, manifest, save, and later network/render/audio formats version
independently. Incompatible change requires migration or recorded epoch break.
Gate K defines one real v1-to-v2 movement migration.

**Reason:** a renderer-only change must not force a save or protocol migration.

**Evidence effect:** none.

### A10. Gate order

**Prior:** `KERNEL.md` called the semantic kernel the first artifact while the
thesis called the visual target pack Gate 0.

**Replacement:** Gate K is a renderer-free semantic preflight that may precede
Gate 0. Gate 0 remains mandatory before any renderer, visual catalog, or Gate 1
implementation.

**Reason:** permit cheap semantic falsification without circular visual design.

**Evidence effect:** none.

### A11. Cold-agent protocol

**Prior:** cold author/debug requirements named the goal but not a reproducible
procedure.

**Replacement:** `docs/evaluation/COLD_AGENT_PROTOCOL.md` defines model-family
separation, blind packets, tools, source restrictions, cycle budgets, human
intervention, evidence capture, and pass/fail rules.

**Reason:** “a stranger succeeded” is meaningless without the conditions.

**Evidence effect:** no cold-agent evidence existed.

### A12. Founding-review provenance

**Prior:** the founding review file claimed to be verbatim with only light
formatting, but portions were materially condensed or reconstructed.

**Replacement:** it is labeled an edited synthesis. The record states what model
identifiers are known, what is not known, and where the previous synthesis
remains in Git history.

**Reason:** provenance-focused work cannot mislabel its own founding evidence.

**Evidence effect:** the reasoning remains usable; quote-level or turn-level
claims from the old synthesis are not treated as raw evidence.

### A13. Greenfield terminology

**Prior:** the architecture exercise was called clean-room despite explicitly
carrying lessons from prior work.

**Replacement:** the exercise is greenfield / vacuum. “Clean” remains a term for
controlled provenance zones only.

**Reason:** separate architectural freedom from independent-reimplementation or
lineage claims.

**Evidence effect:** none.

### A14. Quarantined experiments

**Prior:** temporary workarounds and experiments were categorically forbidden.

**Replacement:** disposable experiments are allowed under `experiments/` or an
explicit experimental branch. They cannot satisfy acceptance or enter the
accepted kernel without clean promotion/rewrite.

**Reason:** research needs cheap falsification; absolute bans encourage agents to
rename experiments as final architecture.

**Evidence effect:** none.

### A15. Workspace and line-count rule

**Prior:** `~1,000 lines` was undefined, and `AGENTS.md` pointed to a thesis
section that did not actually define the workspace.

**Replacement:** the line-count criterion is removed. `KERNEL.md` directly
defines `estate-core`, `estate-schema`, `estate-projection`, `estate-compiler`,
`estate-sim`, and `estate-cli`, plus permitted dependency edges and forbidden
reachability.

**Reason:** structural boundaries and measured budgets are evidence; crate
confetti is not.

**Evidence effect:** none.

## Open questions recorded rather than silently decided

Revision 2 records the following in thesis section 21:

- stable identity across rename/move/split/merge;
- recurring event and scheduler semantics;
- production canonical encoding beyond Gate K;
- additional cross-capability invariants;
- final source-language choice;
- incremental-compilation granularity;
- migration identity and content-version compatibility;
- cold-model roster and rotation;
- signature threat model and trust roots;
- deterministic parallelism;
- save compatibility across changed primitive semantics;
- fixed-point precision;
- rendering projection details;
- final project name.

Recording an open question is not permission for an implementation agent to
choose invisibly. The responsible slice must decide it with evidence or return
to the owner.

## Implementation scope

This decision authorizes SW-A only:

- revise `README.md`, `AGENTS.md`, `THESIS.md`, and `KERNEL.md`;
- correct founding-review provenance;
- add this decision record;
- add the cold-agent protocol;
- open a draft pull request linked to issue #1.

It does not authorize Rust implementation, renderer work, `wgpu`, networking,
audio, an asset pipeline, or a merge to `main`.

## Existing evidence

- Thesis reasoning: preserved in revision 2 and the resolved-disagreements table.
- Kernel proof: none; status remains not started.
- Determinism proof: none.
- Cold-author/debug proof: none.
- Visual target pack: none in this repository.
- Founding transcript: no raw export is stored; the prior edited synthesis
  remains recoverable from commit `e024b06c935c1320f2f76e2fecbc75af1ec9782d`.

Nothing is retroactively called green by this repair.
