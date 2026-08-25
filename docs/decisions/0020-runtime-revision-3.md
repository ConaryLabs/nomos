---
title: RUNTIME.md revision 3 — align the R1-1 comparison count
status: Draft; no owner disposition; does not take effect
number: 0020
date: 2026-08-26
owner: Peter Permenter
issue: 176
supersedes_runtime_revision: 2
proposes_runtime_revision: 3
references:
  - docs/decisions/0017-post-gate-k-runtime-epoch.md
  - docs/decisions/0018-runtime-revision-2.md
---

# RUNTIME.md revision 3 — align the R1-1 comparison count

## Draft boundary

This record is prepared under issue #176 for owner review. It is not yet an
owner decision, does not amend `RUNTIME.md`, and does not establish revision 3.
Revision 2 remains in force until the owner explicitly authorizes the
replacement below and the authorized record and contract change land together.

## Prior wording

`RUNTIME.md` revision 2, section 5, R1-1's `Accepted when` list says:

```text
- `experiments/executable-gaol/compare-effective-facts.sh` reports
  `20 scenarios compared, 0 differences` against the committed
  `rendering-plan.example.json` blocks, with the `"cost": null` spelling on a
  blocked subject the only normalization;
```

The same revision's section 3 accepted-surface summary instead says that the
harness reports `30 scenarios compared, 0 differences` — the original twenty
plus the ten scenarios from the two cold-authored areas. The executable proof
on the frozen candidate reports thirty.

## Proposed replacement wording

Replace only that R1-1 bullet with:

```text
- `experiments/executable-gaol/compare-effective-facts.sh` reports
  `30 scenarios compared, 0 differences` against the committed
  `rendering-plan.example.json` blocks — the original twenty scenarios plus
  the ten from the two cold-authored areas in the quarantined experiment —
  with the `"cost": null` spelling on a blocked subject the only
  normalization;
```

No other acceptance wording changes.

## Reason

Revision 2 contradicts itself about one exact observed count. The original
R1-1 evidence covered four areas and twenty scenarios. The accepted
cold-authoring work added two areas and ten scenarios, and PR #169 repaired the
comparison harness for the current rendering-plan shape and proved all thirty
with zero differences. Its `RUNTIME.md` update correctly recorded that expanded
result in section 3 but left the normative section 5 bullet at twenty.

Thirty is stronger coverage, but that does not authorize silently treating an
exact criterion saying twenty as if it said thirty. This repair makes the
criterion name the evidence the accepted tree actually produces. It does not
weaken a semantic requirement or excuse a failed implementation.

## Effect on existing evidence

No implementation, schema identity, source, package, runtime state, rendering
plan, viewer artifact, existing evidence hash, or recorded output changes. In
particular:

- `nomos.effective_facts@2` and decision 0018's schema-spelling repair are
  unchanged;
- the original twenty-scenario R1-1 receipt remains historical evidence for
  the four-area subset and is not relabelled;
- PR #169's exact-head non-author receipt remains the evidence for the expanded
  thirty-scenario comparison; and
- the frozen combined candidate's implementation tree requires no change.

After the authorized text lands, decision 0019 must bind `RUNTIME.md` revision
3 and its new SHA-256 and must obtain a fresh complete proof on the resulting
candidate before recording an R1 pass.

## Owner disposition

**Pending.** Recommended authorization:

> Authorize the quoted replacement and establish `RUNTIME.md` revision 3. The
> repair aligns the stale R1-1 count with the already recorded thirty-scenario
> comparison proof, changes no implementation or evidence, weakens no semantic
> requirement, and authorizes no game adoption, Gate K retry, or later runtime
> epoch.
