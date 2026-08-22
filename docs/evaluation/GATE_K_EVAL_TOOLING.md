---
title: Gate K cold-agent packet and run tooling
status: Issue 70 pre-formal tooling plan
date: 2026-08-23
protocol: docs/evaluation/COLD_AGENT_PROTOCOL.md revision 2
roster: docs/evaluation/GATE_K_COLD_AGENT_PLAN.md
---

# Gate K cold-agent packet and run tooling

This document fixes the issue #70 harness method before either formal Gate K
attempt. It does not amend the protocol, choose a release candidate, disclose a
formal debug mutation, or spend a Gemini or DeepSeek attempt.

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
- `author-checker`: permitted verification references, the subject output and
  published commands, prompt, and copied candidate binary;
- `debug-checker`: the debugger's output and commands, original permitted debug
  evidence, the hidden seeded mutation, prompt, and copied candidate binary.

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
manifest.

## Budgets and records

The default protocol budgets remain unchanged:

```text
fresh sessions                 1
provider-reported tokens       64,000 maximum
assistant turns                40 maximum
validation/compile cycles      12 maximum
debug diagnostic CLI cycles    12 maximum
operator substantive hints     0
operator retries               0
```

The boundary counts tool calls and CLI cycles before execution, blocks the call
that would exceed a cycle budget, and terminates the run. It stops after the
provider response that first reports a token or turn overrun. The recorder does
not retry. It classifies a protocol violation as `fail`, a recorded substantive
intervention as `assisted`, and a transport/harness failure that prevents fair
evaluation as `inconclusive`. Only a complete within-budget run is eligible for
`pass`, and final task merit still belongs to the independent checker and owner.

A completed record contains `RUN.md`, `plan.json`, `packet-manifest.json`,
`prompt.txt`, the complete sanitized NDJSON event stream, `commands.json`,
subject artifacts, and `checker.json` after independent checking. Missing
identity, transcript, command, artifact, or result fields fail closed. Operator
intervention is always present, including the literal disposition `none`.

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
`docs/evaluation/runs/rehearsal/` and state `formal_attempt: false`.

## Invalidation

Any packet allowlist, public packet document, prompt, budget, runner, recorder,
checker construction, or boundary-extension change after a candidate is tagged
invalidates later exact-head evidence for that candidate. Repairing a rehearsal
harness defect requires a fresh rehearsal; selecting a more favorable prior run
is forbidden.
