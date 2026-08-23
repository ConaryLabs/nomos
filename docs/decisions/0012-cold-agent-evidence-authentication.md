---
title: Cold-agent protocol revision 5 — authenticate the complete evaluation envelope
status: Owner-authorized; effective when merged
number: 0012
date: 2026-08-23
supersedes_protocol_revision: 4
establishes_protocol_revision: 5
owner: Peter Permenter
implementing_reviewer: GPT-5.6
issue: 79
---

# Cold-agent protocol revision 5 — authenticate the complete evaluation envelope

## Decision authority

The owner authorized continued issue #79 repair after a seventh independent
audit reproduced fail-open evidence paths in protocol revision 4 tooling. This
record makes the resulting protocol strengthening explicit. It does not change
the Gate K rubric or authorize another formal attempt.

## Prior wording

Revision 4 required a committed prelaunch reservation and said completion
appended the exact task-receipt hash and outcome. It did not require the close
operation itself to read and validate the receipt or completed launcher. Public
packet documents were hash-bound but partly validated through selected
predicates. Pi provider signatures were removed recursively before semantic
validation, and the execution boundary recorded paths without a complete
path-and-digest identity for Pi, a provider extension, and Bubblewrap.

## Replacement wording

A formal close is derived only from the complete recorded task directory. Its
exact canonical task receipt, exact launcher schema, transcript, commands,
packet manifest, qualification, stderr, boundary, and artifact tree must agree.
The launcher and receipt must bind the one open reservation, candidate, packet,
prompt, session, provider, model, thinking level, committed ledger HEAD, ledger digest,
and status-derived outcome. Close and final assembly use the same semantic
single-record proof for lifecycle, command derivation, qualification, boundary,
accounting, immutable packet, and artifact evidence. A hash, skeletal launcher,
or consistently rehashed invalid record cannot close an attempt.

A reservation cancelled before provider launch is retained as an explicit
`discarded-before-launch` event with a reason rather than fabricated as a task
receipt or left available for silent reuse.

`plan.json`, `packet-manifest.json`, and `task-receipt.json` use exact
allowlisted schemas, strict scalar types, and canonical sorted compact JSON.
All evaluation JSON rejects duplicate keys and non-finite numbers, including
finite syntax that overflows the host numeric representation, before its
declared top-level shape is evaluated. Pi usage fields and UTC timestamps are
strictly typed and ranged.

Each legacy import is admitted only as its exact canonical frozen event. Gate K
final assembly requires the exact four-event frozen inventory; later attempts,
closes, cancellations, or imports require a new candidate plus an explicit
protocol/tooling revision and cannot be silently omitted from disposition.

Raw Pi streams are parsed before sanitization. `textSignature` and
`thinkingSignature` may be removed only from their documented content blocks;
the raw-stream digest is retained. Boundary schema
`nomos.pi_cold_agent_boundary@3` binds the resolved path and SHA-256 of Pi,
Bubblewrap, and any provider extension, and the task receipt repeats that exact
runtime identity.

## Reason

Content addressing does not help when undeclared fields, malformed scalar
values, arbitrary executable paths, or a fabricated close are accepted before
the content is interpreted. Authentication must cover the full evidence
envelope and the actual executables used, not only selected values inside it.

## Effect on existing evidence

The four `gate-k-rc1` formal task records remain byte-for-byte unchanged.
Their legacy boundary schema `@2` and imported close events remain accepted only
at their four frozen receipt hashes, with the absence of prospective prelaunch
proof still explicit. Reassembly continues to produce the same two `fail`
result hashes. No old session is upgraded to revision 5 and no retry is erased
or authorized.

Future formal tasks require boundary schema `@3`, an authenticated runtime
identity, a raw-stream digest, an exact prospective reservation, and a
receipt-backed close event.

## Owner disposition

Approved as a fail-closed protocol repair. The model roster, briefs, rubrics,
zero-coaching rule, unlimited model resource policy, and both existing Gate K
failure verdicts are unchanged.

## New protocol revision

5.
