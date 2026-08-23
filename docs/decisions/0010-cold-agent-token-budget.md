---
title: Cold-agent protocol revision 3 — raise the cumulative token budget
status: Owner-authorized; effective when merged
number: 0010
date: 2026-08-23
supersedes_protocol_revision: 2
establishes_protocol_revision: 3
owner: Peter Permenter
implementing_reviewer: GPT-5.6
issue: 70
---

# Cold-agent protocol revision 3 — raise the cumulative token budget

## Decision authority

The owner directed that the cold-agent token budget need not remain as low as
64,000 while reviewing the first non-formal Opus author rehearsal on 2026-08-23.
After a 256,000-token trial completed at 237,082 tokens, the owner directed that
the ceiling provide substantially more room in line with the selected models'
large context windows. This record supplies the separate owner-authorized
amendment required by issue #70. Protocol revision 3 becomes effective when
this record and its tooling merge; revision 2 remains effective until then.

## Prior wording

`docs/evaluation/COLD_AGENT_PROTOCOL.md` revision 2 set the default
provider-reported total token budget to 64,000.

The issue #70 packet plan and verifier encoded the same 64,000-token maximum.

## Replacement wording

The default provider-reported total token budget is 1,000,000. Gate-specific run
plans may still declare stricter limits before launch. The existing limits of
one fresh session, forty assistant turns, twelve validation/compile cycles,
twelve cold-debug diagnostic CLI cycles, zero substantive hints, and zero
operator retries remain unchanged.

## Reason

Pi reports complete input, output, and cache usage on every assistant response.
The harness correctly sums those provider totals across turns. A non-formal
Opus author rehearsal therefore crossed 64,000 after nine turns even though it
had made the requested content edit and reached successful validation and
compilation. The cap measured repeated conversation context more aggressively
than task complexity or operator cost control.

The 1,000,000 ceiling remains finite and enforced. Turn and CLI-cycle limits
continue to bound meandering behavior independently, while the larger token
budget accommodates providers that report the full cached context on every
turn and leaves room for the more evidence-intensive debug task.

## Effect on existing evidence

No formal Gate K cold-author or cold-debug attempt has begun, so no formal result
is reclassified and the formal attempt counts remain zero.

The two non-formal Opus author transports bound to commits `ac8da47` and
`1676712` remain rehearsal findings only. The completed 256,000-token author
transport bound to `1557943` and its operator-stopped checker are also
superseded by this pre-merge amendment. None is retroactively passed or used as
Gate K evidence. Issue #70 must rebuild the packets and rerun both rehearsals at
a clean commit containing the final amendment.

Existing provider qualification, isolation, packet, command-cycle, turn,
intervention, retry, and checker requirements are unchanged.

## Owner disposition

Approved: replace the default cumulative provider-reported token maximum of
64,000 with 1,000,000 before the formal attempts. Do not weaken any other budget,
eligibility rule, packet boundary, rubric, or attempt-accounting rule.

## New protocol revision

3.
