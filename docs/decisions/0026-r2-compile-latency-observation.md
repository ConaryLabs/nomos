---
title: R2 revision 4 — retain compile latency as an observation, not an ungrounded ceiling
status: Owner-authorized; R2.md revision 4 in force
number: 0026
date: 2026-08-29
owner: Peter Permenter
issue: 199
supersedes_r2_revision: 3
establishes_r2_revision: 4
r2_revision_3_sha256: 625f4bb1ea7c7400a6717c14b51cc6da51b32421e49bba98cf3d7ed9ff4a1254
revision_3_authority: docs/decisions/0025-r2-filesystem-accounting.md
revision_3_authority_sha256: a6a50bca56c4a990b44968ffefc31103a88e48b52904728693a166ba0d66d3ae
issue_193_body_sha256: a1f9d3673ff8ba48591b7d5bb8a3e563aa3d24e16aeb1d75213de1583ddeaedb
r2_revision_1_authorization_pr: 194
r2_revision_1_authorization_pr_body_sha256: bbc334bf2581e040b4f972ce0994d7c3c8ec101b14b088ec5733b58f40c7cc7f
issue_199_revision_3_body_sha256: 0a701b4238fd6b7f23ba0ae40022bc7c23ca450ad1a8f0febc05ab440f6b3c88
r2_1_latency_receipt_sha256: b7b367eee7e34fe384136905f0449f73b464d5426c9f58f41f3e6b4f897b5f5a
formal_red_candidate_commit: fc8b0f8cbf28e0f4eaf84f8e80b5bbe91a881798
formal_red_candidate_tree: e75e5a407826a822f6f1c13905aa8a5a096952f6
formal_red_record_commit: 529eee5564013f8ef39f2109a7b629b819e9e0b4
formal_red_record_tree: b24e1cf73dc7ac4a86bb20bf63d76859a4eb4722
formal_red_author_receipt_sha256: 84862d1df481210869fe8c100cd6091220d5fb19901ea852a29c252f7cc5caab
owner_disposition: repair the contract and rerun affected evidence
---

# R2 revision 4 — retain compile latency as an observation, not an ungrounded ceiling

## Decision authority

The revision-3 formal author proof stopped at the maximum-scene compile median.
The owner then asked which user experience the missing approximately ten
milliseconds affected. Review of the contract, its authorizing decision, issue
#193, PR #194, the R2-1 implementation issue and receipts, and the benchmark
implementation found no recorded derivation of the exact 50 ms median or
100 ms p95 values from an authoring workflow, adopter requirement, frame or tick
budget, throughput target, usability study, or measured baseline.

The owner was presented with the exact disposition
`repair the contract and rerun affected evidence` and the explicit replacement:
retain the maximum-scene compile workload and measurement as evidence, but make
its numeric latency an observation rather than a pass/fail gate until an actual
authoring or adopter workflow supplies a justified target. Peter Permenter
replied `Yes. Proceed` on 2026-08-29. This decision records that authorization
and establishes `R2.md` revision 4.

This repair candidly narrows criterion 4's performance gate. It is not justified
by the size of the miss and does not relabel a failed implementation. The
independent contract defect is that issue #193 required proposed **and
justified** limits and required the contract to decide whether each value was
an acceptance ceiling or a recorded observation, but the selected compile
numbers have no recorded reason. The revision-3 failure prompted that audit; it
is not the reason for the replacement. The same classification repair would be
required for a 49 ms result.

Decision 0023 directed implementation to stop on falsified assumptions and
repair the contract “without weakening it.” This decision narrowly supersedes
that phrase for the two compile-latency thresholds and no other criterion. The
repair is permissible only because the thresholds' missing justification is an
independent authorization defect under section 13, not because the current
implementation missed one of them.

## Evidence and falsified assumption

R2 is an isolated offline presentation proof. It expressly authorizes no editor,
hot reload, live adapter, production deployment, public-player path, or Mortal
Estate integration. The maximum-scene benchmark measures one fresh CLI process
from spawn through synced atomic publication. It includes process startup,
validation, compilation, serialization, file and directory synchronization,
and process exit. It does not measure frame rendering, gameplay response,
viewer navigation, a live observation-to-frame path, or compiler-core time in
isolation.

Issue #193's body, SHA-256 recorded above, required the contract to “propose and
justify” budget limits and to define whether each was a ceiling or observation.
The 50 ms median and 100 ms p95 values first appear in the original R2 contract
history, but neither the issue, contract, authorization PR #194, review
comments, decisions, nor the R2-1 latency receipt states a user, adopter,
workflow, SLA, or perceptual rationale for treating them as a product boundary.

The same repaired compiler binary has produced materially different end-to-end
results on the owner host while preserving the exact fixture and 111,604-byte
output. Retained R2-1 evidence includes passing medians near 38 ms after host
background writes quiesced. The revision-3 formal attempt measured central
samples 59,552,400 ns and 59,587,361 ns, hence median numerator 119,139,761 ns
over denominator 2, with p95 77,475,936 ns. All 100 outputs were byte-identical.
An earlier retained run of the same binary and fixture measured a 58,361,295 ns
median and 77,710,111 ns p95. Those results establish that the measurement is
real and environment-sensitive; they do not supply the missing user-facing
rationale for treating its magnitude as epoch acceptance.

