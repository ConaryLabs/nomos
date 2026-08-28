# R2 revision-3 implementation-author receipt

Status: revision-3 implementation evidence is pending. This receipt records
the authority boundary and provenance routing for the new implementation; it
is not a proof result, an acceptance verdict, or a substitute for a fresh
author run and exact-head non-author rerun.

## Authority and historical boundary

- Issue: #199, `R2 final evidence and owner disposition`
- Contract: owner-authorized `R2.md` revision 3
- Authority: `docs/decisions/0025-r2-filesystem-accounting.md`
- Decision SHA-256 at this routing change:
  `a6a50bca56c4a990b44968ffefc31103a88e48b52904728693a166ba0d66d3ae`
- `R2.md` SHA-256 at this routing change:
  `625f4bb1ea7c7400a6717c14b51cc6da51b32421e49bba98cf3d7ed9ff4a1254`
- Revision-3 issue-body SHA-256:
  `0a701b4238fd6b7f23ba0ae40022bc7c23ca450ad1a8f0febc05ab440f6b3c88`
  over `gh issue view 199 --json body --jq .body`, including the command's
  final LF
- Unchanged R1 contract: `RUNTIME.md` revision 4, SHA-256
  `dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593`

The prior receipt at
`docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md`
is preserved as historical revision-2 red evidence. Its final controlled
recursive-observer run remains bound to candidate
`4b12ed0cff29962885536723beb1e28d31c79acb`, tree
`831edad7b8ce1915c9f79ec2ef5a7ea0806efede`, and the independently audited
failure report SHA-256
`1e5c7b8a5628986b7db85cf557d98f039a864c71e5779243ff1b9d3f95a8beb4`.
Revision 3 does not relabel that run or erase its report.

## Revision-3 scope

Decision 0025 replaces the falsified high-cadence recursive checkout observer
with a dedicated, fixed-capacity XFS filesystem. The 8,192 MiB ceiling,
absolute 50 ms nominal schedule, 100 ms retained-start-gap limit, workload,
isolation, finalization order, and author/non-author proof requirements remain
unchanged. The revision-3 proof contract requires infrastructure that records
XFS identity and counters, uses one persistent sampler, retains raw allocation
rows, performs exact setup/shutdown `ionice -c 3 du -sm --` crosschecks, and
binds the reservation release and post-receipt no-write check. Whether the
current implementation satisfies that contract remains part of the pending
proof.

The revision-3 provenance route covers these changed or newly introduced
evaluation implementation and test files:

- `.github/workflows/nomos-viewer.yml`
- `docs/evaluation/r2-complete-proof-argv.mjs`
- `docs/evaluation/r2-complete-proof-control.sh`
- `docs/evaluation/r2-complete-proof-control.test.sh`
- `docs/evaluation/r2-complete-proof-lib.sh`
- `docs/evaluation/r2-complete-proof-outer.sh`
- `docs/evaluation/r2-complete-proof-receipt.mjs`
- `docs/evaluation/r2-complete-proof-receipt.test.mjs`
- `docs/evaluation/r2-complete-proof-xfs-evidence.mjs`
- `docs/evaluation/r2-complete-proof-xfs-evidence.test.mjs`
- `docs/evaluation/r2-complete-proof-xfs-ledger.mjs`
- `docs/evaluation/r2-complete-proof-xfs-receipt.mjs`
- `docs/evaluation/r2-complete-proof-xfs-receipt.test.mjs`
- `docs/evaluation/r2-complete-proof-xfs-workdir.sh`
- `docs/evaluation/r2-complete-proof-xfs.sh`
- `docs/evaluation/r2-complete-proof-xfs.test.sh`
- `docs/evaluation/r2-complete-proof.sh`
- `docs/evaluation/r2-complete-proof.test.sh`
- `docs/evaluation/r2-filesystem-accounting.mjs`
- `docs/evaluation/r2-filesystem-accounting.test.mjs`
- `docs/evaluation/r2-filesystem-evidence.mjs`
- `docs/evaluation/r2-filesystem-evidence.test.mjs`
- `docs/evaluation/r2-filesystem-sampler.mjs`
- `docs/evaluation/r2-schema-ownership-plants.sh`
- `docs/evaluation/r2-source-provenance.test.sh`

The following revision-2 recursive-observer files are retired and removed from
the active routing case:

