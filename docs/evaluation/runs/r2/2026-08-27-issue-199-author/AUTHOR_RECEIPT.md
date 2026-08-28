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
partial, or fewer-than-three-physical-core topology. It assigns one complete
sibling group to the sampler controller, then splits the remaining complete
groups between disk walks and the proof workload; when that remainder is odd,
the disk-walk side receives the extra group while the workload retains at
least one. On the 12-logical-CPU, six-core reference host, the controller uses
CPUs `0,6`, a bounded pool of 32 persistent workers uses CPUs
`1,2,3,7,8,9`, and the proof workload uses CPUs `4,5,10,11`. The workload
enters its mask before the sampler is launched; all three roles are pairwise
physically disjoint.

Each pool worker is a direct child in the sampler's dedicated session, enters
the walk mask once before reporting readiness, and receives ordinal-bound work
over its private controller-owned channel. It invokes ordinary-CPU-priority,
idle-I/O-priority `du -sm -- <checkout>` without per-sample affinity setup.
Workers retain canonical integer-nanosecond start times taken immediately
before each successful walk. The controller publishes them in chronological
order while independently requiring unique contiguous launch ordinals. All 32
workers are ready before use, but no more than four exact walks may be active
on the reference host's isolated three-group disk mask. A full gate waits
against its own four-second monotonic deadline, remains subject to an earlier
active drain deadline, and never dispatches a fifth walk concurrently. One
controller derives the exact absolute schedule
`origin + ordinal * 50 ms`; it never turns that schedule into a relative delay.
The recorded nominal interval remains 50 ms and the unchanged maximum
consecutive retained-start gap remains 100 ms.
After process closure, the parent requests a drain while the canonical stop
marker is still absent. Request, ready, and stop records are complete decimal
lines published by same-directory atomic rename without overwriting an
existing destination. The controller maintains fixed-origin bridge coverage
until the request-time workers quiesce and always retains at least one
post-intent bridge, including when no worker is live at intent. Before
readiness it validates the complete scheduled-only ledger, bridge timestamp,
unchanged 100 ms maximum gap, and a 75 ms freshness bound. The parent then
writes `host/disk-sampler.stop`; after publishing readiness the controller
starts its own fresh six-second Linux-monotonic wait, revalidates the unchanged
request marker on every poll, and requires the stop timestamp to remain within
the 100 ms handoff window. The parent's separate six-second preparation window
begins before readiness, so a count-dependent controller wait cannot expire
first merely because host scheduling makes one-millisecond sleeps expensive.
After the initial required sample, the controller checks stop again whenever
a full gate frees and immediately after selecting an identity-stable worker.
It launches no scheduled row after stop and launches the distinct final row.
The handoff validator preserves the existing numeric sort, row grammar,
contiguous-ordinal, arithmetic, ordering, gap, bridge, and freshness checks. It
scans the sorted ledger in one `awk` process so validation work itself fits the
freshness bound, but compares and adds absolute nanoseconds as canonical
decimal strings rather than lossy IEEE-754 numbers. Final publication then
independently repeats the full ledger validation in Bash.
Sampler identity includes PID, process group, session, start ticks, and
affinity. Empty live-task procfs `stat` or `status` snapshots receive at most
three immediate reads; a nonempty malformed snapshot fails immediately, and a
third incomplete read remains closed. A successful affinity read is bounded
by a second matching stat identity. Each pool-worker identity additionally
includes its direct sampler parent. A failed process-substitution channel
launch first closes its owned
result descriptor, restoring the descriptor capacity needed for the
controller's identity check, then closes the already-verified dedicated
sampler group. After a successful launch, every child and owned descriptor is
registered before `/proc` identity capture; an unbound launched child
therefore closes through that same group rather than escaping the tracked
pool. Affinity is rechecked after a successful result, before idle reuse,
before stop is sent, and while a live worker is polled for shutdown. A
still-live worker whose structural identity changes during shutdown aborts the
dedicated group instead of being marked reaped. Worker result collection never
waits a live or mismatched PID. A monotonic-clock, deadline-construction, or
sleep failure during orderly shutdown also aborts the group rather than
returning with live children. A saturated four-walk gate receives a fresh
four-second Linux-monotonic deadline but cannot outlive an already-running
drain deadline. Drain-time bridge scheduling and result collection, the
terminal set, and orderly pool shutdown receive their applicable bounded
deadlines. The parent allows its separate six-second monotonic preparation
window before its identity-bound TERM/KILL watchdog proves that the dedicated
session has closed.

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

