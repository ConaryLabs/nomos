---
title: Gate K compiler
status: Implementation reference through SW-F
date: 2026-08-21
applies_to: KERNEL.md sections 1, 2, 4, 9; acceptance 1-3 and 11
---

# Gate K source compiler

SW-C turns one `nomos.source@1` file into a typed Canonical World IR
construction snapshot. SW-D validates executable machine semantics, SW-E adds
ground movement, and SW-F adds compiler-owned light union plus implemented
persistence and diagnostics projections. The public library paths are:

```rust
nomos_compiler::compile_source(source, repository_relative_path)
nomos_compiler::compile_simulation_plan(&world_ir)
nomos_compiler::compile_navigation_plan(&world_ir)
nomos_compiler::compile_persistence_plan(&world_ir)
nomos_compiler::compile_diagnostics_plan(&world_ir)
```

The source language and primitive catalog are documented in
[`docs/authoring.md`](authoring.md); [decision 0002](decisions/0002-source-language.md)
records why that language was chosen.

## Compile stages implemented

1. Parse the schema declaration, catalog declarations, primitive instances,
   typed lattice bindings, catalog references, and graph relations.
2. Build distinct catalog-value and entity symbol tables; duplicates fail.
3. Resolve relation endpoints and primitive catalog references.
4. Reject facts content cannot own: lattice relations, raw transforms, derived
   facts, and canonical-owner declarations.
5. Enforce primitive-specific field and binding shapes.
6. Expand the sealed three-kind catalog into capability bundles, typed machine
   templates, typed transitions, causal interactions, and claim activation
   expressions.
7. Validate transition states, interaction namespaces and handlers, and reject
   every causal cycle before projection.
8. Derive typed ground connectivity and an explicit resolver plan: any active
   blocker wins; otherwise the maximum active traversal cost wins; otherwise
   the positive base cost is `1`.
9. Emit typed fact-ownership receipts and canonical construction-snapshot
   bytes under `nomos.world_ir.construction@3`. Version 2 introduced typed
   provenance; version 3 adds the incompatible light-resolver shape.
10. Resolve entity credentials into command requirements and emit
    `nomos.projection.simulation@3` plus `nomos.projection.navigation@1` with
    byte-identical movement resolver plans. Simulation advances independently
    because its required shape changes; navigation is emitted for the first time.
11. Validate `EmitsLight = union`, positive claim values, activation
    dependencies, subjects, and consumers; project one typed light plan to
    simulation, `nomos.projection.persistence@1`, and
    `nomos.projection.diagnostics@1`.

Parser failures use `EK05xx`; linker and ownership failures use `EK06xx`;
transition/projection validation uses `EK07xx`; movement resolver validation
uses `EK09xx`; typed provenance validation uses `EK10xx`.
Every source rejection carries a repository-relative span and a legal repair
class. The mutation suite plants each ownership/cross-reference violation in
`KERNEL.md` section 9 that belongs to this slice.

## Schema ownership

`nomos-schema` exclusively defines:

- source AST types;
- typed cell, face, and region bindings;
- Canonical World IR entities and relations;
- machine and claim templates;
- typed command/event transitions and phased causal interactions;
- typed movement composition, coherence, connectivity, and resolver subjects;
- typed light-union composition, consumers, and resolver subjects;
- typed fact-ownership receipts: fact IDs, resolved values, projection
  consumers, derivation producers/passes, and causal inputs.

`nomos-compiler` exclusively defines parsing, the sealed primitive catalog,
name resolution, validation, expansion, linking, cycle rejection, and
projection. `nomos-projection` owns runtime-facing simulation, navigation,
persistence, and diagnostics plan types and cannot name `nomos-schema`.
Runtime crates cannot name source or IR types because the
workspace graph denies them a `nomos-schema` edge.

`NamespaceId` means an entity-local semantic namespace. State machines occupy
such namespaces, but static claims do too (`flooded_section.region`). Treating
every namespace as a machine would require fake state machines in the IR, so
SW-C corrected the narrower SW-B naming before it became serialized behavior.

## What SW-C proves

- the exact three-instance fixture is readable on one screen;
- the credential remains a typed catalog value, never a fourth entity;
- primitive references resolve only through the approved three-kind catalog;
- the door, water, and light expand into inspectable typed IR;
- source maps and ownership receipts survive canonical encoding;
- repeated compilation of the same bytes produces identical canonical IR;
- frozen SHA-256 fixtures preserve Nomos construction-v1 and pin every active
  construction-v2 and construction-v3 canonical byte, so a
  shape change without a schema-version change fails the build;
- relevant ownership and cross-reference mutations fail with stable codes and
  source spans;
- the exact six external commands and internal damage handler are catalog
  output, not runtime inventions;
- `combustion.on_enter(burning)` projects to one causal-phase fire-damage edge;
- reversed plan insertion produces identical canonical projection bytes;
- dangling machine/state/handler references and causal cycles fail closed.
- simulation and navigation receive the same typed ground resolver bytes;
- simulation, persistence, and diagnostics receive the same typed light
  resolver bytes;
- dangling claim activations, invalid claim values, mismatched connectivity,
  duplicate resolver identities, and absent subjects fail closed.
- dangling provenance fact edges, unknown producer/pass IDs, unsupported
  producer/pass pairs, and incompatible resolved-value classes fail closed.
- structured provenance and human-readable explanation rendering are separate
  outputs; display wording is not canonical semantics.

## Still unproved

The CLI remains intentionally unimplemented. Acceptance item 3's observable
`nomos inspect` command requires a complete package, including projections;
SW-C proves its expansion/IR half but does not call the criterion satisfied.
Issue #5 records that scope split.

Typed provenance prepares the data needed by `explain-entity`, but the CLI
surface itself remains unimplemented. SW-F resolves light and emits persistence
and diagnostics, but it does not write packages, replay, migrate, or implement
explanations/filesystem CLI commands. `produced_schemas()` now reports the five
actual compiler artifacts. Each incompatible incomplete shape increments the
`nomos.world_ir.construction@N` version. Stable `nomos.world_ir@1` begins only
when the contracted schema is complete.
