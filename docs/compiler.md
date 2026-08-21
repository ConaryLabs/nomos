---
title: Gate K source compiler
status: Implementation reference for SW-C
date: 2026-08-21
applies_to: KERNEL.md sections 1, 2, 4, 9; acceptance 1-3 and 11
---

# Gate K source compiler

SW-C turns one `estate.source@1` file into typed Canonical World IR. The public
library path is:

```rust
estate_compiler::compile_source(source, repository_relative_path)
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
   templates, and typed claim activation expressions.
7. Emit stable fact-ownership receipts and Canonical World IR bytes.

Parser failures use `EK05xx`; linker and ownership failures use `EK06xx`.
Every source rejection carries a repository-relative span and a legal repair
class. The mutation suite plants each ownership/cross-reference violation in
`KERNEL.md` section 9 that belongs to this slice.

## Schema ownership

`estate-schema` exclusively defines:

- source AST types;
- typed cell, face, and region bindings;
- Canonical World IR entities and relations;
- machine and claim templates;
- fact-ownership receipts.

`estate-compiler` exclusively defines parsing, the sealed primitive catalog,
name resolution, validation, expansion, and linking. Runtime crates cannot name
the source or IR types because the workspace graph denies them an
`estate-schema` edge.

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
- relevant ownership and cross-reference mutations fail with stable codes and
  source spans.

## Still unproved

The CLI remains intentionally unimplemented. Acceptance item 3's observable
`estate inspect` command requires a complete package, including projections;
SW-C proves its expansion/IR half but does not call the criterion satisfied.
Issue #5 records that scope split.

Typed interactions, resolver plans, composition/coherence rules, projections,
runtime transactions, package compilation, migration, replay, explanations,
and cold-agent gates belong to later slices. Canonical World IR will grow those
contracted fields before Gate K can pass.