Candidate `41cd82765f88ce22d8bc66baa55db2cdab81e1c8` (tree
`836a76944faa63ac154d5cc188943eed18bb0cc3`) then ran once in a new fresh,
detached, standalone author clone. Commands 1 through 24 exited zero. Within
command 25 all 111 Node tests passed, but the dedicated hung-sampler plant
failed. Its deliberately blocked workers reached the independent 32-worker
cap; scanning that active set made the controller's nominal four-second,
400-iteration wait outlive the parent watchdog, so the controller closed
without emitting the worker-timeout diagnostic the plant required. Failure
cleanup then retained two outer-sampler gaps of 105,693,440 ns and 111,628,800
ns. That cleanup intentionally bypasses the normal request/ready handoff after
an already-red workload and is not green-path terminal evidence. No command
after 25 ran, and no final receipt, evidence manifest, disk summary, compile
benchmark, or browser benchmark was emitted. The preserved failure report is
`/data/dev/src/nomos-r2-author-41cd827.yklBao/author-failure-report.md`,
SHA-256
`2238ca6f8b4912e4057702835ee0c4a0acb6141a2b2112b11f8c752028cb388b`.
It binds both external streams, the command ledger, raw sampler rows,
environment, command-25 logs, and the retained hung-sampler fixture. This run
remains red.

The following repair replaces iteration counting with an absolute
Linux-monotonic deadline. The same deadline begins when drain intent is first
observed, is checked while bridge launches continue, and is reused while that
active set is reaped; scan work therefore cannot silently extend the bound.
The terminal worker set gets a fresh deadline. The hung plant now leaves one
request-time worker blocked while later bridges complete, proving deadline
closure without conflating it with the separately retained 32-worker-cap and
parent-watchdog plants. A controller-only scripted monotonic-clock plant
advances one second per probe and requires exactly the six values from one
through six seconds, including the pool-readiness deadline and the single
active-set deadline, so the former 400-iteration loop cannot pass by waiting
longer. Disk publication and summary helpers move into the already-separated
disk-control library so routinely edited code files remain below 1,000 lines;
their behaviour is unchanged.

Candidate `35abe10213aa6c12b58bf2e328979351e499d8ff` (tree
`98c67276e58cff3d9ff00dfcb2e1af1bcbfa5dc3`) then ran once in a new fresh,
detached, standalone author clone. All 33 workload commands exited zero. The
clean release build completed in 22.58 seconds; the compile-latency median was
40,848,116 ns and the p95 was 54,298,290 ns; peak checkout disk was 1,357 MiB.
All four measurements were below their unchanged ceilings, and the browser
smoke reproduced the committed contact-sheet bytes. The observer retained
6,156 complete, contiguous scheduled rows but rejected nine successful-attempt
start gaps over 100,000,000 ns during browser smoke and the clean release
build; the maximum was 158,897,664 ns. Worker-side affinity setup and attempt
startup remained exposed to observer-core contention, so more frequent nominal
launch deadlines did not ensure retained-start coverage. Handoff validation
therefore did not publish readiness, the stop marker, a terminal row, disk
summary, evidence manifest, or final receipt. The preserved failure report is
`/data/dev/src/nomos-r2-author-35abe10.fXY8zx/author-failure-report.md`,
SHA-256
`e08e67d527cd8d518ae80400458eeedbeda56d79c6c562ed143ad877dfa92ef5`.
It binds both external streams, the complete command ledger, rejected raw and
sorted sampler rows, drain request, environment, build measurement, compile
summary, browser receipt, and contact sheet. This run remains red and will not
be retried at that commit.

