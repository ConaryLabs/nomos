# R2 revision-4 implementation-author receipt

Status: the revision-4 author and exact-head non-author proofs passed through
first hosted-repair candidate `7b16d364276d10ca772cd2eb8ca03a4b6081de45`,
tree `30db2b7adf4abdf305086bacb3547294f50620e7`. Its second public run
made every non-R2 job green and exposed two further pre-XFS hosted-runner
defects. The second narrow repair is pending a fresh author proof, exact-head
Luna Max rerun, and hosted reruns. Owner visual judgment, owner R2 disposition,
and merge disposition also remain pending. This receipt records implementation
evidence; it is not an acceptance verdict.

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

The revision-4 provenance route covers these changed workflow, evaluation
source, and test files:

- `.github/workflows/nomos-viewer.yml`
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

All other unchanged R1/R2 source, schema, fixture, presentation,
browser-evidence, and proof-harness rows retain their existing historical
producing receipts. This record does not reattribute those bytes. The
provenance register, checker, and producing receipts remain control evidence
bound by the eventual candidate commit/tree and final receipt under the
existing self-binding rule.

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

## Retained third revision-4 formal red

Candidate `c1b50cfb4930ec6fb2298b235464133855135b58`, tree
`036ad93f1d2f0729b79a219bdf8bdc5b100f087c`, was run once from the fresh,
detached, non-shallow, clean source
`/data/dev/src/nomos-r4-checkpoint-candidate.vQg7jb` with fresh work directory
`/data/dev/src/nomos-r4-checkpoint-xfs-run.cxAdBh`. Its complete clean-head
portable preflight had passed before launch. The candidate-native inner proof
passed all 33 ordered commands, assembled and independently verified its
receipt, and exported byte-identical evidence. The supervisor exited zero.
The outer receipt assembler exited one with exact error `inner evidence
manifest does not bind exported output`; the wrapper reported `R2 XFS wrapper:
RED (receipt=1 supervisor=0)` and emitted no `wrapper-receipt.json`. The run is
therefore formal red, and no retry was launched from either retained path.

The two-checkpoint repair worked exactly as intended. The pre-format snapshot
recorded logical size `8,589,934,592`, `st_blocks` `16,777,216`, 512-byte block
units, and allocated size `8,589,934,592`. After sync, ordinary unmount, loop
detach, and proof of no remaining association, both the post-teardown snapshot
and live inode recorded logical size `8,589,934,592`, `st_blocks` `16,777,224`,
and allocated size `8,589,938,688`. The facts bind each checkpoint separately,
so the prior extra-contractual equality refusal did not recur.

Read-only diagnosis proved that the inner manifest and exported regular files
do bind. The canonical export inventory contains 2,344 rows: 1,882 regular
files and 462 directories. Excluding the manifest and final receipt themselves
leaves 1,880 regular files, exactly the 1,880 byte-path-sorted manifest rows,
with zero missing paths, extra paths, digest drift, or ordering difference. The
outer validator incorrectly projected every inventory row other than those two
paths, including directory rows that have no SHA-256, while the contract's
evidence manifest and the inner assembler cover regular files. The earlier
positive outer-receipt fixture was flat and therefore contained no directory
row. This is a proof-validator and test-fixture defect, not output mutation,
finalization timing, a product result, or a contract defect. The narrow repair
filters the manifest comparison to canonical inventory rows whose type is
`file`; the full source/export inventory continues to bind directory paths and
modes. A nested-file positive fixture makes the prior implementation fail and
the repaired implementation pass.

The retained diagnostic evidence is:

- inner receipt SHA-256:
  `7d040b5246b79cf48b8fb6c726d294af0504a0b8ee591bb0d4209b7879123e28`;
- inner `EVIDENCE.sha256` SHA-256:
  `eae5d98e48108e3516be8ee43bfb3f80f2d4c86647dd6ba7b27f9ebc21c9b1b2`,
  with all 1,880 listed regular files hash-valid;
