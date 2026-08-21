---
title: Gate K workspace
status: Implementation reference for SW-B
date: 2026-08-21
applies_to: KERNEL.md sections 5, 7, 8, 10; acceptance 12, 15, 16
---

# Gate K workspace

`KERNEL.md` section 10 defines the crates and the permitted dependency edges.
This document says where they live and how to run the proof. Contract revision
3 pins the implementation choices SW-B originally had to make.

## Crate map

```text
crates/nomos-core        stable IDs, canonical bytes, hashing, checked
                          arithmetic, diagnostics, world packages
crates/nomos-schema      authoring source and Canonical World IR construction
                          schemas
crates/nomos-projection  simulation/navigation/persistence/diagnostics
                          projection schemas
crates/nomos-compiler    parse, link, expand, validate, migrate, project
crates/nomos-sim         runtime state, command transactions, replay,
                          effective-fact resolution
crates/nomos-cli         the `nomos` command surface and orchestration
xtask                     workspace tooling; the boundary checker
```

Edges, verbatim from section 10:

```text
nomos-schema      -> nomos-core
nomos-projection  -> nomos-core
nomos-compiler    -> nomos-core, nomos-schema, nomos-projection
nomos-sim         -> nomos-core, nomos-projection
nomos-cli         -> nomos-core, nomos-compiler, nomos-sim, nomos-projection
```

No crate in the workspace has a third-party dependency. `Cargo.lock` contains
seven entries, all of them local. Decision 0005 makes that a deliberate,
temporary Gate K constraint: it protects the offline proof and audit surface
while the semantic kernel is small, but it is not a permanent repository
constitution. Later gates may admit a dependency only through a separate
owner-authorized decision and the review criteria in that record.

## Running the proof

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

`.github/workflows/verify.yml` runs exactly these, plus a determinism step and
disk/time budgets. To run the determinism step locally:

```bash
hashes() {
  cargo test --locked -p nomos-core --test determinism "$@" \
    -- --nocapture --test-threads=1 | grep '^HASH ' | sort
}
hashes > /tmp/debug.txt
hashes --release > /tmp/release.txt
diff -u /tmp/debug.txt /tmp/release.txt
```

## The boundary checker

```bash
cargo xtask boundary [--manifest-path <path/to/Cargo.toml>]
```

Exit codes follow `KERNEL.md` section 8: `0` clean, `1` violations, `2` invalid
usage, `3` environment failure. It reads `cargo metadata --all-features` and
enforces five rules:

| Rule | What it refuses |
| --- | --- |
| `membership` | a workspace member that is not a section 10 kernel crate or declared tooling; a kernel crate that has gone missing |
| `permitted-edges` | any edge between workspace crates that section 10 does not permit, including dev-dependency edges |
| `cycles` | a dependency cycle among kernel crates — including the dev-dependency cycles that Cargo itself allows |
| `forbidden-dependency` | a renderer, windowing, engine, audio, networking, watcher, or hot-reload crate anywhere transitively reachable from a kernel crate |
| `tooling-isolation` | `xtask` reaching a kernel crate |

`--manifest-path` points the check at a copy of the workspace, which is how the
planted-violation receipt is produced without disturbing this one.

### What the checker cannot see

Section 10 also forbids canonical schema types from being defined in more than
one crate. That is a property of the source, not of the dependency graph, and
`cargo metadata` cannot see it. The Canonical World IR construction type is
defined in `nomos-schema`; every crate that must not see it lacks the edge that
would let it. The checker proves the missing edges. Local schema-ID uniqueness
tests, compile-fail boundary doctests, and the compiler crossing test support
the explicit source-review receipt required by revision 3. None of those checks
is claimed as a semantic-uniqueness proof by itself.

## Contract choices first implemented by SW-B

### `xtask` is a seventh workspace member

Contract revision 3 declares six kernel crates plus `xtask`. `xtask`
builds no kernel artifact and is not reachable from any kernel crate. It exists
as a separate member for one reason: the boundary checker must not sit inside
the graph it checks. As a subcommand of `nomos-cli` its own dependencies would
be inside the kernel graph, and the forbidden list would need exceptions carved
for the checker — precisely the shape of hole this project is trying not to
have. The `tooling-isolation` rule proves the separation holds, and the
`membership` rule means an eighth member cannot be added quietly.

### The world package writer lives in `nomos-core`

