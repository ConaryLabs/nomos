---
title: Cold-agent protocol revision 4 — reserve formal attempts before launch
status: Owner-authorized; effective when merged
number: 0011
date: 2026-08-23
supersedes_protocol_revision: 3
establishes_protocol_revision: 4
owner: Peter Permenter
implementing_reviewer: GPT-5.6
issue: 79
---

# Cold-agent protocol revision 4 — reserve formal attempts before launch

## Decision authority

The owner directed issue #79's structured-evidence repair to proceed after an
independent audit showed that finished receipts could not prove an abandoned
provider transport had never occurred. This record supplies the explicit
protocol amendment required by section 11. Revision 4 becomes effective with
this record and its tooling; revision 3 governed the four frozen Gate K tasks.

## Prior wording

Section 8 required all formal attempts against the same brief to be reported.
The task plan and finished receipt also declared one fresh session and zero
operator retries. No durable record existed before provider launch, so a
transport abandoned before recording could be omitted without detection.

## Replacement wording

Before every formal provider task, the operator appends and commits a
hash-chained reservation naming the candidate, packet manifest, prompt, shape,
provider, model, thinking level, attempt ID, and nonce. The launcher refuses a
formal task unless that exact reservation is the one open entry in the committed
ledger. Completion appends the exact task-receipt hash and outcome. No later
reservation may be made while an earlier one remains open.

## Reason

A post-run `operatorRetries: 0` assertion authenticates only the recorded run.
Prelaunch reservation creates evidence before the action it accounts for and
makes discarded, failed, or inconclusive transports visible. The hash chain and
Git commit bind ordering without adding a service or third-party dependency.

## Effect on existing evidence

The four Gate K tasks remain byte-for-byte frozen and retain their owner verdicts.
They predate revision 4, so the ledger imports each as a closed legacy event by
exact candidate, packet-manifest, identity, task-receipt hash, and outcome. This
does not manufacture retroactive prelaunch proof; it makes the limitation and
complete known inventory explicit. No retry or new formal attempt is authorized.

Any future formal run requires a newly authorized candidate and a prospective
committed reservation. Rehearsals remain non-formal and do not consume entries.

## Owner disposition

Approved as a strengthening of attempt accounting. Fresh-session, no-coaching,
no-retry, packet isolation, model roster, rubrics, resource accounting, and the
two Gate K failure verdicts remain unchanged.

## New protocol revision

4.