- equal source and export inventory digest:
  `77e8a165fc0be9d93b8134352118826bdea4da55992ed4087b3b6f37c4c34557`,
  over 2,344 rows; export-inventory document SHA-256
  `49e05cccd6f9db1bd901761422d4700757724ef35deaf04094f24609e6a28035`;
- compile-summary SHA-256:
  `2d46f7b28093b235b7524f2158826d166548139fc242b29a408931594943946c`,
  recording median numerator `114562392` ns over denominator `2` (57,281,196
  ns), p95 `77071650` ns, and 100 separately published 111,604-byte outputs,
  all with SHA-256
  `aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`;
- raw compile-sample table SHA-256:
  `fc507f8322e1962025fc6ac6f832ac185b68f9c681bcffc2844221f6e95bf2c5`;
- supervisor-facts SHA-256:
  `882e02cd87203ca662157f2fa71adee271b0fe0c3adc1de232b8ec3a2df5e2da`;
- pre-format and post-teardown image-stat SHA-256 respectively
  `80ad97fd657fac7c80f9ee3eefe09c1f7b00b5914356abd1fdbe1a1858bbeab2`
  and `11c40235dc8cdf1d00dd0a7a7e236e4e38324332e4c2c862a4a0e05e9a8ef4db`;
- wrapper-command and execution-ledger SHA-256 respectively
  `227d5f41a003830859d5f7757eb23be82578375e4fb15943f91923f7d6085e8b`
  and `27b0689daede3e5f49c5e08c6c8c0b4e3af0d36af3337bc88e3e4be477bb6050`;
- clean host-monitor SHA-256:
  `d0590c27884032d41364cca1615575b11bfa4bffa1c1038a49829c8995a0f869`;
  and outer receipt-stderr SHA-256:
  `fc4612d2ddd8ca9ca8316c87e1ddee08fd41680c05c2e08454f7f928f2e6df35`.

The inner clean release build took 10.88 seconds; peak checkout allocation was
1,547 MiB; the 3,642-sample filesystem trace had maximum gap `74,748,991` ns;
XFS capacity was `8,511,139,840` bytes; the distribution was 805,600 bytes in
14 files; and both browser lanes, all 20 browser samples, bounded process
closures, and zero-external-request checks passed. These measurements do not
promote the run.

Teardown unmounted and detached `/dev/loop1`; no proof mount, holder, or image
association remains. The before/after loop inventories are byte-identical at
SHA-256
`e951f122f209cb4a215522a5b5e708d1a855da1e65e9aedfa014b849f4be6a74`
and contain only the unrelated, untouched `/dev/loop0`. Preserve both exact
third-run paths and all contents. Nothing from this run may be resumed,
spliced, relabelled, or carried into the repaired candidate's proof.

## Passing revision-4 author proof

The clean implementation candidate is
`52d126f235a0ab31cf8b48f8a87bd3a400c7437d`, tree
`9a5e520339950c14ecb180298ec5ff392700437d`. Before the formal run, that exact
head passed the complete portable preflight: formatting, locked offline
workspace Clippy and tests, dependency boundaries, release compiler, packet,
schema, 100-row provenance register and 10 plants, adopter neutrality and 5
plants, 39 complete-proof refusal plants, XFS shell validation, maximum
fixture, scene signatures, 132 Node tests, the 14-file 805,600-byte build, and
20 browser launches with zero external requests. The worktree was clean.

The candidate was then run once from fresh detached, non-shallow, clean source
`/data/dev/src/nomos-r4-manifest-candidate.2K7kNz` with fresh work directory
`/data/dev/src/nomos-r4-manifest-xfs-run.OqSRS1`. The candidate-native wrapper
returned `R2 XFS wrapper: PASS`. Its inner and outer receipts both report
`pass`; a separate in-memory invocation of the candidate's outer assembler
recomputed a byte-identical receipt. A read-only Sol Ultra audit independently
rehash-validated all 117 outer evidence bindings, all 1,879 manifest-listed
regular files, and the complete 2,343-row source/export inventory, and found no
blocker.