On 2026-08-28 the owner authorized an implementation repair, not a contract or
ceiling change. The repair preserves the failed rerun as red evidence, does not
relabel it, and requires new author and non-author executions at the repaired
exact head before any R2 disposition.

That repair reserves a third complete physical-core role for the controller,
moves the proof workload to its disjoint mask before sampler creation, and
replaces per-sample worker creation with the bounded ready-before-use pool
described above. The successful-attempt timestamp, exact `du -sm -- <checkout>`
invocation, idle I/O class, 32-walk concurrency bound, 50 ms nominal absolute
schedule, 100 ms retained-start-gap limit, and 8,192 MiB ceiling are unchanged.
The former monolithic proof library and plant suite are decomposed into a
sampler library and source-only disk-plant files so every routinely edited code
file remains below 1,000 lines.

The first local source freeze for that repair,
`236ee629025d815db4a800c354a81100ea78ec77`, did not enter the formal author
protocol. Its exact-head local matrix first failed the real pool-affinity plant
at the parent-to-controller stop handoff. The retained development fixture
`target/r2-complete-proof-plants.Wx51oo` has 47 complete scheduled rows, all at
17 MiB, with an 8,424,704 ns minimum and 17,100,288 ns maximum retained-start
gap. Its matching request and ready markers bind
`1787902277384374304`; all 32 worker result streams report success, but no stop
marker, terminal row, or published ledger exists. A later traced execution was
green, so the evidence identifies a host-sensitive handoff race rather than a
failed walk; “green on retry” was not accepted, and this local candidate was
retired without a formal run.

The follow-up source repair changes neither handoff ordering nor its 100 ms
freshness ceiling. It replaces the controller's former 5,000-poll post-ready
wait with the fresh six-second monotonic window described above. The real
process-affinity plant publishes a canonical stop directly because drain
request/ready/stop ordering is independently exercised; this keeps its verdict
about the pool and exact walks independent of a second host-scheduled parent
handoff. Concurrent development rehearsals then exposed two more test-only
count/timing assumptions: the positive plant's nominal five-second readiness
poll could end while otherwise valid workers were still starting, and its
two-second cap-plant walks could finish before dispatch 33. The positive plant
now uses explicit 12-second readiness and 8-second row-accumulation monotonic
deadlines. The cap plant holds its exact walks behind an unreleased gate, so
the 32-worker boundary cannot disappear when the host is slow. Two concurrent
complete 40-plant suites pass with those repairs. These development rehearsals
are not acceptance evidence.

A subsequent lifecycle audit made the channel-launch plant prove the real
post-process-substitution failure rather than only a generic redirection
failure. The controller exhausts dynamic descriptors only after Bash has
created the process-substitution channel; the child records its actual
`/proc` parent, process group, and session, while the controller's second
self-identity read supplies deterministic synchronization. That plant exposed
that an identity-bound group abort itself needed one free descriptor. The
source now closes the already-owned result descriptor before the abort and the
plant requires exit 137 plus complete session closure. The same audit found
that shutdown clock/deadline/sleep helper failures could return before live
workers closed. Those branches now abort the verified group, and a
caller-sensitive monotonic-clock plant requires that path to exit 137, publish
no ledger, and close the whole session.

The first attempted consecutive rehearsal after that repair remained red. In
retained fixture `target/r2-complete-proof-plants.bMisRz`, the history plant
published a complete 35-row ledger, made 240 process probes, removed its state,
and otherwise satisfied the bounded-active-set assertion, but missed the
plant's arbitrary 40-row minimum because it published stop after a fixed 0.8
second sleep. That is a host-sensitive fixture assumption, not evidence of
quadratic process history, and it was not accepted on retry. The history plant
is now separately decomposed and waits against explicit 12-second readiness
and 8-second row-accumulation monotonic deadlines before publishing stop. Its
probe ceiling, production sampler, disk method, concurrency bound, schedules,
and acceptance ceilings are unchanged.

