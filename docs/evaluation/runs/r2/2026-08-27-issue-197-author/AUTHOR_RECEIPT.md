# R2-2 primary implementation-author receipt

Status: implementation source frozen; clean candidate author proof passed;
exact-head non-author proof pending.

## Authority and baseline

- Issue: #197, `R2-2: implement the offline consumer and independent second scene`
- Baseline commit: `cc47a7235f92d0ed460c7db5d178448b12fdba02`
- Baseline tree: `2bce614d2df94464c20042cdf059a7b22ec39c09`
- Contract: owner-authorized `R2.md` revision 1
- Contract SHA-256: `2f671ffe87ebbc7076aa1e25474c5d114df1f03316c71be768e9d39b44b20c0c`
- Issue-body SHA-256: `7d0a1245e49e095a3f8643d9f05cb14bd81121e7bf8913f8faf13a6c4e782e0b`
  over `gh api repos/ConaryLabs/nomos/issues/197 --jq .body`, including the
  command's final LF
- Author: Codex primary agent, GPT-5 family

## Frozen implementation and packet

- Implementation commit: `f64d374ee001ef0e51b66e3b2b4078ad6d1d770e`
- Implementation tree: `2cae2ec952ccd21cff9c6e9e91424922551b35e5`
- Release compiler SHA-256:
  `dde136c1f2abd66e68ec395ce2fcfb427eec62e17f150d1bf35776a9da41e264`
- Renderer catalog SHA-256:
  `6259520fbf318ae0393ea4ae69649864acb154db4034d081435416be2ffa9323`
- Second-scene packet manifest SHA-256:
  `d5708087cf7967a420667c56a7b02ed052b7058ed8545af06e6771170003c948`
- Packet inventory: 42 regular files; the manifest excludes only itself.

## Consulted inputs

The implementation used the baseline repository authority named by
`AGENTS.md`, the exact issue body, the issue-frozen catalog, the accepted R1
Three.js vendor manifest/files, and the owner authorization conversation. No
external project repository, content payload, target frame, palette, image,
prose, schema, coordinates, mechanic, or governance document was consulted or
copied as an implementation input.

## Method and scope

The author added the isolated browser consumer, canonical decoder, renderer,
numeric UI, offline build, CDP smoke harness, semantic signature tool, packet
delivery/audit tooling, manifest and neutrality checks, tests, and control
records. The first scene and compiler remained byte-unchanged from R2-1. The
second scene and its evidence entered unchanged from the independent packet
author. `CANDIDATE_PROOF.md` binds the clean candidate measurements and exact
digests; PR #198 and the Luna Max receipt bind the final review head.

Commands used include repository reads, `apply_patch`, SHA-256 checks, Node's
test runner, the offline build, the Chromium smoke, the four accepted Cargo
workspace commands, and the R2 schema/provenance/neutrality proofs. Red
development runs are not called evidence; final receipts bind the exact green
head.
