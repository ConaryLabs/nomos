---
title: Founding adversarial review — edited synthesis and provenance
status: Historical design record; non-authoritative
review_date: 2026-08-20/21
record_revision: 2
supersedes_record_revision: 1
contract_decision: docs/decisions/0001-contract-repair.md
---

# Founding adversarial review — edited synthesis and provenance

## Provenance correction

This file is an **edited synthesis**, not a verbatim transcript.

The first repository version described the review as verbatim with only light
formatting. That was inaccurate: portions of the relayed exchange were condensed,
reconstructed, or normalized before being committed. Contract revision 2
corrects the claim rather than pretending the repository's founding evidence is
cleaner than it is.

The previous long-form synthesis remains recoverable from Git history at commit:

```text
e024b06c935c1320f2f76e2fecbc75af1ec9782d
```

That historical blob is useful for reasoning and sequence, but it must not be
cited as an exact turn-by-turn export or quote-level source. No complete raw
client export is stored in this repository as of this revision.

## Participants

- **Owner and referee:** Peter Permenter.
- **Claude-side reviewer:** the client label recorded during the exchange was
  `Claude Fable 5`, operating through Claude Code. The exact underlying provider
  model identifier was not independently captured in the repository; this is a
  provenance limitation, not an invitation to guess.
- **OpenAI-side reviewer:** GPT-5.6 Pro, operating through ChatGPT, as reported by
  the originating session.

Peter relayed the positions between the two model systems, rejected sunk-cost
reasoning, and made the project-level dispositions.

## Scope of the founding exercise

The question was greenfield and deliberately non-binding:

> If a game runtime were built from scratch for LLMs to author consistently,
> what technologies and approaches would best use their strengths and fence
> their weaknesses?

The exchange carried lessons from prior project experience, so it was not a
clean-room or independent-reimplementation process. It was a vacuum architecture
exercise: existing code and rulings were excluded from the decision, while
observed failure modes remained evidence.

## Synthesis of the review

### Round 1 — renderer as art director

The initial proposal argued that LLMs are better at writing deterministic text
and programs than producing matching raster scenes. It favored a small closed
visual vocabulary, declarative scenes, shader-defined style, plain-file state,
and a renderer that guarantees consistency.

The first counterposition accepted the core thesis but warned against turning a
visual compiler into an unnecessary custom-engine hostage situation. It added:

- semantic scenes rather than raw transforms;
- governed raster rather than a blanket ban;
- material/effect DSLs rather than routine raw shader authoring;
- structural, statistical, perceptual, and multimodal visual proof rather than
  universal byte-identical screenshots;
- voxel-authored, mesh-rendered environments;
- explainable pixels and forensic render passes.

### Vacuum reset — custom runtime all the way down

Peter explicitly removed existing-project sunk costs and asked for the best
architecture in a vacuum.

The custom-runtime position then strengthened:

- Rust plus `wgpu` for native and browser-compatible rendering;
- a typed semantic spatial model;
- fixed-tick deterministic simulation;
- content-addressed artifacts;
- a CLI-first agent surface;
- no raw transforms in content;
- a cold-author test.

The counterposition moved toward the custom runtime but relocated the center:

- the Canonical World IR, not the compiler executable, is the architecture's
  shared truth;
- spatial facts require a lattice while relations and state require a graph;
- semantic primitives must compile across visual, physical, behavioral,
  persistence, network, and diagnostic consequences;
- server-authoritative fixed ticks fit a persistent tactical RPG better than peer
  lockstep;
- one workspace with hard crate boundaries is preferable to one giant crate;
- the project should build one game's runtime, not a general engine.

### Capability layer

The review's load-bearing addition was the capability layer.

Without it, each primitive would independently emit into every subsystem:
`N primitives × M subsystems` bespoke emitters. With a small sealed capability
basis, primitives become bundles and each projection compiler implements its
consequence per capability family.

The wording converged to:

> Capabilities own contracts and obligations. Projection compilers own the
> consequences.

The capability basis was further split into typed namespaces with explicit
composition laws. State remained in independent namespace-local machines rather
than flattened product states.

### Resolver and transaction model

Independent machine states can make overlapping claims. The review therefore
added:

- pure derived interactions that calculate facts without mutating another
  machine's state;
- causal interactions that send typed events to a target machine;
- deterministic phase ordering;
- atomic command transactions;
- composition algebra per capability;
- runtime resolution of effective facts from the complete current state;
- source-mapped causal receipts.

The initial synthesis did not clearly distinguish compile-time claim preparation
from command-time effective-fact resolution. Issue #1 and contract revision 2
made that separation explicit and added cross-capability coherence, including a
single movement disposition rather than contradictory independent answers.

### Versioned and sealed boundaries

The review converged on:

- Canonical World IR versioned from the first commit;
- independently versioned simulation, navigation, render, audio, persistence,
  save, replay, and protocol formats;
- symbolic IDs in source and content addresses in builds;
- immutable world packages and separate mutable runtime state;
- provenance closure and development taints;
- a separate hot-reload development binary that cannot be linked into release;
- no shippable raw-transform escape hatch.

### Proof before machinery

The final practical direction was to build a renderer-free executable semantic
kernel first:

- one door;
- one water region;
- one extinguishable light;
- namespace-local machines;
- capability resolution;
- projections as inspectable JSON;
- deterministic commands and hashes;
- one migration;
- explanation tools;
- mutation tests;
- a cold author and cold debugger.

Separately, a human-approved visual target pack must exist before any renderer or
visual primitive catalog is allowed to grow. Contract revision 2 names the
semantic preflight **Gate K** and keeps the visual target pack as **Gate 0**.

## Durable resolutions

The authoritative current resolutions are in `THESIS.md` section 20. The most
important are:

- Canonical World IR at the center;
- typed lattice plus world graph plus linker;
- primitives over a sealed capability basis;
- capabilities own contracts, projection compilers own consequences;
- namespace-local machines;
- compile-time resolver preparation separated from command-time effective facts;
- cross-capability coherence rather than subsystem improvisation;
- deterministic server-authoritative ticks and zero authoritative client
  prediction for deliberate pulses;
- immutable packages and separate runtime state;
- independent versioned boundaries;
- one Rust workspace with hard dependency edges;
- Gate K before Gate 0, and Gate 0 before renderer work;
- cold-agent and non-author reproduction as proof obligations.

## Limits of this record

- It is not a raw transcript.
- It does not preserve exact wording for every turn.
- The Claude-side exact underlying model identifier is not known from the
  stored evidence.
- It is historical context, not authority over `THESIS.md`, `KERNEL.md`, or an
  adopting project.
- Git history preserves the prior synthesis, including its inaccurate provenance
  label; revision 2 does not rewrite history, only the current claim.

For executable requirements, read `KERNEL.md`. For the owner-authorized repair,
read `docs/decisions/0001-contract-repair.md`.
