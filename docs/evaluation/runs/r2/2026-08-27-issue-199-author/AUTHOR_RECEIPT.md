# R2 final-proof implementation-author receipt

Status: final-proof source authored for issue #199; the exact candidate and
author/non-author executions are bound by the generated external proof receipts,
not recursively by this source receipt.

## Authority and baseline

- Issue: #199, `R2 final evidence and owner disposition`
- Baseline commit: `6cbce64cb867aef24faf227e62bdfc585bbcbd5d`
- Baseline tree: `6dada35f44e178f0d6cafc5ac2b5c94ab3fd0522`
- Contract: owner-authorized `R2.md` revision 2
- Contract SHA-256:
  `770740bad1c85cf7ea9dcd16f8c25e01766064d3b59d7f0bb9d438c289a6e638`
- Revision-2 authority: decision 0024, SHA-256
  `0356b3918a5c2643c36e16555e8ef78155bf893a8c3c21e4f75263f8289feea0`
- Unchanged R1 contract: `RUNTIME.md` revision 4, SHA-256
  `dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593`
- Issue-body SHA-256:
  `8ffd30e7a213e991732ea6031743542eb68d9b80fe6d4989ed58052617352dcc`
  over `gh api repos/ConaryLabs/nomos/issues/199 --jq .body`, including the
  command's final LF
- Author: Codex primary agent and its bounded GPT-5 family implementation
subagents

Decision 0024's front matter retains
`4320dbdee1fcc52204809a9896c6e5a9a460d033a1ceba2c2aab7091bc55f929`,
the pre-repair issue body against which the owner authorized that immutable
decision record. The decision's replacement wording then explicitly required
the issue wording and its digest binding to be updated. The final issue body
and this proof therefore bind `8ffd30e7...`; the two digests describe the
ordered before/after authority states rather than disagreeing authorities.

## Consulted inputs

The implementation used the repository authority named by `AGENTS.md`, the
exact issue body, the accepted R1 and authorized R2 contracts and decisions,
the existing R1/R2 proof scripts, the committed R2 fixtures and evidence, and
the owner authorization conversation. The implementation did not consult or
copy an adopter repository, payload, frame, palette, asset, prose, schema,
coordinate set, mechanic, or governance document.

## Scope and method

The final slice adds a decomposed orchestration harness, an independent receipt
assembler/verifier, their plant tests, source-provenance routing, operational
CI wiring, and an evidence-only extension to the R2 browser smoke receipt. The
extension retains the already-checked per-launch browser facts so the final
verifier can audit them independently; it changes no rendering or visual
behaviour. The slice does not edit an R1 crate/viewer/contract, an R2
compiler/decoder/catalog/renderer/UI, either scene or expected plan, the
second-author packet, or the committed browser evidence.

The R1 viewer's unchanged tests require `dist/` beside their source. To satisfy
that accepted path assumption without writing the committed application tree,
the harness copies and digest-verifies the exact tracked viewer files plus the
required generated runtime inputs beneath its supplied output, stages the
distribution in that output-local mirror, and requires all 104 unchanged tests
to pass with zero skips. This is proof topology only; it changes no accepted R1
byte or meaning.

The harness validates and digest-records host tools before proving an outer
network connection, creating a fresh network namespace, and proving that the
same connection is blocked inside it. It enables loopback only, drops
privileges back to the invoking user, and runs the proof beneath a read-only
root/filesystem view whose only persistent writable roots are the supplied
output and checkout-local `target/`. Its independent verifier recomputes the
recorded counts, hashes, timing arithmetic, ceilings, distribution trees,
mount controls, and process/network closure before it emits a PASS receipt.
Generated evidence is kept external to the candidate and binds the exact
candidate commit/tree; committing a generated run would move the candidate and
require a new run.

The checkout-wide disk observer uses the process's allowed CPU list and
requires at least three allowed CPUs. On the 12-CPU reference host it pins the
sampler controller to CPU 0, ordinary-priority `du -sm -- <checkout>` walks to
CPUs 1–5 with idle I/O priority, and proof workloads to CPUs 6–11.
Workers retain canonical integer-nanosecond start times taken immediately
before each successful walk, and the controller publishes them in
chronological order while independently requiring unique contiguous launch
ordinals. A distinct final row must start after the canonical timestamp stored
in `host/disk-sampler.stop`. Sampler identity includes PID, process group,
session, start ticks, and affinity; shutdown is bounded and proves that the
dedicated session has closed.

## Preserved execution history and repair disposition

Candidate `5581e8977170157b85245bd5eec06bffc60640e4` first completed an author
run whose generated receipt has SHA-256 `c21f8837b0bfd4e0dd589d09e600f9d6c4ed0b493a7dfd3e26c66bb5be29f046`
and whose `EVIDENCE.sha256` has SHA-256
`6cd2f976a084083e87b56406da88087c9a0a0a0fdf4bd0ea2249a0257f6daebb`.
The external author execution record has SHA-256
`50406bdec9c8125b72effa6448995462611ce99a36bc3de057f163583a4810e9`.

The formal Luna Max exact-head rerun of that same candidate failed closed. All
33 workload commands exited zero, but retained disk-sample gaps of 114 ms and
104 ms exceeded the unchanged 100 ms ceiling. No final receipt was emitted.
The preserved reviewer report is
`/data/dev/src/nomos-r2-reviewer-5581e89.XaYGWP/reviewer-report.md`, SHA-256
`b11b36a24203f0ba4c00766ad7428aa6b9f039ecb2ff8b783b41cd63a7a982cc`.

The first authorized repair candidate,
`0e04aeefbb28dd30162edad07d7713585c6b0c0d` (tree
`702f4b68b308ca44669438eb36b45d1971ab18fb`), also failed closed in a
fresh, detached, standalone author clone. Commands 1–24 exited zero; command
25 exited 1 because its long-history plant rejected the live schedule, while
the outer observer accumulated 32 active `du` walks and failed its concurrency
bound. No final receipt or evidence manifest was emitted. The preserved
failure report is
`/data/dev/src/nomos-r2-author-0e04aee.t5UFQP/author-failure-report.md`,
SHA-256
`37d3a402bed56d70ce195e94f2b2cef59b3429024bc61b22cbd14ac358d36212`.
It binds the external streams, command ledger, command-25 logs, and unfinished
raw sampler rows. This run remains red.

The subsequent implementation repair leaves the ceiling and exact `du -sm`
walk unchanged. It replaces thousands of persistent per-launch
acknowledgement files inside the measured checkout with one ordinal-bound,
ephemeral acknowledgement, restores ordinary CPU priority while retaining
idle I/O priority and the isolated CPU set, and makes the bookkeeping-history
plant deterministic rather than host-schedule-sensitive.

On 2026-08-28 the owner authorized an implementation repair, not a contract or
ceiling change. The repair preserves the failed rerun as red evidence, does not
relabel it, and requires new author and non-author executions at the repaired
exact head before any R2 disposition.

Commands used during implementation include repository reads, `apply_patch`,
shell and Node syntax/tests, the four accepted workspace checks, output-local
R1/R2 rehearsals, and the final standalone network-isolated proof. Development
failures are not evidence. The PR and external author/non-author receipts bind
the exact final green commands, environment, outputs, and candidate identity.
