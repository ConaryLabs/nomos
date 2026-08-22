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
| Cold author | Google Gemini 3.7 Flash High through Pi and the pinned `pi-antigravity` provider | DeepSeek V4 Pro through Pi's built-in provider |
| Cold debugger | DeepSeek V4 Pro through Pi's built-in provider | Google Gemini 3.7 Flash High through Pi and the pinned `pi-antigravity` provider |

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

The original 2026-08-21 clients reported:

```text
agy 1.1.17
gemini-3.7-flash-high    Gemini 3.7 Flash (High)

Reasonix v1.29.0 (9eaa3b295, linux/amd64)
deepseek-pro/deepseek-v4-pro
```

Issue #45 subsequently falsified `agy` 1.1.17 as a provable formal boundary.
On 2026-08-22 issue #49 qualified a common replacement transport:

```text
Pi 0.84.2
antigravity/gemini-3.7-flash    Gemini 3.7 Flash    high
deepseek/deepseek-v4-pro        DeepSeek V4 Pro     max
anthropic/claude-opus-5         Claude Opus 5       max (supplemental only)
```

Gemini runs use high effort, the maximum supported by this route. DeepSeek runs
use maximum effort. Exact model resolution is evidence, not an invocation
assumption:

- Pi's boundary record and terminal assistant event must both resolve Gemini to
  `antigravity/gemini-3.7-flash` at `high`;
- those same records must resolve DeepSeek to
  `deepseek/deepseek-v4-pro` at `max`.

Client versions may change before the formal runs. Each `plan.json` records the
then-current version, exact invocation, resolved model, and provider. A route
that cannot resolve the exact subject during preflight blocks launch. If a
formal invocation proceeds under an unexpected model or silent fallback, the
attempt fails. If export loss prevents post-run identity confirmation despite a
correct preflight and runtime label, the attempt is inconclusive. Neither case
permits an unreported retry.
The operator preflights the exact route before formal invocation. During the
formal invocation, the harness must record resolved identity before the first
model response or tool call and confirm it again in the exported result. A
post-task identity failure is reported against that attempt; it never erases an
unfavorable subject result or creates an unreported retry.

### Historical `agy` preflights and falsification

Issue #17 proved that `agy --print --model ... <prompt>` silently sends the
seven-byte string `--model` as the prompt. Every Gemini print-mode lane must put
the prompt immediately after `-p` and must pass this repository's cheap harness
from the exact target worktree before a formal attempt:

```bash
docs/evaluation/agy-print-preflight.sh
```

The harness pins `gemini-3.7-flash-high`, high effort, the exact worktree, and
streaming JSON; disables slash-command expansion; and requires a completed
terminal-tool `pwd` event whose output is the worktree. A greeting, scratch
directory, missing tool event, model mismatch, non-success result, or nonzero
client exit blocks launch. Its prompt-first argument ordering is regression
tested in CI without contacting the provider.

The successful 1.1.17 repair receipt is under
`docs/evaluation/runs/tooling/2026-08-22-agy-print-mode-repair/`. The three
failed 2026-08-21 calls retain zero evidentiary value. Passing this transport
preflight is necessary but not sufficient for a formal run: the init event's
effective tools and all freshness/ablation requirements below still require
separate inspection and approval.

Issue #45 tested that boundary on 2026-08-22 with a new project and conversation,
sandboxing, slash-command expansion disabled, and a custom main agent declaring
only `view_file`, `replace_file_content`, and `run_command`. `agy` 1.1.17 still
reported 57 tools in its init event, including browser, web, MCP, persisted
knowledge, messaging, scheduling, and subagent capabilities. The event omitted
the project ID, context-source set, and memory state. Model text claiming some
forbidden tools were unavailable is not effective-configuration evidence.

The exact receipt is under
`docs/evaluation/runs/tooling/2026-08-22-agy-formal-boundary-falsification/`.
The committed guard remains the fail-closed test for any attempt to revive the
`agy` route:

```bash
docs/evaluation/agy-formal-boundary-preflight.sh
```

On 1.1.17 it correctly exits `1` with `AGY_FORMAL_BOUNDARY BLOCKED`. This
falsification remains evidence even though issue #49 qualifies Pi as the new
transport; it does not weaken the protocol or spend a formal attempt.

### Mandatory Pi boundary preflight

Every Gemini or DeepSeek formal launch now begins with the matching lane of:

```bash
docs/evaluation/pi-cold-agent-preflight.sh gemini
docs/evaluation/pi-cold-agent-preflight.sh deepseek
```

Pi is pinned by npm integrity and installed-tree digest. The Gemini adapter is
separately named and pinned; package discovery remains disabled, and only that
provider entry point plus the repository boundary extension are loaded
explicitly. The launcher starts a fresh ephemeral JSON session from a clean
exact commit, rejects a provider/model/thinking/worktree mismatch, disables
discovered extensions, skills, templates, themes, context files, built-in
tools, project trust, and session persistence, and proves the effective tool
catalog is exactly the boundary extension's `bash`.

That tool runs only inside Bubblewrap with a read-only host root, the target
checkout as its sole read-write host mount, a cleared allowlisted environment,
and an unshared network namespace. A pre-provider self-test proves the exact
commit and rejects outside reads, outside writes, credential environment, and
external network access. The offline fixture matrix and sanitized authenticated
receipts are under
`docs/evaluation/runs/tooling/2026-08-22-pi-provider-qualification/`. A failed
preflight blocks launch; a passing neutral probe spends no formal attempt.

## Freshness and tool boundary

Each formal subject starts in a new conversation and isolated task workspace.
It receives only the content-addressed packet allowed by the applicable
protocol section. It receives no repository history, source outside the packet,
issue or pull-request discussion, founding conversation, prior evaluation
transcript, personal/project memory, web access, connector, skill expansion,
subagent, or other model.

The operator must inspect the client's effective tools and context before
launch. For both selected families, the Pi preflight must prove a fresh
ephemeral session, no discovered context or resources, the exact single-tool
catalog, and the isolated command boundary. If either lane cannot prove that
state at run time, that subject is ineligible until the owner approves a new
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

Peter authorized direct `agy` with Gemini 3.7 Flash High and the original
Reasonix/DeepSeek route on 2026-08-21. After the `agy` boundary falsification,
he authorized qualifying Pi as the common non-Codex transport on 2026-08-22.
Issue #49 changes only those transports: Gemini remains the cold author,
DeepSeek remains the cold debugger, and each remains the other's independent
checker. Claude through Pi is supplemental only. The updated routing becomes
effective when the owner merges the issue #49 change.
