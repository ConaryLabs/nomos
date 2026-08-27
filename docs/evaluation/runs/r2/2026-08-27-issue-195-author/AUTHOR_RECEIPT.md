# R2-1 implementation-author receipt

Status: implementation commit frozen; exact-head non-author binding pending.

## Authority and packet

- Issue: #195, `R2-1: implement the strict observed-scene carrier and compiler`
- Baseline commit: `a3954b5fe1d36c6e0e939cbb3de6d625468d69e6`
- Baseline tree: `e0489cdc7f8469f06b896a2b0a9d7f05887448ce`
- Contract: owner-authorized `R2.md` revision 1
- Contract SHA-256: `2f671ffe87ebbc7076aa1e25474c5d114df1f03316c71be768e9d39b44b20c0c`
- Issue-body SHA-256: `4ffbbd1602558900ff7cdadf29acf466c80d344bfecbe20add8809d215d3910b`
  over `gh api repos/ConaryLabs/nomos/issues/195 --jq .body`, including the
  command's final LF
- Detached input manifest: `PACKET.sha256`; its digest is recorded below after
  the manifest enters the candidate
- Author: Codex primary agent, GPT-5 family
- Packet-manifest SHA-256:
  `3a1070c18dcc43f93784c2fbe9cb98d613b4b406817c40887588c540242aad5d`

The detached packet is the baseline Nomos contract/issue material and local
implementation patterns enumerated by `PACKET.sha256`. Each repository row is
the exact blob at the baseline commit. The issue row is the exact retrieval
described above.

## Consulted inputs

The complete consulted-input set is exactly the packet manifest plus the owner
authorization conversation establishing issue #195 as the active slice. No
adopter repository, payload, target frame, palette, asset, prose, schema,
coordinate set, or mechanic was opened, read, copied, or used as an
implementation input.

In particular, neither The Mortal Estate nor Cairn repository was consulted.
The generic first scene and all carrier vocabulary derive only from `R2.md` and
issue #195.

## Commands and method

The author used repository reads (`sed`, `rg`, `find`, `git show`), exact hash
checks (`sha256sum`), `apply_patch` for source edits, Cargo formatting/check/test
commands, the crate's own compiler CLI, the Node maximum-fixture generator and
test, and the R2 schema-ownership and source-provenance checkers. The maximum
workload was measured by the committed Node harness. Two red development runs
and the exact implementation-commit pass are recorded in `LATENCY_RECEIPT.md`.

## Implementation candidate

- Commit: `13d778ddbd5c4428b9a809a014fbad19c1abd94d`
- Tree: `76921e507f584f55e05cf68e08f72e8d14cf7aff`
- Compiler-produced plan receipt: `COMPILER_RECEIPT.md`
- Maximum latency receipt: `LATENCY_RECEIPT.md`

The later evidence commits add only control records/checkers around this frozen
implementation. Clean detached author proof is bound in `CANDIDATE_PROOF.md`.
The PR and non-author receipt bind the exact review head and tree. Owner merge
disposition remains pending.
