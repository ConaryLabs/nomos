---
title: Gate K authoring source language
status: Implementation decision for SW-C; accepted on merge
number: 0002
date: 2026-08-21
issue: 5
contract_revision: 2
---

# Gate K authoring source language

## Decision

Gate K uses a small line-oriented `.estate` language with explicit declaration
terminators and whitespace-separated typed values. SW-C implements source
schema version `estate.source@1` exactly as documented in `docs/authoring.md`.

This resolves the open source-language question in `THESIS.md` section 21. It
does not amend or weaken `KERNEL.md`; it chooses a representation for the
already-required source facts.

## Why this language

The Gate K authoring surface has three jobs: remain readable on one screen,
preserve source spans without lossy conversion, and prevent content from
supplying derived facts. A purpose-built grammar makes those boundaries visible
and rejects forbidden concepts by name.

JSON was rejected because its object shape would expose compiler/schema
structure as authoring ceremony and would make repeated declarations difficult
to diagnose before map-key collapse. YAML and TOML were rejected because adding
a third-party parser would break the zero-third-party, offline workspace, while
implementing a partial lookalike would create misleading edge semantics. Rust
or another general scripting language was rejected because arbitrary code is a
contractual non-goal.

## Grammar properties

- one declaration or field per line;
- comments and blank lines are ignored;
- every file declares its source schema;
- entity, primitive-kind, and catalog-value references are distinct types;
- relation kinds come from a sealed vocabulary (`owns` in Gate K);
- primitive parameters name the catalog namespace they accept;
- lattice bindings use signed integer cells plus closed direction names;
- entity bodies end with `end`; indentation is presentation only;
- unknown statements fail closed;
- raw transforms, derived facts, content-authored fact owners, and lattice
  relations receive dedicated stable diagnostics rather than becoming unknown
  extension points.

## Consequences

The parser is intentionally small enough to audit and ship without a parser
generator. Language growth requires explicit grammar documentation and tests;
there is no generic property bag for features to smuggle themselves through.
The source schema versions independently of Canonical World IR and projection
schemas, as required by `KERNEL.md` section 6.
