---
title: Package-bound semantic explanations
status: SW-N implementation and evidence reference
date: 2026-08-22
applies_to: KERNEL.md sections 8 and 9; acceptance 14
---

# Package-bound semantic explanations

SW-N implements the two Gate K read-only explanation commands:

```text
nomos explain-entity <world/> <entity>
nomos explain-transition <run/> <entity> --tick <tick> --world <world/>
```

They render canonical JSON from reconstructed typed meaning. They do not parse
source, decode a second semantic representation, publish an artifact, or edit an
input tree.

## Entity explanation

`explain-entity` strictly opens the complete compiled world before selecting an
entity. Its report includes:

- the source-mapped linked entity and approved primitive expansion;
- independent namespace-machine definitions, claim templates, and interactions;
- active initial claim reasons and effective movement/light facts;
- typed fact-ownership receipts, resolved values, projection consumers, and
  derivation steps;
- the verified package digest and relevant source, construction, stable IR,
  package-member, projection, and runtime-state schema identities.

The fixture reports remain semantically distinct: `north_gate` explains two
independent initial blockers, `flooded_section` explains traversal cost `3`, and
`brazier_02` explains positive light emission.

## Transition explanation

`explain-transition` performs the decision-0009 sequence exactly:

1. strictly open and semantically reconstruct `--world`;
2. strictly open the exact six-file run bundle against that world;
3. re-execute every committed request and require byte-identical states, logs,
   receipts, and hashes through the existing opener;
4. select the receipt by resulting tick and verify that the requested entity
   owns either the initiating command or an ordered transition step;
5. render the unresolved request, resolved command, ordered local/causal steps,
   typed causes, active claims added/removed, complete effective facts before and
   after, projection deltas, source mapping, tick, and resulting state hash.

The accepted `fixtures/gaol.commands` and `fixtures/gaol.replay` stay unchanged.
They prove `north_gate` at tick 4. `fixtures/gaol-seven.commands` is separate
seven-command evidence that proves `brazier_02` at tick 7.

## Stable selection failures

The strict package/run openers retain all existing integrity, semantics,
re-execution, symlink, and special-file diagnostics. SW-N adds only three report
selection codes after those boundaries pass:

```text
EK0825  well-formed entity is absent from the verified world
EK0826  requested tick is absent from the committed run prefix
EK0827  requested entity is unrelated to the selected transition
```

Malformed CLI grammar remains `EK0001` with exit `2`; semantic rejection is exit
`1`; environment/I/O failure remains exit `3`.

## Evidence and limits

Focused subprocess tests hash the exact canonical stdout for all three entity
reports and both required transition reports. They also cover argument order and
arity, non-UTF-8 arguments, invalid/absent/unrelated entities, absent ticks,
wrong packages/semantics, forged evidence, forbidden entry types, root symlinks,
and byte-for-byte input immutability.

SW-N adds no persisted schema, package member, run member, package locator,
dependency, alternate runtime path, or contract change. It does not satisfy the
remaining execution matrix, budget, final schema-ownership, or formal cold-agent
gates.
