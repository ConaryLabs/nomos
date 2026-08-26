---
title: RUNTIME.md revision 4 — repair the owner-disposition history
status: Owner-authorized; RUNTIME.md revision 4 in force
number: 0021
date: 2026-08-26
owner: Peter Permenter
issue: 184
supersedes_runtime_revision: 3
establishes_runtime_revision: 4
references:
  - docs/decisions/0018-runtime-revision-2.md
  - docs/decisions/0019-r1-final-disposition.md
  - docs/decisions/0020-runtime-revision-3.md
---

# RUNTIME.md revision 4 — repair the owner-disposition history

## Decision authority

Peter Permenter authorized this record on 2026-08-26. It replaces exactly the
stale section 10 wording quoted below, updates the contract metadata, and
establishes `RUNTIME.md` revision 4. No acceptance criterion, implementation
requirement, evidence requirement, or prior verdict changes.

## Prior wording

`RUNTIME.md` revision 3, section 10 says:

```text
## 10. Owner disposition

**Authorize.** Recorded by Peter Permenter on 2026-08-25. Revision 1 takes
effect as written, with section 3 resolved to option (a): kernel crates may gain
read-only R1 surface under the conditions stated there. No further amendments.
R1-1 is the first slice under acceptance.
```

## Replacement wording

Replace section 10 with:

```text
## 10. Owner disposition and revision history

Peter Permenter authorized revision 1 on 2026-08-25, with section 3 resolved to
option (a): kernel crates may gain read-only R1 surface under the conditions
stated there. R1-1 was the first slice under acceptance. Owner-authorized
decision 0018 established revision 2, and owner-authorized decision 0020
established revision 3.

Decision 0019 accepted all five R1 criteria and closed the epoch as this
repository's runtime baseline without authorizing game adoption. Decision 0021
repairs this revision history and establishes revision 4; it changes no
criterion, implementation, evidence, or verdict. No further R1 implementation
slice is authorized by this contract. Any later contract amendment must follow
section 8, and any later runtime epoch, capability family, or game adoption
requires a new owner decision under decision 0019's consequences.
```

The frontmatter changes `status` and `contract_revision` to revision 4, records
this decision as `revision_4_authority`, and records decision 0019 as the final
disposition. No other wording changes.

## Reason

Revision 3's metadata correctly says revision 3 is in force, but its original
revision-1 disposition footer still says “Revision 1 takes effect,” “No further
amendments,” and “R1-1 is the first slice under acceptance.” Decisions 0018 and
0020 had already established revisions 2 and 3, and decision 0019 had already
accepted all five criteria and closed R1. The footer therefore contradicts the
same document's metadata and the controlling owner records.

A cold reader following the mandated reading order found the contradiction
while auditing the fresh-machine handoff. Leaving it in place would make the
active contract ambiguous about both its revision and whether R1 work remains.
The replacement preserves the original revision-1 disposition as history and
then records the later authorized state explicitly.

## Effect on existing evidence

No implementation, schema identity, source, package, runtime state, compiled
artifact, digest, budget, workflow receipt, or acceptance criterion changes.
Decision 0019 remains the final verdict on its exact revision-3 candidate and
SHA-256; revision 4 does not relabel or rerun that candidate. It records the
already-authorized revision and disposition history around unchanged criteria.

Gate K remains failed under decision 0013, round two remains terminated
incomplete under decision 0016, and decision 0019's no-adoption and no-later-
epoch boundaries remain in force. Existing R1 proof remains evidence for every
unchanged criterion. This repair requires only document consistency checks,
the ordinary workspace proof, and an exact-head non-author audit.

## Owner disposition

**Authorize.** Peter Permenter authorizes the quoted replacement and establishes
`RUNTIME.md` revision 4. This repair closes a contradictory historical footer,
changes no accepted criterion or observed evidence, weakens nothing, and
authorizes no new implementation slice, Gate K attempt, later runtime epoch,
capability family, or game adoption.