Candidate `ba06cc9df3a9e08f3b241cd5c3fecd70339eeee8` (tree
`7e1329f46add263f515575820c16d079fe004fe9`) then passed its exact-head local
matrix, including the workspace checks, 111 Node tests, 87-row provenance
inventory, nine provenance plants, and complete 42-plant suite. A read-only
Luna Max lifecycle audit nevertheless blocked candidate promotion before any
formal proof. It found that active-set wait setup could return before aborting
live work, partial readiness reused the wait-reaped flag, and four plants
either disarmed cleanup early or treated an indeterminate session inspection
as closure. This candidate remains a local red review result and did not enter
the standalone author protocol.

The follow-up separates `worker_ready` from `worker_reaped`; every successfully
launched child now remains wait-owned even if another readiness record fails.
Monotonic or deadline setup failure while an active sample set is waiting now
aborts the verified dedicated group. Deterministic plants make worker 31 emit
a malformed readiness record after workers 0-30 are ready and require 32
distinct explicit waits, then fail the first active-set wait clock probe and
require exit 137 with no publication. The blocked-stop, scripted-deadline,
capture-mismatch, and history plants keep their cleanup PID armed and require
session inspection status exactly 1 before accepting closure.

A second read-only Luna Max audit then found that the extracted history plant
had retained its cleanup PID but had launched an ordinary background function,
not a session leader. Its session-membership query could therefore report
closure for an ID that had never owned the workers. This was found before a
candidate freeze or formal run. The plant now launches under `setsid`, proves
that its root owns both its process group and session before accepting
readiness, and keeps the PID armed until status-1 session closure. The parent
trap kills that exact process group, with a root-PID fallback if session setup
itself fails.

Deterministic plants reject partial or fewer-than-three-group topology and
bind the exact six-core split. A real process plant verifies one affinity
operation for each of the 32 persistent direct children and proves that more
than 32 exact fake-`du` invocations all remain in the walk-only mask while the
controller remains in its own mask. That plant derives its retained ledger
timestamps from contiguous ordinals so unrelated host scheduling cannot
duplicate the separately planted exact gap boundary; it still invokes every
exact walk, while the retry plant independently proves authentic attempt-time
retention. The source assertion itself requires more than 32 completed exact
walks, so fewer samples cannot satisfy that verdict.
The cap plant starts the complete 32-worker pool, holds and counts exactly four
walks, advances only the launch-slot clock through its four-second deadline,
proves that a fifth exact walk never starts, publishes no ledger row, and
proves complete session closure. A real descriptor-exhaustion plant makes
process substitution start a
held child while the controller's dynamic request-descriptor duplication
fails, then proves the failed channel launch closes the sampler session. A
second held-child mutation refuses startup identity capture and proves that a
registered process-substitution child cannot survive its controller. A live
affinity mutation moves an idle worker onto the controller mask and proves
that shutdown refuses the drifted identity without leaking the session. A
separate live structural-identity mutation keeps an idle worker alive while
the controller observes a changed process-group tuple, and proves that the
dedicated group is aborted rather than the worker being forgotten as reaped.
Exact-integer handoff plants distinguish adjacent nanoseconds above 2^53,
reject a one-nanosecond origin/elapsed mismatch, reject a 100,000,001 ns
retained-start gap, and refuse control-marker values above signed 64-bit range.
The post-ready wait both accepts a stop first published on synthetic poll 5,001
and expires with the exact diagnostic at its synthetic six-second monotonic
boundary. Existing retry, chronological-publication, exact-gap,
absolute-schedule, drain, terminal-order, deadline, identity-mismatch, and
parent-watchdog plants remain green. Two consecutive complete 42-plant suites
passed after the subsequent readiness, active-wait, and exact-session-closure
repairs. A further complete 42-plant suite passed after the history plant's
dedicated-session correction. These runs are development rehearsals, not
acceptance evidence.

