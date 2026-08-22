---
title: Gate K compiler
status: Compiler implementation reference; current through SW-M
date: 2026-08-22
applies_to: KERNEL.md sections 1, 2, 4, 9; acceptance 1-3 and 11
---

# Gate K source compiler

SW-C turns one `nomos.source@1` file into a typed Canonical World IR
construction snapshot. SW-D validates executable machine semantics, SW-E adds
ground movement, SW-F adds compiler-owned light union plus all four projections,
and SW-G promotes the complete schema into stable World IR before projection.
The public library paths are:

```rust
nomos_compiler::compile_source(source, repository_relative_path)
nomos_compiler::promote_world_ir(&construction_ir)
nomos_compiler::compile_world(source, repository_relative_path)
nomos_compiler::compile_simulation_plan(&stable_world_ir)
nomos_compiler::compile_navigation_plan(&stable_world_ir)
nomos_compiler::compile_persistence_plan(&stable_world_ir)
nomos_compiler::compile_diagnostics_plan(&stable_world_ir)
nomos_compiler::compile_world_package(source, repository_relative_path)
nomos_compiler::migrate_world_ir_v1_to_v2(&legacy_stable_world_ir)
nomos_compiler::migrate_world_package_v1(legacy_package_root)
nomos_cli::compile_and_write_world(source, repository_relative_path, destination)
nomos_cli::migrate_and_write_world(legacy_package_root, destination)
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
12. Validate the complete construction snapshot, derive one tagged initial
    `movement_disposition_ground` row per subject, and promote a distinct
    `nomos.world_ir@2` carrying explicit source, construction, compiler,
    primitive-catalog, ownership, and provenance versions.
13. Compile every projection only from the stable type, emit the exact schema
    ownership registry and typed compiler receipts, and validate the complete
    seven-member semantic package set before filesystem publication.
14. On the isolated migration path, strictly validate a complete stable-v1
    package, regenerate and compare every legacy projection and receipt, convert
    only its movement rows to v2, then regenerate and validate all active
    package members.

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
- stable `nomos.world_ir@2`, the migration-only strict v1 type, their movement
  rows, and the package schema registry with typed authoritative owners.

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
- construction-v3 bytes remain frozen while stable-v1 and stable-v2 receive
  independent golden hashes; construction evidence is never relabelled into
  the stable line;
- the public projection APIs accept `StableWorldIr`, not construction IR;
- complete package members, build receipts, schema ownership, and initial state
  reproduce byte-for-byte across clean compilation.

## Slice history and remaining work

At SW-C, the CLI was intentionally unimplemented: acceptance item 3's
observable `nomos inspect` command required the complete package added by later
slices. Issue #5 records that historical scope split. SW-H now exposes
`validate`, `compile`, and `inspect`; SW-J adds `run` and `command` over the
verified package boundary; SW-K adds `replay`, and SW-M adds strict `migrate`
without exposing legacy execution through ordinary runtime commands.

Typed provenance prepares the data needed by `explain-entity`, but the CLI
explanation surface remains unimplemented pending owner disposition of issue
#62.
`produced_schemas()` reports construction evidence, stable IR, all
four projections, the registry, and compiler receipts. Construction versions
remain preserved evidence; stable `nomos.world_ir@1` is frozen as migration
input while new compilation emits only `nomos.world_ir@2`.