- `docs/evaluation/r2-complete-proof-disk-plants.sh`
- `docs/evaluation/r2-disk-control-lib.sh`
- `docs/evaluation/r2-disk-history-plant.sh`
- `docs/evaluation/r2-disk-lane-plants.sh`
- `docs/evaluation/r2-disk-overload-plant.sh`
- `docs/evaluation/r2-disk-sampler-lib.sh`
- `docs/evaluation/r2-disk-slot-race-plants.sh`
- `docs/evaluation/r2-disk-terminal-order.test.sh`
- `docs/evaluation/r2-procfs-read-plants.sh`

The unchanged process-closure files
`r2-complete-proof-process.mjs` and `r2-complete-proof-process.test.mjs`,
along with other unchanged evaluation sources, remain bound to their existing
historical producing receipts. This receipt is not used to reattribute those
bytes.

## Revision-3 development rehearsals retained as red

Two local rehearsal runs predate the next frozen candidate. They are retained
as failure evidence and do not satisfy any author-proof, non-author, CI, owner
judgment, or acceptance requirement.

- Candidate `2f2c27c40241621c934844f9b087b6069ac04d78`, tree
  `2a054de5477874721cde10797f4aa8b26d39d87c`, source
  `/data/dev/src/nomos-r2-candidate.Gzpnma`, work
  `/data/dev/src/nomos-r2-xfs-run.My4lZV`: the wrapper failed before the inner
  proof because network isolation attempted passwordless `sudo` only after the
  privilege/capability drop. The wrapper receipt SHA-256 is
  `f3f9c90fb6cbd6e28af837bb922dba145f62af76f71ac75eb515c0dd06c7c018`;
  the supervisor-facts SHA-256 is
  `d27e5d9ca8f3518d2cc6e1ba28e3e48dd442cc7a1b798a07262bd56d078ddb3a`.
  Teardown was clean. This run remains red.
- Candidate `5d579db688d1e6d6a72a9706f7c3619545486328`, tree
  `5f3dc0c53875d2ca5c77f24c48eef05e3cabf49e`, source
  `/data/dev/src/nomos-r2-candidate.Y9eWAI`, work
  `/data/dev/src/nomos-r2-xfs-run.FvfjEo`: commands 1-32 passed and command 33
  failed its compile-median ceiling. The measured median was 58,361,295 ns
  (`116722590 / 2`) against 50,000,000 ns; p95 was 77,710,111 ns against
  100,000,000 ns. The wrapper receipt SHA-256 is
  `6aa2294947cb12f9db24e081d989c659309e5e3afd96130d9e57f683d151954a`;
  supervisor-facts SHA-256 is
  `aee1b6c19a721871df40fd4d8bcdd90ea9ee3e21caf308d64b9b2c2ba8442bea`;
  compile summary SHA-256 is
  `b437dc0eef7dd220dc6fcda3922960390f2395ca1bfbdda586491db6d2aa5cec`;
  and samples SHA-256 is
  `04b356dafb0c0b3659e2b3ed6d285b593af114a55a3a258c408d3d423f9617ec`.
  Teardown was clean. A later standalone diagnostic under the same transient
  host load passed; it does not relabel this rehearsal.
- Candidate `965dd6f6e252c927a9b7123a2fa8dd49b597cea5`, tree
  `763aebfb54b2d2465eea291d19d7783e9f9d9e45`, source
  `/data/dev/src/nomos-r2-candidate.LgLJYn`, work
  `/data/dev/src/nomos-r2-xfs-run.syhfxA`: the frozen author attempt failed
  before image creation because GNU `find` did not descend through the pinned
  work descriptor when that descriptor symlink was its command-line root. The
  wrapper receipt SHA-256 is
  `f69486772da355504527e268a7922d1a525b21e6e12a38a0142e651fb00d4a8f`;
  supervisor-facts SHA-256 is
  `ef3b1a2d6b553cd57aa5aaf33cd251a5a8f75c35a284d87c3118fe9bca7fbba3`;
  and supervisor stderr SHA-256 is
  `5dcee23c3a3710a5ab9e60c68285612253da6e6ad47878a3aea7cb029975a52c`.
  No proof loop, mount, or backing image was created; the pre/post host loop
  inventories were identical. This attempt remains setup-red.
