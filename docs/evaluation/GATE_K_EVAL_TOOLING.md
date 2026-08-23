---
title: Gate K cold-agent packet and run tooling
status: Issue 70 pre-formal tooling plan
date: 2026-08-23
protocol: docs/evaluation/COLD_AGENT_PROTOCOL.md revision 3
roster: docs/evaluation/GATE_K_COLD_AGENT_PLAN.md
---

# Gate K cold-agent packet and run tooling

This document fixes the issue #70 harness method before either formal Gate K
attempt. Decision 0010 separately removes resource ceilings while preserving
complete accounting. This tooling does not choose a release candidate, disclose
a formal debug mutation, or spend a Gemini or DeepSeek attempt.

## Boundary composition

The runner first executes the matching repository-owned Pi neutral preflight
against the exact clean candidate checkout. A subject or checker launch then
uses the same pinned Pi installation, provider route, model identity checks,
system prompt, single `bash` tool, boundary extension, cleared environment, and
Bubblewrap isolation. The ordinary repository is never mounted for the task
launch.

The task mount has two layers:

1. the complete content-addressed packet is mounted read-only at `/workspace`;
2. exactly one declared packet directory is remounted read-write:
   `workspace/` for a cold author or `output/` for a cold debugger/checker.

Every packet directory other than a currently empty declared writable root must
be the prefix of at least one manifested regular file. Input trees, recorded
artifact trees, checker construction, verification, and finalization reject
empty descendant directories. This makes directory topology a consequence of
content-addressed file paths and prevents an unmanifested directory name from
carrying a hidden answer or unrelated context.

For packet runs, the otherwise empty `/tmp`, `/dev`, and `/home/subject`
directories are read-only, and the process filesystem is remounted read-only.
The boundary self-test proves the device filesystem is empty and writes fail at
the packet root, `/tmp`, `/home/subject`, the process filesystem, and the sandbox
root before it proves that the one declared task directory accepts a write.
Source-only neutral qualification keeps its disposable `/tmp` because Cargo
needs temporary storage; that worktree is a separate preflight boundary and is
never the callable task packet.

The task and checker prompts name `/tmp`, `/home`, `/etc`, and `/workspace/..`
as forbidden explicitly. A checker must inspect every subject command and
reject a model-requested outside-path access even if Bubblewrap denied it; a
checker that probes those paths itself is likewise ineligible. This is task
merit review on top of the structural sandbox, not shell parsing by the runner.

The extension verifies the candidate binding, packet-manifest digest, binary,
read-only packet root, declared writable path, absent credentials, absent
outside-host paths, and unshared network before the first provider request.

## Packet identities

`gate-k-eval-packet.sh` builds four shapes:

- `author`: public orientation, the Gate K base-fixture excerpt, authoring and
  compiler references, exact generated CLI help, one source fixture, a writable
  source copy, brief, prompt, and copied candidate binary;
- `debug`: public orientation plus runtime, explanation, and compiler references,
  CLI help, verified compiled world, failing input, failure artifacts, brief,
  prompt, and copied candidate binary;
- `author-checker`: permitted verification references, the subject output,
  published commands, binding task receipt, prompt, and copied candidate binary;
- `debug-checker`: the debugger's output, commands, binding task receipt,
  original permitted debug evidence, the hidden seeded mutation, prompt, and
  copied candidate binary.

Every build requires the exact forty-character candidate commit, a clean Git
worktree at that commit, and an absent output directory outside the worktree.
It builds `nomos` in release mode before packet construction. Packet files use
fixed modes and repository-independent relative paths. `plan.json` and
`packet-manifest.json` use canonical compact JSON with sorted object keys; the
manifest records every other file's relative path, byte size, mode, SHA-256,
and schema identity where the file declares one. Its own SHA-256 is the packet
identity recorded by the launcher.

