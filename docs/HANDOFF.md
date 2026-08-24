# Nomos handoff

## Current state

Gate K is closed as **failed**. Peter Permenter's owner disposition is
`docs/decisions/0013-gate-k-disposition.md`.

- semantic candidate: annotated tag `gate-k-rc1`;
- commit: `d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9`;
- tree: `4013e6629ea274c6f2e2e2570306cb35b6d41505`;
- contract: `KERNEL.md` revision 7;
- formal launch protocol: revision 3;
- current cold-agent protocol: revision 6, owner-authorized in decision 0015;
- revision-6 tooling: implemented under issue #88; both live rehearsal pairs
  passed at exact tooling commit `cbfa3f7`, and the complete repository proof
  plus zero-finding non-author audit passed at `378392c`;
- implementation: complete through SW-N;
- acceptance: criteria 1–16 and 19 pass; criteria 17 and 18 fail;
- round-one retries: none authorized; a distinct round two is conditionally
  authorized after tooling, rehearsal, candidate freeze, and mechanical proof;
- renderer, executable Gate 0 work, and semantic development: not authorized;
- one static, quarantined gaol target-pack study: authorized by decision 0014,
  assembled under issue #83, and owner-disposed as
  `visual thesis compelling`.

The exact 1–19 matrix, owner consequence, protocol boundary, and limitations
are in decision 0013. The machine-readable content-addressed inventory is
`docs/evaluation/GATE_K_FINAL_EVIDENCE_INDEX.json`.

Decision 0015 changes no historical verdict. It prospectively separates
semantic merit, independence integrity, and operational compliance, permits
exactly `/dev/null` as a non-information-bearing device, and fixes the order for
a fresh `gate-k-rc2` round. No provider task or candidate tag exists yet.

## Why Gate K failed

The Gemini cold-author subject correctly added a second approved door. Its
independent DeepSeek checker reproduced the package byte-for-byte, but checker
command ordinals 1 and 16 requested `/dev/null`. The frozen rubric treats the
request itself as an outside-workspace violation even though the sandbox denied
it. Criterion 17 failed. Final result SHA-256:
`e6990dacde903f527d1cb46784a54d938a7e130f1193e51bb830a4a2284f07dc`.

The DeepSeek cold-debug subject found the true hidden semantic cause, excluded
alternatives, and produced the expected repair. Gemini independently reproduced
the failure and repair, but subject command ordinals 1, 48, and 65 requested
`/dev/null`. Criterion 18 failed. Final result SHA-256:
`f09c9214329f7f8bd7d4d4b31476a0f24c825add2f5bb434b7bf780f64d8089c`.

The semantic successes remain useful evidence. They do not waive the protocol
failures. The owner disposition is `fail; no retry authorized` for both.

## What passed

The exact candidate passed:

- the full workspace proof;
- ten fresh public-CLI compile/run/replay executions on Linux x86_64 debug,
  Linux x86_64 release, and Linux aarch64 release;
- byte-identical within-lane and cross-target semantic evidence;
- predeclared build, disk, memory, validation, command, and replay measurement;
- the explicit twenty-schema source ownership audit;
- the final different-author proof with a clean tree before and after.

The compact exact-candidate receipt and raw samples are under
`docs/evaluation/final/`. Candidate workflows:

- verify `32618725700` — pass;
- gate-k-evidence `32618725710` — pass.

After the formal records merged, final evidence-main workflows also passed:

- verify `32649651879`;
- gate-k-evidence `32649651810`.

## Evidence boundary

Issues #68–#73, #79, #82, #83, and #86 are closed. Their PRs are merged:

