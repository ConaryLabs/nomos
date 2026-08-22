---
title: Thesis reference — forensics and resolved disagreements
status: Companion to THESIS.md revision 2
date: 2026-08-21
---

# Thesis reference — forensics and resolved disagreements

This companion holds the detailed reference material for `THESIS.md` sections
19 and 20. It is kept separately for readability; guiding documents are not
subject to the repository's approximately 1,000-line code-organisation rule,
and the extraction changes no thesis claim.

## Section 19 details — forensics, explanation, and the visual proof ladder

The runtime should answer semantic “why” questions. The following prototype-era
spellings preserve the vocabulary of thesis revision 2; active Nomos acceptance
and command grammar are governed by `KERNEL.md` and the effective decisions:

```text
estate explain-entity north_gate
estate explain-transition north_gate --tick 4
estate why-blocked guard_04 --destination cell(8,4)
estate why-visible thief_02 --to guard_04
estate trace-event combat_18842
estate explain-material wet_keep_stone
estate explain-save north_gate
estate explain-replication goblin_17
estate explain-pixel scene/guard_post 612 344
```

A result cites source, primitive, compiler passes, generated IDs, active machine
state, claims, effective facts, rules fired, dependencies, tick, and event
sequence.

Conventional tools expose implementation state. This exposes semantic causality.

### Render contact sheet

Every formal render produces:

- beauty;
- neutral-lit;
- silhouette;
- material ID;
- entity ID;
- depth;
- normals;
- light-only;
- navigation;
- collision;
- annotated warnings.

### Visual proof ladder

1. Structural assertions.
2. Image statistics.
3. Perceptual comparison on a pinned runner.
4. Multimodal review over the contact sheet.

Byte-identical pixels are required only where a pinned deterministic runner can
actually promise them. Across arbitrary machines, they are fantasy bureaucrat
bait. Goldens are receipts. They are not taste.

## Section 20 ledger — resolved disagreements

| Position A | Position B | Resolution | Why |
| --- | --- | --- | --- |
| Browser/WebGL + TypeScript as engine center | Godot + content compiler | Rust + `wgpu` custom runtime; browser Workbench later | The agent-facing language is the product; backend popularity is the wrong boundary. |
| Zero raster | Governed raster | Governed raster | Ungoverned production caused drift; the style compiler is the fence. |
| Byte-identical screenshots everywhere | Tiered visual proof | Tiered ladder | Structure, statistics, pinned perceptual comparison, then multimodal review. |
| Compiler is the engine | Canonical IR is the center | IR at the center | Prevents the compiler from becoming a renderer, server, and small municipality. |
| One grid for everything | Lattice plus graph | Lattice + graph + linker | Spatial and relational facts need different owners. |
| Per-primitive subsystem emitters | Sealed capability basis | Capabilities | N primitives × M systems is a very long switch statement. |
| Capabilities own consequences | Projection compilers own consequences | Capabilities own contracts; projections own consequences | Keeps capability definitions semantic and testable. |
| Flat capability bag | Typed namespaces and algebra | Namespaces + composition | Prevents type-correct gibberish and ambiguous winners. |
| Per-capability algebra is sufficient | Cross-capability coherence required | Composite effective facts/invariants | Simulation and navigation cannot choose different truths. |
| One machine per entity | One machine per namespace | Namespace-local machines | Avoids product-state explosion. |
| Precompute final deltas per transition | Resolve from whole current state | Precompute dependencies; resolve effective facts at command time | A ward can keep a door blocked after `open`. |
| Raw transforms discouraged | Raw transforms impossible in content | Removed; tainted laboratory only | An escape hatch an agent can reach will be reached at 2 a.m. |
| Broad procedural generation | Derivation / bounded variation / synthesis split | Derivation encouraged, variation sparing, synthesis forbidden | Deterministic geometry is compilation; “decorate freely” is oatmeal. |
| Peer lockstep | Server-authoritative fixed tick | Server-authoritative | Persistent game, one authoritative clock. |
| Narrow prediction | Zero authoritative prediction | Zero for deliberate pulses | No second clock with an opinion about the goblin. |
| One universal IR for save/replay/network | Versioned derived boundaries | Independent schemas | The constitution does not regulate sewer pipes. |
| Mutable `.world` command target | Immutable package + separate state | Immutable evidence | Replays and migrations need the original input intact. |
| One crate | Hard-bounded workspace | Workspace | Build graph enforces ownership and keeps renderer out of server/sim. |
| `~1,000 lines` acceptance | Structural boundaries and measured budgets | Remove line quota | Line quotas encourage crate confetti, not architecture. |
| Gate 0 before every executable artifact | Semantic kernel first | Gate K before Gate 0; Gate 0 before renderer | Cheap semantic falsification without allowing visual machinery to start. |
| Founding record is verbatim | It was condensed | Edited synthesis with explicit provenance | A provenance-focused project cannot mislabel its own founding evidence. |
| “Clean-room” architecture exercise | Prior lessons were imported | Greenfield / vacuum | Clean zones remain a provenance concept, not a false historical claim. |
| Hot reload behind a flag | Separate development binary | Separate binary, proven unlinked | A flag is still a release code path. |
