---
title: Contract revision 4 — World IR construction lineage
status: Owner-authorized; effective when merged
number: 0004
date: 2026-08-21
issue: 15
supersedes_contract_revision: 3
establishes_contract_revision: 4
owner: Peter Permenter
implementing_reviewer: GPT-5.6
---

# Contract revision 4 — World IR construction lineage

## Decision authority

The owner reviewed and approved the finished replacement wording. Contract
revision 4 becomes effective when PR #16 merges. Revision 3 remains effective
until then.

## Decision

Distinguish incomplete Canonical World IR construction snapshots from the stable
World IR migration line. The repair preserves every first-commit versioning and
compatibility obligation; it does not authorize a package, runtime, or Gate K
acceptance artifact.

## Problem

SW-C emitted `estate.world_ir@1` while the type still lacks required Gate K
transition, interaction, composition, coherence, resolver, projection, and
movement semantics. Adding those fields under the same identity would silently
change a persisted schema. Advancing that incomplete type to version 2 would
consume the version that section 6 reserves for the required movement migration.

The contract therefore contradicts the incremental slice plan: it requires a
stable Canonical World IR from the first commit while also reserving its first
incompatible version change for a representation that cannot exist until later.

## Amendment A1 — distinguish construction snapshots from stable World IR

### Prior wording and evidence

KERNEL section 6 said:

> Version from the first commit:
>
> - Canonical World IR;

THESIS section 12 said:

> The Canonical World IR is constitutional from the first commit:

The six following THESIS bullets require an explicit schema version, canonical
serialization, migration support, fixture coverage, compatibility receipts,
and build failure on silent incompatible change. KERNEL also requires migration
or an epoch break for every incompatible persisted change and reserves stable
`estate.world_ir@1` to `estate.world_ir@2` for the movement representation.

SW-C nevertheless named its deliberately incomplete linker output
`estate.world_ir@1`.

### Replacement

Incomplete pre-Gate snapshots use the separately versioned identity
`estate.world_ir.construction@N`:

- every persisted construction snapshot declares that identity and a version;
- an incompatible construction change increments `N` and requires either a
  migration or an explicit construction epoch break;
- a construction snapshot is canonical build evidence, but is not a valid Gate K
  package, cannot occupy a package's `world-ir.json` member, and cannot satisfy
  the required stable migration;
- every construction snapshot obeys the same first-commit canonicalization,
  versioning, compatibility, fixture, and fail-closed obligations;
- SW-C's parser/linker snapshot is `estate.world_ir.construction@1`;
- SW-D's transition/interaction expansion is
  `estate.world_ir.construction@2`;
- the incompatible SW-C to SW-D change is an explicit construction epoch break;
- stable `estate.world_ir@1` is first assigned to the complete section 4 schema
  with the version-1 movement representation; and
- the required stable `estate.world_ir@1` to `estate.world_ir@2` movement
  migration remains unchanged. Construction versions may not be relabelled as
  stable versions to claim that evidence.

This is an explicit schema-identity correction, not continuity between the
mistaken SW-C name and the future stable schema.

The exact contract replacements are:

> Version from the first commit:
>
> - Canonical World IR, including incomplete construction snapshots;

and:

> The Canonical World IR lineage is constitutional from the first commit:

The accompanying construction rules state that every snapshot obeys the full
list of constitutional requirements, that an incomplete build cannot emit a
valid package, and that every incompatible construction change increments its
version and requires a migration or explicit construction epoch break.

### Reason

Construction artifacts need honest, independently evolving identities while
the schema is still incomplete. Reserving the stable identity until its first
contract-complete shape preserves both first-commit versioning and the exact
migration Gate K is required to prove.

### Effect on existing evidence

The SW-C bytes named `estate.world_ir@1` are invalidated as evidence of the
final stable schema. Their Git history and review receipts remain evidence of
the mistake. The corrected bytes change only the schema identity to
`estate.world_ir.construction@1`.

SW-C evidence for parser behavior, linker ordering, spans, typed symbol tables,
ownership, diagnostics, and repeatability remains valid. No package, runtime,
movement, migration, replay, or Gate K pass depended on the mistaken identity;
none of those proofs existed. Source and projection schema identities, runtime
state, package mechanics, and the frozen core hash fixture are unchanged.

### Evidence limits preserved

This amendment does not weaken any Gate K behavior, determinism, migration,
runtime, package, cold-agent, or platform criterion. It only prevents an
incomplete intermediate artifact from impersonating the stable migration line.

## Owner disposition

On 2026-08-21, Peter Permenter explicitly approved decision 0004 as written and
authorized merging PR #16. The amendment is approved in full.

On merge, this record establishes contract revision 4 and supersedes revision
3. SW-D implementation issue #14 may then proceed and repair issue #15 may
close.

## Related non-normative record maintenance

README, AGENTS, HANDOFF, compiler/workspace documentation, crate descriptions,
and API documentation are updated to label the current output as a construction
snapshot. Before disposition they distinguish effective revision 3 from proposed
revision 4; this authorization advances them to revision 4 effective on merge.
