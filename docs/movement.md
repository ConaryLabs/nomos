---
title: Gate K ground movement resolution
status: Movement implementation reference; current through SW-J
date: 2026-08-22
applies_to: KERNEL.md sections 1, 2, 4; acceptance 5
---

# Gate K ground movement resolution

SW-E proves that effective movement is composed from compiler-owned semantics,
not hard-coded door behavior in the runtime. The boundary is:

```text
nomos-schema WorldIr construction@3 (Nomos epoch)
  -> nomos-compiler resolver validation and projection
  -> nomos-projection shared MovementResolverPlan@1
  -> nomos-sim command-time effective facts
```

The same resolver value is embedded in `nomos.projection.simulation@3` and
`nomos.projection.navigation@1`. A canonical-byte equality test prevents those
two consumers from acquiring different movement laws.

## Composition and coherence

For the `ground` channel, the construction snapshot explicitly records:

1. blocking claims compose by **any active blocker**;
2. traversal costs compose by **maximum active cost**;
3. blocking is resolved before cost;
4. every subject carries compiler-derived lattice connectivity;
5. no active cost uses the positive base cost `1`.

Claims retain stable `ClaimRef` identities, typed `Always`, `StateEquals`,
`Any`, `All`, and `Not` activation expressions, typed boolean/cost values, and
source spans. The compiler rejects dangling namespaces or states, wrong value
kinds, absent/cross-entity subjects, invalid connectivity, duplicate
law/coherence/subject identities, and zero costs. The runtime also validates
the projected law flags and fails closed if a referenced machine or state is
missing instead of treating the activation as false.

`MovementDisposition::Blocked` contains a sorted, nonempty list of every active
blocking claim. `MovementDisposition::Traversable` contains the positive
maximum active cost and the sorted claims that supplied that maximum; its
reason list is empty only when the base cost applies.

## Transaction timing

`prepare_transaction` resolves movement once against the input state and once
after all local and causal transitions settle. Both immutable fact sets are
available on `PreparedTransaction`. The input state remains unchanged, and any
command, causal, or resolver failure discards the staged clone without exposing
partial facts.

## Exact fixture evidence

The compiled `fixtures/gaol.nomos` integration proof observes:

| State change | `north_gate` result |
| --- | --- |
| initial | blocked by `portal` and `ward` |
| unlock, then open | blocked by `ward` |
| unseal after open | traversable at base cost `1` |
| ignite, causing target-owned destruction | blocked by `ward` |

`flooded_section` is always traversable at cost `3`, supplied by its region
claim. These results are computed from the projected activation expressions;
`nomos-sim` contains no door, ward, water, or primitive-catalog branch.

## Evidence boundary

SW-E's resolver remains ground-movement-only. SW-F now carries its before/after
facts into committed typed receipts alongside light facts; the movement plan
and navigation schema do not change. SW-G packages those artifacts, SW-H
exposes their filesystem validation, compilation, and inspection, and SW-J
executes runtime commands into verified run bundles. Replay, migration,
explanations, and the multi-target/formal cold-agent evidence remain open.