Candidate `c15b6cdc371bac427a40ddb6c7bfee226e6b2771` (tree
`69aed5b098e365f6389b2bf46709f6aba1f07523`) passed a clean exact-head local
matrix and three read-only Luna Max audit lanes, then ran once in a fresh,
detached `git clone --no-local` author checkout. All 33 workload commands
exited zero. The clean release build took 18.40 seconds, maximum-scene compile
latency recorded a `79050673/2` ns median and 44,802,909 ns p95, browser smoke
reproduced the committed contact sheet, and peak checkout disk was 1,356 MiB.
The observer nevertheless rejected 40 retained-start gaps over 100,000,000 ns
among 5,907 scheduled rows; the maximum was 286,006,134 ns during R2 browser
smoke. No drain readiness, stop marker, terminal row, disk summary, evidence
manifest, or final receipt was emitted. The preserved report is
`/data/dev/src/nomos-r2-author-c15b6cd.ToCgmn/author-failure-report.md`,
SHA-256
`b65c89132ec28cdf7f12ed66ce0dee1cdfe570ed5236e8956a29b39402541066`.
It binds both external streams, the complete command ledger, rejected raw and
sorted rows, drain request, environment, build and compile measurements,
browser receipt, and contact sheet. This formal run remains red and will not
be retried at that commit.

Post-failure diagnostics found no concurrency-cap diagnostic or exhausted
retry in the formal artifacts. The two-phase controller fell about 47.99
seconds behind its 25 ms union timeline: actual starts averaged 33.09 ms, and
32 concurrent final-tree walks took roughly 0.36--0.53 seconds each. A single
absolute 50 ms phase without an active gate remained red in development under
the exact browser workload: retained fixture
`/data/dev/src/nomos-r2-single-phase-probe.76MnlN` had 322 private rows, eight
gaps above 100 ms, and a 227,920,101 ns maximum. An immediate four-walk refusal
then correctly exposed that backpressure must wait rather than fail as soon as
the gate is full.

The selected repair removes the optional second phase, retains all 32
prestarted and identity-bound workers, and admits at most four live exact
walks. When all four are busy, the controller continues from the fixed-origin
schedule after a bounded slot wait; authentic worker-side starts and the
unchanged 100 ms validator, not nominal deadlines, decide the result. The
deterministic schedule plant records exact `(origin, ordinal, 50 ms)` helper
arguments while ordinal 1 retains a deliberately 7 ms-late worker timestamp.
The real overload plant starts all 32 workers, holds exactly four exact walks,
and requires the fifth to remain unlaunched through a scripted timeout and
complete session abort.

Three consecutive exact-browser development probes with this repair retained
144, 144, and 142 rows with maximum gaps of 59,079,158 ns, 64,089,631 ns, and
59,145,073 ns. A clean release-build probe retained 328 rows with a 58,846,725
ns maximum. All four had zero gaps over 100 ms; the browser probes reproduced
the exact contact-sheet digest. The terminal-order plant and complete
42-plant suite then passed. These are load-bearing development diagnostics,
not acceptance evidence; only a new exact candidate may enter the formal
author protocol.

A final pre-freeze Luna Max source audit found two launch-boundary races before
that new formal run. A stop marker published while a saturated gate collected
a completed walk could be skipped by the gate's early success return, and the
gate's fresh four-second timeout could outlive the already-established drain
deadline. The controller now checks the post-initial stop boundary both after
gate collection and immediately before the request write, and every scheduled
launch boundary enforces the shared drain deadline. Three production-path
plants publish stop while a held gate frees, publish stop during the final
identity-stable worker selection, and advance a saturated gate beyond its
earlier drain deadline. They require no extra scheduled dispatch, the distinct
terminal row where success remains possible, exact status-137 session abort
where the drain deadline expires, no premature ledger publication, and proved
session closure. Depending on live host scheduling, the previously existing
hung-drain plant reaches that same shared deadline either at the next
scheduled-launch boundary or in request-time result collection; the synthetic
slot plant independently pins the former. The complete 45-plant suite passed
after these repairs; this remains development evidence, not acceptance.

