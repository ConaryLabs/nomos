---
title: Typed forensic provenance
status: Implementation reference for issue 24
date: 2026-08-21
applies_to: KERNEL.md sections 4 and 9; acceptance 3
---

# Typed forensic provenance

`nomos.world_ir.construction@2` replaces the construction-v1 ownership
receipt's display strings with a typed causal record. This is an incompatible
construction-snapshot change, not the assignment of stable
`nomos.world_ir@1`.

Each receipt contains:

- a closed `FactIdentity` for entity identity, spatial anchor, spatial binding,
  credential, or graph relation;
- a closed `ResolvedFactValue` carrying an `EntityId`, `Binding`,
  `CatalogValueId`, or typed relation edge;
- a `ProjectionConsumer` set rather than arbitrary consumer identifiers;
- one or more `DerivationStep` records with a typed producer, compiler pass,
  and typed inputs.

Derivation inputs distinguish references to canonical facts, approved
primitive kinds, and declared catalog values. `WorldIr::new` verifies every
fact edge against the complete receipt set and every primitive/catalog input
against the compiled world. It also verifies that each receipt root names an
actual entity or relation and that each resolved catalog value was declared.
Spatial and credential values must exactly match the corresponding compiled
entity record; carrying the right value variant is not sufficient.
This makes a causal edge directly navigable without
parsing a label such as `binding/typed_lattice`.

Fact collections order by the typed `FactIdentity` value. Derivation steps and
their inputs also use typed canonical order. Human-readable `Display` output is
not an ordering key and cannot move canonical bytes.

The producer and pass vocabularies are closed. Unknown producer IDs fail with
`EK1003`, unknown pass IDs with `EK1004`, and unsupported typed combinations
with `EK1005`. Dangling fact edges fail with `EK1001`; resolved values whose
type or identity conflicts with the fact fail with `EK1002`; missing typed
primitive or catalog inputs fail with `EK1006`; a non-canonical fact owner fails
with `EK1007`. A typed producer/pass pair must also be valid for its fact class,
not merely present in the global vocabulary.

## Structured and readable output

`FactOwnershipReceipt::to_canonical()` exposes the structured causal record
retained in World IR and used by `nomos explain-entity`; it remains available to
a future richer diagnostics projection.
`FactOwnershipReceipt::render_text()` produces human-readable text from those
types. The renderer is deliberately downstream: changing prose cannot change
canonical World IR bytes, fact identities, or causal edges.

Issue #24 supplied the typed semantic input and presentation boundary. SW-N now
renders it only after strict package verification; presentation remains
downstream from canonical meaning and cannot change package bytes.
