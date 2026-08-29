# R2 revision-4 implementation-author receipt

Status: revision-4 implementation evidence is pending. This receipt records
the authority boundary and provenance routing for the contract-classification
repair. It is not a proof result, acceptance verdict, or substitute for a fresh
author run and exact-head non-author rerun.

## Authority and historical boundary

- Issue: #199, `R2 final evidence and owner disposition`
- Contract: owner-authorized `R2.md` revision 4
- Contract SHA-256 at this routing change:
  `81c31f3ef5f9f4919f33fcc89f27e03eed344f84f44b1f6e9e04a19ac363ad8b`
- Authority: `docs/decisions/0026-r2-compile-latency-observation.md`
- Decision SHA-256 at this routing change:
  `b23bfa6275d8579b6782aa24b70b2edaae13b3960ba8ff8e9d79810a48149c73`
- Owner disposition: `repair the contract and rerun affected evidence`,
  authorized by Peter Permenter's `Yes. Proceed` reply on 2026-08-29
- Revision-4 issue-body SHA-256:
  `a1282d0802a45fc7d11872dec8156a745fac65f98d782ff209a7ab38eff209b2`
  over `gh issue view 199 --json body --jq .body`, including the command's
  final LF
- Unchanged R1 contract: `RUNTIME.md` revision 4, SHA-256
  `dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593`

The revision-3 implementation receipt at
`docs/evaluation/runs/r2/2026-08-28-issue-199-revision-3-author/AUTHOR_RECEIPT.md`
and all external revision-3 failure evidence remain historical records. In
particular, candidate `fc8b0f8cbf28e0f4eaf84f8e80b5bbe91a881798`, tree
`e75e5a407826a822f6f1c13905aa8a5a096952f6`, remains a red revision-3 author
attempt whose compile observation was a median numerator of `119139761` ns
over denominator `2` and a p95 of `77475936` ns. Its historical author receipt
has SHA-256
`84862d1df481210869fe8c100cd6091220d5fb19901ea852a29c252f7cc5caab`.
Revision 4 neither edits nor relabels that attempt.

## Revision-4 scope

Decision 0026 repairs an independently defective acceptance classification.
The exact maximum-scene workload, prebuilt release binary, 10 warmups, 100
retained new-process samples, unique same-filesystem outputs, synced atomic
publication timing interval, raw evidence, even-count median, nearest-rank p95,
and environment binding remain required. A valid measurement is now recorded
as an observation; its magnitude is not an acceptance verdict. Every other R2
acceptance ceiling and proof requirement remains unchanged.

The repair changes no compiler, decoder, catalog, renderer, UI, scene,
expected-plan, packet, contact-sheet, or accepted R1 byte. It changes only the
benchmark's classification output and the final proof machinery that validates
and records that output, plus the provenance plant that enforces this distinct
route.

The revision-4 provenance route covers exactly these changed evaluation source
and test files:

- `docs/evaluation/measure-r2-compile.mjs`
- `docs/evaluation/r2-complete-proof-receipt.mjs`
- `docs/evaluation/r2-complete-proof-receipt.test.mjs`
- `docs/evaluation/r2-complete-proof.sh`
- `docs/evaluation/r2-source-provenance.test.sh`

All unchanged R1/R2 source, schema, fixture, presentation, browser-evidence,
workflow, and proof-harness rows retain their existing historical producing
receipts. This record does not reattribute those bytes. The provenance
register, checker, and producing receipts remain control evidence bound by the
eventual candidate commit/tree and final receipt under the existing
self-binding rule.

## Proof obligation and status

The prior revision-3 proof stopped at ordered command 33 and emitted no final
evidence manifest or passing receipt. It cannot be resumed, promoted, or
reclassified. Revision 4 therefore requires a fresh candidate-native author
proof on a newly created dedicated 8,192 MiB XFS filesystem. If that author
proof passes, the same exact head still requires the issue's independent Luna
Max XFS rerun. Public CI can supplement those proofs; it does not replace them.

No passing revision-4 author proof, exact-head non-author proof, owner visual
judgment, or owner R2 disposition is bound by this source receipt. Focused
tests may establish implementation facts but do not make R2 green or accepted.

## Clean-room and adopter boundary

This repair is Nomos contract and proof infrastructure only. It does not
consult, copy, or embed The Mortal Estate or another adopter's repository,
payload, frame, palette, asset, prose, coordinate set, mechanic, schema, or
governance document. The Mortal Estate may use Nomos and feed lessons back
through separately authorized Nomos changes, but neither project becomes
authority for the other through this receipt.
