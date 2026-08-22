---
title: Gate K package evidence boundary
status: Package implementation reference; current through SW-K
date: 2026-08-22
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
hashes, publication, and filesystem structure. SW-G adds the semantic layer:
`nomos-compiler::CompiledWorld` owns the exact seven members and validates their
schemas and agreement. The read side returns an
`nomos-compiler::OpenedCompiledWorld` only after reconstructing the complete
typed stable IR and regenerating every projection.
`nomos-cli::write_compiled_world` and `compile_and_write_world` complete
assembly and semantic validation before entering the generic writer, and
`open_compiled_world` crosses that same typed semantic boundary after generic
integrity checks. SW-H exposes this orchestration through `compile` and
`inspect`; SW-J consumes the opened result through `run` and `command`. Runtime
causal receipts never enter an input package and live only in separate run
outputs.

## Stable artifacts and ownership

| Member | Schema | Authoritative type owner |
| --- | --- | --- |
| `manifest.json` | `nomos.package.manifest@1` | `nomos-core::package::PackageManifest` |
| `world-ir.json` | `nomos.world_ir@1` | `nomos-schema::StableWorldIr` |
| `simulation.json` | `nomos.projection.simulation@3` | `nomos-projection::SimulationPlan` |
| `navigation.json` | `nomos.projection.navigation@1` | `nomos-projection::NavigationPlan` |
| `persistence.json` | `nomos.projection.persistence@1` | `nomos-projection::PersistencePlan` |
| `diagnostics.json` | `nomos.projection.diagnostics@1` | `nomos-projection::DiagnosticsPlan` |
| `schemas.json` | `nomos.package.schemas@1` | `nomos-schema::SchemaRegistry` |
| `compiler-receipts.json` | `nomos.compiler_receipts@1` | `nomos-compiler::CompilerReceipts` |

The registry has one row for every top-level persisted artifact schema. Source
and construction schema IDs are provenance inside stable IR and compiler
receipts, not separately persisted package artifacts; both are still checked
exactly on semantic open.

This source review enumerates every canonical type newly entering a complete
package. Repository search confirms no second crate defines any of these
schemas; crate edges prevent projection/runtime code from naming IR types.
This began as an SW-G receipt, not the final Gate K ownership receipt. SW-I and
SW-J subsequently stabilized the current run-output types, and SW-K stabilizes
the strict replay-log input; migration schema ownership remains outstanding.

Stable `nomos.world_ir@1` is not construction v3 with a new label. It preserves
the construction schema as provenance, adds explicit compiler/catalog versions,
and carries the contract-required initial v1 movement rows. Its frozen fixture
digest is `555017cf5e13a33b4bb5b18bae14b7577fd1fc38abf89b1f6f475874600fa493`.
The complete fixture package manifest digest is
`f1af0cc92ea44fd09ba93815bb99cc6c24517b56888f39be33a9d47b1299bab7`.

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
Fault-injection tests cover compile, member assembly, semantic validation,
staging write, and staged verification; each proves that its failure occurs
without a published destination. Semantic validation is repeated after open as
defence in depth, but publication never relies on a post-publication semantic
check.
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

`open_compiled_world` then refuses the wrong semantic member set, exact-field
or schema/version mismatches, construction bytes relabelled as stable IR,
incomplete ownership or compiler receipts, and receipt/member digest
disagreement. It strictly reconstructs every stable-IR entity, primitive
expansion, capability, claim, machine transition, interaction, resolver, and
provenance edge through the authoritative Rust types. The reconstructed IR must
re-encode byte-for-byte, so persisted semantic arrays must already have their
declared canonical order rather than becoming valid through decoder sorting.
The compiler also re-derives the sealed primitive expansions, primitive field
shapes, movement/light resolver plans, stable-v1 initial movement, approved
relations, and exact non-positional ownership/derivation edges. It then
regenerates simulation, navigation, persistence, and diagnostics projections
and requires byte-identical package members.

The resulting `OpenedCompiledWorld` carries the typed stable IR, all four typed
projections, exact registry, verified compiler receipts, generic package, and
package digest. Runtime initialization consumes its regenerated simulation
plan; no package consumer decodes an initialization subset independently.
These checks use `EK0411`–`EK0413`. Regression tests mutate nested command and
handler rows, claims, provenance, semantic ordering, and valid IR meaning while
refreshing both the compiler receipt artifact hash and the generic manifest;
semantic open still rejects every case.

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

SHA-256 proves byte identity, not authenticity. Compiler receipts bind exact
source and artifact bytes inside the package, but they are not signatures.
SW-H exposes this boundary through filesystem `validate`, `compile`, and
`inspect` commands, and SW-J consumes it without changing the input package to
write run directories and execute runtime commands. SW-K replays a package-bound
canonical history through the same immutable output boundary. The repository
still does not implement signing, guarantee power-loss durability, migrate, or
satisfy whole-Gate-K acceptance.
