---
title: One projected-activation evaluator — record
status: R1 slice record; acceptance disposition with the owner
date: 2026-08-25
issue: 136
branch: r1/issue-136-activation-evaluator
accepts_against: issue #136 acceptance; RUNTIME.md §3 (revision 1, option (a))
registers: nothing; no new canonical schema identity
applies_to: RUNTIME.md §3
---

# One projected-activation evaluator

## What moved

`ProjectedActivation` had two private evaluators that nothing proved equal:

- `crates/nomos-sim/src/resolver.rs:155` `activation_is_true`, reached by
  `resolve_movement` and `resolve_light`;
- `crates/nomos-compiler/src/projection.rs:620` `projected_activation_is_true`,
  called from `initial_movement_v1` and `initial_movement_v2` at lines 125 and
  165.

`KERNEL.md` §10 permits `nomos-compiler` → core/schema/projection and
`nomos-sim` → core/projection and no edge between them, so the one evaluator now
lives in `nomos-projection` beside the type both crates already consume, at
`crates/nomos-projection/src/movement.rs`. Both private copies are deleted.

## The signature

```rust
pub fn activation_is_true<L>(
    activation: &ProjectedActivation,
    state_equals: &L,
) -> Result<bool, Diagnostic>
where
    L: Fn(&NamespaceId, &Ident) -> Result<bool, Diagnostic>;
```

The state lookup is a closure, not a `&BTreeMap<NamespaceId, Ident>`, because
the two callers do not look up the same thing. The compiler holds exactly such a
map — every machine's authored initial state — and its closure captures it by
reference. The runtime resolver has no map: it reads the machine table of the
`SimulationPlan` *and* the live `SimulationState`, and reports which of the two
is missing. Handing it a `&BTreeMap` would mean materialising one per
`resolve_movement` and `resolve_light` call, which is allocation churn on the
runtime path for no gain; a closure over the two borrows it already holds costs
nothing. The compiler's lookup allocates nothing either — it is a `get` on the
map it already builds.

The lookup owns its diagnostic, which is the second reason for this shape. The
two callers report a missing namespace differently and both spellings are
accepted diagnostics:

| Caller | Code | Message |
| --- | --- | --- |
| `nomos-compiler` | `EK0903` `RESOLVER_ACTIVATION_NAMESPACE_MISSING` | ``initial movement activation namespace `{namespace}` does not exist`` |
| `nomos-sim` | `EK0908` `RESOLVER_RUNTIME_REFERENCE_MISSING` | ``claim activation namespace `{namespace}` is absent from the simulation plan`` |
| `nomos-sim` | `EK0908` `RESOLVER_RUNTIME_REFERENCE_MISSING` | ``claim activation state `{state}` is absent from `{namespace}` `` |
| `nomos-sim` | `EK0908` `RESOLVER_RUNTIME_REFERENCE_MISSING` | ``claim activation namespace `{namespace}` is absent from current state`` |

Unifying that text would change accepted diagnostics, which `RUNTIME.md` §3
option (a) forbids. Each message stays with its caller, byte for byte:
`machine_state_lookup` in `crates/nomos-sim/src/resolver.rs` and
`initial_state_lookup` in `crates/nomos-compiler/src/projection.rs`.

## Where the two evaluators actually differed

They were not byte-identical, and the difference is recorded rather than papered
over. On an empty `any` or `all` group the runtime copy returned `EK0907`
`RESOLVER_PLAN_INVALID`, ``runtime received an empty `{kind}` activation
group``; the compiler copy returned vacuous truth — `false` for empty `any`,
`true` for empty `all`. The shared function keeps the strict runtime behaviour,
message included.

That is unreachable in the compiler and therefore changes no compiler
diagnostic. `initial_movement_v1` and `initial_movement_v2` obtain their plan
from `movement_plan`, which calls `validate_activation`
(`crates/nomos-compiler/src/projection.rs`) on every claim before
`project_activation` builds a `ProjectedActivation`; that validation refuses an
empty group with `EK0907`, "claim activation groups must not be empty". No
compiled world can carry one. The runtime copy checks it anyway because a
package on disk is not necessarily one this compiler produced.

The runtime lookup also verifies that the required state exists in the named
machine before comparing, which the compiler copy never did. That check moved
into `machine_state_lookup` unchanged, so it still applies on the runtime path
and still does not apply on the compile-time path.

## The equivalence test

`crates/nomos-cli/tests/activation_evaluator.rs`, in the one crate that depends
on both compiler and runtime:
`compile_time_initial_movement_equals_the_resolver_at_the_initial_state`
compiles `fixtures/gaol.nomos` and all four `experiments/executable-gaol/areas/*/world.nomos`,
and asserts that every stable-v2 initial movement row equals what
`nomos_sim::resolve_movement` resolves against `SimulationState::initialize` —
same subjects, same blocked/traversable disposition, same effective cost, same
ordered reasons. A second test proves the comparison is not vacuous: every one
of the five worlds carries at least one `state_equals` predicate, so the state
lookup is actually exercised. Inverting the compiler lookup's comparison fails
the first test on `north_gate`.

## Gate K visibility

Nothing Gate K-visible changed. The frozen hash-domain regression in
`crates/nomos-core/tests/determinism.rs` and the golden fixtures run under
`cargo test --workspace --locked`, and every artifact and command output for the
five worlds — 63 files across `validate`, `compile`, `inspect`,
`entity-catalog`, `run`, `explain-entity`, and `effective-facts` — is
byte-identical to the same files produced by the binary built at `origin/main`
(`52296dc`). No canonical schema identity is added, so
`docs/evaluation/R1_SCHEMA_OWNERSHIP.md` is untouched and
`docs/evaluation/r1-schema-ownership.sh` still passes.

## Open point for the owner

`RUNTIME.md` §5 R1-1 is an accepted criterion and it names this function:
"`activation_is_true` (`resolver.rs:155`) staying private so the projected law
flags stay in the path." That line reference no longer resolves, and the
function is now `pub` in `nomos-projection`. Issue #136 dispositions the move
explicitly and calls it a separate slice, so this change follows the issue and
does not edit the accepted §5 text; repairing that wording is an owner decision
under `RUNTIME.md` §8. The criterion's substance is unaffected: the shared
function evaluates one activation node and composes no movement disposition and
no light fact, so effective facts still come only from `resolve_movement` and
`resolve_light`, with the projected-law flags in the path, and R1-1's "must not
add a second implementation of activation evaluation" is strengthened by this
change, not weakened.
