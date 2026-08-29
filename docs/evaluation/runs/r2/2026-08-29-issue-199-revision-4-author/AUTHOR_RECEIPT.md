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
- `docs/evaluation/r2-complete-proof-xfs-evidence.mjs`
- `docs/evaluation/r2-complete-proof-xfs-receipt.mjs`
- `docs/evaluation/r2-complete-proof-xfs-receipt.test.mjs`
- `docs/evaluation/r2-complete-proof-xfs.sh`
- `docs/evaluation/r2-complete-proof-xfs.test.sh`
- `docs/evaluation/r2-complete-proof-xfs-workdir.sh`
- `docs/evaluation/r2-complete-proof.sh`
- `docs/evaluation/r2-source-provenance.test.sh`

All unchanged R1/R2 source, schema, fixture, presentation, browser-evidence,
workflow, and proof-harness rows retain their existing historical producing
receipts. This record does not reattribute those bytes. The provenance
register, checker, and producing receipts remain control evidence bound by the
eventual candidate commit/tree and final receipt under the existing
self-binding rule.

## Retained first revision-4 formal red

Candidate `ff35834dc98cf15a1f8d659adb67fe6f81718a40`, tree
`ed547a0ab4378a3259dc10827c9c7dfdf533e481`, was run once from fresh standalone
source `/data/dev/src/nomos-r4-candidate.wbrd6Z` with fresh work directory
`/data/dev/src/nomos-r4-xfs-run.1djISn`. The candidate-native inner proof
passed all 33 ordered commands, assembled and verified its inner receipt, and
exported byte-identical evidence. The outer wrapper remained formally red and
emitted no `wrapper-receipt.json`: final receipt assembly correctly refused
the backing image with `image file size or allocation differs from the receipt
facts`. An inner pass cannot be promoted through an absent wrapper receipt and
none of this run may be resumed, spliced, relabelled, or reused for another
candidate.

The wrapper first recorded the required logical and allocated size of
`8,589,934,592` bytes, then invoked
`/usr/sbin/mkfs.xfs -f -l internal /dev/loop1`. Its retained stdout ends with
`Discarding blocks...Done.` After teardown the image remained logically 8 GiB
but had only `1,480,175,616` allocated bytes. The formatter's default discard
had propagated through the loop device and punched holes in the backing file.
This is a proof-harness defect against the existing fully allocated,
non-sparse-image requirement, not a product result or another contract defect.
The repair adds and receipt-binds `-K`, the `mkfs.xfs` no-discard option; the
strict post-run allocation check remains unchanged.

The retained inner evidence is diagnostic old-head evidence only:

- inner receipt SHA-256:
  `4aeafd1523c2734963b1aeba539f267ab7a276a831d61e7b69a33bc675f5e39b`;
- inner `EVIDENCE.sha256` SHA-256:
  `0f9dda92144aa8e292cd5976c8d1f9ec75f86f33602e9b5676d1a156ced07377`,
  with all 1,883 listed entries hash-valid;
- equal source and export inventory digest:
  `798f29146ca6bd306e72f910557c28fa71edf0673619a3fc1480104f34197207`;
- compile summary SHA-256:
  `32680dbeb4754b91c354c659fcf8917dc793c101d166a76694a42e34463aff74`,
  recording median numerator `117194938` ns over denominator `2`, p95
  `69426702` ns, and 100 separately published 111,604-byte outputs, all with
  SHA-256
  `aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`;
- raw compile-sample table SHA-256:
  `d0ec4d743ad72a4ce6f02099c18c89cac4af85c533dfa5f3e3efe97ebeee1ed9`;
- supervisor-facts SHA-256:
  `fda0524a1f2e15c1aa831a27364e40480fcb1de963b90bcf311ceba10a4bcaa7`;
- initial image-stat SHA-256:
  `80ad97fd657fac7c80f9ee3eefe09c1f7b00b5914356abd1fdbe1a1858bbeab2`;
- formatter-stdout SHA-256:
  `d7319c37618228543ac2e59887f15e07d6b01e188e2777569915026034b6c2d4`;
  and outer receipt-stderr SHA-256:
  `b98219aa570aa310a14853ca02e730cfedbad2cb9cf75ce6e0c786e1dd6e1d16`.

Teardown unmounted and detached `/dev/loop1` with no holder. The before/after
loop inventories are byte-identical at SHA-256
`e951f122f209cb4a215522a5b5e708d1a855da1e65e9aedfa014b849f4be6a74`
and contain only the unrelated, untouched `/dev/loop0`; the clean host-monitor
SHA-256 is
`afbb5224506774022a4c261e3a772883640f8d80e3d95d8af1934c18be8a7f62`.
A separate controlled 512 MiB diagnostic retained at
`/data/dev/src/nomos-xfs-allocation-diag.emvjOo` reproduced the allocation loss
with default formatting and preserved all `536,870,912` allocated bytes with
`mkfs.xfs -K`, including after a normal XFS mount, 64 MiB create/delete, sync,
and unmount. XFS defaults to runtime `nodiscard`, so this repair does not add
an unnecessary mount-argv change.

## Retained second revision-4 formal red

Candidate `00987913615b266f2fa792edb37db7b9304da439`, tree
`abe227ef4e33d23578a535427a3b999a2cdcfb61`, was run once from the fresh,
detached, non-shallow, clean source
`/data/dev/src/nomos-r4-k-candidate.akDvtZ` with fresh work directory
`/data/dev/src/nomos-r4-k-xfs-run.h5H2w9`. The candidate-native inner proof
passed all 33 ordered commands, assembled and verified its receipt, and
exported byte-identical evidence. The supervisor exited zero. The outer receipt
assembler exited one with exact error `image stat evidence differs from the
image file`; the wrapper reported `R2 XFS wrapper: RED (receipt=1
supervisor=0)` and emitted no `wrapper-receipt.json`. This is therefore formal
red regardless of the valid diagnostic inner evidence.

