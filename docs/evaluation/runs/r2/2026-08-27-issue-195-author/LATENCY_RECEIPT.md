# R2-1 maximum-scene latency receipt

Status: cold-review-repaired candidate passed; all earlier red runs retained.

## Frozen workload and method

- Implementation commit: `13d778ddbd5c4428b9a809a014fbad19c1abd94d`
- Implementation tree: `76921e507f584f55e05cf68e08f72e8d14cf7aff`
- Maximum fixture bytes: `98421`
- Maximum fixture SHA-256:
  `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909`
- Method: prebuilt locked release binary, 10 unrecorded fresh-process warmups,
  then 100 recorded fresh processes to unique nonexistent output paths on the
  same filesystem; elapsed time is integer `process.hrtime.bigint()` nanoseconds
  from immediately before spawn through successful synced atomic publication.
- Summary calculation: even-count median is the sum of sorted samples 50 and
  51 divided by two; p95 is sorted sample 95.
- Ceiling: median at most `50,000,000 ns`; p95 at most `100,000,000 ns`.
- Host: `remi`, Linux `7.0.0-30-generic`, x86_64, 12 logical CPUs.
- Node: `v22.22.1`.

## Exact implementation-commit result

- Release binary SHA-256:
  `8948aa69c094e6af964ddef6c46506cdce1bb18f75ddbced036b18f951e7cff4`
- Output: `111604` bytes, SHA-256
  `aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`
  for every warmup and recorded process.
- Median numerator: `81,695,811 ns`; median: `40,847,905.5 ns`; pass.
- p95: `51,493,304 ns`; pass.
- Raw-sample SHA-256:
  `163129263111a9a8bcc25cb5b491b3cbd0ea042235a4515be8555859ecbd0a09`
- Summary SHA-256:
  `257ee860713b0f101d325609f07694b1710fe1c8932374172f6d4b315b05aa7a`
- Retained directory:
  `target/r2-latency-issue-195-13d778d/` (`112` regular files: 10 warmup
  outputs, 100 recorded outputs, `samples.tsv`, and `summary.json`).

Command:

```text
node docs/evaluation/measure-r2-compile.mjs --binary target/release/nomos-observed-scene --fixture fixtures/r2/maximum-observed-scene.json --output target/r2-latency-issue-195-13d778d
```

## Retained red evidence

No ceiling or method changed.

1. The initial implementation rebuilt semantic documents with a quadratic
   per-character UTF-8 suffix validation. Binary SHA-256
   `2605ba1fb2c78e67a7d293883450ab4cb7e894645623c4f977283b577cc1dc7e`;
   median numerator `493,373,029 ns` (median `246,686,514.5 ns`), p95
   `267,848,683 ns`; both red. Summary SHA-256
   `37cccdd48d6fb17b414cd1460bfb012b73a8499268190a5c4be51e42e0cdc49f`;
   retained directory `target/r2-latency-issue-195-author/`.
2. Linear string-run parsing removed that algorithmic cost, but the compiler
   still reconstructed the full semantic plan after exact staged-byte
   comparison. Binary SHA-256
   `b3d248a59f37d4834fd93515d2f4a36bdb1571c0aae7310fafa383336d61c61d`;
   median numerator `111,320,896 ns` (median `55,660,448 ns`), p95
   `60,550,057 ns`; median red, p95 pass. Summary SHA-256
   `8e6fa1c30b94d260c623b41bb9a943c6366cf9904e4c268bac9c5528b7554a1a`;
   retained directory `target/r2-latency-issue-195-author-r2/`.

The passing implementation keeps both file and parent-directory `sync_all`
calls. Its staged re-verification reopens the bytes, compares them exactly with
the typed compiler output, and parses them under the canonical profile; it does
not redundantly reconstruct semantics that were already validated before
encoding.

The clean detached candidate at commit
`74478ace9e7397c8dd4339c806f33041b0500194` repeated the exact method with the
same binary and output digests: median numerator `79,784,505 ns` (median
`39,892,252.5 ns`) and p95 `47,682,882 ns`, both pass. Its raw-sample SHA-256 is
`5445c7678b906242f2ae3776bdaef980c32a2d5c1218a809498e12dee97dd087` and
summary SHA-256 is
`68ab4747660f391a356a8b58b9799cc5c8d0d2b6c860587cf6e02f08f5d8c71f`.

## Post-audit repair result

Commit `c87bddee7ec872fe47053f4684974dbb919b523c`, tree
`b872fd8f942a74b1ef39f77a84eaa45edc32af17`, repaired identity-phase path
precedence and prevented cleanup from deleting a pre-existing file that merely
collided with the compiler's staging name. It repeated the exact method above
without changing the workload, ceiling, calculation, or synced publication:

- release binary SHA-256:
  `a853d88b973e6540bfd79c65f363f4052340a3d739901d5959d63e4faed54121`;
- output: `111604` bytes, SHA-256
  `aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`
  for every warmup and recorded process;