The manifest deliberately cannot hash itself. It declares that exclusion, and
the launcher rejects any path not enumerated by the manifest plus the manifest
itself. A rebuild comparison is over the complete directory, including the
manifest. Debug worlds, run bundles, forensics, and checker debug evidence use
exact relative-file allowlists. All copied trees reject nested Git metadata,
repository source, prior transcripts, reviews, and credential-like files; the
verifier independently repeats the shape allowlist after construction. Checker
construction accepts one complete recorder-produced subject record rather than
independent artifact and command paths. Both builder and verifier recompute the
artifact-tree and command hashes bound by the copied subject task receipt.

## Constraints, accounting, and records

The protocol separates independence constraints from resource accounting:

```text
fresh sessions                 1
operator substantive hints     0
operator retries               0
provider-reported tokens       recorded, no ceiling
assistant turns                recorded, no ceiling
tool calls and commands         recorded, no ceiling
```

The boundary does not parse shell text or terminate a valid run because of
resource use. The complete event stream and ordered command record preserve the
measured usage for owner review. The recorder does not retry. It classifies a
recorded substantive intervention as `assisted` and a transport/harness failure
that prevents fair evaluation as `inconclusive`. Final task merit belongs to the
independent checker and owner.

Before final assembly, an independent reviewer examines every subject and
checker command and writes `nomos.gate_k.command_adjudication@1` JSON. The
record binds the candidate, both task receipts, both complete command files,
their reviewed command counts, the adjudicator, and the owner disposition. Each
finding binds its subject/checker command ordinal and SHA-256 and records the
outside-workspace path token and reason. `gate-k-eval-validate-adjudication.py`
validates that structure and every binding without trying to infer shell
semantics. This is deliberate: arbitrary Bash can contain nested interpreters,
heredocs, comments, and quoted scan data, so a partial shell parser is neither a
sound access detector nor a reliable way to distinguish evidence text from an
operand. A bound finding takes precedence over transport or intervention state
and mechanically derives overall `fail`, even when `checker.json` self-reports
`pass`; finalization refuses an absent, incomplete, stale, or internally
contradictory adjudication.

The adjudication has this exact top-level shape (digests and identities shown as
placeholders):

```json
{
  "schema": "nomos.gate_k.command_adjudication@1",
  "candidateCommit": "<40 lowercase hex>",
  "subjectTaskReceiptSha256": "<sha256>",
  "checkerTaskReceiptSha256": "<sha256>",
  "subjectCommandsSha256": "<sha256>",
  "checkerCommandsSha256": "<sha256>",
  "reviewedAllCommands": true,
  "reviewedCommandCounts": {"subject": 1, "checker": 1},
  "findings": [],
  "verdict": "pass",
  "reason": "<independent review disposition>",
  "adjudicator": "<identity>",
  "ownerDisposition": "<owner identity and disposition>"
}
```

Each finding has exactly `record`, `commandOrdinal`, `commandSha256`, `kind`,
`pathToken`, and `reason`. Version 1 supports only the
`outside_workspace_path` kind. An empty array requires adjudication verdict
`pass`; one or more findings require `fail`. That verdict concerns the command
review. The finalizer still derives the overall `pass`, `fail`, `assisted`, or
`inconclusive` run verdict from the complete protocol record, with a command
finding taking highest precedence.

`commands.json` is not trusted as a self-reported inventory. Recording and
finalization both derive its exact canonical bytes from paired
`tool_execution_start` and `tool_execution_end` events in `transcript.ndjson`.
The derivation rejects missing, duplicate, or unpaired tool-call IDs. The
adjudication validator then requires exact command-document and row fields,
contiguous integer ordinals, unique nonempty call IDs, completed Bash calls,
boolean error flags, and one nonempty shell-command argument. It also checks the
adjudication candidate directly against both bound task receipts.

