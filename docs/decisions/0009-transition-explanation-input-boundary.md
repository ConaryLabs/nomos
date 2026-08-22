---
title: Contract revision 7 — bind transition explanations to verified worlds
status: Owner-authorized; effective when merged
number: 0009
date: 2026-08-22
issue: 62
supersedes_contract_revision: 6
establishes_contract_revision: 7
owner: Peter Permenter
implementing_reviewer: GPT-5.6
---

# Contract revision 7 — bind transition explanations to verified worlds

## Decision authority

The owner authorized the recommended disposition while directing work on issue
#62 on 2026-08-22. This record supplies the amendment required by `AGENTS.md`
before the Gate K command contract changes. Contract revision 7 becomes
effective when this record and its implementation merge; revision 6 remains
effective until then.

## Problem

KERNEL section 8 specified `explain-transition` with only a run directory,
entity, and tick. A verified run bundle records the digest of its input package
and the digest of its simulation semantics, but it deliberately contains
neither a package path nor the package's typed `SimulationPlan`.

The accepted run opener therefore requires both inputs:

```rust
open_run_bundle(root: &Path, world: &OpenedCompiledWorld)
```

The world is needed to reconstruct persisted runtime states, validate legal
entities, namespaces, and machine states, verify the complete simulation
digest, bind the exact package bytes, and re-execute the committed evidence. A
digest proves identity after bytes are supplied; it is not a locator from which
those bytes or semantics can be recovered.

The old command wording could be implemented only by weakening explanation to
an incompletely verified forensic read or by changing the accepted run-bundle
format. Neither consequence was stated by revision 6.

## Amendment A1 — transition-explanation verification and input boundary

### Prior wording

KERNEL section 8 specified:

```text
nomos explain-transition runs/gaol/ north_gate --tick 4
nomos explain-transition runs/gaol/ brazier_02 --tick 7
```

It did not supply the world package required to authenticate and re-execute the
run evidence.

### Replacement wording

KERNEL section 8 specifies:

```text
nomos explain-transition runs/gaol/ north_gate \
  --tick 4 \
  --world build/gaol.world/

nomos explain-transition runs/gaol-seven/ brazier_02 \
  --tick 7 \
  --world build/gaol.world/
```

`explain-transition` first opens and semantically verifies the supplied world,
then strictly opens and re-executes the supplied run against that world before
selecting the requested receipt. Package or simulation identity disagreement,
invalid state semantics, malformed evidence, or failed re-execution fails
closed through the existing stable diagnostic boundary.

The run bundle remains the exact six-file format accepted by SW-J. It does not
embed, copy, or locate a package and does not gain a package-path field.

### Reason

This preserves the strongest already accepted evidence boundary and makes the
missing input explicit at the CLI. It avoids both unauthenticated standalone
interpretation and a format migration unrelated to explanation.

### Effect on existing evidence

The correction is prospective. Existing packages, run bundles, replay logs,
canonical schemas, fixture bytes, digests, hashes, receipts, and completed proof
records do not change. The later explanation implementation will consume the
existing typed package and run evidence through their current strict openers.

No prior command claimed to implement `explain-transition`, so no accepted CLI
output or compatibility surface is removed.

### Owner disposition

Approved: add the explicit `--world <world/>` input and retain fully verified,
package-bound re-execution. A weaker standalone forensic read and a run-bundle
format change are not authorized for Gate K transition explanations.

### New contract revision

7.

## Amendment A2 — tick-seven evidence

### Prior wording

KERNEL section 8 named `brazier_02 --tick 7` under `runs/gaol/`, while the
accepted five-command `fixtures/gaol.commands` and `fixtures/gaol.replay`
execution extinguishes `brazier_02` at tick 5.

### Replacement wording

The tick-7 example uses `runs/gaol-seven/`, a separate valid seven-command run
derived from the same verified world. The primary five-command fixture and its
replay remain unchanged.

### Reason

The distinct run name makes the example reproducible without rewriting accepted
fixture evidence merely to move one already proved transition two ticks later.
It also keeps the tick-7 acceptance example rather than weakening it to the
currently convenient primary-fixture tick.

### Effect on existing evidence

`fixtures/gaol.commands`, `fixtures/gaol.replay`, their package bindings, and all
existing runtime and migration goldens remain byte-for-byte valid. The future
explanation slice supplies separate seven-command evidence and proves the exact
tick-7 output.

### Owner disposition

Approved: the tick-7 example may be demonstrated by a separate valid run and is
not required to replace or mutate the checked-in primary command/replay fixture.

### New contract revision

7.

## Evidence limits preserved

This decision repairs the command input and example-evidence boundary only. It
does not implement either explanation command, weaken strict package or run
opening, add a persisted schema, migrate an artifact, alter canonical bytes,
change diagnostic exit classes, add a dependency, satisfy explanation
acceptance, or launch a formal cold-agent run.