Decisive author evidence is:

- outer wrapper receipt SHA-256:
  `ec7ef34f671774d557fea48801c6739fbce4382dafc7d6566e15649863a5e0db`;
- inner receipt SHA-256:
  `0b94e9445cc84557e0cb8f09c0a3cb1695f1835cfa39bef04459cf51af77f6ac`;
- inner evidence-manifest SHA-256:
  `c0dcc9b468f38c45be910c9cca53d25e33f59db3caeb2df533fdc24c2bd4c4a6`;
- equal source/export inventory digest:
  `1d3ef3280b0bb9524e884e3525dd0bad76615d7cbd61f301940a14ccdb15fa75`;
- compile-summary and raw-sample SHA-256 respectively
  `af6051166c649c54eec9e4ae84e723636ef1a8164b23e75142e340df90f642c0`
  and `f1d94280e27c2dd33d54707ee894341cd157834bf5615af98684b2a4cc39e26a`;
- wrapper-command and execution-ledger SHA-256 respectively
  `b3f0a9ab59ffbdb14fd0f9f4fb9e197647aef7cd256b6e8a1807c35436ff9afa`
  and `22de34e52462fe4d3bd3e806e119ee3e96bc4bba709d505374b959da5d5cb976`;
- clean host-monitor SHA-256:
  `9b53187439aec918a7bc5a4628a4f01d822bdf848126b9039f2c6aeb94fe9726`.

All 33 ordered proof commands passed. The clean release build took 10.39
seconds. Peak checkout allocation was 1,551 MiB; 2,527 filesystem samples had
maximum gap `64,167,911` ns; XFS capacity was `8,511,139,840` bytes. The
compile observation retained 100 identical 111,604-byte outputs and recorded
median numerator `128598343` ns over denominator `2` and p95 `88,680,729` ns.
The two-scene browser proof recorded combined p95 `637,752,598` ns and zero
external requests. Process and write-boundary closure passed.

The image was exact 8 GiB logical and allocated before formatting. The
post-teardown snapshot and live inode agreed at exact 8 GiB logical and
`8,589,938,688` allocated bytes. Teardown unmounted and detached `/dev/loop1`,
left no holder, image association, or proof mount, and retained byte-identical
before/after loop inventories containing only the unrelated `/dev/loop0`.

## Passing exact-head Luna Max non-author proof

An independent OpenAI `gpt-5.6-luna` agent at Max reasoning reran the complete
candidate-native proof against the identical commit and tree. It used wholly
fresh detached, non-shallow, clean source
`/data/dev/src/nomos-luna-max-candidate.fMDUJB` and fresh work directory
`/data/dev/src/nomos-luna-max-xfs-run.IGFzbG`; it did not reuse or modify the
author paths. The wrapper returned `R2 XFS wrapper: PASS`, and the reviewer
independently recomputed the exact outer receipt in memory with the candidate's
module. No finding or proof defect remained.

Decisive non-author evidence is:

- outer wrapper receipt SHA-256:
  `9e9902f92a712b5e8983015a24fddd89313accdb7a9a7d22ce3892200f985c4d`;
- inner receipt SHA-256:
  `d90cbe24ffa1ebbc71526b42ab1546864e401750e8f9d781c4c528d0c8cb54bc`;
- inner evidence-manifest SHA-256:
  `3322631f4ececa5ae0bfc763ad4cac0ef2161991dccfd0806fe3b77482971712`;
- equal source/export inventory digest:
  `d2704ea4b9437700c96ef837326b30833e059483e25ab799ef6eaca2e1fc4e06`,
  over 2,345 canonical rows;
