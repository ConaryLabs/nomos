# R2-1 repaired clean detached candidate proof

Status: author proof passed; exact-head non-author proof pending.

## Candidate binding

- Candidate commit: `32ae37d61cd71bb7be11c43de56b83a64bf006a3`
- Candidate tree: `2152165c2e07c0e23a023d17975723bc293bf348`
- Detached proof worktree: `/data/dev/src/nomos-issue-195-proof-3`
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
docs/evaluation/r2-source-provenance.test.sh
node docs/evaluation/r2-maximum.test.mjs
```

Results:

- boundary: clean, including all 18 xtask tests and every R2 dependency class;
- R2 schema register: 2 identities, SHA-256
  `948dae85d6cbcedc0ffc55629bfbf633a8b934bb99e23a84a11bada43fe11531`;
- R2 source register: 31 rows, SHA-256
  `9640718f61f0033500fa2844927f0ea358ceb5748834b895196b7c4fbfb9f228`;
- exact maximum fixture: 98,421 bytes, SHA-256
  `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909`.

The committed-archive provenance plant suite passed all seven
missing, extra, symlink, digest-drift, unknown-origin, unlicensed, and dangling-
receipt cases.

## Exact-head latency confirmation

The detached candidate built the release compiler and ran the exact 10+100
fresh-process method into `target/r2-latency-32ae37d/`. All 110 outputs were
retained and had the same 111,604-byte SHA-256
`aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`.

- Binary SHA-256:
  `dde136c1f2abd66e68ec395ce2fcfb427eec62e17f150d1bf35776a9da41e264`
- Median numerator: `76,740,574 ns`; median: `38,370,287 ns`; pass.
- p95: `43,190,202 ns`; pass.
- Raw-sample SHA-256:
  `3feda73d21ea1c8482c5930a8cc1f295054d6aba5caab324f4df238c836a0821`
- Summary SHA-256:
  `dfd259f931a6fda1f59da42733179b13b7b877eff62a5d5ae1fd7996ae83f8b9`

Every earlier red run and the contemporaneous old-binary control remain
recorded in `LATENCY_RECEIPT.md`; neither the ceiling nor method changed.

## Cold-review repairs

A Luna Max cold review of the preceding candidate reran all eight required
commands successfully and then found four defects. The repaired candidate:

1. binds a decoded plan's source digest to the exact canonical source rebuilt
   from its copied facts;
2. rejects `i64::MIN` crop dimensions without arithmetic overflow or panic;
3. enforces lexical aggregate-bound precedence even when unsigned dimensions
   exceed `i64::MAX`; and
4. proves R2 dependency edges by Cargo package identity, so an external package
   merely named `nomos-core` cannot impersonate the workspace member.

Each repair has a direct regression test. The fourth also has a planted
external-namesake boundary failure. Publication staging remained sound under
review and was not weakened.

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
- Process closure: pass in 19 ms; no browser, server, smoke, or play process
  remained.
- Viewer-build receipt SHA-256:
  `03f79d195f388d7aea105365cc0e8d28697d06ce031387ae10fac408175cd227`.
- Smoke receipt SHA-256:
  `7a08305c11db26de7999a80e7676d52e4a65d112639bd09c5a017fd370d0d18f`.
- Session SHA-256:
  `9e05713fe5568fa76ff5e346e4f4de351972c774827a0e58318b910c7add1cea`.

The host denied `unshare --net` with `Operation not permitted`, so this run does
not claim to replace the accepted network-namespace adoption receipt. The R1
browser proof still enforced Chrome's host-resolver isolation, recorded every
request, observed zero external requests, and passed its negative control. R2-1
adds no browser, network, or dependency path.
