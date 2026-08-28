---
title: R2 revision 3 — replace recursive observation with bounded filesystem accounting
status: Owner-authorized; R2.md revision 3 in force
number: 0025
date: 2026-08-28
owner: Peter Permenter
issue: 199
supersedes_r2_revision: 2
establishes_r2_revision: 3
r2_revision_2_sha256: 770740bad1c85cf7ea9dcd16f8c25e01766064d3b59d7f0bb9d438c289a6e638
revision_2_authority: docs/decisions/0024-r2-final-proof-finalization-order.md
revision_2_authority_sha256: 0356b3918a5c2643c36e16555e8ef78155bf893a8c3c21e4f75263f8289feea0
issue_199_revision_2_body_sha256: 8ffd30e7a213e991732ea6031743542eb68d9b80fe6d4989ed58052617352dcc
controlled_recursive_observer_candidate_commit: 032913203113843fb775b21d335cff4f8970c714
controlled_recursive_observer_candidate_tree: 22ea37581edca558b78f5698218c89ccc3f1ad4b
controlled_recursive_observer_report_sha256: 26d84e003e7ae312dc1dda562fde73c313a044c69d9ea447289d53f23c3e508c
final_recursive_observer_candidate_commit: 4b12ed0cff29962885536723beb1e28d31c79acb
final_recursive_observer_candidate_tree: 831edad7b8ce1915c9f79ec2ef5a7ea0806efede
final_recursive_observer_record_commit: 157c7fe2466f9bbe75f84eadd69df66da5369271
final_recursive_observer_record_tree: a93823252b8b3fba348a0ec10c3ffb63d946f5dd
final_recursive_observer_report_sha256: 1e5c7b8a5628986b7db85cf557d98f039a864c71e5779243ff1b9d3f95a8beb4
r1_contract: RUNTIME.md revision 4
r1_contract_sha256: dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593
---

# R2 revision 3 — replace recursive observation with bounded filesystem accounting

## Decision authority

Peter Permenter authorized this record exactly as proposed on 2026-08-28. It
was prepared under issue #199 after the final controlled implementation of the
revision-2 recursive disk observer missed the unchanged retained-start-gap
limit in a fresh standalone development rehearsal. Section 13 requires a
falsified measurement assumption to be recorded and repaired rather than
hidden by further implementation tuning. This decision takes effect as written
and establishes `R2.md` revision 3.

The repair changes only the peak-checkout-disk measurement method and the proof
infrastructure needed to make that method falsifiable. The 8,192 MiB ceiling,
absolute 50 ms nominal sample schedule, 100 ms maximum retained-start gap,
workload, isolation, finalization order, author proof, exact-head non-author
proof, and owner verdict all remain required.

## Prior wording

`R2.md` revision 2 section 9 defines the disk measurement as follows:

```text
| peak standalone checkout including `.git`, local target, and proof outputs | from proof start through process closure, sample `du -sm` on that checkout every 100 ms and retain the maximum; the output directory must be inside the checkout | 8,192 MiB |
```

The paragraph immediately following the ceiling table says:

```text
The peak-disk sampler uses an absolute 50 ms nominal schedule and fails if any
two consecutive retained sample-start timestamps are more than 100 ms apart;
it may therefore sample more frequently, but never less frequently, than the
stated 100 ms coverage.
```

Section 9 proof-list item 7 calls this the “checkout-wide disk sampler,” and the
opening proof paragraph requires a clean standalone checkout but does not give
that checkout a dedicated capacity-bounded filesystem.

Issue #199 repeats the method under “Falsifiable acceptance and ceilings”:

```text
- A sampler starts before step 1 and samples `du -sm` over the complete checkout on an absolute `50 ms` nominal schedule. It fails if consecutive retained sample-start timestamps are more than `100 ms` apart, covers `.git`, target, proof outputs, and process closure, retains initial/final and timestamped raw rows, and measures at most `8,192 MiB`.
```

## Falsified assumption

Revision 2 assumed that repeated recursive `ionice -c 3 du -sm -- <checkout>`
walks could observe an actively compiling checkout with no two retained sample
starts more than 100 ms apart on the owner reference host. The final controlled
candidate reserved disjoint physical-core groups, prestarted a fixed worker
pool, limited live walks to three across three exact lanes, and kept the 50 ms
absolute launch schedule. Its fresh one-shot rehearsal nevertheless retained
19 gaps above 100 ms; the maximum was 120,139,206 ns. All 33 workload commands
passed, but the sampler correctly made the proof red.

