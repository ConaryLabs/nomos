---
title: Contract revision 6 — adopt the Nomos identity
status: Owner-authorized; effective when merged
number: 0007
date: 2026-08-21
issue: 31
supersedes_contract_revision: 5
establishes_contract_revision: 6
owner: Peter Permenter
implementing_reviewer: GPT-5.6
---

# Contract revision 6 — adopt the Nomos identity

## Decision authority

The owner authorized issue #31 and directed that the rename land before typed
forensic provenance or stable World IR promotion. This record supplies the
required acceptance repair before implementation changes identity-bearing
paths, schemas, commands, or evidence. Contract revision 6 becomes effective
when this record and its implementation merge; revision 5 remains effective
until then.

## Decision

**Nomos** is the project, runtime, authoring system, CLI, crate namespace, and
active schema namespace. **The Signed World** remains the name of the
architectural thesis that Nomos tests.

The naming hierarchy is:

```text
Nomos             project, runtime, and authoring system
The Signed World  architectural thesis
.nomos            semantic authoring source extension
nomos             CLI binary
nomos-*           Rust crate namespace
nomos.*           active schema namespace
```

The short project description is:

> Nomos — a semantic game runtime designed for AI authors.

## Amendment A1 — project, workspace, CLI, and source identity

### Prior wording

The repository and README used `signed-world` as the project identity. KERNEL
section 10 named six `estate-*` crates, section 8 named an `estate` binary, and
sections 1, 4, and 10 described `.estate` authoring source. Cargo metadata and
the active fixture used the same working identities.

### Replacement wording

The active repository identity is `ConaryLabs/nomos`. The six Gate K crates and
their directories are:

```text
nomos-core
nomos-schema
nomos-projection
nomos-compiler
nomos-sim
nomos-cli
```

Their dependency graph is structurally identical to revision 5. The binary is
`nomos`; the authoring extension is `.nomos`; and the base fixture is
`fixtures/gaol.nomos`. No `estate-*` facade crate, `estate` binary, or `.estate`
compatibility input is introduced.

### Reason

The implementation is still pre-Gate, has no released package or stable World
IR, and has only a small command surface. Delaying the cutover would make a
working identity into an accidental compatibility promise across crates,
schemas, packages, replay, diagnostics, and tooling.

### Effect on existing evidence

The crate graph, language grammar, primitive catalog, compiler behavior,
projection ownership, runtime behavior, and package structure do not change.
Commands and paths in completed receipts remain evidence of the names that
existed when those proofs ran. They are not edited. New proof uses Nomos names.

### Owner disposition

Approved as an identity-only pre-Gate cutover. The Signed World thesis name is
retained.

### New contract revision

6.

## Amendment A2 — active schema namespace and construction epoch

### Prior wording

Revision 5 used active `estate.*` schema identities. The incomplete linker line
had advanced through `estate.world_ir.construction@3`, while the contract
reserved a future stable `estate.world_ir@1` to `estate.world_ir@2` movement
migration.

### Replacement wording

Every active project-owned schema moves to the `nomos.*` namespace, including:

```text
estate.source                         -> nomos.source
estate.world_ir.construction          -> nomos.world_ir.construction
estate.world_ir                       -> nomos.world_ir
estate.projection.simulation          -> nomos.projection.simulation
estate.projection.navigation          -> nomos.projection.navigation
estate.projection.persistence         -> nomos.projection.persistence
estate.projection.diagnostics         -> nomos.projection.diagnostics
estate.runtime_state                  -> nomos.runtime_state
estate.replay_log                     -> nomos.replay_log
estate.package.manifest               -> nomos.package.manifest
estate.hash_domain_fixture             -> nomos.hash_domain_fixture
```

This is an explicit construction epoch break:

```text
closed prototype epoch:
  estate.source@1
  estate.world_ir.construction@1..3

active Nomos epoch:
  nomos.source@1
  nomos.world_ir.construction@1

future stable line:
  nomos.world_ir@1 -> nomos.world_ir@2
```

The Nomos construction schema starts at version 1 because its schema name and
epoch are new. It does not claim byte or schema compatibility with any
`estate.world_ir.construction@N` artifact. No stable `estate.world_ir@1` will be
created.

### Reason

Keeping an obsolete namespace in canonical bytes would make the rename
cosmetic and create permanent cross-project vocabulary. Incrementing the old
construction version would instead imply continuity that this deliberate
identity reset rejects. Schema name plus version, not the version integer by
itself, defines identity.

### Effect on existing evidence

All canonical envelopes containing an active schema ID change bytes and hash.
The prototype construction hashes remain committed and are classified as
historical evidence. The implementation freezes a new Nomos construction-v1
golden hash and a new Nomos hash-domain fixture. The cutover receipt records the
old-to-new relationship; no old receipt or hash is rewritten.

Semantic parser, linker, ownership, transition, resolver, and transaction tests
must continue to pass. Any change beyond identity-bearing bytes is outside this
decision and requires its own disposition.

### Owner disposition

Approved as a pre-Gate construction epoch break, not a compatibility migration.

### New contract revision

6.

## Amendment A3 — provenance and historical naming

### Prior wording

The contract did not distinguish active identity references from historically
accurate occurrences of the former working names.

### Replacement wording

Historical decisions, review transcripts, completed run receipts, command
captures, commit messages, repository URLs, hashes, and paths retain the names
that were true when recorded. Current docs and open future-facing instructions
use Nomos. Every remaining legacy-name match must be classified as historical
evidence or an explicit provenance note; otherwise it is a defect.

GitHub redirects from `ConaryLabs/signed-world` are acceptable for immutable
historical links. Active Cargo and repository metadata use
`https://github.com/ConaryLabs/nomos`.

### Reason

Mass-rewriting history would destroy provenance. Leaving obsolete names in
active instructions would recreate the retired identity. The audit boundary
must distinguish those cases explicitly.

### Effect on existing evidence

Existing evidence remains byte-for-byte unchanged and continues to describe
the prototype epoch honestly. New evidence identifies Nomos and may refer back
to the prototype only through explicit provenance language.

### Owner disposition

Approved. Provenance preservation is part of acceptance, not an exception to
the rename.

### New contract revision

6.

## Evidence limits preserved

This decision changes identity only. It does not promote stable World IR,
define compiler-receipt content, change language grammar or runtime semantics,
add compatibility aliases, implement signing, commit runtime state, replay,
migration, the CLI command surface, renderer work, or formal cold-agent gates.
