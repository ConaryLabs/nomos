---
title: Gate K package evidence boundary
status: Implementation reference for issue 22 and contract revision 5
date: 2026-08-21
applies_to: KERNEL.md sections 5-7; acceptance 12
---

# Gate K package evidence boundary

Contract revision 5 makes one directory mean one verified package. Its root
contains only regular files:

```text
manifest.json
world-ir.json
simulation.json
navigation.json
persistence.json
diagnostics.json
schemas.json
compiler-receipts.json
```

The generic `nomos-core::package` layer validates names, canonical bytes,
hashes, publication, and filesystem structure. It deliberately does not decide
which semantic members a complete Gate K package requires; later compiler/CLI
orchestration owns that exact artifact set. When that orchestration lands,
`compiler-receipts.json` is a required canonical manifest member. Runtime
causal receipts never enter an input package and live only in run outputs.

## Publication

`WorldPackage::write` performs these steps:

1. refuse an existing destination without following symlinks;
2. reject duplicate names and noncanonical member bytes before disk mutation;
3. create a uniquely named sibling staging directory;
4. write every member and the canonical manifest into staging;
5. reopen and fully verify the staged package;
6. recheck that the destination is absent;
7. publish with one same-filesystem rename.

Any reported failure removes the staging directory and leaves the requested
destination absent. Existing destinations are never merged into or replaced.
The atomicity claim covers the supported local-filesystem, single-publisher
boundary. It is not a claim of crash durability, distributed-filesystem rename
semantics, or coordination with an external process racing to create the same
destination.

## Verification

`WorldPackage::open` refuses unless:

- the root is a real directory rather than a symlink;
- `manifest.json` and every declared member are regular files;
- the manifest, nested schema object, and every member row have exactly their
  declared fields;
- member rows are unique and strictly ordered by canonical member name;
- the package digest matches the canonical manifest body;
- the root contains no undeclared file or directory;
- every member matches its recorded size and SHA-256 digest; and
- every hash-valid member independently parses as canonical bytes.

The reader lexically normalizes trailing separators and inspects entry types
without following existing final-component symlinks before reading. Tests cover
root spellings with and without a trailing separator, manifest and member
symlinks, declared and undeclared directories, canonical unknown fields with
recomputed digests, duplicate and unsorted rows, and noncanonical members whose
hashes were maliciously updated.

The caller must own a quiescent package tree for the duration of `open`; an
external process concurrently replacing entries is outside this path-based
integrity boundary. This verifier does not claim to be a hostile-filesystem
sandbox or authenticity mechanism. That limitation is explicit because SHA-256
binds bytes but supplies neither access control nor provenance.

## Evidence limits

SHA-256 proves byte identity, not authenticity or provenance. This slice does
not define the compiler-receipt schema, assemble a complete world package,
implement signing, guarantee power-loss durability, or satisfy package CLI,
migration, replay, or whole-Gate-K acceptance.
