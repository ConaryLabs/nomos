---
title: Cold-agent protocol revision 3 — account for resources without ceilings
status: Owner-authorized; effective when merged
number: 0010
date: 2026-08-23
supersedes_protocol_revision: 2
establishes_protocol_revision: 3
owner: Peter Permenter
implementing_reviewer: GPT-5.6
issue: 70
---

# Cold-agent protocol revision 3 — account for resources without ceilings

## Decision authority

The owner directed that the cold-agent token budget need not remain as low as
64,000 while reviewing the first non-formal Opus author rehearsal on 2026-08-23.
After higher-token trials and a false diagnostic-cycle rejection, the owner
directed that independent runs have no token, turn, or tool-use ceilings.
Resource use remains measured and may inform the predeclared model or effort
level of a later run. This record supplies the separate owner-authorized
amendment required by issue #70. Protocol revision 3 becomes effective when this
record and its tooling merge; revision 2 remains effective until then.

## Prior wording

`docs/evaluation/COLD_AGENT_PROTOCOL.md` revision 2 set these default ceilings:

```text
provider-reported total tokens      64,000
assistant turns                     40
validation/compile cycles           12
cold-debug diagnostic CLI cycles    12
```

The issue #70 packet plan, verifier, and runner encoded and enforced those
ceilings.

## Replacement wording

Tokens, assistant turns, tool calls, and exact ordered commands are recorded
without a resource ceiling. Resource use does not terminate or fail an otherwise
valid run. One fresh session, zero substantive hints, and zero operator retries
remain binding because they define independence rather than cost.

## Reason

Pi reports complete input, output, and cache usage on every assistant response.
The harness correctly sums those provider totals across turns. A non-formal
Opus author rehearsal therefore crossed 64,000 after nine turns even though it
had made the requested content edit and reached successful validation and
compilation. The cap measured repeated conversation context more aggressively
than task complexity or operator cost control.

A later 1,000,000-token trial confirmed that even the small author checker could
legitimately report 800,996 cumulative tokens while independently reproducing
the package. The debug rehearsal then exposed the deeper flaw in tool-cycle
enforcement: it had executed eleven diagnostic calls, but reproduction commands
quoted inside an evidence here-document made the pre-execution text scanner
predict twenty-one. The harness rejected evidence-writing syntax rather than
observed tool use.

Complete transcripts and ordered commands already make resource use auditable.
Hard ceilings added parser and termination failure modes without strengthening
the causal claim. If observed use becomes unreasonable, the owner can change a
later run's predeclared model or effort level.

## Effect on existing evidence

No formal Gate K cold-author or cold-debug attempt has begun, so no formal result
is reclassified and the formal attempt counts remain zero.

The two non-formal Opus author transports bound to commits `ac8da47` and
`1676712` remain rehearsal findings only. The completed 256,000-token author
transport bound to `1557943` and its operator-stopped checker are also
superseded by this pre-merge amendment. The author pairs bound to `f762531` and
`51907e3`, plus the tool-cycle-rejected debug transport at `51907e3`, remain
rehearsal findings that motivated the final design. None is retroactively passed
or used as Gate K evidence. Issue #70 must rebuild the packets and rerun both
rehearsals at a clean commit containing the final amendment.

Existing provider qualification, isolation, packet, intervention, retry, and
checker requirements are unchanged. Turn, token, tool-call, and ordered-command
accounting remain required.

## Owner disposition

Approved: remove token, turn, validation/compile-cycle, and diagnostic-cycle
ceilings before the formal attempts. Preserve complete resource accounting. Do
not weaken eligibility, fresh-session, no-coaching, no-retry, packet, isolation,
rubric, or attempt-accounting rules.

## New protocol revision

3.
