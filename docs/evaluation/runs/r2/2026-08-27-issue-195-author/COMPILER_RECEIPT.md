# R2-1 compiler-produced plan receipt

Status: reproduced from the frozen implementation commit.

## Compiler binding

- Implementation commit: `13d778ddbd5c4428b9a809a014fbad19c1abd94d`
- Implementation tree: `76921e507f584f55e05cf68e08f72e8d14cf7aff`
- Release binary SHA-256:
  `8948aa69c094e6af964ddef6c46506cdce1bb18f75ddbced036b18f951e7cff4`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- Cargo: `cargo 1.98.0 (797e8a9bc 2026-08-05)`
- Host: `remi`, Linux `7.0.0-30-generic`, x86_64

## Input and output

- Input: `fixtures/r2/scenes/scene_one.json`
- Input bytes: `1211`
- Input SHA-256:
  `e4c04e7d6806aaba3e3ba9e0b94ba761442e00d5dd17a14581c29e1de22c41aa`
- Expected plan: `fixtures/r2/plans/scene_one.json`
- Plan bytes: `2305`
- Plan SHA-256:
  `717b91f3f35d815bfa9f9cc777b38f8a091f7a6339d786c57360e94ffe4c7699`

## Reproduction

From the exact implementation commit:

```text
cargo build --release --locked -p nomos-observed-scene
target/release/nomos-observed-scene compile --input fixtures/r2/scenes/scene_one.json --out target/r2-scene-one-13d778d.json
cmp fixtures/r2/plans/scene_one.json target/r2-scene-one-13d778d.json
```

The command exited zero, wrote no stdout or stderr, preserved the input, and
produced the expected plan bytes exactly. The fresh output remains retained at
`target/r2-scene-one-13d778d.json` in the author worktree.