The prior no-discard repair worked. Both the operation facts and ordered ledger
bind exact formatter argv
`/usr/sbin/mkfs.xfs -f -K -l internal /dev/loop1`, and formatter output contains
no discard line. The pre-format snapshot recorded exact logical and allocated
size `8,589,934,592`, `st_blocks` `16,777,216`, and 512-byte block units. After
ordinary sync, unmount, loop detach, and proof of no holder or image
association, the live image still had exact logical size `8,589,934,592` and
`st_blocks` `16,777,224`, or `8,589,938,688` allocated bytes: a harmless 4,096
byte increase, not a hole or underallocation. A read-only diagnosis found 98
mapped data extents covering exactly 8 GiB; the additional host-XFS block is
consistent with extent-map metadata created when writes split the original
unwritten extent.

The contract requires exact 8 GiB logical size and `st_blocks * 512` allocated
bytes **at least** 8 GiB. It does not require pre-format and post-teardown
allocation counts to be equal or impose a host-backing-allocation ceiling. The
sole outer refusal was therefore an extra-contractual temporal comparison:
the verifier compared a deliberately retained pre-format stat snapshot for
exact equality with the post-teardown inode. This is a proof-harness defect,
not a product result, performance failure, or new contract defect. The repair
retains separately named pre-format and post-teardown stat evidence, validates
each independently against the existing exact-size/fully-allocated rule, binds
facts to their corresponding checkpoint, and compares the live inode only to
the post-teardown snapshot. It requires no equality or ordering between the two
allocation counts.

The retained inner evidence remains diagnostic old-head evidence only:

- inner receipt SHA-256:
  `bad5efc54bdfe8f1d17ef239880fe49b57582f290ae47990aca5489d7c7bcbe3`;
- inner `EVIDENCE.sha256` SHA-256:
  `c640340b79ff805d139bd4691cda09a4bdf0c3fad59bb51f4e14b20409eaed6d`,
  with all 1,879 listed entries hash-valid;
- equal source and export inventory digest:
  `496420950d42d0555081a3fc3c206a809049f6335843c5feae77f29ec33d201c`,
  over 2,343 rows;
- compile-summary SHA-256:
  `34a8d78a83349ee2dd36e88ec83a65f4965c12b1b73eb2269373dea42005b3e5`,
  recording median numerator `116912362` ns over denominator `2` (58,456,181
  ns), p95 `64134818` ns, and 100 separately published 111,604-byte outputs,
  all with SHA-256
  `aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`;
- raw compile-sample table SHA-256:
  `9ad5f5b6ac22e542c17761dca0cec2632128a878ad218545e7ffbf7822ec7f22`;
- supervisor-facts SHA-256:
  `1bc66d9a98c5a450f261c994b55a898dc23756e7b6df18d2d296b6262adb0e3b`;
- initial image-stat SHA-256:
  `80ad97fd657fac7c80f9ee3eefe09c1f7b00b5914356abd1fdbe1a1858bbeab2`;
- initial filefrag SHA-256:
  `c3265c646ef80a590ce8945f619e922c6e84ca0322ba5aedbb9c74d4cf676751`;
- formatter-stdout SHA-256:
  `cff8c93a40832108d074128c8ff37828e33eb54357f670edd2ed5d627088779e`;
- wrapper-command ledger SHA-256:
  `d9aec42c7c1a88a87f8b714a5ae6b6a235ad4b6eef9613fecea9c17c60501a05`;
- wrapper-execution ledger SHA-256:
  `dd8c00ad83ecc8bc4c9dfd44852243741f7f92f6beef83318e381b4b0411a457`;
- host-monitor SHA-256:
  `33654f762e99d401c9de642249e52cd16ddc91f059fefdd89e5fed202b20842d`;
  and receipt-stderr SHA-256:
  `6dc5da52c39be25efe18708a9c4c0c1931f35335f9fdd5cbff45b7e9fe696ace`.

The inner build took 18.54 seconds; peak checkout allocation was 1,544 MiB;
the 3,967-sample filesystem trace had maximum gap `97,894,050` ns; XFS capacity
was `8,511,139,840` bytes; the distribution was 805,600 bytes in 14 files; and
the two browser lanes, their 20 samples, bounded process closures, and zero
external-request checks passed. These measurements do not promote the run.

Teardown unmounted and detached `/dev/loop1`; no proof mount, holder, or image
association remains. The before/after loop inventories are byte-identical at
SHA-256
`e951f122f209cb4a215522a5b5e708d1a855da1e65e9aedfa014b849f4be6a74`
and contain only the unrelated, untouched `/dev/loop0`. Preserve both exact
second-run paths and all contents. Nothing from this run may be resumed,
spliced, relabelled, or carried into the repaired candidate's proof.

## Proof obligation and status

The prior revision-3 proof and both retained revision-4 wrapper-red attempts
cannot be resumed, promoted, or reclassified. Revision 4 therefore requires a
fresh candidate-native author proof on a newly created dedicated 8,192 MiB XFS
filesystem from wholly new source and work paths. If that author proof passes,
the same exact head still requires the issue's independent Luna Max XFS rerun.
Public CI can supplement those proofs; it does not replace them.

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