Candidate `8b5dc036d097e2a7b3c5341e3f4c344787795c5c` (tree
`1863b4c873ccf23aebd985243145b65c2c00f3da`) then passed the clean exact-head
matrix and three read-only Luna Max audit lanes. Its single formal author
invocation used a full, detached, clean `git clone --no-local --no-hardlinks`
at `/data/dev/src/nomos-r2-author-8b5dc03.cb2qiJ/checkout`, but failed the
harness output preflight with exit 1. The operator had created checkout-local
`target/` while incorrectly requiring the exact
`target/r2-complete-proof` path to be absent; the harness requires that exact
output path to exist already as a real directory. Standard output was empty
and standard error was exactly
`R2 complete proof: FAIL: output must already exist as a real directory`.
No command ledger, sampler session, build, browser process, evidence artifact,
or generated receipt was created, and the checkout remained clean. The
preserved report is
`/data/dev/src/nomos-r2-author-8b5dc03.cb2qiJ/author-failure-report.md`,
SHA-256
`adf3c189cc7c2e5b11b9c87c78265bfd69240a5a4afa93253ca77658a643789e`.
This operator-preparation failure is formal red evidence and will not be
retried at that commit. The next candidate changes only this source receipt;
its preparation must create and validate the exact empty output directory
before the one formal invocation.

Candidate `15d7504053a2d40dc55c24ce13c121683c3f2698` (tree
`50db0dc24483b0f9aeaf451e3fbcc33b8fabfcfc`) passed its clean exact-head
matrix and three read-only Luna Max audit lanes, then ran exactly once in a
fresh, detached, full, clean `git clone --no-local --no-hardlinks` author
checkout whose exact output directory existed and was empty. Commands 1--32
exited zero. Command 33 produced all 100 byte-identical maximum-scene compile
outputs, then failed its unchanged p95 ceiling: the median numerator was
`79436952/2` ns, p95 was 165,693,556 ns, and eight samples at ordinals 84--91
exceeded 100,000,000 ns. The observer retained 3,196 scheduled rows and one
terminal row at a 1,356 MiB peak, but rejected 12 retained-start gaps over
100,000,000 ns; the maximum was 138,697,920 ns. Browser smoke still reproduced
the committed contact-sheet digest, and the clean release build exited zero in
19.58 seconds. The preserved independently audited report is
`/data/dev/src/nomos-r2-author-15d7504.xZojZY/author-failure-report.md`,
SHA-256
`eb8af44b4bfcce7030d00f9355f33ce1c0dd9e37eee9686e54d36bc288a1e75e`.
It binds both external streams, the complete command ledger, rejected private
disk rows, environment and measurements, compile outputs, browser receipt, and
contact sheet. No accepted disk summary, evidence manifest, or final receipt
was emitted. This exact candidate remains formal red and will not be retried.

The next implementation repair changes no contract, command, schedule,
timestamp, or ceiling. It keeps the complete 32-worker identity-bound pool but
tightens admission from four active exact walks to two: one per isolated
physical-core group in the reference host's walk mask. Deterministic overload,
terminal-deadline, stop-boundary, and drain-deadline plants now saturate that
two-walk gate and prove that no third scheduled walk crosses its launch
boundary. The unchanged authentic retained-start timestamps and 100 ms
validator remain the cadence authority; only a new exact candidate may enter
the author protocol.

The first live browser probe in the long-lived implementation checkout was
discarded as unrepresentative: retained plant fixtures had grown that checkout
to 3,033 MiB, and the observer correctly rejected eight gaps with a
1,399,887,872 ns maximum. A fresh standalone development clone carrying the
same source diff then reproduced the browser contact-sheet digest with 148
rows, a 230 MiB peak, zero gap violations, and a 72,450,560 ns maximum. Its
first observed compile run hit a host-wide slow interval: the observer itself
remained green at 78,611,200 ns maximum gap, but the compile median was
`117887660/2` ns. An immediately adjacent no-observer control also failed the
median at `110895997/2` ns, while its next no-observer run passed at
`77141488/2` ns median and 41,261,155 ns p95. The next observed run passed at
`77553966/2` ns median and 41,409,743 ns p95 while retaining 96 disk rows with
zero violations and a 69,942,016 ns maximum gap.

