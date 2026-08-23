---
title: Gate K RC1 mechanical evidence receipt
status: Preserved exact-candidate evidence
date: 2026-08-23
candidate: d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9
candidate_tag: gate-k-rc1
workflow_run: 32618725710
---

# Gate K RC1 mechanical evidence

This is the compact repository copy of the load-bearing receipts from the
successful `gate-k-evidence` workflow run `32618725710`. The run checked out
exact candidate `d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9`. The complete downloaded
artifacts remain identified by their GitHub artifact IDs and archive SHA-256
digests below; this record does not pretend that issue or PR prose is evidence.

## Determinism

Each native lane built once and performed ten fresh process-level
`compile`/`run`/`replay` executions. Ordinary run and replay bundles were
byte-identical, all ten executions within each lane were byte-identical, and
the three lanes shared the same semantic bytes.

| Lane | Profile | Receipt SHA-256 | Artifact ID | Archive SHA-256 |
| --- | --- | --- | ---: | --- |
| Linux x86_64 | debug | `8878a307a1fe0403c70e4d4c9d84a38e3f24af69fc950bdd109cb894f242acd0` | `9487728929` | `5f810bcf747fb553e3e48f943a1ebc4b52a09db6fd620ed69ea0003d509fad7b` |
| Linux x86_64 | release | `684617f15463d9acf53329b2128d56061c12ea855933cb4166a5802205bf203a` | `9487733469` | `a65cf71039658c6994e7562147a5f3d0f7c0f7f2d228f9f2a633f7215ca1b595` |
| Linux aarch64 | release | `36fe8a5d72ea8ee8dd34e58be354d699bcc4d2e0e7608e6d4b6e0581e177dbed` | `9487731461` | `3f7e9996f04454cc5c8c4ffcee8d9a940ad86efe97e38adb0352ee007b7bc3c1` |
| Cross-target | all | `55af1a8372043118e11827a8c3b20e2ba84746e8f86e91168b09f348d64e4be3` | `9487736044` | `1bbbcb14d393acfdeee3db980f408280e60c428f3294e23608b777393199f9a7` |

The cross-target semantic digest table hashes to
`75fa9d29b736fc93d30a49260c9c698376fe3fdfcffc75d7ef0852e6d398bfbe`.
Its exact rows are:

```text
6ee9f7af8cf382ef1f3cc11c51ee608026310508ee9408129759e9e87c441d89  gaol.run/causal-receipts.json
ef30d6a25026a162111e3e180a65a0e536f94b0b8ef0d73027bdbedd865ad6d9  gaol.run/command-log.json
fdeb5db8257ba41927de65fb50b08edc54cace49ae332ced327f4a8cad051a4d  gaol.run/final-state.json
9d6cf31816a8a1f7887858ea9d8a69307b2d67b4c0cfa6489fb8b08658bcfa6b  gaol.run/initial-state.json
e13f4ce9339967e51a06353a1315fa82c0e5a45172073f48af02b1fe0d5a8e7a  gaol.run/result.json
1989700685bf430aca5f1394ef279c9e720011e4834fd48df36e8ed275595320  gaol.run/state-hashes.json
cd1e5578c6368590a709058e0b772aadb05bb8c3e96424b52998459fb7a65494  gaol.world/compiler-receipts.json
da208fbc883dc9aa64748dd44a9c940956e1842a5be101156cd88c1204fd31e4  gaol.world/diagnostics.json
f963dd317fa57f5e626ce5281f9e35dcd9790db05236f2125ef8f1c395beda2a  gaol.world/manifest.json
e22e585399cb902bebbd3cdb00dfbd7497344d51c76454aa14404de831c93f71  gaol.world/navigation.json
40a0622cfb1ef13af32328208bc5618a6508e111cf4e375a3d00d20571d0c540  gaol.world/persistence.json
724b92a6ca4d9e25a5d35114b753b7be40e386627283e1235ca127145a2ac3b9  gaol.world/schemas.json
0cdb454be9876dccdc05369e91fb9bdea017973d77f7803d23a921a969a1ef3c  gaol.world/simulation.json
c55f09fe400465c637c75cbef0ea3a0ef06bd4a585c0ead78b1a50a63d790b7c  gaol.world/world-ir.json
```

## Measured budgets

The predeclared method is in `docs/evaluation/GATE_K_EVIDENCE_PLAN.md`. The
exact-candidate budget artifact is GitHub artifact `9487733963`, archive
SHA-256 `8474a96dd8eda6658bd858bad70b37f8f663eaeac8bfa947cf109fe4f992f153`.
Its receipt hashes to
`fc1c06e110be389fb29db072e75af1e28c0c63f074278b156eb91cdab17eaf48`.

- clean release build: `35.326886396 s`;
- sampled peak/final target disk: `19,424 / 19,424 KiB`;
- build maximum RSS: `373,008 KiB`;
- three warmups and twenty process-level measured samples per operation;
- validate min/median/p95/max: `11.780384 / 12.041085 / 12.433112 / 12.450346 ms`;
- command min/median/p95/max: `12.735133 / 13.085078 / 13.346224 / 18.405442 ms`;
- replay min/median/p95/max: `16.985785 / 17.624397 / 17.965241 / 20.842133 ms`;
- throughput: `56.508886` complete replays/s and `282.544428` committed
  commands/s.

The preserved raw samples are
`gate-k-rc1-budget-raw-samples.tsv`, SHA-256
`f1aec3642cb95f070254af3657827809fcc2ad74e499d81e0da282c7c33535ee`.
The artifact's summary and throughput tables hash to
`4ca6ba9478a4a653323e56538eb8e508d7c700d0f5da66ae5799cc33900babb7`
and `acb754520e3959f4c8083dbb99a63ee1417896e64145edf650725f2ad2c68190`.

## Schema ownership

`docs/evaluation/SCHEMA_OWNERSHIP.md` is the explicit source-level review.
The exact-candidate workflow receipt reports twenty identities, zero duplicate
meanings, and exact intentional compile/migration receipt profiles. The receipt
hashes to `3bc22b07d6abbba56d2bfb1b6b80138e169d2be9d615e3f157925bb93612ebd1`.
GitHub artifact `9487725597` has archive SHA-256
`04bb9e9135beb52b044efa40cd8b7c51f949f265e56ea78685adea3c6fe7c6eb`.

## Environment boundary

The budget lane recorded Ubuntu 24 runner image `20260816.277.1`, Linux
`6.17.0-1022-azure`, x86_64, Rust `1.98.0`, Cargo `1.98.0`, three warmups,
twenty samples, and a 50 ms disk sampler. The per-lane artifacts retain each
runner's complete environment and CPU identity. All receipts record clean trees
before and after execution.
