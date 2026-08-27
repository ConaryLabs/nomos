# R2-1 clean detached candidate proof

Status: author proof passed; exact-head non-author proof pending.

## Candidate binding

- Candidate commit: `74478ace9e7397c8dd4339c806f33041b0500194`
- Candidate tree: `cd2d05f4022154100173c7ac2f64c48fff315e59`
- Detached proof worktree: `/data/dev/src/nomos-issue-195-proof`
- Host: `remi`, Linux `7.0.0-30-generic`, x86_64, 12 logical CPUs
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- Node: `v22.22.1`
- Browser: `HeadlessChrome/152.0.7977.64`

The detached worktree began with no local build, generated content, wasm,
viewer distribution, smoke output, or R2 measurement output. All committed
paths remained unchanged throughout the proof.

## R2 and workspace proof

The following commands passed from the detached candidate:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
docs/evaluation/r2-schema-ownership.sh
docs/evaluation/r2-source-provenance.sh
node docs/evaluation/r2-maximum.test.mjs
```

Results:

- boundary: clean, including all 17 xtask tests and every R2 dependency class;
- R2 schema register: 2 identities, SHA-256
  `948dae85d6cbcedc0ffc55629bfbf633a8b934bb99e23a84a11bada43fe11531`;
- R2 source register: 31 rows, SHA-256
  `41481c7a44db30479e5db2c9ac03a6ac23db0d8c18c530cfc986067bfa145fb3`;
- exact maximum fixture: 98,421 bytes, SHA-256
  `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909`.

The committed-archive provenance plant suite separately passed all seven
missing, extra, symlink, digest-drift, unknown-origin, unlicensed, and dangling-
receipt cases.

## Exact-head latency confirmation

The detached candidate built the release compiler and ran the exact 10+100
fresh-process method into `target/r2-latency-74478ac/`. All 110 outputs were
retained and had the same 111,604-byte SHA-256
`aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`.

- Binary SHA-256:
  `8948aa69c094e6af964ddef6c46506cdce1bb18f75ddbced036b18f951e7cff4`
- Median numerator: `79,784,505 ns`; median: `39,892,252.5 ns`; pass.
- p95: `47,682,882 ns`; pass.
- Raw-sample SHA-256:
  `5445c7678b906242f2ae3776bdaef980c32a2d5c1218a809498e12dee97dd087`
- Summary SHA-256:
  `68ab4747660f391a356a8b58b9799cc5c8d0d2b6c860587cf6e02f08f5d8c71f`

The two earlier red runs remain recorded in `LATENCY_RECEIPT.md`; neither the
ceiling nor method changed.

## Unchanged R1 proof

The documented six-area proof was rebuilt from the same fresh detached
candidate in the required order: executable-gaol verification, locked offline
wasm build, native `nomos-play`, viewer build, all 104 Node tests, then required-
Chromium smoke and native replay.

- Six areas verified; public distribution: 24 files, 1,386,650 bytes.
- Wasm: 421,195 bytes, SHA-256
  `e8e03c125667ad937939f4a628b67df9ff813a88823fecd859784ed241673c97`.
- Browser: WebGL2; 6 areas, 65 moves, traversal cost 95; 23 loopback requests,
  zero external requests; negative external fetch failed as required.
- Native replay: 77 commands and receipts; pass; chain head
  `43a1b2164f18bc54738d0402013419659576e2d866c3fca630321a2ca641f143`.
- Process closure: pass in 21 ms; no browser, server, smoke, or play process
  remained.
- Viewer-build receipt SHA-256:
  `03f79d195f388d7aea105365cc0e8d28697d06ce031387ae10fac408175cd227`.
- Smoke receipt SHA-256:
  `a4be57f3c2616f507d1fa03455866743c078a7e3bd1cfd6194d5fc6897161235`.
- Session SHA-256:
  `9e05713fe5568fa76ff5e346e4f4de351972c774827a0e58318b910c7add1cea`.

The host denied `unshare --net` with `Operation not permitted`, so this run does
not claim to replace the accepted network-namespace adoption receipt. The R1
browser proof still enforced Chrome's host-resolver isolation, recorded every
request, observed zero external requests, and passed its negative control. R2-1
adds no browser, network, or dependency path.
