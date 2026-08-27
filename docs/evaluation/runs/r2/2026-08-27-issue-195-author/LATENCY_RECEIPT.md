# R2-1 maximum-scene latency receipt

Status: exact implementation-commit run passed; earlier red runs retained.

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