The same fresh clone was then expanded to 1,840 MiB, above the prior formal
peak, with debug and two release build trees. Browser smoke again reproduced
the exact contact sheet while retaining 148 rows with zero violations and a
79,287,808 ns maximum gap. The maximum-scene benchmark then passed at
`79624417/2` ns median and 44,438,981 ns p95 while the observer retained 104
rows at a 1,852 MiB peak with zero violations and a 90,762,496 ns maximum gap.
These load probes and the green 45-plant suite are development evidence, not
acceptance; they justify freezing a new candidate without relabeling any prior
run.

Candidate `ab0c219f5a442756ba1e9c5c64624a91388fd8e9` (tree
`6013d5935777e39f02d1aca73b02f434be0e54b2`) then passed the clean exact-head
matrix and three read-only Luna Max audit lanes. Its single formal author
invocation used a fresh, detached, full, clean
`git clone --no-local --no-hardlinks` at
`/data/dev/src/nomos-r2-author-ab0c219.2BEGSn/checkout`, with the exact
checkout-local output directory already present, real, and empty. All 33
workload commands exited zero. The clean release build took 16.65 seconds;
100 compile outputs were byte-identical, with median numerator `90984889/2`
ns and p95 57,869,989 ns; browser smoke reproduced the exact committed contact
sheet; and peak checkout disk was 1,355 MiB. All four workload and resource
ceilings passed.

The observer nevertheless rejected its drain handoff. Its 2,897 complete,
contiguous scheduled rows contained 57 retained-start gaps over 100,000,000
ns, with a maximum of 146,686,568 ns between ordinals 1,336 and 1,337. The
drain-request marker exists, but validation therefore published no
drain-readiness marker, stop marker, terminal row, public disk rows beyond the
header, disk summary, evidence manifest, or final receipt. The only outer
diagnostic was the then-generic
`R2 disk sampler: one or more scheduled samples failed`; the row evidence and
missing handoff publications establish a retained-gap validation failure, not
a failed `du` worker or workload command. The preserved independently audited
report is
`/data/dev/src/nomos-r2-author-ab0c219.2BEGSn/author-failure-report.md`,
SHA-256
`886e1e0f185c221afecd37b38a214340890a56ae060f8a0c07c0f0e40dd052e9`.
It binds both external streams and every retained artifact available from the
failed run. This exact candidate remains formal red and will not be retried.

Development probes then separated the concurrency cap from physical-core
allocation. Three active walks on the former two-group disk mask produced one
108,816,640 ns red browser maximum followed by a 90,198,272 ns green maximum.
With one controller group, three disk groups, and two workload groups on the
same six-group host, two browser probes passed at 84,058,368 ns and 77,169,664
ns maximum gaps. An observed maximum-scene compile passed at median numerator
`78446561/2` ns and p95 43,529,976 ns while the disk observer's maximum was
85,816,832 ns. An observed clean release build took 23.51 seconds while its
maximum retained gap was 82,980,096 ns. The checkout was 1,865--1,906 MiB,
larger than every prior formal peak. These short probes are development
evidence only.

The selected repair therefore keeps the fixed 50 ms schedule, exact disk
method, authentic successful-start timestamps, 100 ms retained-gap ceiling,
32-worker identity pool, and every workload/resource ceiling unchanged. It
admits at most three walks and derives a topology split that reserves the first
complete sibling group for the controller, gives the disk side the extra group
when the remaining count is odd, and still leaves the workload at least one
complete group. On the reference host this is controller CPUs `0,6`, one
shared disk-worker mask `1,2,3,7,8,9` spanning three complete sibling groups,
and workload CPUs `4,5,10,11`. Deterministic plants saturate exactly three
walks, withhold the fourth, and bind that topology. Drain-handoff gap rejection
now emits the same exact retained-gap diagnostic already used by final summary
validation, so a future failure identifies the violated criterion directly.
Only a new exact candidate may enter the author protocol.

The first post-repair terminal-order run then retained a development red at
`target/r2-disk-terminal-order.OEAqyd`: request, readiness, and stop markers
were complete, but the parent-side helper refused closure. A focused live-task
diagnostic established that this host's procfs can return an empty
`/proc/<pid>/status` snapshot while the same task's stat identity and affinity
remain stable: 20 of 21,768 single reads were empty. The unmodified handshake
reproduced three false failures in 120 iterations even though its child exited
zero. A status-only retry still allowed another development red, so the repair
was completed across both identity snapshots and definitive session closure
rather than assigning an unproved cause to that second run.