- compile-summary and raw-sample SHA-256 respectively
  `204a4394aa95d5f7be69c2b6fc7df0f84aa53a4e89de9970423e3bf497ab5d39`
  and `9ddd6d7a7b2b3bff203fc4fc043be317117730c16bd5b4c30956f7ba0583d668`;
- wrapper-command and execution-ledger SHA-256 respectively
  `49275d4b62251624bada40b69e1fdab18e42f9ff326156490573fa66b1ad50fd`
  and `dd11ad4652e691a174ff123c07ea7cbe694268f431756f93b04f0cf2089dbadf`;
- clean host-monitor SHA-256:
  `69dda48250b255afe1739d7e9ed321b7f316c33ce1bb6a61391b1ad26fb4c48c`.

All 33 ordered commands and the same 14/14, 104/104, and 132/132 test groups
passed. The clean release build took 10.14 seconds. Peak checkout allocation
was 1,552 MiB; 2,514 filesystem samples had maximum gap `53,372,691` ns. The
compile observation retained 100 identical 111,604-byte outputs and recorded
median numerator `129040131` ns over denominator `2` and p95 `96,861,044` ns.
The 805,600-byte, 14-file distribution and all 20 browser launches passed with
combined p95 `671,572,235` ns and zero external requests. Process and
write-boundary closure passed.

The non-author image recorded the same exact pre-format and post-teardown
allocation checkpoints as the author proof. Teardown unmounted and detached
`/dev/loop1`, left no holder, image association, or proof mount, and retained
byte-identical loop inventories at SHA-256
`e951f122f209cb4a215522a5b5e708d1a855da1e65e9aedfa014b849f4be6a74`
containing only the unrelated, untouched `/dev/loop0`.

## Passing record-only-head Luna Max proof

Committing the preceding control summary created candidate
`acc8c02133f956f22de182bc6b67ff002c1553c5`, tree
`968d4c77217da955814cb112ac4cd1d95b2d9fe8`. A fresh OpenAI `gpt-5.6-luna`
Max reviewer ran the complete candidate-native wrapper from detached clean
source `/data/dev/src/nomos-final-luna-candidate.k1YYy3` and fresh work
`/data/dev/src/nomos-final-luna-xfs-run.X8m6rn`. The wrapper returned `R2 XFS
wrapper: PASS`; both the reviewer and the implementation author independently
recomputed the outer receipt byte-for-byte. No finding remained.

Decisive record-head evidence is:

- outer wrapper receipt SHA-256:
  `0c17c0445c35e9ba282a04f2f0bdcebf91917cd6c1420f2e7940435da582c4f7`;
- inner receipt SHA-256:
  `c83a31b51078e8bf1216f96987c99d610e1c0061b03bc66903d1510a80771182`;
- inner evidence-manifest SHA-256:
  `4453031d4b8f51245cea542da3b44c7204a5976c5f39fea54231b0fc15163be2`;
- equal 2,344-row source/export inventory digest:
  `91f3aed9f7669dea1d956d4f350bc829b29c556b36e16cf751fdec59f5d4ba31`,
  covering 1,882 files and 462 directories while the manifest binds 1,880
  evidence files.

All 33 ordered commands passed, as did the 14/14, 104/104, and 132/132 test
groups. The clean release build took 22.22 seconds. Peak checkout allocation
was 1,553 MiB; 3,296 filesystem samples had maximum gap `78,320,767` ns. The
distribution remained 805,600 bytes across 14 files. Browser combined p95 was
`1,222,937,745` ns with zero external requests. The compile observation was
median numerator `259723758` ns over denominator `2` and p95 `207,390,954` ns
across 100 deterministic outputs; those magnitudes are observations, not
acceptance ceilings. Process and write-boundary closure passed.