The falsified assumption is therefore that the exact compile-latency numbers
were justified R2 acceptance boundaries. Compile correctness, determinism,
durable publication, workload size, measurement integrity, and the value of
retaining median and p95 observations remain intact.

## Prior wording

`R2.md` revision 3 criterion 4 says:

```text
4. **The combined candidate is reproducible and stays inside measured ceilings.**
   A clean network-isolated checkout passes the complete R1 and R2 proof,
   reproduces the canonical fixtures and built public artifact, meets every
   section 9 ceiling, and receives an exact-head non-author rerun. The owner then
   explicitly accepts or rejects R2; green proof alone does not admit it.
```

The section 9 heading is:

```text
## 9. Proof and measured ceilings
```

Proof items 7 and 8 say:

```text
7. the budget-producing workload while one prestarted persistent filesystem
   sampler continues; and
8. terminal finalization: while the sampler remains active, prove that no
   other proof process remains in the fresh network namespace; stop and wait
   for the sampler and retain its final row; recompute every ceiling from raw
   evidence; prove unchanged HEAD/tree and a clean worktree; repeat the strict
   namespace process-closure check; construct the closed evidence manifest;
   and only then assemble and independently verify the final receipt.
```

Section 9 introduces its table with:

```text
### Acceptance ceilings

These are ceilings, not observations or promises for every machine.
```

Its compile row is:

```text
| maximum-scene compile latency | the exact committed maximum fixture above; prebuilt release binary; 10 unrecorded process warmups, then 100 new processes from spawn through synced atomic publication to unique nonexistent output paths on one filesystem; warm OS file cache is retained, output is never reused, and all raw samples and outputs are retained | median 50 ms, p95 100 ms |
```

The post-table rule says:

```text
A budget miss is red evidence. Changing a ceiling or its method requires the
contract-repair process; a slow runner is recorded, not silently excluded.
```

Section 11's final-evidence target says:

```text
3. **R2 final evidence and disposition.** New issue after R2-2 lands; one bound
   combined candidate, complete network-isolated proof, refreshed ceilings,
   exact-head non-author rerun, owner visual verdict, and owner R2 verdict.
```

Section 12's applicable stop bullet says:

```text
- red required proof, non-reproducible canonical/public artifact, leaked
  process, external runtime request, or budget miss; or
```

Issue #199 repeats the compile criterion:

```text
- The exact maximum fixture is `98,421` bytes with SHA-256
  `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909`.
  A prebuilt release compiler runs 10 unrecorded warmups and 100 retained new
  process samples to unique same-filesystem outputs. Recomputed median is at
  most `50 ms` and nearest-rank p95 at most `100 ms`; sample count, ordinals,
  paths, output bytes/digests, numerator/denominator, and p95 all verify.
```

The benchmark summary contains `median_ceiling_ns`, `median_pass`,
`p95_ceiling_ns`, and `p95_pass`; its command fails solely when a correctly
measured latency exceeds either number. The final receipt verifier independently
repeats those comparisons.

## Replacement wording

Replace criterion 4 with:

```text
4. **The combined candidate is reproducible, stays inside measured ceilings,
   and records the required compile observation.** A clean network-isolated
   checkout passes the complete R1 and R2 proof, reproduces the canonical
   fixtures and built public artifact, meets every section 9 acceptance
   ceiling, records maximum-scene compile latency under section 9's fixed
   workload and method, and receives an exact-head non-author rerun. The owner
   then explicitly accepts or rejects R2; green proof alone does not admit it.
```

Rename section 9 to `Proof, measured ceilings, and required observations`.
Replace proof items 7 and 8 with:

```text
7. the required maximum-scene compile-latency observation while one
   prestarted persistent filesystem sampler continues; and
8. terminal finalization: while the sampler remains active, prove that no
   other proof process remains in the fresh network namespace; stop and wait
   for the sampler and retain its final row; recompute every acceptance ceiling
   and required observation from raw evidence; prove unchanged HEAD/tree and a
   clean worktree; repeat the strict namespace process-closure check; construct
   the closed evidence manifest; and only then assemble and independently
   verify the final receipt.
```

Remove only the compile-latency row from the `Acceptance ceilings` table. Keep
every other row and method unchanged. Immediately after that table add:

```text
### Required compile-latency observation

Maximum-scene compile latency is a required recorded observation, not an
acceptance ceiling. It uses the exact committed maximum fixture above; a
prebuilt release binary; 10 unrecorded process warmups; then 100 new processes
measured from spawn through synced atomic publication to unique nonexistent
output paths on one filesystem. The warm OS file cache is retained, output is
never reused, and every raw sample and output is retained.

The proof independently recomputes the even-count median and nearest-rank p95
and records both with the complete environment. Observation evidence is valid
only when all 110 processes complete, every output is byte-identical, every
retained sample and path verifies, and the summary arithmetic recomputes.
Failure of any of those requirements is red. The magnitude of a correctly
recorded median or p95 is not itself pass or fail. No latency ceiling may be
restored until an identified authoring or adopter workflow supplies a measured,
owner-authorized rationale.
```

