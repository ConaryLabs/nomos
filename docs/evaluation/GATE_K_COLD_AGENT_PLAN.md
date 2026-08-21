---
title: Gate K cold-agent eligibility and roster plan
status: Owner-authorized routing; effective on merge
date: 2026-08-21
issue: 7
protocol: docs/evaluation/COLD_AGENT_PROTOCOL.md revision 1
---

# Gate K cold-agent eligibility and roster plan

This plan resolves the mixed-family eligibility question before any formal
Gate K cold-author or cold-debug run begins. It selects third-family subjects
for the whole-kernel claims instead of trying to divide the kernel into
convenient authorship slices after seeing results.

This is an evaluation routing decision, not a Gate K contract revision. The
formal runs still have to satisfy every criterion in the cold-agent protocol and
`KERNEL.md`; this plan does not count as either run.

## Principal implementation authorship

The implemented kernel has mixed principal authorship:

- SW-B workspace foundations were authored through Claude-family lanes;
- SW-C source schema, parser, typed resolution, primitive expansion, and
  ownership linker were authored through GPT/Codex;
- later Gate K implementation is expected to continue through GPT/Codex unless
  a merged record says otherwise.

The SW-C GPT/Codex attribution is owner-supplied provenance recorded by issue
#7; its implementation commit has no model trailer. The uncertainty cannot
weaken eligibility because this plan excludes GPT rather than admitting it.

Claude and GPT families are therefore excluded as formal whole-Gate-K subjects.
They may operate the harness or review evidence only where the protocol permits
and where they are not adjudicating their own work alone.

## Predeclared roster

| Formal role | Subject route | Independent checker route |
| --- | --- | --- |
| Cold author | Google Gemini 3.7 Flash High through Antigravity `agy` | DeepSeek V4 Pro through direct Reasonix |
| Cold debugger | DeepSeek V4 Pro through direct Reasonix | Google Gemini 3.7 Flash High through Antigravity `agy` |

The cold author and cold debugger use separate fresh sessions. A checker also
uses a fresh session. The cold-author checker receives the committed subject
output plus the published reproduction commands. After the cold-debug subject
finishes, its checker additionally receives the hidden seeded mutation required
by protocol section 7 so it can confirm the diagnosis. The hidden mutation is
never exposed to the subject. No conversation is resumed across roles.

Both selected families have performed evaluation-only reruns in this repository:
DeepSeek checked the merged SW-C proof, and Gemini checked the checkout-v7 CI
change. Neither authored or designed Gate K. Those runs do not disqualify a new
subject under protocol section 2, but their conversations must not be resumed or
made available to a formal subject.

The receipts are:

- `docs/evaluation/runs/gate-k/2026-08-21-deepseek-v4-pro-sw-c-rerun/`;
- `docs/evaluation/runs/ci/2026-08-21-gemini-3.7-flash-checkout-v7/`.

The owner and adjudicator is Peter Permenter. The operator may be Mira/Codex,
but the operator supplies the predeclared packet verbatim and may not coach.

## Verified routes at plan time

On 2026-08-21 the available clients reported:

```text
agy 1.1.17
gemini-3.7-flash-high    Gemini 3.7 Flash (High)

Reasonix v1.29.0 (9eaa3b295, linux/amd64)
deepseek-pro/deepseek-v4-pro
```

Gemini runs use high effort, the maximum supported by this route. DeepSeek runs
use maximum effort. Exact model resolution is evidence, not an invocation
assumption:

- the `agy` event log must resolve to `Gemini 3.7 Flash (High)`;
- the Reasonix `result.json` session identifier and metrics must resolve to
  `deepseek-v4-pro` and `deepseek-pro/deepseek-v4-pro`.

Client versions may change before the formal runs. Each `plan.json` records the
then-current version, exact invocation, resolved model, and provider. A model
alias, silent fallback, or unresolved identifier makes the run inconclusive.
The operator preflights the exact route before formal invocation. During the
formal invocation, the harness must record resolved identity before the first
model response or tool call and confirm it again in the exported result. A
post-task identity failure is reported against that attempt; it never erases an
unfavorable subject result or creates an unreported retry.

## Freshness and tool boundary

Each formal subject starts in a new conversation and isolated task workspace.
It receives only the content-addressed packet allowed by the applicable
protocol section. It receives no repository history, source outside the packet,
issue or pull-request discussion, founding conversation, prior evaluation
transcript, personal/project memory, web access, connector, skill expansion,
subagent, or other model.

The operator must inspect the client's effective tools and context before
launch. For Gemini, the run plan must demonstrate that a fresh Antigravity
project/conversation does not expose persisted project context; slash-command
expansion and nonessential integrations are disabled. For DeepSeek, retrieval
and subagent tools are ablated. If either client cannot prove the declared
boundary at run time, that subject is ineligible until the owner approves a new
plan.

The protocol's default budgets and zero-substantive-hint rule apply. No model
fallback or cross-model rescue is allowed inside a formal attempt. A transport
restart follows the protocol exactly and remains disclosed.

## Eligibility invalidation

This roster must be reconsidered before launch if any of these occurs:

- Gemini or DeepSeek materially authors or designs Gate K implementation;
- the selected exact model is unavailable or silently resolves to another
  family or tier;
- the client cannot disable or disclose persisted context and forbidden tools;
- the formal task, packet, budget, or pass rubric changes;
- principal authorship of the evaluated kernel changes materially.

An invalidated plan blocks the formal run. It does not permit substituting a
same-family subject or inventing slice boundaries after results are known.

## Owner disposition

Peter authorized direct `agy` with Gemini 3.7 Flash High and the existing direct
Reasonix/DeepSeek and Claude Code routes for other-family work on 2026-08-21.
For the whole Gate K formal gates, this record selects Gemini as cold author and
DeepSeek as cold debugger, with cross-family independent checking as listed
above. The roster becomes effective when the owner merges this record.