A completed record contains `RUN.md`, `plan.json`, `packet-manifest.json`,
`prompt.txt`, the complete sanitized NDJSON event stream, `commands.json`,
the complete subject and checker artifact trees, `checker.json` after
independent checking, and `adjudication.json`. Finalization recomputes both
artifact-tree digests after copying them into the durable run. It also proves
that each task receipt agrees with its plan, packet manifest, and sandbox
boundary; that the packet's candidate marker is the exact candidate plus one
newline; that the sandbox independently matched the supplied candidate binary;
that the candidate exists in repository history; and that the checker packet
manifest binds the exact supplied subject receipt, commands, and every artifact.
Checker command/reason arrays are validated element by element. The output must
be outside both immutable input records. Missing
identity, transcript, command, artifact, or result fields fail closed. Operator
intervention is always present, including the literal disposition `none`.
Checker prompts declare the finalizer-owned `nomos.gate_k.checker_result@1`
schema and its required verdict, command, and reason fields before launch.
Sanitization removes only provider `textSignature` and `thinkingSignature`
fields, matching the existing Pi qualification receipt; `plan.json` declares
that exact loss limit before launch. No message, tool event, usage row, tool
result, or model identity is removed.

## Non-formal rehearsals

The committed rehearsal inputs are explicitly ineligible:

- the author task adds an approved `extinguishable_light`, not the formal second
  `iron_barred_door`;
- the debug task inserts a second `unlock` after the first successful unlock.
  The rejected run preserves a committed causal prefix, allowing the debugger
  to prove that syntax, package integrity, and credential authorization are not
  the cause and to identify the repeated state transition instead.

The debug mutation record is supplied only to its checker packet. It is not the
formal DeepSeek mutation. Rehearsal records live under
`docs/evaluation/runs/rehearsal/` and state `formalAttempt: false`.

The first completed issue #70 rehearsal set targeted exact clean candidate
`71093eb46805c6811100e4b552595048a11b5346`. Both the author subject/checker
pair and debug subject/checker pair completed without operator intervention or
retry. Their complete records are stored at
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author/` and
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-debug/`. Every task
receipt states `formalAttempt: false`; these records spend no Gemini or
DeepSeek formal attempt and cannot satisfy acceptance 17 or 18. The non-author
audit of `97a9c7c` invalidated their pass disposition: it found fail-open copied
tree allowlists, a writable packet-run `/tmp`, and omitted checker reproduction
artifacts in the durable final record. They remain immutable rehearsal findings,
not passing issue #70 evidence. The repaired harness requires fresh author and
debug rehearsal pairs. An uncommitted follow-up author rerun proved the repaired
filesystem denied two subject-requested `/tmp` paths, but its checker treated
those attempts as a minor deviation and passed. That run is not promoted. The
prompts and rubric now make any model-requested outside-path access a rejection,
including when the sandbox denies it.

The replacement r3 rehearsal set targets exact clean candidate
`0072f9970cbc88c8936f3741b8cf9f48495a8c13`. Both subject/checker pairs passed,
and each checker distinguished forbidden-path strings quoted in the subject's
own boundary-compliance prose from actual command operands. Neither subject nor
checker requested an outside-path access. The finalizer preserved both complete
checker artifact trees and recomputed all four subject/checker tree digests.
Durable records live at
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author-r3/` and
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-debug-r3/`. A neutral
Claude qualification probe twice emitted two turns and failed closed before a
task launch; replacement neutral probes passed without changing either task
packet. No failed probe received a task prompt or spent a rehearsal/formal
attempt.

A replacement non-author audit at evidence commit
`18525aa37ab1bf06d2f64a50684f657f26c0d7ae` invalidated r3. Bubblewrap's
packet-run device mount exposed writable `/dev/null` and `/dev/shm`; the r3
author and checker used `/dev/null` despite the outside-path rubric. The audit
also proved checker construction accepted caller-substituted artifacts and
commands because they were not bound to the subject task receipt. Packet runs
now expose an empty `/dev`, remount `/proc` read-only, test those properties,
and require one complete subject record whose receipt hashes the copied command
and artifact trees. Both rehearsal pairs require replacement r4 evidence.