The failed assumption is the suitability of a recursive directory traversal as
a high-cadence observer, not the 8,192 MiB resource ceiling. More pools, lanes,
polling, or concurrent walks would continue making the observer compete with
the workload it measures and would not establish a reliable bound.

## Replacement wording

Replace section 9's opening complete-proof paragraph with:

```text
Before final disposition,
`docs/evaluation/r2-complete-proof-xfs.sh --source <clean-candidate-checkout>
--work <empty-host-directory>` requires canonical, non-overlapping source and
work paths with no symlink traversal and a real empty work directory. Exact
command
`/usr/bin/fallocate --posix --length 8589934592 <work>/filesystem.xfs`
must exit zero. The wrapper syncs that file and requires
its exact logical length and `stat.st_blocks * 512` allocated bytes to be at
least 8,589,934,592; it retains `filefrag` extent evidence plus the work-host
filesystem identity, mount options, quota-state output, and `statvfs` counters
before and after allocation. It attaches an exact-size loop device, formats it
as XFS with an internal log and no realtime device, and mounts it at `<work>/fs`
in a private mount namespace. One full standalone detached checkout at
`<work>/fs/checkout` is that filesystem's sole visible top-level entry. The
wrapper records the fundamental allocation unit
as `f_frsize` from `statvfs`/`stat -f %S`; the persistent sampler records Linux
`statfs.f_bsize`, and this XFS format requires the two values to be equal. The
XFS data-block capacity — `f_frsize * f_blocks` — must be no more than
8,589,934,592 bytes from before the checkout is created until the complete proof
process closes.
The wrapper then
invokes `docs/evaluation/r2-complete-proof.sh --output <empty-directory>` with
the output inside that checkout. All required host tools are installed first;
the inner harness removes external network routes before step 1, permits
loopback only for static smoke servers, and restores no route until every proof
process has closed. The backing image, loop device, private mount, and any
post-proof evidence export are host proof infrastructure, not checkout content
or a runtime dependency; the wrapper receipt records their exact paths,
commands, tool versions, image allocation and size, loop-device size,
filesystem identity and capacity, mount options and propagation, candidate
identity, result, and exported-evidence digest. The host-side paths
`<work>/export`, `<work>/export.sha256`, `<work>/wrapper-receipt.json`,
`<work>/proof.stdout`, and `<work>/proof.stderr` are siblings of `<work>/fs`,
never descendants of that XFS mount. The wrapper writes the exported output at
`<work>/export/target/r2-complete-proof` and rejects a symlink, special node, or
canonical path disagreement in either source proof output or export.

`wrapper-receipt.json` has schema identity `nomos-r2-xfs-proof/1` and exact
top-level fields `receipt`, `outcome`, `candidate`, `image`, `loop_device`,
`filesystem`, `mount`, `invocation`, `export`, and `teardown`. Those objects bind
the facts named above; `invocation` binds argv, working directory, original
uid/gid, exit status, stdout, and stderr, `export` binds its canonical inventory
and the inner evidence-manifest digest when one exists, and `teardown` requires
both unmounted and loop-detached to be true. The wrapper may return success only
when its receipt outcome and the inner proof are both `pass`.

One long-lived privileged supervisor starts in a new mount namespace and runs
`mount --make-rprivate /` before provisioning. It alone owns provisioning,
mount, proof supervision, export, ordinary unmount, and loop-device detachment.
It validates a clean source candidate, then uses the original uid/gid with no
capabilities to run `git clone --no-local --no-hardlinks`, detach the exact
source HEAD, create the output, rerun the standalone-checkout checks, and invoke
the proof. The supervisor is the only repository program run with privilege;
it does not source or invoke a file from the clone until after dropping to that
uid/gid with no capabilities, and no clone hook or checkout program runs as
root. Bubblewrap uses this exact bind order: system root read-only, canonical
checkout read-only at its identical path, canonical checkout `target/`
writable, and canonical proof output writable. After the proof child closes,
the supervisor exports only the proof output, without dereferencing links, to
the validated host-side export;
it rejects non-regular nondirectory entries and requires byte-identical
canonical source/export inventories. With its working directory outside XFS,
it proves no process holds the mount, syncs it, runs ordinary `umount` without
lazy or forced options, proves the mount is absent, detaches the exact loop
device, and proves the device has no open holder or image attachment.

An unprivileged host-namespace monitor records mount and loop state before the
supervisor starts and after it exits and rejects any surviving `<work>/fs`
mount, image attachment, or new proof-owned loop device. A red run preserves
the image, export, logs, exit status, and wrapper metadata; teardown still
occurs and must not erase the failure.
```