The bounded procfs reader now retries only an empty snapshot up to
three immediate attempts. Nonempty files with a missing canonical row,
duplicate rows, malformed values, or malformed stat content fail on the first
attempt; three incomplete status reads return the prior closed status, and
three incomplete stat reads preserve process absence. Identity is read again
after affinity to exclude a PID-reuse splice. Session closure retries an
indeterminate scan only inside its existing 100-probe, 10 ms polling bound and
accepts only a definitive absent result. The decomposed source-only procfs
plant injects transient success, persistent failure, and immediate malformed
failure for both readers, while the parent handshake injects an indeterminate
closure scan. The complete 45-plant suite passed; separate live diagnostics
then completed 20,000 identity checks and 100 parent handshakes with zero
failures. These are development results, not acceptance evidence.

Development commit `d2e87d7f5e9e1047c1b750b621aa8cc63384cb0f` (tree
`90ad0293bbcb17ceb42cd9ba16d67a31080d854a`) then ran the exact complete
harness once in the fresh, detached, full, clean standalone clone
`/data/dev/src/nomos-r2-rehearsal-d2e87d7.p9WTZF/checkout`. This was
deliberately a development rehearsal whose result could be recorded into a
different final tree, not a formal author attempt. All 33 workload commands
exited zero. The clean release build took 23.57 seconds; maximum-scene compile
latency passed with median numerator `78446703/2` ns and p95 43,397,020 ns;
browser smoke reproduced the exact contact sheet; and peak checkout disk was
1,356 MiB.

The three-walk observer remained red at full scale. It retained 3,337 complete
scheduled rows with p50 49,963,598 ns, p90 54,529,075 ns, p95 57,228,026 ns,
and p99 63,697,137 ns consecutive-start gaps, but 14 gaps exceeded the
unchanged 100,000,000 ns ceiling. The maximum was 112,018,736 ns between
chronological rows with launch ordinals 1,770 and 1,771. Violations occurred
during R1 viewer tests, provenance plants, R2 viewer tests, R2 browser smoke,
and the clean release build rather than one isolated command. The exact new
retained-gap diagnostic preceded the existing generic sampler diagnostic;
drain validation correctly withheld readiness, stop, terminal, disk summary,
evidence manifest, and final receipt. The preserved report is
`/data/dev/src/nomos-r2-rehearsal-d2e87d7.p9WTZF/rehearsal-failure-report.md`,
SHA-256
`e4d043fa620b58c5a0db8172e398935b8ac4c482d8d198bbf7d040f49e45f895`.
This development commit remains red and will not be retried.

The selected follow-up changes only fixed observer capacity from three to four
active walks while retaining the controller-one/disk-three/workload-two
physical-group split. It does not alter the 32-worker pool, exact disk method,
idle I/O class, authentic timestamps, absolute 50 ms schedule, 100 ms gap
ceiling, workload, or any resource ceiling. The full rehearsal left large
workload margin—56.6 ms at compile p95 and 36.43 seconds at clean build—while
its cadence misses were only 0.27--12.02 ms over the limit. The fourth
in-flight walk supplies bounded coverage without reducing the workload to one
physical group. Deterministic overload, stop-selection, drain-deadline, and
terminal-order plants saturate exactly four walks and prove that a fifth is
withheld. A new development commit must pass full standalone rehearsals before
another exact candidate may enter the formal author protocol.

Commands used during implementation include repository reads, `apply_patch`,
shell and Node syntax/tests, the four accepted workspace checks, output-local
R1/R2 rehearsals, failure-injection plants, and fresh-checkout load probes.
Development failures and rehearsals are not acceptance evidence. The repaired
source still lacks a green formal standalone author proof; a new exact
candidate's external receipt and the later exact-head non-author receipt must
bind the exact green commands, environment, outputs, and candidate identity.