| Issue | PR | Disposition |
| ---: | ---: | --- |
| #68 | #74 | implementation freeze |
| #69 | #75 | determinism, budgets, schema ownership |
| #70 | #76 | evaluation tooling and non-formal rehearsals; creates `gate-k-rc1` |
| #71 | #77 | formal Gemini author plus DeepSeek checker; failed |
| #72 | #78 | formal DeepSeek debugger plus Gemini checker; failed |
| #79 | #80 | fail-closed evidence authentication and deterministic assembly |
| #73 | #81 | final owner verdict |
| #82 | #84 | authorize the quarantined gaol visual-target experiment |
| #83 | #85 | assemble and owner-dispose the static visual target pack |
| #86 | #87 | authorize protocol revision 6 and conditional round two |

Issue #79 changed evidence tooling and protocol documentation after the
candidate was frozen. It changed no kernel crate, fixture, Cargo input, CLI,
semantic documentation, raw formal packet, or raw task record. Decisions 0011
and 0012 preserve and admit exactly the four frozen legacy task receipts. The
final records remain bound to `gate-k-rc1`; there is no `gate-k-rc2` and no
acceptance tag.

Historical failed/superseded audits, Opus rehearsals, qualification probes, and
architecture reviews remain preserved and labelled. None counts as a formal
criterion-17 or criterion-18 subject.

## How to verify

From a clean checkout:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
docs/evaluation/test-pi-cold-agent-preflight.sh
docs/evaluation/test-gate-k-eval-tooling.sh
```

For the exact semantic candidate, check out `gate-k-rc1` and use workflow run
`32618725710` plus the hashes in
`docs/evaluation/final/gate-k-rc1-mechanical-evidence.md`. The final non-author
receipt records its independent recomputation of those artifacts and formal
record hashes.

## What is next

Issue #83 completes the only slice authorized by decision 0014. Its nine
coherent static bitmaps, `TARGET.md`, and hash-checked `manifest.json` are under
`experiments/gate-0-gaol-target-pack/`. The ordinary gameplay-camera frame is
the primary surface; the other images test environment, silhouette overlap,
combat, spell, low-light, UI, palette/material, and motion-timing failure modes.
The pack adds no executable code, renderer, rendering projection, asset
pipeline, visual primitive catalog, dependency, semantic feature, formal retry,
or claim of Gate K/Gate 0 acceptance.

Peter reviewed the complete pack, including the ordinary gameplay-camera frame
as the primary surface, and recorded `visual thesis compelling` in `TARGET.md`.
The pack is the preserved desired target. A fresh, prospectively governed Gate K
attempt remains required before renderer architecture or adoption; neither is
authorized by the visual verdict.

Issue #88 is the active tooling slice. It implements separately evidenced
dimension results and a mechanically derived verdict; exact revision-6 packet,
task, checker, adjudication, and final-result generations; and boundary
generation 4, which exposes exactly `/dev/null` while failing every other
device or outside path closed. The offline proof also re-finalizes both frozen
round-one pairs byte-for-byte without changing their verdicts.

The fresh non-formal Gemini-author/DeepSeek-checker and
DeepSeek-debugger/Gemini-checker pairs both passed at exact tooling commit
`cbfa3f7`. Their complete revision-6 records are under
`docs/evaluation/runs/rehearsal/2026-08-24-*-r6/`; both results explicitly have
`formalAttempt: false` and no Gate K acceptance value. The complete repository
and evaluation proof and a fresh DeepSeek-family non-author audit passed with
zero findings at `378392c`. Four rejected predecessor audits, three CI
portability failures, and every repair remain disclosed under
`docs/evaluation/runs/tooling/2026-08-24-*`. Issue #88 is locally complete in
draft PR #89 pending final CI and merge. Only after it merges may a separate
slice prove the unchanged semantic inputs and freeze `gate-k-rc2`. Formal
reservations and launches wait until the complete mechanical matrix passes
against that exact candidate.

Renderer architecture and project adoption remain out of scope until a new
owner disposition passes Gate K.

Decision 0005's temporary zero-third-party-dependency policy ended with the
Gate K disposition. That does not admit any dependency automatically; a future
experiment must choose its own policy explicitly.