The image was exact 8 GiB logical and allocated before formatting and exact 8
GiB logical with `8,589,938,688` allocated bytes after teardown. Ordinary
unmount and exact `/dev/loop1` detachment passed with no holder, association,
or proof mount. Before/after loop inventories were byte-identical and contain
only the unrelated, untouched `/dev/loop0`.

## First public-workflow reds and repair

Draft PR #201 published exact head `acc8c02`. Public `verify` run 33234517046
and every `gate-k-evidence` job in run 33234517031 passed. In `nomos viewer`
run 33234517044, the R1 offline job and ordinary viewer/browser job passed, but
the two R2 jobs were formal red:

- detached job 99053026380 supplied `CHROME_BIN=$(command -v google-chrome)`.
  GitHub's runner command path traversed a symlink, so the wrapper's canonical
  path guard correctly refused it before image allocation or XFS setup;
- portable job 99053026507 reached the XFS shell-validation suite, where Node
  22.23 represented the `/proc/self/fd` entrypoint differently across
  `process.argv[1]` and `import.meta.url`. The asymmetric comparison did not
  enter the receipt helper CLI. This lane also started no XFS work.

Both GitHub job logs are retained immutable red evidence. The empty artifact
diagnostics are expected because both failures preceded evidence creation; the
workflow's fail-closed `if-no-files-found: error` remains unchanged.

The repair does not weaken either boundary. The detached job now passes
`realpath -e --` of the discovered Chrome executable. Its job-scoped shell
plant prevents the already-canonical portable lane from masking future drift.
The receipt helper now uses the loader-provided `import.meta.main` boolean and
retains a symmetric-realpath fallback for older Node versions. Canonical and
descriptor-spelled CLI executions both require status 2 plus the exact usage
prefix, with captured status/output reported on failure. The workflow, helper,
XFS shell suite, and 11-plant provenance routing are attributed to this
revision-4 receipt. The 100-row provenance register SHA-256 is
`2782c821f0e90eb214abe72f7cd0c35be74198610868096c5c5ca2647140263b`.
No compiler, decoder, renderer, UI, scene, plan, packet,
contact-sheet, runtime, acceptance ceiling, or product byte changed.

## Passing first hosted-repair candidate proofs

The first hosted-portability repair was committed as candidate
`7b16d364276d10ca772cd2eb8ca03a4b6081de45`, tree
`30db2b7adf4abdf305086bacb3547294f50620e7`. Its exact-head portable
preflight passed formatting, locked workspace Clippy and tests, boundaries,
byte-identical plans and packet, the 100-row provenance register and 11 plants,
5 neutrality plants, 39 complete-proof plants, XFS shell validation, 132 Node
tests, the 805,600-byte 14-file build, and 20 browser launches with zero
external requests.

The fresh author proof used detached clean source
`/data/dev/src/nomos-r2-repaired-author-candidate.StgxqS` and fresh work
`/data/dev/src/nomos-r2-repaired-author-xfs-run.LwpKU4`. The wrapper returned
PASS, and a separate author-side invocation recomputed its receipt
byte-for-byte. Decisive evidence is:

- outer wrapper receipt SHA-256:
  `9ab7f585d13f536e1a86987d4e6afb561d2198d60690b0ce85f8b928136dcc2e`;
- inner receipt SHA-256:
  `0eb538eabc5af194a60c0e73e9319011b87d59f1dddfef41d5c32630e3e4ba93`;
- inner evidence-manifest SHA-256:
  `dee7e38160899cf306cc1db76f9e536884bb3267021df3f2a19069b8a1bcdbc3`;
- equal 2,346-row source/export inventory digest:
  `9b0a17ab64b28e8a95553f55e49375f9ca8992b1efa351575211a061d93628be`,
  covering 1,884 files and 462 directories while the manifest binds 1,882
  evidence files.

