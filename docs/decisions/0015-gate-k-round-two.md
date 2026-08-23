---
title: Prospectively governed Gate K round two
status: Owner-authorized; formal launch remains conditional
number: 0015
date: 2026-08-23
owner: Peter Permenter
implementing_reviewer: GPT-5.6
issue: 86
gate_k_disposition: docs/decisions/0013-gate-k-disposition.md
visual_disposition: experiments/gate-0-gaol-target-pack/TARGET.md
contract_revision: 7
supersedes_protocol_revision: 5
establishes_protocol_revision: 6
future_candidate: gate-k-rc2
---

# Prospectively governed Gate K round two

## Decision authority

Peter Permenter reviewed the complete quarantined gaol target pack, recorded
`visual thesis compelling`, and authorized the next prerequisite for a fresh
Gate K round: a prospective protocol and operating-order decision.

Decision 0013 remains unchanged. Gate K failed, criteria 17 and 18 remain
failed for `gate-k-rc1`, and all four formal task records and both failed final
results remain immutable. Decision 0014 also remains unchanged: the static
target pack is a desired visual target, not renderer evidence or Gate K
acceptance.

This decision authorizes a separately governed round two only after its
protocol tooling, rehearsal, exact candidate, and mechanical evidence pass. It
does not itself launch a provider, create `gate-k-rc2`, or authorize renderer
work.

## Prior wording

Cold-agent protocol revision 5 treated every requested path outside the one
declared task output subtree identically:

> A model-requested boundary probe or outside-path access fails the task rubric
> even when denied; any successful undeclared read or write is a harness
> failure, not subject evidence.

It gave each run exactly one verdict—`pass`, `fail`, `assisted`, or
`inconclusive`—without separately recording semantic merit, independence
integrity, and operational compliance.

Decision 0013 authorized no retry and required a new owner decision,
prospective protocol/tooling revision, and new frozen candidate before any
future attempt.

## Replacement wording

Protocol revision 6 records three independently adjudicated dimensions for
every formal subject and checker:

1. **semantic merit** — whether every task-specific authoring, debugging, or
   checking result and explanation criterion is correct;
2. **independence integrity** — whether model eligibility and fresh-session
   requirements hold, only declared information entered the task, and no
   substantive outside help occurred;
3. **operational compliance** — whether every declared tool, path, execution,
   permitted-change, evidence-accounting, and record-completeness requirement
   was obeyed.

Each dimension is `pass`, `fail`, or `inconclusive`, with evidence. The run's
single overall verdict remains `pass`, `fail`, `assisted`, or `inconclusive`
and is derived in this order:

1. substantive help produces `assisted`;
2. otherwise, any failed dimension produces `fail`;
3. otherwise, any inconclusive dimension produces `inconclusive`;
4. only three passing dimensions, which collectively cover every declared run
   criterion, produce `pass`.

Only overall `pass`, followed by the required independent checker and all
applicable `KERNEL.md` criteria, can satisfy a cold gate. Separate dimensions
preserve diagnostic information; they do not allow one success to compensate
for another failure.

For future formal runs, the exact device path `/dev/null` is the sole declared
filesystem exception outside the task workspace. Reading it, writing it, or
redirecting a command stream to it is operationally allowed because it admits
no project information and preserves the independence boundary. It remains
command-accounted and cannot replace required transcript, diagnostic, or
artifact evidence.

Every other outside-workspace path request fails operational compliance even
when the sandbox denies it. A subject-requested successful undeclared access
also fails operational compliance; if undeclared information enters the task,
independence integrity fails. An unrequested harness exposure is recorded as a
harness failure and cannot become passing subject evidence. The finalizer must
fail closed in every case.

The fresh-session, model-eligibility, zero-coaching, zero-retry-after-model-
failure, content-addressed packet, attempt ledger, exact runtime identity,
complete transcript and command accounting, resource accounting without
ceilings, and independent-checker rules are unchanged.

## Reason

The `gate-k-rc1` records answered two questions differently. The agents and
checkers completed the semantic work correctly and received no hidden project
information, while requested `/dev/null` redirections violated the frozen
absolute path rule. The overall failures were correct under that rule, but the
single verdict concealed the positive semantic and independence observations.

`/dev/null` is a conventional non-information-bearing sink. Treating it as
equivalent to a source, hidden branch, home directory, or undeclared packet
path added operational friction without strengthening blindness. Declaring
that one exception prospectively removes the discovered proxy error while
retaining strict failure for every information-bearing or undeclared path.

The compelling static target pack makes a fresh proof worth its cost. Reusing
or relabelling the old attempts would not.

## Effect on existing evidence

None. The four `gate-k-rc1` task records remain byte-for-byte unchanged under
their original protocol and rubric. Their `/dev/null` requests remain forbidden
for those runs, criteria 17 and 18 remain failed, and decision 0013 remains the
only disposition of that candidate. Protocol revision 6 is not retroactive.

Round two must use fresh sessions, new reservations and launch events, new task
records, a fresh exact candidate named `gate-k-rc2`, and a new final owner
disposition. Every failed and superseded historical attempt remains disclosed.

## Authorized operating order

Work proceeds only through separately falsifiable issues in this order:

1. merge this decision and protocol revision;
2. implement revision-6 schemas, finalization, boundary handling, and tests,
   then pass non-formal author and debugger rehearsals plus a non-author audit;
3. prove the semantic candidate inputs against `gate-k-rc1`, freeze the exact
   `gate-k-rc2` candidate, and rerun the complete mechanical matrix;
4. reserve and launch one fresh Gemini-family cold-author subject;
5. reserve and launch one fresh DeepSeek-family cold-debug subject;
6. run a fresh DeepSeek-family checker for the author result and a fresh
   Gemini-family checker for the debugger result;
7. assemble the complete evidence and record an explicit owner verdict.

The exact provider model identifiers, clients, thinking levels, prompts,
packets, hashes, and run rubrics are pinned before their reservations. A model
failure receives no operator retry. A transport failure follows the existing
authenticated inconclusive path rather than silently consuming or restoring an
attempt.

No formal reservation or launch is authorized until steps 1–3 are green. No
renderer architecture, executable visual experiment, or adoption is authorized
unless the new final owner disposition passes Gate K.

## Owner disposition

Approved prospectively. Begin the revision-6 tooling slice after this decision
merges. Preserve the failed round-one record without qualification. Do not
spend a formal attempt, create the candidate tag, or begin renderer work in
this slice.

## New protocol revision

6.