Replace proof-list item 7 with:

```text
7. the budget-producing workload while one prestarted persistent filesystem
   sampler continues;
```

Replace the peak-checkout-disk row with:

```text
| peak standalone checkout including `.git`, local target, and proof outputs | continuously enforce a fully preallocated dedicated XFS capacity `f_blocks * f_frsize` of at most 8,589,934,592 bytes from before checkout creation through proof-process closure, requiring `statfs.f_bsize == f_frsize`; during the inner proof, one prestarted persistent process samples on an absolute 50 ms nominal schedule and retains raw counters plus `(f_blocks - f_bfree) * f_frsize`; the filesystem contains only the checkout and the output directory is inside it; run exact `ionice -c 3 du -sm -- <checkout>` setup and shutdown scope crosschecks outside the live sampling interval | 8,192 MiB |
```

Replace the paragraph immediately following the ceiling table with:

```text
The persistent sampler records a monotonic sample-start timestamp, mount ID,
canonical mountpoint and source, major:minor device, XFS UUID, `f_type`, root
device and inode, bound `f_frsize`, observed `f_bsize`, `f_blocks`, `f_bfree`,
`f_bavail`, allocated bytes, allocated MiB, and kind for every row. It requires
`f_type == XFS_SUPER_MAGIC`, `0 <= f_bavail <= f_bfree <= f_blocks`,
`f_bsize == f_frsize`, `capacity_bytes = f_blocks * f_frsize`,
`allocated_bytes = (f_blocks - f_bfree) * f_frsize`, and
`allocated_mib = ceil(allocated_bytes / 1,048,576)`. It uses an absolute 50 ms
nominal schedule and fails if any two consecutive retained sample-start
timestamps are more than 100 ms apart. The fixed capacity continuously prevents
an over-ceiling allocation between samples; a workload write or publication
failure is still red. Mount ID, mountpoint, source, device, UUID, type, root
identity, block size, and total blocks are invariant for the sampler lifetime;
an unreadable or replaced counter source is drift and fails.

Before live sampling and after sampler shutdown and raw-ledger finalization, the
harness runs exactly `ionice -c 3 du -sm -- <canonical-checkout>` from the
checkout. Each invocation runs once with no retry, exits zero, writes nothing
to stderr, and writes exactly one
`<canonical unsigned MiB><TAB><canonical physical checkout path><LF>` row to
stdout. The proof retains its argv, working directory, status, stdout, stderr,
start/end timestamps, and an immediately following `statfs` snapshot. Each
`du_mib` must satisfy
`du_mib <= ceil(allocated_bytes / 1,048,576)` for its associated snapshot;
equality is not required because filesystem metadata and deleted-open files
remain charged by `statfs`. The shutdown crosscheck occurs before the final
summary, evidence manifest, and receipt. The proof fails on a non-XFS filesystem;
filesystem or device identity drift; block-size or total-block drift;
malformed, negative, out-of-range, or internally inconsistent counters;
capacity above 8,589,934,592 bytes; any visible top-level filesystem entry
other than the checkout; any checkout, target, or output path on another
device; a sparse or underallocated backing image; an image or loop-device size
other than 8,589,934,592 bytes; a non-private mount, external log, realtime
device, or missing `rw`, `nodev`, or `nosuid` mount option; or an unexpected
nested mount. Inside the measured checkout subtree, the only permitted nested
writable mounts are bubblewrap's exact canonical `target/` and proof-output
binds. Each must retain the checkout XFS UUID, major:minor device, filesystem
type, block size, capacity, and canonical source and target; every other nested
mount or any path or identity drift fails. The checkout root itself is
explicitly read-only bound at its identical canonical path so the private XFS
mount remains visible beneath bubblewrap's system-root bind; it must retain the
same XFS identity and canonical source and target. Bubblewrap's read-only
system-root bind is outside the measured checkout subtree.

Before the setup crosscheck and live sampling, exact command
`/usr/bin/fallocate --posix --length 16777216 <output>/host/finalization.reserve`
must exit zero and create that exact regular file. Its length must be 16,777,216
bytes and its allocated block bytes from
`stat` must be at least that length; its path, length, and allocated blocks
`R_allocated` are retained. Every raw allocation row therefore charges the
reservation. After the sampler's terminal row and raw-ledger flush, the
shutdown `du` and immediately following `statfs` checkpoint close the reported
maximum and record allocated bytes `A_before`. Only then is the reservation
unlinked and `/usr/bin/sync -f <output>` completes successfully. Before any
finalization file is written, another `statfs` records allocated bytes
`A_after` and requires `A_before >= R_allocated` and
`A_after <= A_before - R_allocated`; any failed command or inequality is red.
The summary, manifest, and receipt are then written and verified. A no-write
`statfs` check after receipt verification must not exceed the closed maximum;
otherwise the proof fails.
That last check creates no recursive evidence, and its success is bound by the
top-level harness result and the author and non-author execution receipts.
```

