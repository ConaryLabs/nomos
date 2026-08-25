---
title: R1 clean-checkout offline and budget evidence
status: Exact combined-candidate CI evidence; final disposition pending
date: 2026-08-26
candidate: bf9e11b25a37591401033d76b94ac875a1cb92c1
candidate_tree: df7b1a9c023f5c9b4943b61f39c13f6b67668ead
workflow_run: 32908589982
workflow_job: 97997912940
artifact_id: 9585756215
artifact_sha256: 8180c7ee3e267e6ff9b371a982189a6161a3c308a092d59d215bc535aadf104d
issue: 178
originating_issue: 172
---

# R1 clean-checkout offline and budget evidence

This is the compact repository copy of the load-bearing receipt from successful
`nomos viewer` workflow run `32908589982`, job `97997912940`, `R1 offline
build, artifact, budgets`. The job checked out exact combined candidate
`bf9e11b25a37591401033d76b94ac875a1cb92c1`, tree
`df7b1a9c023f5c9b4943b61f39c13f6b67668ead`. GitHub artifact `9585756215`,
`r1-adoption-evidence`, is 1 497 140 bytes and has archive SHA-256
`8180c7ee3e267e6ff9b371a982189a6161a3c308a092d59d215bc535aadf104d`;
its compact `receipt.txt` hashes to
`2e6b8fe887c33431ae99d186233a673d31e2bd55a87ad3b27562bfe7bc5d9228`.

The earlier implementation-head receipt at
`bdd2229219bfb3b9efdf6c64f0d865f3202a4d82`, run `32905965046`, artifact
`9584836533`, remains immutable historical evidence. This record supersedes its
transcription because the combined corrective merges changed the exact wasm
and public-artifact bytes; it does not relabel or alter the earlier artifact.

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
`b96dd69de3e3266efe4c404df5e6fddcc342360afdda2f3833ce943aaa36996f`.

## Measurements

Three warmups precede twenty process-level measured samples for validation,
kernel replay, and play replay. Times include process startup. File sizes are
the sum of regular-file bytes rather than allocated disk blocks.

| Field | Observation |
| --- | ---: |
| Clean release workspace build | 22.224724296 s |
| Validation | 15.692150 ms median; 15.904666 ms p95 |
| Kernel replay | 226.912927 committed commands/s; five commands per replay |
| Six-area play replay | 932.278176 committed commands/s; 82.272551 ms median; 83.615599 ms p95; 77 commands per replay |
| Compiled accepted-fixture package | 20 492 bytes; 8 regular files |
| Staged and scanned public artifact | 1 386 650 bytes; 24 regular files |
| Authoritative wasm runtime | 421 195 bytes; SHA-256 `e8e03c125667ad937939f4a628b67df9ff813a88823fecd859784ed241673c97` |
| Cold edit-to-visible frame | 27 740 ms; navigation-to-first-frame component 2 821 ms |

The edit-to-visible interval starts before content compilation/capture, when no
derived content, wasm runtime, or public artifact exists. It ends when the page
sets `data-ready`, after `renderer.present()` has synchronously submitted the
first WebGL render. It therefore excludes the smoke lane's subsequent route
play and native replay. Proof-only viewer tests run before the interval and are
also excluded.

The raw Gate K operation samples hash to
`e3fcdc45bfa71e49880decc4a7a70d2d4ac08dbacf3a2a67f79ddc1424ead22d`;
the raw play-replay samples hash to
`90363f85056ee947d353dab5b4f83ecdbe1be2ed8baed4f8e22dd32489e7bd42`.
The build, kernel-operation, kernel-throughput, and play-replay summary tables
hash respectively to
`4de7a7f4c4026e38451bcaef9ea0cef152170b3699e1e05a2adaf56d89b769bc`,
`2893bb2c4e6f32c526458b27c1b3619340d2d98ffa4fce246200a4eb6c21c762`,
`11866c5d0a28497eaa89a28f35c4a428193e2d91c804c96e51c56155f2a7da59`,
and `b6953e8ca961e3641ab0a0dca707e5e18273406087f5a51bb4c15677a668c68f`.
The staged-build and browser receipts hash to
`2cbf870c5825d7e61cf2da6c7f8ec924db90f5a48e931dbfae6a6ddf1f77e0c7`
and `e457726c2ae81f897c682275c5ad4d27eec1dc8b2649a5f2e2497883291cd7f5`.

## Environment and limits

The runner was GitHub `ubuntu24` x86_64 image `20260816.277.1`, Linux
`6.17.0-1022-azure`, Rust and Cargo `1.98.0`, Node `22.23.2`, and Chrome
`151.0.7922.137`. These are observations of that runner, not portable targets
or guarantees.

Independent reviewer `/root/final_r1_proof` downloaded this exact artifact,
recomputed its load-bearing hashes, reproduced the wasm size and digest in a
fresh local build, and passed the complete implementation proof on the same
candidate. That review correctly kept the overall verdict incomplete because
the prior repository copy did not yet bind these measurements and issue #176's
contract-text contradiction remained open. This evidence refresh resolves only
the former; it does not pre-approve the later combined candidate or decision
0019.
