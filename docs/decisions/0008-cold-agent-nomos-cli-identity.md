---
title: Cold-agent protocol revision 2 — use the Nomos CLI identity
status: Owner-authorized; effective when merged
number: 0008
date: 2026-08-22
issue: 47
supersedes_protocol_revision: 1
establishes_protocol_revision: 2
gate_k_contract_revision: 6
owner: Peter Permenter
implementing_reviewer: GPT-5.6
---

# Cold-agent protocol revision 2 — use the Nomos CLI identity

## Decision authority

The owner directed completion of issue #47 on 2026-08-22. That issue's
falsifiable acceptance authorizes only the stale CLI-identity correction below.
This record supplies the amendment required by section 11 of the cold-agent
protocol before the active formal-run tool policy changes. Protocol revision 2
becomes effective when this record and its implementation merge; revision 1
remains effective until then.

## Problem

Cold-agent protocol revision 1 still names the prototype-era `estate` binary in
its active default tool policy. Contract revision 6 and decision 0007 establish
`nomos` as the active CLI identity. A future formal packet cannot execute the
stale name honestly, and the transport work in issues #45 and #49 was not
authorized to amend the protocol silently.

## Amendment A1 — active formal-run CLI identity

### Prior wording

Section 4, “Tool policy,” specified:

> - execute the published `estate` CLI;

### Replacement wording

Section 4, “Tool policy,” specifies:

> - execute the published `nomos` CLI;

### Reason

This aligns the formal evaluation instructions with the owner-authorized Nomos
identity already used by the repository, binary, active source format, and
schemas. It removes an impossible invocation without adding a compatibility
alias or changing what the subject may do.

### Effect on existing evidence

The correction is prospective. Prototype-era decisions, review records,
transcripts, receipts, command captures, paths, hashes, and model outputs retain
the `estate` names that were true when recorded. They remain immutable evidence
and are not rewritten or reinterpreted.

Existing cold reviews and evaluation reruns keep their recorded verdicts. The
`agy` boundary falsification from issue #45 remains a tooling falsification. The
Pi qualification from issue #49 remains transport and isolation evidence, not
a formal cold-author or cold-debug attempt. No prior run gains or loses formal
status because of this identity correction.

### Owner disposition

Approved as an identity-only correction. The tool scope, packet boundary,
budgets, pass rubric, verdict rules, subject roster, attempt accounting, and
existing run dispositions do not change. No formal attempt is launched by this
decision.

### New protocol revision

2.

## Active-instruction audit

The implementing change audits active cold-agent instructions for the obsolete
CLI invocation while excluding immutable run and review evidence from the
absence claim. After replacement, this command must return no match:

```bash
rg -n -i 'execute the published `estate` CLI' \
  README.md THESIS.md KERNEL.md docs/evaluation/COLD_AGENT_PROTOCOL.md \
  docs/evaluation/GATE_K_COLD_AGENT_PLAN.md docs/HANDOFF.md
```

Legacy `estate` names elsewhere remain permitted only where their surrounding
text explicitly identifies prototype-era history or provenance. Decision 0007
continues to govern that classification.

## Evidence limits preserved

Gate K contract revision 6 does not change. This decision does not implement
the CLI command surface, alter issue #45's Antigravity verdict, change the Pi
routes, substitute a model or family, relax context or tool isolation, modify a
budget or rubric, rewrite historical evidence, or launch a formal cold-agent
run.
