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

The checkout-wide disk observer reads each allowed logical CPU's Linux
`thread_siblings_list` and fails closed on an absent, malformed, contradictory,
or fewer-than-two-physical-core topology. It assigns complete physical-core
sibling groups to the observer and the remaining complete groups to the proof
workload, giving an odd extra group to the workload. On the 12-logical-CPU,
six-core reference host, the sampler controller is pinned to CPU 0,
ordinary-priority `du -sm -- <checkout>` walks use CPUs `0,1,2,6,7,8` with idle
I/O priority, and proof workloads use CPUs `3,4,5,9,10,11`. Thus neither role
shares a physical core with the other.
Workers retain canonical integer-nanosecond start times taken immediately
before each successful walk, and the controller publishes them in
chronological order while independently requiring unique contiguous launch
ordinals. One controller derives two explicit absolute 50 ms phases from the
fixed origin: even ordinals are `origin + n * 50 ms`, and odd ordinals are
`origin + 25 ms + n * 50 ms`. It never turns either phase into a relative
delay. Their union samples every 25 ms, as the contract expressly permits,
while the recorded nominal interval remains 50 ms and the unchanged maximum
consecutive retained-start gap remains 100 ms.
After process closure, the parent requests a drain while the canonical stop
marker is still absent. Request, ready, and stop records are complete decimal
lines published by same-directory atomic rename without overwriting an
existing destination. The controller maintains fixed-origin bridge coverage
until the request-time workers quiesce and always retains at least one
post-intent bridge, including when no worker is live at intent. Before
readiness it validates the complete scheduled-only ledger, bridge timestamp,
unchanged 100 ms maximum gap, and a 75 ms freshness bound. The parent then
writes `host/disk-sampler.stop`; the controller requires its timestamp to
remain within the 100 ms handoff window, launches no scheduled row after it,
and launches the distinct final row.
Sampler identity includes PID, process group, session, start ticks, and
affinity. Worker capture and reaping never block on a live or mismatched PID;
one shared deadline per active worker set and the parent's identity-bound
TERM/KILL watchdog bound shutdown and prove that the dedicated session has
closed.

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

Candidate `3e9e1732264f69aec7daec4b6b7f3f8cf1851105` (tree
`87fcb8c5de70de2e0cae4d00fc3ca43e6a80a04f`) then failed closed in a new
fresh, detached, standalone author clone. All 25 closed command-ledger rows,
including 111 Node tests and 37 shell plants in command 25, exited zero. The
outer observer nevertheless accumulated 32 active walks while command 25's
synthetic receipt suite deleted its large accumulated fixture set, so the
harness rejected the sampler's lost identity. No final receipt or evidence
manifest was emitted. The preserved failure report is
`/data/dev/src/nomos-r2-author-3e9e173.qzqf3H/author-failure-report.md`,
SHA-256
`1938c1c3c629d47274e1434a72853e195400ea553fa68c57eecba3cda9c7d545`.
It binds the external streams, complete 25-row command ledger, command-25
logs, and unfinished raw sampler rows. This run remains red.

The following repair changes no verifier assertion, existing mutation plant's
failure criterion, disk method, or ceiling. It adds a terminal-order plant and
shortens the asynchronous plant's fake walk without changing the closure or
cadence behaviour that plant asserts. The synthetic receipt test constructs a
clean, detached,
standalone SHA-1 repository from `git archive HEAD` containing every exact
source byte read by the verifier or fixture builder. Its synthetic commit and
tree exercise all standalone-root and candidate-binding assertions; the
top-level final verifier, rather than this unit fixture, binds the real complete
candidate tree. The test creates one path-bound synthetic output, snapshots its
canonical files in memory, and between sequential cases removes only generated
extras and restores only the missing or changed baseline files. It therefore
no longer accumulates outputs or recursively deletes and recopies a complete
fixture for every plant. This reduces the measured transient fixture peak from
roughly 507 MiB to 16 MiB and removes the sustained metadata churn. Like the
neighboring viewer tests, it retains its final scratch fixture beneath
`host/tmp`; those bytes remain inside the measured write boundary and the final
evidence manifest binds them instead of racing the live observer with an
unrequired recursive teardown. The shell plant suite likewise retains its
closed fixture beneath checkout-local `target/`, which is within the measured
and permitted boundary; the final clean-worktree and write-boundary checks
still fail closed on any input or outside-target write.

Candidate `1639e9dd5b1dee84d101967869889236c94038ce` (tree
`032905e8d419fd36df030493597a691d21de4077`) then failed closed in another
fresh, detached, standalone author clone. Commands 1–32 exited zero, including
111 Node tests, all 37 shell plants, and the separate terminal-order plant in
command 25. Command 33 rejected a compile-latency median numerator of
`167806723/2` ns against the unchanged 50 ms ceiling; its 93,089,412 ns p95
remained below the unchanged 100 ms ceiling. Independently, the checkout-wide
observer rejected retained-start gaps of 196,455,424 ns and 129,176,064 ns
during command 15's three full-tree schema-plant archives and recursive
teardown. Peak checkout disk was 1,803 MiB, below its unchanged 8,192 MiB
ceiling. No final receipt or evidence manifest was emitted. The preserved
failure report is
`/data/dev/src/nomos-r2-author-1639e9d.5BfHUQ/author-failure-report.md`, SHA-256
`aff26e03866fb88724f134abe97fce75aeae110aea2bd2e077151ad6f26bf21d`.
It binds the external streams, complete command ledger, command-25 and
command-33 logs, and unfinished raw sampler rows. This run remains red. A
post-failure diagnostic without the observer measured the same binary at a
`78503626/2` ns median, supporting observer/workload interference rather than
a compiler regression.