- An operator invocation intended for candidate
  `00f79f1faedbeedf5ae889c16d6bea8952972a72` used the development checkout's
  wrapper instead of the frozen source wrapper. The source-binding guard
  rejected it before supervisor setup and left work
  `/data/dev/src/nomos-r2-xfs-run.9SqI3T` empty. It is not a candidate proof.
- Candidate `00f79f1faedbeedf5ae889c16d6bea8952972a72`, tree
  `213a59583141f7a2f702429045249c0eedce8f89`, source
  `/data/dev/src/nomos-r2-candidate.yg1gre`, work
  `/data/dev/src/nomos-r2-xfs-run.mugsce`: the candidate-native attempt failed
  before image creation because the strict wrapper-tool validator compared its
  sorted 48-key register with an expected array whose `sh` and `sha256sum`
  entries were not in sorted order. The complete TSV register SHA-256 is
  `3b03a74c3652b1e8cc832715f1d568710f4f4625331aac8b930eabc3c962919a`;
  its JSON projection SHA-256 is
  `b6daa8d60c1a47fce00b5f9a2171dc0e3f9888a8a84f197909ebf086f7963963`;
  supervisor stderr SHA-256 is
  `a29d8c7d2d07b2723ce570dfb19babf891ef1719460dcad31acfd6cd3e4c2659`;
  and receipt stderr SHA-256 is
  `327bd97ddf07ae97fb17ee077652f4410426d417d6ea7150c464990e6bc602b9`.
  No proof loop, mount, backing image, or final receipt was created. The
  pre/post host loop inventories are byte-identical at SHA-256
  `e951f122f209cb4a215522a5b5e708d1a855da1e65e9aedfa014b849f4be6a74`.
  This attempt remains setup-red.
- Candidate `dfba1ad17e6a604cd343aac81bf48ded669d273d`, tree
  `62302d0c06e2171881d54a661a066e96cc947d0c`, source
  `/data/dev/src/nomos-r2-candidate.XywDQm`, work
  `/data/dev/src/nomos-r2-xfs-run.ds1Dk6`: the candidate-native attempt fully
  allocated the 8,192 MiB image, attached `/dev/loop1`, formatted and mounted
  XFS, then failed before the clone because `findmnt` canonicalized the
  descriptor-spelled mount target while the identity check compared it with
  `/proc/self/fd/11/fs`. The mount command recorded status zero and empty
  stdout/stderr; the XFS-info SHA-256 is
  `cff8c93a40832108d074128c8ff37828e33eb54357f670edd2ed5d627088779e`;
  the UUID evidence SHA-256 is
  `82372fb5fd19c1d47cb807238bee3d9d31c31ce56049a5e5a6ffb6897d90f11b`;
  and supervisor stderr SHA-256 is
  `5d2472a0415cfb1dd93c573d884b4ec43202f4e3780d1001d0dc4f7f94f628a7`.
  Failure-facts assembly then exposed eight undefined jq bindings and emitted
  no final receipt; receipt stderr SHA-256 is
  `327bd97ddf07ae97fb17ee077652f4410426d417d6ea7150c464990e6bc602b9`.
  Cleanup unmounted and detached the proof filesystem. Pre/post loop
  inventories are byte-identical at SHA-256
  `e951f122f209cb4a215522a5b5e708d1a855da1e65e9aedfa014b849f4be6a74`,
  and `/dev/loop0` remained the unrelated pre-existing Conary loop. This
  attempt remains setup-red.

## Clean-room and adopter boundary

The implementation is Nomos proof infrastructure only. It does not consult,
copy, or embed The Mortal Estate, Cairn, or any other adopter's repository,
identity, payload, frame, palette, asset, prose, coordinate set, mechanic,
schema, or governance document. The standalone checkout and external wrapper
paths are proof topology; they are not runtime dependencies or adopter content.
Any adopter remains outside this repository's authority and must be admitted by
that project's own explicit decision and evidence.

## Proof status

No passing revision-3 author candidate, generated final evidence manifest, or
final proof receipt is bound by this record. Local focused tests and syntax
checks may establish implementation facts, but they do not satisfy the
privileged XFS run, the complete measured proof, the exact-head non-author
rerun, or the owner's merge disposition. Until those records exist, R2
revision 3 remains pending and must not be called green or accepted.

The source-provenance inventory and per-file SHA-256 values have been refreshed
for the current implementation bytes. That closure does not convert this
routing receipt or any retained rehearsal into a proof result; the eventual
author result must bind its own exact candidate commit/tree and generated
evidence.