The first post-repair author pair at `0b807bf9c9fcf9007f0e2d660241f229a0be69e1`
proved the new structural boundary and receipt binding, but the subject still
typed one reflexive `2>/dev/null` fallback. The empty `/dev` denied it, the
subject disclosed it, and the checker correctly returned `reject`. That failed
pair is preserved at
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author-r4-rejected/`.
The shared and task prompts now state the legal alternative explicitly: capture
output in the declared writable subtree and remove the file afterward.

The clean r5 author and debug pairs target exact candidate
`c1b9f355fa32f8ba749b62aa8d15bd05e9c62808`. Both independent checkers passed.
The author pair recorded 16/19 assistant turns, 23/28 tool calls, and
265,474/564,830 provider-reported tokens. The debug pair recorded 16/18 turns,
26/23 tool calls, and 529,484/523,550 provider-reported tokens. All four task
sessions are distinct, record no operator intervention or retry, and state
`formalAttempt: false`. Every actual command path stayed inside `/workspace`;
the checkers independently reproduced the authored package and the failing and
repaired debug runs. Durable records live at
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author-r5/` and
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-debug-r5/`. These are
issue #70 rehearsal evidence only and do not spend or satisfy either formal
Gate K attempt.

The non-author audit of evidence head
`d317122887dbacb106c257944ee6e0a2d3237e1f` invalidated r5 despite finding its
actual sessions and receipts clean. Copy, manifest, artifact hashing, and
finalization preserved empty directories while binding only regular files; an
empty directory name could therefore disclose an expected answer without
changing a packet or receipt digest. Packet construction, verification,
recording, and finalization now reject every empty descendant directory, and
the offline suite reproduces the attack at each boundary. Both shapes require
clean replacement r6 pairs.

The clean r6 author and debug pairs target exact repaired candidate
`c800c98a67f2599b5522a84d42a7549600d53d1f`. Both independent checkers passed.
The author pair recorded 14/20 assistant turns, 21/32 tool calls, and
208,252/770,315 provider-reported tokens. The debug pair recorded 19/15 turns,
30/19 tool calls, and 611,607/393,092 provider-reported tokens. All four task
sessions are distinct, record no intervention or retry, state
`formalAttempt: false`, and contain no empty artifact directory. Every actual
command operand stayed inside `/workspace`; checker-only forbidden-path strings
were quoted scan patterns, not access attempts. Durable records live at
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author-r6/` and
`docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-debug-r6/`.

## Invalidation

### Formal `gate-k-rc1` finalizer finding

The four formal sessions against exact candidate
`d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9` completed in their predeclared
order. The Gemini author subject produced a correct second door, and the
DeepSeek author checker reproduced it byte-for-byte. That checker nevertheless
requested `/dev/null` at command ordinals 1 and 16, then improperly self-waived
the violation and returned `pass`. The prior finalizer trusted that self-verdict
and would not encode the evidence-backed failure.

Issue #79 adds the hash-bound structured adjudication and a regression
containing a `pass` checker result plus an independently recorded forbidden
redirection. The finalizer refuses a finding paired with an adjudication
self-verdict of `pass`, and a valid bound finding mechanically produces overall
`fail`. A companion fixture records no finding for forbidden-path strings used
only as quoted grep, sed, or Python scan data. Further regressions reject stale
subject/checker pairing, phantom candidate bindings, and output nested beneath
immutable input evidence. They also reject commands absent from the transcript,
malformed command or checker-result rows, and a self-consistent receipt chain
whose packet marker names different candidate bytes. The DeepSeek debug
subject's ordinals 1, 48, and 65 are recorded through the same structured path;
its Gemini checker correctly returned `reject` independently.

Peter Permenter dispositioned both formal attempts `fail` on 2026-08-23. Their
subject and checker records are immutable, and no retry is authorized or
planned. This tooling repair does not retroactively change those sessions or
their candidate binding. It does invalidate `gate-k-rc1` for any later
exact-head evidence: a future launch would require a newly frozen
`gate-k-rcN`, combined-head proof, and explicit owner authorization.

Any packet allowlist, public packet document, prompt, constraint, runner, recorder,
checker construction, or boundary-extension change after a candidate is tagged
invalidates later exact-head evidence for that candidate. Repairing a rehearsal
harness defect requires a fresh rehearsal; selecting a more favorable prior run
is forbidden.
