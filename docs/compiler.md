---
title: Gate K compiler
status: Implementation reference through SW-E
date: 2026-08-21
applies_to: KERNEL.md sections 1, 2, 4, 9; acceptance 1-3 and 11
---

# Gate K source compiler

SW-C turns one `estate.source@1` file into a typed Canonical World IR
construction snapshot. SW-D validates its executable machine semantics and
projects a runtime-only simulation plan. SW-E adds compiler-owned ground
movement composition and a shared resolver projected to simulation and
navigation. The public library paths are:

```rust
estate_compiler::compile_source(source, repository_relative_path)
estate_compiler::compile_simulation_plan(&world_ir)
estate_compiler::compile_navigation_plan(&world_ir)
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
9. Emit stable fact-ownership receipts and canonical construction-snapshot
   bytes under `estate.world_ir.construction@3`.
10. Resolve entity credentials into command requirements and emit
    `estate.projection.simulation@2` plus `estate.projection.navigation@1` with
    byte-identical movement resolver plans. Simulation advances independently
    because its required shape changes; navigation is emitted for the first time.

Parser failures use `EK05xx`; linker and ownership failures use `EK06xx`;
transition/projection validation uses `EK07xx`; movement resolver validation
uses `EK09xx`.
Every source rejection carries a repository-relative span and a legal repair
class. The mutation suite plants each ownership/cross-reference violation in
`KERNEL.md` section 9 that belongs to this slice.

## Schema ownership

`estate-schema` exclusively defines:

- source AST types;
- typed cell, face, and region bindings;
- Canonical World IR entities and relations;
- machine and claim templates;
- typed command/event transitions and phased causal interactions;
- typed movement composition, coherence, connectivity, and resolver subjects;
- fact-ownership receipts.

`estate-compiler` exclusively defines parsing, the sealed primitive catalog,
name resolution, validation, expansion, linking, cycle rejection, and
projection. `estate-projection` owns the runtime-facing plan types and cannot
name `estate-schema`. Runtime crates cannot name source or IR types because the
workspace graph denies them an `estate-schema` edge.

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
- a frozen SHA-256 fixture pins every construction-v2 canonical byte, so a
  shape change without a schema-version change fails the build;
- relevant ownership and cross-reference mutations fail with stable codes and
  source spans;
- the exact six external commands and internal damage handler are catalog
  output, not runtime inventions;
- `combustion.on_enter(burning)` projects to one causal-phase fire-damage edge;
- reversed plan insertion produces identical canonical projection bytes;
- dangling machine/state/handler references and causal cycles fail closed.
- simulation and navigation receive the same typed ground resolver bytes;
- dangling claim activations, invalid claim values, mismatched connectivity,
  duplicate resolver identities, and absent subjects fail closed.

## Still unproved

The CLI remains intentionally unimplemented. Acceptance item 3's observable
`estate inspect` command requires a complete package, including projections;
SW-C proves its expansion/IR half but does not call the criterion satisfied.
Issue #5 records that scope split.

SW-E resolves only effective ground movement facts. It does not resolve light,
commit snapshots, hash state, write packages, replay, migrate, or implement
explanations/CLI commands. Persistence and diagnostics projection schema names
remain planned ownership only; `produced_schemas()` reports construction IR,
simulation, and navigation as the artifacts actually implemented. Each
incompatible incomplete shape increments the
`estate.world_ir.construction@N` version. Stable `estate.world_ir@1` begins only
when the contracted schema is complete.