- median numerator: `81,313,106 ns`; median: `40,656,553 ns`; pass;
- p95: `53,060,187 ns`; pass;
- raw-sample SHA-256:
  `cbf84539d58ca764a751556449529d942b12adaa52c3cbc359d2a7660173c4f6`;
- summary SHA-256:
  `2e6fcfa133d04319c02e4484269cc969e52c2476758b2f9df9a2caf6f275111f`;
  and
- retained directory: `target/r2-latency-c87bdde/` (`112` regular files).

The clean detached evidence commit
`50897ad8b9429d0a62693b5769645b6bd314feca`, tree
`db8666633fdd11548dafff0a48af0982d5ad9a46`, repeated that binary and output
digest: median numerator `76,119,018 ns` (median `38,059,509 ns`) and p95
`42,049,356 ns`, both pass. Its raw-sample SHA-256 is
`9e3ac3902a814c65dbaa31b22ec859b79441a5aa51d320406b7e3fa4ee87a206` and
summary SHA-256 is
`1e104212c204aa14cd2ad623504fe4b1dfd486f7c88ecb4e0c69d0e5c6bea9e2`.

## Cold-review repair result and contemporaneous red evidence

Luna Max found four validation and boundary defects. Commit
`4c532de7b41f14cb66c3062a8384c19ebe484a81`, tree
`7e1a181b0fc8fe1c7bbab2b2ac80ab2b1923b7bb`, repairs them. Its release binary
SHA-256 is
`dde136c1f2abd66e68ec395ce2fcfb427eec62e17f150d1bf35776a9da41e264`;
the compiled output remains exactly 111,604 bytes with SHA-256
`aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`.

Two consecutive exact repaired-binary measurements ran while the host was
servicing sustained background writes and were red. They are retained rather
than excluded:

1. median numerator `205,207,952 ns` (median `102,603,976 ns`), p95
   `149,212,887 ns`; raw SHA-256
   `ff2e0f505c9cf0ce5c2e79e7ad7a8700f96b62693953870dd8d18efde47e403a`,
   summary SHA-256
   `e9495b3c01378494a1be2698671a8f69326988c49864a2e44e4a77f06d65c671`,
   retained at `target/r2-latency-4c532de/`.
2. median numerator `216,175,449 ns` (median `108,087,724.5 ns`), p95
   `143,731,872 ns`; raw SHA-256
   `9fa64c9a4d3965787c8b0e62c514a7eb62281c418a18563ece3d100771d6cb5f`,
   summary SHA-256
   `de2455cbde386042ff9f2c881665466fe439705826537b08a7a969a277f4f857`,
   retained at `target/r2-latency-4c532de-r2/`.

A contemporaneous control using the previously passing `a853d88b...` binary
was also red: median numerator `213,092,832 ns` (median `106,546,416 ns`), p95
`458,331,296 ns`; raw SHA-256
`e715c1a58a601a076631ba6c43e2c107500fd8eaf54bb87865f162c26080ca2c`,
summary SHA-256
`a5a87e215937b21cba7c60293f90f2695358332cf5991d51e974ae97d2001c5b`.
This control records the host-wide condition; it does not turn either repaired
run green.

After background writes quiesced, the repaired binary repeated the unchanged
method and passed: median numerator `75,155,960 ns` (median `37,577,980 ns`),
p95 `41,394,197 ns`; raw SHA-256
`3db7e8404b8d1bc78674be1913d14ee967ea5672180363b11ecee59f99e5073b`,
summary SHA-256
`9984434b483bf51cace00f170397581f36821aa27fe43c6107ea34e049660d3d`.
All 112 files are retained at `target/r2-latency-4c532de-r3/`. No workload,
ceiling, calculation, publication sync, or sample was changed or discarded.

## Exact repaired-candidate confirmation

The clean detached candidate at commit
`32ae37d61cd71bb7be11c43de56b83a64bf006a3`, tree
`2152165c2e07c0e23a023d17975723bc293bf348`, repeated the unchanged method
with the repaired binary and output digests:

- release binary SHA-256:
  `dde136c1f2abd66e68ec395ce2fcfb427eec62e17f150d1bf35776a9da41e264`;
- output: `111604` bytes, SHA-256
  `aa36d6befffa48870d8f6cee00663139ec301bb1b606b9270e5e7984566cd6f0`
  for every warmup and recorded process;
- median numerator: `76,740,574 ns`; median: `38,370,287 ns`; pass;
- p95: `43,190,202 ns`; pass;
- raw-sample SHA-256:
  `3feda73d21ea1c8482c5930a8cc1f295054d6aba5caab324f4df238c836a0821`;
- summary SHA-256:
  `dfd259f931a6fda1f59da42733179b13b7b877eff62a5d5ae1fd7996ae83f8b9`.

All 112 files are retained at
`/data/dev/src/nomos-issue-195-proof-3/target/r2-latency-32ae37d/`.
