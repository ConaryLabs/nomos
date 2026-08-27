---
title: R2 revision 2 — repair final-proof finalization order
status: Owner-authorized; R2.md revision 2 in force
number: 0024
date: 2026-08-27
owner: Peter Permenter
issue: 199
supersedes_r2_revision: 1
establishes_r2_revision: 2
r2_revision_1_sha256: 2f671ffe87ebbc7076aa1e25474c5d114df1f03316c71be768e9d39b44b20c0c
issue_199_body_sha256: 4320dbdee1fcc52204809a9896c6e5a9a460d033a1ceba2c2aab7091bc55f929
r1_contract: RUNTIME.md revision 4
r1_contract_sha256: dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593
---

# R2 revision 2 — repair final-proof finalization order

## Decision authority

Peter Permenter authorized this record exactly as proposed on 2026-08-27. It
was prepared under issue #199 after implementation of the final proof exposed a
dependency contradiction in `R2.md` revision 1 and repeated it in the issue's
exact ordered proof. Section 13 requires the contradiction to be recorded and
repaired rather than silently reinterpreted. This decision takes effect as
written and establishes `R2.md` revision 2.

The repair changes only how the proof's terminal evidence is ordered and where
the unavoidable non-recursive boundary is stated. It changes no semantic
criterion, schema, source or compiled artifact, visual target, frozen digest,
budget ceiling, R1 meaning, or owner verdict.

## Prior wording

`R2.md` revision 1 section 9 orders its last two proof steps as follows:

```text
7. the budget measurements and receipt assembly; and
8. a final clean-worktree and process-closure check.
```

The same section then requires:

```text
Its receipt binds commit, tree, toolchain, commands, environment,
source/artifact digests, counts, timings, peak disk, process closure, and every
result.
```

Its peak-disk method also runs “from proof start through process closure.” Issue
#199 repeats the order by requiring step 7 to “assemble the bound
evidence/receipt” before step 8 stops the sampler and proves closure, while its
deliverables require the final receipt to bind both process closure and the
evidence-manifest digest.

## Contradiction

A final receipt cannot digest-bind process-closure and final disk-sampler bytes
that do not exist until the following step. Nor can the evidence manifest bind
a sampler file that continues changing after the manifest is hashed. Moving
closure earlier without a repair violates the literal order; assembling the
receipt earlier makes its required bindings false. Including the receipt's own
verification result in the receipt would also create a hash recursion.

This is an ordering contradiction, not a failed implementation or a reason to
weaken a proof. The dependency-correct order is to close and measure the
evidence first, then construct and verify the manifest and receipt that bind
it.

## Replacement wording

Replace R2 section 9 proof-list items 7 and 8 with:

```text
7. the budget-producing workload while the checkout-wide disk sampler
   continues; and
8. terminal finalization: while the sampler remains active, prove that no
   other proof process remains in the fresh network namespace; stop and wait
   for the sampler and retain its final row; recompute every ceiling from raw
   evidence; prove unchanged HEAD/tree and a clean worktree; repeat the strict
   namespace process-closure check; construct the closed evidence manifest;
   and only then assemble and independently verify the final receipt.
```

Replace the paragraph immediately following that list with:

```text
The script runs with `LC_ALL=C`, refuses an output directory outside the
standalone checkout, and writes only beneath that supplied empty directory and
the checkout-local `target/`; it never edits an input or committed fixture. The
ordered command ledger binds the executable workload in steps 1–7. Step 8's
terminal checks, manifest construction, receipt assembly, and receipt
verification occur after that ledger closes so the finalized artifact does not
claim to contain its own construction or verification result. Their success is
bound by the top-level harness exit and by each author and non-author execution
receipt. The final receipt binds commit, tree, toolchain, the closed workload
ledger, environment, source/artifact digests, counts, timings, peak disk,
pre-sampler-stop and post-sampler-stop process closure, every workload result,
and the evidence-manifest digest. The evidence manifest excludes only itself
and the final receipt.
```

Append this sentence to the peak-disk measurement method:

```text
The sampler uses an absolute 50 ms nominal schedule and fails if any two
consecutive retained sample-start timestamps are more than 100 ms apart; it may
therefore sample more frequently, but never less frequently, than the stated
100 ms coverage.
```

Apply the same replacement order and non-recursive boundary to issue #199's
“Exact ordered proof” and final-receipt deliverable. Update the issue's R2
revision/digest and issue-body digest bindings only after the repaired contract
and issue wording are exact.

## Reason

The replacement makes the written order follow the data dependency that the
existing requirements already impose. The two closure checks strengthen the
claim: the first occurs while disk measurement is live and allows only the
sampler subtree; the second occurs after that subtree has closed. The 50 ms
absolute schedule gives the 100 ms sampling requirement a falsifiable
non-real-time-Linux interpretation without increasing its maximum permitted
gap.

The explicit non-recursive boundary prevents a receipt from claiming to bind
its own construction or verification. The author and exact-head non-author
receipts remain the independent records that the top-level harness returned
success after final verification.

## Effect on existing evidence

No accepted R1 evidence changes. The frozen R2 catalog, packet, scenes, plans,
signatures, and committed contact sheet remain byte-identical and retain their
existing evidentiary roles. No R2 final-disposition evidence exists yet: all
issue #199 implementation rehearsals predate candidate freeze and are
development evidence only.

This authorization establishes R2 revision 2. The issue #199 acceptance wording
and its digest binding must be updated before candidate freeze, and the author
proof, exact-head Luna Max rerun, and required CI must all run from the repaired
candidate. Nothing may be relabelled from an earlier run.

## Owner disposition

**Authorize.** Peter Permenter replied `authorized` to the explicit request to
authorize decision 0024 and R2 revision 2. The quoted replacements take effect
exactly. They repair the impossible finalization order without weakening a
criterion or ceiling and authorize issue #199 to proceed through candidate
freeze and proof. They do not accept R2 or authorize merge, adoption, or any
later epoch.