A package is a directory of named canonical byte members with a hashed
manifest. That is canonical bytes and hashing, which is `nomos-core`'s charter.
It also has to be reachable from more crates than `nomos-projection` is:
`world-ir.json` and `schemas.json` are `nomos-schema` artifacts, and
`nomos-schema` may depend only on `nomos-core`. Putting the writer in
`nomos-projection` would either strand the schema crate or require an edge
section 10 forbids.

The module knows nothing about member *meaning*. It enforces names, bytes,
hashes, and immutability. What `simulation.json` must contain stays in
`nomos-projection`.

Contract revision 5 closes the directory boundary around that generic layer.
Writes use a verified sibling staging directory and one same-filesystem rename;
reads require exact manifest shapes, ordered unique rows, canonical members,
and regular-file-only roots without symlinks or undeclared subtrees. Compiler
receipts become the hashed `compiler-receipts.json` member, while runtime causal
receipts remain separate run artifacts. [`packages.md`](packages.md) records the
supported local-filesystem/single-publisher limit and the evidence still absent.

### SHA-256 in-crate under the Gate K dependency policy

The state hash is the constitutional identity of authoritative state, and
`docs/thesis-open-questions.md` records the signature threat model as open.
Implementing SHA-256 in `nomos-core` — 110 lines, proved against the published
FIPS 180-4 vectors including the multi-block and padding-boundary cases —
removes third-party code from the hash domain entirely, and lets the whole
workspace build and test with no network access, which the cold-agent protocol
depends on. If the owner prefers the `sha2` crate, it is a contained swap: the
`sha256` function is the only thing that would change, and the frozen fixture
hash would prove the swap changed nothing. Such a swap is forbidden during Gate
K under decision 0005. The local implementation is the current integrity/hash
mechanism; it is not evidence that package signing or adversarial cryptography
has been solved.

### Identifiers are ASCII, and that is how NFC is satisfied

Revision 3 requires stable identifier segments and canonical object
field names to match `[a-z][a-z0-9_]*`. Every character in that ASCII set is
NFC-invariant, so a validated name is normalized by construction without
carrying versioned Unicode tables into the hash domain. Composite IDs add only
their schema-declared separators between validated segments.

This fails closed rather than silently: both spellings of a composed character
are refused, so nothing unnormalised can enter an artifact. String *values*
carry any UTF-8 and are emitted as UTF-8, never as escapes. When the kernel
needs non-ASCII identifiers it needs a real NFC implementation and an owner
decision; it does not need this rule relaxed quietly.

### Escape spelling

Revision 3 pins the spelling first implemented by SW-B: `\b \f \n \r
\t` for those five control code points and `\u00xx` with lowercase hex for every
other code point below `U+0020`. `\/` is never emitted and is refused on read.
`U+007F` is not a JSON control character and is emitted raw.

### The strict reader is defined by the encoder

`parse_canonical` parses, re-encodes, and compares bytes. Anything the profile
forbids changes the re-encoded bytes and is refused. The consequence worth
knowing: the reader cannot drift away from the encoder, because it *is* the
encoder plus a comparison.

One asymmetry follows from it. The byte profile has a single spelling for an
integer, so the reader returns `Int` for values that fit `i64` and `Uint` above
that. Round-trip equality is guaranteed on bytes, not on the Rust variant.
Decoders read integers through a helper that accepts either.

### The frozen fixture is not the runtime-state schema

`crates/nomos-core/tests/golden/hash-domain-fixture.json` and the hash literal
in `tests/determinism.rs` are frozen forever. The fixture is *shaped* like the
Gate K initial state so it exercises entity ordering, namespace ordering,
integer costs, and a catalog reference — but it carries its own schema name. If
it tracked the real runtime-state schema, a legitimate schema change would look
exactly like a canonicalisation regression, and the test would stop meaning
anything.

## Budgets

Acceptance 16 requires measured numbers. SW-B measures build time and target
size; validation latency, command latency, and replay throughput have nothing to
measure yet and are recorded as not-applicable rather than as passing. The CI
`Budgets` step prints `du -sh target` and `df -h` on every run.

## Not proved by SW-B

- The section 7 execution matrix. CI proves x86_64 debug and release; **Linux
  aarch64 release is unproved**, and the ten-runs-per-target count is not
  implemented. Gate K is not green until both exist.
- Everything from `nomos validate` onward: section 8's command surface, the
  v1-to-v2 migration, the mutation suite, and the cold-agent gates.
