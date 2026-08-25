---
title: R1 clean-checkout offline and budget evidence
status: Author and CI evidence; non-author candidate rerun pending
date: 2026-08-26
candidate: bdd2229219bfb3b9efdf6c64f0d865f3202a4d82
workflow_run: 32905965046
artifact_id: 9584836533
issue: 172
---

# R1 clean-checkout offline and budget evidence

This is the compact repository copy of the load-bearing receipt from the
successful `nomos viewer` workflow run `32905965046`, job `R1 offline build,
artifact, budgets`. The job checked out exact branch head
`bdd2229219bfb3b9efdf6c64f0d865f3202a4d82`. GitHub artifact `9584836533`,
`r1-adoption-evidence`, has archive SHA-256
`8e17731c3db4fd2d9859430e8133fc3a3a11c7dfc0e7e63ef864d06837160c72`;
its compact `receipt.txt` hashes to
`be77b3b9652157575265cd3ae0a8c2de9edc740234763b2477c4fcfcf4089e0a`.

The artifact contains the environment, exact command ledger, build timing and
disk samples, all operation samples, the compiled fixture and base run used by
the measurements, workspace and viewer test output, the staged-build receipt,
the browser receipt and screenshots, the recorded six-area session, and all
twenty-three play-replay outputs. This compact copy identifies those bytes; it
does not substitute issue or pull-request prose for them.

## Offline boundary

The workflow provisioned the pinned Rust toolchain and Node runtime, then ran
`docs/evaluation/r1-adoption-evidence.sh target/r1-adoption` in a fresh network
namespace. The namespace had no IPv4 or IPv6 default route; loopback was its
only enabled interface. The script refuses an ordinary networked environment
and set `CARGO_NET_OFFLINE=true` before any repository build.

From a clean checkout with no derived target, content, wasm, or public-site
output, that isolated process:

1. built the complete workspace in release mode and tested the complete
   workspace with Cargo offline;
2. compiled and captured all six committed study areas;
3. built the authoritative wasm runtime with `--offline`;
4. ran the viewer tests, staged the public site, and passed its origin,
   forbidden-input, credential, and build-path scan;
5. loaded the site through loopback Chromium, whose own resolver additionally
   denied every non-local host, reached the final escape, and replayed the
   browser's 77-command session through the release native runtime; and
6. left the Git commit and tracked tree unchanged.

The receipt reports `workspace_build_offline yes`, `workspace_test_offline
yes`, `public_artifact_offline yes`, and clean state before and after. Its exact
command ledger hashes to
`60c423b6f9082873a9becfe70d85b7fa32630c7948cf61dd9c28d565d0958a96`.

## Measurements

Three warmups precede twenty process-level measured samples for validation,
kernel replay, and play replay. Times include process startup. File sizes are
the sum of regular-file bytes rather than allocated disk blocks.

| Field | Observation |
| --- | ---: |
| Clean release workspace build | 17.344341824 s |
| Validation | 9.783460 ms median; 9.988658 ms p95 |
| Kernel replay | 349.668386 committed commands/s; five commands per replay |
| Six-area play replay | 1 206.731111 committed commands/s; 63.476209 ms median; 65.063893 ms p95; 77 commands per replay |
| Compiled accepted-fixture package | 20 492 bytes; 8 regular files |
| Staged and scanned public artifact | 1 387 887 bytes; 24 regular files |
| Authoritative wasm runtime | 422 432 bytes; SHA-256 `70addbe7662caab4af2d0147c09dc8e839dd282c617a99cd325ced026d0d3a0f` |
| Cold edit-to-visible frame | 27 771 ms; navigation-to-first-frame component 2 056 ms |

The edit-to-visible interval starts before content compilation/capture, when no
derived content, wasm runtime, or public artifact exists. It ends when the page
sets `data-ready`, after `renderer.present()` has synchronously submitted the
first WebGL render. It therefore excludes the smoke lane's subsequent route
play and native replay. Proof-only viewer tests run before the interval and are
also excluded.

The raw Gate K operation samples hash to
`ea6acad0b1fe99200c028b600e79c2b5ede9223fbd0dd515633ff2304041b500`;
the raw play-replay samples hash to
`6b3d577af6e0771355a94b40adbf3811590fc245d11c6743e7151f76dcdb0cd8`.
The build, kernel-operation, kernel-throughput, and play-replay summary tables
hash respectively to
`e89b32e0bd3539b2035c4a27a590b0038c01c745fc2da6a75d5b7fbc8472290b`,
`a3227157f08a598563e00c72bcffd8a29f745b457689c80fdf56887cab4cae7e`,
`1d369235bb03463626ed16e4ec738866e6b807290606163cd01bd80658e55f66`,
and `ad4d6cd5d257fd5193b41cd54b0cdaa4c114830f7c7f425d35806a1350c5c3aa`.
The staged-build and browser receipts hash to
`7ad93fc176ef6a7ebf6e8743051d7b069b947cdcd3602879d8257e379e959647`
and `2962b70d487b6459ac1ff28f86457a519d40713bd030934dc07a7a4686d6be4a`.

## Environment and limits

The runner was GitHub `ubuntu24` x86_64 image `20260823.283.1`, Linux
`6.17.0-1022-azure`, Rust and Cargo `1.98.0`, Node `22.23.2`, and Chrome
`151.0.7922.173`. These are observations of that runner, not portable targets
or guarantees.

This record is author and CI evidence. CI is not the non-author reviewer
required by `AGENTS.md`; the frozen R1 candidate still needs its final
different-author rerun before an adoption disposition can call it green. This
follow-up documentation commit records the successful implementation-head
measurement and does not rewrite its samples as measurements of a later tree.