All 33 commands and the 14/14, 104/104, and 132/132 test groups passed. The
clean build took 17.16 seconds. Peak checkout allocation was 1,572 MiB; 3,471
samples had maximum gap `91,251,965` ns. The distribution was 805,600 bytes in
14 files. Browser combined p95 was `594,261,149` ns with zero external
requests. The compile observation was median numerator `362135183` ns over
denominator `2` and p95 `236,481,415` ns. Those magnitudes are observations,
not acceptance ceilings.

The required exact-head Luna Max proof used wholly separate detached clean
source `/data/dev/src/nomos-r2-repaired-luna-candidate.ouSBZl` and fresh work
`/data/dev/src/nomos-r2-repaired-luna-xfs-run.Njk1nu`. It returned PASS, and
the reviewer independently recomputed the outer receipt byte-for-byte.
Decisive evidence is:

- outer wrapper receipt SHA-256:
  `37a192a87f700b8a2a0fa16f74197f8eda9a409417d0836577be4a566747aa66`;
- inner receipt SHA-256:
  `b486a56484cae48ee214e2a44df9c3d1c46a1631a80e356f71d88209caf6de5b`;
- inner evidence-manifest SHA-256:
  `55e6e142b4c237f2a3e1c5e4e9061f9f3e9ecd07daaab94e6ddf20c61049bac5`;
- equal 2,344-row source/export inventory digest:
  `9abb212a4ba2626f095e00257aaf77d3842d0f31d7d0fc0c116c3a509b63e4ca`,
  covering 1,882 files and 462 directories while the manifest binds 1,880
  evidence files.

The same command and test groups passed. The clean build took 10.22 seconds.
Peak checkout allocation was 1,568 MiB; 2,767 samples had maximum gap
`93,172,072` ns. The distribution remained 805,600 bytes in 14 files. Browser
combined p95 was `639,246,684` ns with zero external requests. The compile
observation was median numerator `130683956` ns over denominator `2` and p95
`86,925,130` ns.

Both images were exact 8 GiB logical and allocated before formatting, and both
recorded the allowed 4 KiB post-teardown allocation increase. Ordinary unmount
and exact `/dev/loop1` detachment passed with no holder, association, or proof
mount. The before/after inventories were byte-identical and contained only the
unrelated, untouched `/dev/loop0`. A read-only Sol Ultra audit independently
checked the author evidence and found no blocker.

## Second public-workflow reds and repair

Draft PR #201 published exact head `7b16d36`. Public `verify` run 33236083997
and `gate-k-evidence` run 33236083960 passed. In `nomos viewer` run 33236083963,
ordinary viewer/browser job 99057201671 and R1 offline-budget job 99057201691
passed. The two R2 jobs remained formal red:

- portable job 99057201605 passed the 100-row provenance register and 11
  plants, 5 neutrality plants, and 39 complete-proof plants. Its XFS shell
  validation then invoked `/usr/bin/node` and stopped with status 127 because
  `actions/setup-node@v5` had installed Node 22.23.2 under
  `/opt/hostedtoolcache/node/.../bin/node`. The later missing-artifact failure
  is secondary to that early fail-closed stop;
- detached job 99057201685 stopped in wrapper tool recording with
  `tool disappeared while recording: node`. Fallback facts record
  `setup_failed:true`, every image/loop/mount operation at its not-run status,
  no image, and byte-identical empty proof-loop inventories. Its public receipt
  sandbox independently stopped at
  `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`, exposing the
  Ubuntu 24.04 AppArmor restriction on unprivileged user namespaces.

Both logs are immutable red evidence and neither job reached fallocate, image,
loop, mount, or XFS setup. Downloaded detached artifact 9709965583 is retained
at `/data/dev/src/nomos-r2-hosted-red-7b16d36.cAuhFq`; its archive SHA-256 is
`7fef19b7534a247dc98b669034b352823a3757296ee0cb117e66ae1b8f7258c3`.
The workflow correctly had no red backing image to upload. The unrelated
`/dev/loop0` remained untouched.

