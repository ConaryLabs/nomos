# R2-1 clean detached candidate proof

Status: author proof passed; exact-head non-author proof pending.

## Candidate binding

- Candidate commit: `50897ad8b9429d0a62693b5769645b6bd314feca`
- Candidate tree: `db8666633fdd11548dafff0a48af0982d5ad9a46`
- Detached proof worktree: `/data/dev/src/nomos-issue-195-proof-2`
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
  `e23ffa6dfe7573308184885edd23ade16e4149d3bf9907a0b6a323d9c926200e`;
- exact maximum fixture: 98,421 bytes, SHA-256
  `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909`.

The committed-archive provenance plant suite separately passed all seven
missing, extra, symlink, digest-drift, unknown-origin, unlicensed, and dangling-
receipt cases.

## Exact-head latency confirmation

The detached candidate built the release compiler and ran the exact 10+100
fresh-process method into `target/r2-latency-50897ad/`. All 110 outputs were
retained and had the same 111,604-byte SHA-256
`aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`.

- Binary SHA-256:
  `a853d88b973e6540bfd79c65f363f4052340a3d739901d5959d63e4faed54121`
- Median numerator: `76,119,018 ns`; median: `38,059,509 ns`; pass.
- p95: `42,049,356 ns`; pass.
- Raw-sample SHA-256:
  `9e3ac3902a814c65dbaa31b22ec859b79441a5aa51d320406b7e3fa4ee87a206`
- Summary SHA-256:
  `1e104212c204aa14cd2ad623504fe4b1dfd486f7c88ecb4e0c69d0e5c6bea9e2`

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
- Process closure: pass in 24 ms; no browser, server, smoke, or play process
  remained.
- Viewer-build receipt SHA-256:
  `03f79d195f388d7aea105365cc0e8d28697d06ce031387ae10fac408175cd227`.
- Smoke receipt SHA-256:
  `ca321ec5e3c7a84a7f8bf7b8981b5e43f82337692026a39ee67607fd274caac1`.
- Session SHA-256:
  `9e05713fe5568fa76ff5e346e4f4de351972c774827a0e58318b910c7add1cea`.

The host denied `unshare --net` with `Operation not permitted`, so this run does
not claim to replace the accepted network-namespace adoption receipt. The R1
browser proof still enforced Chrome's host-resolver isolation, recorded every
request, observed zero external requests, and passed its negative control. R2-1
adds no browser, network, or dependency path.
