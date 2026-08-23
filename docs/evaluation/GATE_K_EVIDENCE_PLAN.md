---
title: Gate K mechanical evidence plan
status: Predeclared methodology for issue 69
date: 2026-08-23
candidate: post-SW-N implementation-complete tree
---

# Gate K mechanical evidence plan

This plan fixes the measurement method before final values are observed. It
closes only the determinism, measured-budget, and canonical-schema-ownership
evidence gaps assigned to issue #69. It changes no semantic contract and sets no
performance threshold after seeing results.

## Determinism matrix

The dedicated `.github/workflows/gate-k-evidence.yml` workflow uses native
GitHub-hosted Linux runners:

| Lane | Runner label | Profile | Executions |
| --- | --- | --- | ---: |
| Linux x86_64 debug | `ubuntu-24.04` | debug | 10 |
| Linux x86_64 release | `ubuntu-24.04` | release | 10 |
| Linux aarch64 release | `ubuntu-24.04-arm` | release | 10 |

Each lane checks out one exact commit, installs the repository-pinned Rust
toolchain, builds `nomos` once in its declared profile, and invokes
`gate-k-determinism.sh`. Every execution gets a fresh uniquely named directory
and runs the public active-v2 path:

```text
nomos compile fixtures/gaol.nomos --out <fresh>/gaol.world
nomos run <fresh>/gaol.world --commands fixtures/gaol.commands --out <fresh>/gaol.run
nomos replay <fresh>/gaol.world --log fixtures/gaol.replay --out <fresh>/gaol.replay.run
```

Within each execution, ordinary run and replay bundles must be byte-identical.
Within a lane, all package members and all run members must be byte-identical
across the ten executions. The cross-target job downloads all three lane
artifacts and requires their preserved package and run baselines to be
byte-identical.

The per-target receipt preserves the exact commit, initial/final tracked-tree
state, runner image and architecture, kernel and CPU descriptions, Rust/Cargo
versions, build profile, commands, every artifact SHA-256 digest, the package
manifest digest, World IR digest, simulation-semantics digest, complete state
hash sequence, and the ten per-run checksum tables. The cross-target receipt
records the three source receipts and the final shared semantic checksum table.

The older `nomos-core` hash-domain fixture remains in `verify.yml`. It is a
canonicalization regression only; it is not part of this process-level matrix.

## Budget method

`gate-k-budgets.sh` runs on `ubuntu-24.04` x86_64. Required host tools are Bash,
GNU `time`, GNU `date`, `du`, `awk`, `sort`, and the pinned Rust/Cargo toolchain.
It writes build products into a new dedicated `CARGO_TARGET_DIR`; a pre-existing
directory is refused. Cargo caches and the installed toolchain are not deleted,
and their presence is recorded.

The clean release workspace build is:

```text
cargo build --workspace --release --locked
```

Wall time and maximum resident set size come from process-level GNU `time`.
While the process is alive, a sampler records `du -sk` for the dedicated target
directory every 50 milliseconds. The receipt retains every disk sample, the
sampled peak, and final target size. Peak disk is therefore observed during the
build rather than inferred from the final directory.

The release binary then performs three fixed warmups and twenty measured
process-level samples for each operation:

- `nomos validate fixtures/gaol.nomos`;
- one `nomos command` from a strictly decoded verified six-file run state to a
  fresh output directory; and
- `nomos replay` of the accepted five-command replay to a fresh output
  directory.

Each sample records monotonic-enough host nanosecond timestamps around the whole
process and GNU `time` maximum RSS. Output publication is inside the measured
command/replay interval. Raw warmup and measured rows are retained. Summaries
report minimum, median (mean of the two middle values for an even sample count),
nearest-rank p95, and maximum. Replay throughput is computed from the aggregate
measured wall time as complete replays per second and committed commands per
second; the script verifies the accepted replay contains five committed rows.

These are environment-specific observations, not universal performance
guarantees. Issue #69 records values without optimizing or moving thresholds.

## Schema-ownership review

The final source review is recorded separately in
`SCHEMA_OWNERSHIP.md`. It enumerates the expected twenty identities and checks
their authoritative types, encoders, strict decoders or regeneration checks,
persisted boundaries, consumers, and legacy status directly in source. Package
registries, compiler consumed/produced schema lists, projection `all_schemas`,
runtime schema functions, tests, and active documentation are corroborating
evidence; none substitutes for the source review.

## Evidence disposition

Workflow artifacts are immutable evidence attached to the exact workflow run.
The PR receipt names their run, commit, and artifact digests. If any later
candidate-affecting change occurs, the complete dedicated workflow reruns on the
new exact head. This issue does not create an RC tag, run a formal cold agent, or
claim Gate K green.