Replace the post-table budget rule with:

```text
A miss against an acceptance ceiling is red evidence. A missing, malformed,
non-reproducible, or arithmetically invalid required observation is red
evidence; the magnitude of a correctly recorded compile-latency observation is
not. Changing an acceptance ceiling or any measurement method requires the
contract-repair process; a slow runner is recorded, not silently excluded.
```

Replace section 11's final-evidence target with:

```text
3. **R2 final evidence and disposition.** New issue after R2-2 lands; one bound
   combined candidate, complete network-isolated proof, refreshed acceptance
   ceilings and compile-latency observation, exact-head non-author rerun, owner
   visual verdict, and owner R2 verdict.
```

Replace section 12's applicable stop bullet with:

```text
- red required proof, non-reproducible canonical/public artifact, leaked
  process, external runtime request, acceptance-ceiling miss, or missing,
  malformed, drifted, or nonreproducible compile observation; or
```

In issue #199, replace the compile bullet with:

```text
- The exact maximum fixture is `98,421` bytes with SHA-256
  `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909`.
  A prebuilt release compiler runs 10 unrecorded warmups and 100 retained new
  process samples to unique same-filesystem outputs. Sample count, ordinals,
  paths, output bytes/digests, numerator/denominator, and nearest-rank p95 all
  verify. Median and p95 are retained recorded observations with no numeric
  acceptance ceiling.
```

Issue #199's proof finalization must recompute every acceptance ceiling and the
compile observation. Its non-author attack and refusal plants must distinguish
acceptance-ceiling enforcement from compile-observation integrity. Its stop
line uses `acceptance-ceiling miss` and separately refuses a missing, malformed,
drifted, or nonreproducible compile observation.

The benchmark output summary removes the four compile-verdict fields and adds
exact field `measurement_role: "recorded_observation"`. It retains the complete
environment, binary and fixture bindings, 10/100 counts, output digest, median
numerator and denominator, and p95. A successful measurement prints `RECORDED`
rather than `PASS`. The receipt verifier requires that role, recomputes all
arithmetic and byte/path bindings from raw evidence, and imposes no latency
comparison.

No compiler, decoder, catalog, renderer, UI, scene, expected plan, second-scene
packet, contact sheet, fixture, benchmark workload, durability operation, or
remaining ceiling changes.

## Reason

An acceptance threshold needs an identified consequence. R2 has none for the
specific 50/100 ms values. Retaining a number as a gate solely because it was
written earlier would turn the proof into ritual rather than evidence. Raising
the median to 60 ms merely to admit the current result would be equally
arbitrary, and optimizing solely toward 50 ms would preserve the same
unsupported premise. Recording the exact end-to-end observation preserves
information and regression visibility without claiming an unsupported product
promise.

This is not permission to omit performance acceptance from a future editor,
live adapter, adopter mapping, or production path. Such a path must establish a
workload and threshold from its own user-facing or operational requirement and
receive separate authority.

## Effect on existing evidence

No accepted R1 contract, implementation, artifact, workflow, receipt, or
verdict changes. The R2 schemas, compiler, decoder, catalog, renderer, UI,
scenes, plans, maximum fixture, packet, signatures, contact sheet, and all
frozen content digests remain byte-identical. The build, disk, distribution,
browser, filesystem-sampler cadence, process-closure, isolation, and teardown
ceilings and methods remain unchanged.

Every revision-1 through revision-3 latency result retains its exact historical
meaning. In particular, the revision-3 attempt at candidate
`fc8b0f8cbf28e0f4eaf84f8e80b5bbe91a881798`, tree
`e75e5a407826a822f6f1c13905aa8a5a096952f6`, remains a formal red under
revision 3. Its 32 passing commands, command-33 failure, raw samples, partial
export, wrapper receipt, and clean teardown are not relabelled as a revision-4
pass and cannot satisfy revision-4 evidence.

Issue #199 must bind revision 4 and its exact new contract, decision, and issue
body digests before candidate freeze. The proof harness, measurement summary,
receipt verifier, tests, and source-provenance routing must implement the
replacement fail closed. Then one fresh complete candidate-native 8 GiB XFS
author proof must run from new standalone source and work paths. A passing
author result still requires a separate fresh exact-head Luna Max rerun and the
applicable hosted workflows before the owner visual and final R2 verdicts.

## Owner disposition

**Repair the contract and rerun affected evidence.** Peter Permenter replied
`Yes. Proceed` to the explicit request to record this exact disposition and
reclassify the unchanged maximum-scene compile measurement as an observation
rather than an unjustified numeric acceptance gate. Establish `R2.md` revision
4 and permit issue #199 to update only the contract, proof/evidence machinery,
tests, provenance, and handoff needed for the repair. This is not `accept R2`,
does not authorize merge or adoption, and does not authorize compiler or other
frozen product-byte changes.
