---
title: Contract revision 5 — package evidence boundary
status: Owner-authorized; effective when merged
number: 0006
date: 2026-08-21
issue: 22
supersedes_contract_revision: 4
establishes_contract_revision: 5
owner: Peter Permenter
implementing_reviewer: GPT-5.6
---

# Contract revision 5 — package evidence boundary

## Decision authority

The owner accepted issue #22 at its recorded slice boundary, then authorized
proceeding with that slice on 2026-08-21. This record supplies the explicit
contract repair that #22 requires before package implementation can continue.
Contract revision 5 becomes effective when this record and its implementation
merge; revision 4 remains effective until then.

## Problem

KERNEL section 5 names `receipts/` inside the immutable world-package layout,
while the same section places runtime causal receipts in separate run outputs.
The SW-B writer created the package directory but excluded it from the manifest,
and the verifier deliberately ignored it. Anything placed there could change
without changing the package digest.

That contradicts the claim that a verified package has one exact immutable
structure. It also leaves two plausible authorities for runtime receipts.

## Amendment A1 — one authority for compiler and runtime receipts

### Prior wording

KERNEL section 5 specified:

```text
build/gaol.world/
  manifest.json
  world-ir.json
  simulation.json
  navigation.json
  persistence.json
  diagnostics.json
  schemas.json
  receipts/
```

It separately specified `runs/<run>/causal-receipts.json` for runtime output.
The contract did not say whether the package directory held compiler receipts,
runtime receipts, or both, and did not say whether its contents were hashed.

### Replacement wording

Gate K world packages contain only regular files at their root. The example
layout replaces `receipts/` with:

```text
  compiler-receipts.json
```

`manifest.json` declares and hashes every other package entry, including
`compiler-receipts.json`. Compiler, linker, validation, and invariant receipts
that belong to a compiled build live in that canonical member. Runtime causal
receipts live only in the separately versioned run output
`causal-receipts.json`; commands, runs, and replays never write receipts into an
input world package. No unmanifested directory or file is permitted inside a
verified package.

### Reason

Build receipts describe how the immutable package was produced and therefore
belong inside its hash boundary. Runtime receipts describe execution against
that input and therefore belong beside mutable run state. A single canonical
member is inspectable with ordinary tools and avoids inventing recursive
directory-manifest semantics for Gate K.

### Effect on existing evidence

No accepted Gate K package exists. SW-B package tests prove canonical member
hashing, basic tamper detection, and refusal to overwrite an existing output;
those proofs remain useful but did not establish a complete package boundary.

The empty unmanifested `receipts/` directory emitted by SW-B is invalidated as
package-layout evidence and is removed. Historical Git and rerun receipts
remain evidence of what SW-B actually tested. Canonical member bytes, SHA-256,
source/IR/projection schemas, runtime semantics, and the frozen core hash fixture
are unchanged. The manifest body shape remains `estate.package.manifest@1`;
this repair tightens structural validation without changing its encoded fields.

### Owner disposition

Approved as the issue #22 repair: compiler/build receipts are canonical hashed
package members, runtime causal receipts remain separate run artifacts, and the
unverified subtree is removed.

### New contract revision

5.

## Amendment A2 — exact publication and verification boundary

### Prior wording

KERNEL section 5 said that a compiled package is immutable evidence and that
commands and migrations never modify it in place. It did not define how a new
directory becomes visible or which filesystem entry types are legal.

### Replacement wording

A package writer validates all inputs, writes a complete package to a fresh
sibling staging directory on the same filesystem, verifies that staged package,
and publishes it with one rename. A failed write removes its staging directory
and leaves the requested destination absent. An existing destination is never
replaced or merged into. Atomicity is claimed only for the documented supported
local-filesystem, single-publisher boundary.

On open, the manifest, nested schema object, and member rows have exact field
sets. Member rows are unique and strictly ordered by member name. Every declared
member is a regular file whose bytes are canonical and match the recorded size
and digest. The package root contains only the regular `manifest.json` plus its
declared regular member files; symlinks, special files, and undeclared files or
directories are refused.

### Reason

Sequential writes into the destination can strand a partial path that the
immutability rule then refuses to replace. Hash checking alone also does not
prove that decoded structure was exact or that filesystem traversal did not
cross an undeclared entry type. Publication and verification must cover the
same structure the package claims as evidence.

### Effect on existing evidence

Existing successful package bytes and manifest digests remain unchanged. Tests
that relied on the ignored `receipts/` directory change under amendment A1.
New failure-injection and tamper tests extend the evidence; they do not upgrade
the current construction snapshots into valid stable World IR packages.

### Owner disposition

Approved as the fail-closed publication and read boundary required by issue #22.

### New contract revision

5.

## Evidence limits preserved

This repair does not implement package orchestration, stable World IR,
compiler-receipt schema content, signing/authenticity, runtime commit, replay,
migration, CLI commands, aarch64 evidence, the ten-run matrix, or formal cold
agents. It makes the generic directory boundary safe enough for those later
slices to rely on; it does not claim any of them complete.