The second repair treats Node as the caller-selected development tool it is.
The public wrapper resolves and canonicalizes it before PATH collapse, requires
Node 22 or newer under a clean environment, refuses a Node directory that
shadows any declared inner or wrapper tool, and passes the exact path
positionally through the supervisor. The supervisor records and hashes that
path after dropping identity and every capability; statfs and export execute it
only after the same drop. Public host-check and receipt sandboxes invoke the
same absolute path. Facts record it in the export operation, and validation
derives the expected argv from the canonical, live-rehashed Node tool record.
No caller-selected Node byte executes as root and no CI-only `/usr/bin` shim is
created.

Each hosted R2 job now provisions and validates `sysctl`, records the original
`kernel.apparmor_restrict_unprivileged_userns` value, temporarily writes only
that gate to zero, and runs an unprivileged production-shaped probe before the
candidate step. The probe retains `setpriv --no-new-privs`, fresh network and
PID namespaces, `--unshare-net`, a read-only root, zero inherited/permitted/
effective/bounding/ambient capabilities, enabled IPv4/IPv6 loopback only, and
loopback-only routes. An `if: always()` step restores and verifies the exact
original gate value before artifact upload. The candidate proof is never run
with sudo. This provisions the already-required topology; it changes no R2
criterion or isolation boundary.

The producing source SHA-256 values for this repair are:

- workflow: `a631522dbd9b5639ec69a91c2d143cfe43725593af567b2271df6395e58264f2`;
- XFS evidence validator:
  `2ccdb094e420105e0b33c896ede5ba7f64e1250c2c992434baa8624f283d50f3`;
- XFS receipt test:
  `5d11c18e0b75a5c5092d7221e3f66a5de2f073198b873091e4e0c6480ca64926`;
- XFS work-directory helper:
  `d36e99a6f8db448686cf0439e17cf70ead0db3f0bd2ebe2750e29edb5da36538`;
- XFS wrapper:
  `3479b848be905afef6f42089d790ef03501f62c6a91df5583d83d662f06c8fda`;
- XFS shell-validation suite:
  `a0a77a2513e4d3ef1c92bd715cd42c1931515c9c271ba154cf10549413939516`.

The 100-row provenance checker and all 11 provenance plants pass at register
SHA-256
`6ab5bd7eb266878b1ab8bd3f8811791f36438ecf938c36475981c63f7f45e4f5`.
The toolcache-shaped Node plant, path-shadow refusal, dynamic operation/tool
binding mutations, exact hosted-sandbox probe, workflow YAML parse, focused
receipt tests, and ShellCheck pass. Three read-only Sol Ultra reviews found no
remaining code, portability, or workflow blocker. No new 8 GiB proof has been
launched for these bytes.

## Proof obligation and status

The prior revision-3 proof, three retained revision-4 wrapper-red attempts, and
both sets of retained public-run job reds cannot be resumed, promoted, or
reclassified. The passing runs through `7b16d36` remain evidence about their
exact candidates; they do not make the second hosted-portability repair green.
Because that repair changes workflow and proof bytes, freeze one new combined
candidate and run a fresh complete author XFS proof from new source/work paths.
The same exact head then requires a fresh Luna Max complete XFS rerun. No
evidence-summary commit may follow it.

R2 is not yet admitted. Both fresh local proofs, all applicable public hosted
workflows, owner visual judgment, the owner's explicit R2 disposition, and the
distinct merge disposition remain required. After the repaired exact-head
rerun, no additional local XFS run is warranted unless the head changes again
or later evidence produces a new failure.

## Clean-room and adopter boundary

This repair is Nomos contract and proof infrastructure only. It does not
consult, copy, or embed The Mortal Estate or another adopter's repository,
payload, frame, palette, asset, prose, coordinate set, mechanic, schema, or
governance document. The Mortal Estate may use Nomos and feed lessons back
through separately authorized Nomos changes, but neither project becomes
authority for the other through this receipt.