Apply the same method, wrapper invocation, failure conditions, and issue-body
digest binding to issue #199 before candidate freeze. The loopback backing
image is expressly authorized proof infrastructure. It is never shipped,
admitted as runtime source, or counted as checkout content.

## Reason

Kernel filesystem accounting is constant-work with respect to checkout size.
It does not repeatedly enumerate the source, Git object database, targets, and
proof output while those trees are being created. A dedicated filesystem makes
its scope exact: every allocatable data block belongs to the one checkout, and
the same immutable capacity enforces the ceiling even if Linux delays a sample.

The two retained `du` executions keep a recognizable independent scope check at
setup and shutdown without placing recursive traversal in the hot loop. Using
allocated bytes from `f_blocks - f_bfree` is conservative: filesystem metadata,
deleted-but-open files, and other allocated blocks inside the dedicated
filesystem remain charged even when a directory walk would not name them.
Fully allocating the backing image before formatting prevents later host-space
exhaustion from masquerading as an R2 checkout-budget failure; exact image and
loop sizes plus the smaller measured XFS capacity close both sides of that
claim.

This is a measurement-method repair, not an exclusion of a slow runner. It
removes the mechanism whose measured overhead falsified its own timing
assumption while keeping both the cadence and resource limit exact.

## Effect on existing evidence

No accepted R1 contract, implementation, artifact, workflow, receipt, or
verdict changes. The frozen R2 catalog, packet, scenes, plans, signatures, and
committed contact sheet remain byte-identical and retain their existing roles.
The four R2 criteria, all non-disk ceilings, network and write isolation, 33
workload commands, finalization order, non-author requirement, and owner stop
line are unchanged.

Every revision-2 development rehearsal remains what it recorded. The controlled
six-lane candidate at `032913203113843fb775b21d335cff4f8970c714`, tree
`22ea37581edca558b78f5698218c89ccc3f1ad4b`, retains failure report
`/data/dev/src/nomos-r2-rehearsal-lanes-a.ELNOo9/rehearsal-failure-report.md`,
SHA-256
`26d84e003e7ae312dc1dda562fde73c313a044c69d9ea447289d53f23c3e508c`.
The final recursive-observer candidate at
`4b12ed0cff29962885536723beb1e28d31c79acb`, tree
`831edad7b8ce1915c9f79ec2ef5a7ea0806efede`, is valid red evidence; commit
`157c7fe2466f9bbe75f84eadd69df66da5369271`, tree
`a93823252b8b3fba348a0ec10c3ffb63d946f5dd`, binds failure report
`/data/dev/src/nomos-r2-rehearsal-lanes3-a.u8Zv71/rehearsal-failure-report.md`,
SHA-256
`1e5c7b8a5628986b7db85cf557d98f039a864c71e5779243ff1b9d3f95a8beb4`.
Both are immutable red development evidence. Nothing from either run is
relabelled or rerun as green, and no formal R2 author proof exists yet.

The issue #199 acceptance wording and digest must be updated to revision 3
before candidate freeze. The repaired implementation requires fresh local
plants and development rehearsals, followed by a new formal author run and a
new exact-head non-author run. No proof from revision 2 can satisfy revision 3.

## Owner disposition

**Authorize.** Peter Permenter replied `Authorized` to the explicit proposal to
establish R2 revision 3; replace live repeated
`ionice -c 3 du -sm -- <checkout>` traversal with persistent kernel-backed
filesystem accounting on a dedicated fixed-capacity XFS filesystem containing
only the standalone checkout; retain the exact setup and shutdown `du`
crosschecks, absolute 50 ms schedule, unchanged 100 ms maximum gap, 8,192 MiB
ceiling, workload, isolation, author and non-author proofs, and historical red
evidence; and permit a loopback backing image as proof infrastructure. This
authorization repairs the method and allows issue #199 implementation to
continue. It does not accept R2 or authorize merge, adoption, or a later epoch.