The next authorized implementation repair preserves every red run and every
ceiling. The previous numeric CPU masks were logically disjoint but, on the
reference host, paired every observer CPU with a workload CPU on the same
physical SMT core. The repaired harness partitions complete sibling groups and
records the read topology and resulting masks. It also restores a single exact
absolute 50 ms schedule; a controller-level fake-clock plant adds 7 ms of work
after every launch and proves that later launches remain on the fixed-origin
deadlines rather than drifting as relative sleeps. Command 15 now archives only
the complete crate scope,
the schema checker, and its three registers through
`docs/evaluation/r2-schema-ownership-plants.sh` into three roughly 2 MiB plants;
each plant proves its exact regular-file inventory, installs its named
mutation, requires the mutation-specific diagnostic, and is retained beneath
the proof output's `host/tmp`, where the closed evidence manifest binds it,
rather than recursively deleted while the observer is live. No accepted
source, checker assertion, `du -sm` method, successful-attempt
timestamp, process-closure assertion, or budget ceiling changes.

Candidate `42b92ada6865e7998c4a8bf8e37781760164f693` (tree
`a1813b1df3f46d45abc2023efef7d868f9cab1af`) then ran in a new fresh,
detached, standalone author clone. All 33 workload commands exited zero. The
compile-latency median numerator was `81521596/2` ns and the p95 was
`46609834` ns, both below their unchanged ceilings. The physical-core split
therefore removed the prior compile interference, and the schema plants no
longer produced a cleanup spike. Peak checkout disk was 1,356 MiB. The
observer nevertheless rejected eight retained-start gaps over 100,000,000 ns;
the maximum was 103,188,736 ns. Chronological rows show that successful retry
timestamps reordered around otherwise present controller launches. No final
receipt or evidence manifest was emitted. The preserved failure report is
`/data/dev/src/nomos-r2-author-42b92ad.qulGiw/author-failure-report.md`,
SHA-256
`fd116ea091e0178c88180c4266c37f63224a53a96aad84dea74c3c83047de8b6`.
It binds the external streams, complete command ledger, raw and sorted sampler
rows, environment, and compile outputs. This run remains red.

The selected follow-up repair changes no contract, evidence timestamp, disk
command, concurrency bound, or ceiling. It runs the sampler's one controller
as two fixed-origin 50 ms phases offset by 25 ms, which decision 0024
explicitly permits as more-frequent sampling. Deterministic plants bind both
phase sequences, fixed-origin rather than relative scheduling, a delayed
successful retry, chronological publication, the exact 100 ms gap boundary,
terminal ordering, and fail-closed overload behaviour. Physical-core role
isolation and the sparse retained schema plants remain unchanged.

Candidate `cc32f4ce25bd80175cf7646f6962288a9b403b31` (tree
`6f10f17975c43560e24f6caf079aebb52de58a01`) then ran in a new fresh,
detached, standalone author clone. All 33 workload commands exited zero. The
compile-latency median numerator was `82800004/2` ns and the p95 was
`46214693` ns; peak checkout disk was 1,358 MiB. All three remained below
their unchanged ceilings. The sampler retained 4,505 complete rows but
rejected five steady-state retained-start gaps from 102,449,920 ns through
112,933,376 ns, plus a 136,123,648 ns final scheduled-to-terminal gap. Its
controller had accumulated roughly 30.4 seconds of fixed-origin schedule lag
because every production launch synchronously waited for a worker
acknowledgement. The final gap separately occurred while the controller waited
for a live scheduled walk before launching its terminal row. No final receipt,
evidence manifest, or disk summary was emitted. The preserved failure report
is `/data/dev/src/nomos-r2-author-cc32f4c.TiBgeF/author-failure-report.md`,
SHA-256
`efb01708d632b1c56b669b7cda345e71eeaac85697f0f7c45ec497a74f5022ca`.
It binds the external streams, complete command ledger, raw and sorted sampler
rows, environment, and compile outputs. This run remains red.

The next repair keeps the two absolute 50 ms phases and every existing
ceiling. Production launches no longer wait for an acknowledgement file;
PID/start identity, the 32-worker cap, exit status, exact successful-attempt
timestamps, complete contiguous ordinals, and chronological publication still
fail closed. The six-argument acknowledgement form is retained only for the
direct retry plant so successful-attempt timestamp semantics remain
independently exercised. Terminal shutdown now uses atomically published
pre-stop intent, forces a post-intent bridge even with an empty active set,
validates the scheduled handoff ledger and freshness, and atomically publishes
readiness. Only then does the parent atomically publish the canonical stop
marker, after which no scheduled row is launched and the distinct terminal
walk begins. Plants additionally bind partial-publication invisibility,
multiple delayed timestamps arriving out of launch order, and bounded cleanup
of a hung dedicated sampler session. This resolves shutdown coverage without
reinterpreting “stop,” fabricating a timestamp, or changing the accepted
method.

On 2026-08-28 the owner authorized an implementation repair, not a contract or
ceiling change. The repair preserves the failed rerun as red evidence, does not
relabel it, and requires new author and non-author executions at the repaired
exact head before any R2 disposition.

Commands used during implementation include repository reads, `apply_patch`,
shell and Node syntax/tests, the four accepted workspace checks, output-local
R1/R2 rehearsals, and the final standalone network-isolated proof. Development
failures are not evidence. The PR and external author/non-author receipts bind
the exact final green commands, environment, outputs, and candidate identity.
