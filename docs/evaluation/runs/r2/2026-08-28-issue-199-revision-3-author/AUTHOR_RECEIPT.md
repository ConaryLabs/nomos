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

- `docs/evaluation/r2-complete-proof-control.sh`
- `docs/evaluation/r2-complete-proof-control.test.sh`
- `docs/evaluation/r2-complete-proof-lib.sh`
- `docs/evaluation/r2-complete-proof-outer.sh`
- `docs/evaluation/r2-complete-proof-receipt.mjs`
- `docs/evaluation/r2-complete-proof-receipt.test.mjs`
- `docs/evaluation/r2-complete-proof-xfs-evidence.mjs`
- `docs/evaluation/r2-complete-proof-xfs-evidence.test.mjs`
- `docs/evaluation/r2-complete-proof-xfs-receipt.mjs`
- `docs/evaluation/r2-complete-proof-xfs-receipt.test.mjs`
- `docs/evaluation/r2-complete-proof-xfs.sh`
- `docs/evaluation/r2-complete-proof-xfs.test.sh`
- `docs/evaluation/r2-complete-proof.sh`
- `docs/evaluation/r2-complete-proof.test.sh`
- `docs/evaluation/r2-filesystem-accounting.mjs`
- `docs/evaluation/r2-filesystem-accounting.test.mjs`
- `docs/evaluation/r2-filesystem-evidence.mjs`
- `docs/evaluation/r2-filesystem-evidence.test.mjs`
- `docs/evaluation/r2-filesystem-sampler.mjs`
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

## Clean-room and adopter boundary

The implementation is Nomos proof infrastructure only. It does not consult,
copy, or embed The Mortal Estate, Cairn, or any other adopter's repository,
identity, payload, frame, palette, asset, prose, coordinate set, mechanic,
schema, or governance document. The standalone checkout and external wrapper
paths are proof topology; they are not runtime dependencies or adopter content.
Any adopter remains outside this repository's authority and must be admitted by
that project's own explicit decision and evidence.

## Proof status

No fresh revision-3 author candidate commit/tree, generated final evidence
manifest, or final proof receipt is bound by this record. Local focused tests
and syntax checks may establish implementation facts, but they do not satisfy
the privileged XFS rehearsal, the complete measured proof, the exact-head
non-author rerun, or the owner's merge disposition. Until those records exist,
R2 revision 3 remains pending and must not be called green or accepted.

The inventory table and per-file SHA-256 values are intentionally updated in a
separate provenance step after the revision-3 working tree is final. This
receipt therefore records routing intent and authority context without claiming
hash closure for the moving candidate.
