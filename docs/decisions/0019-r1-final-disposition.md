---
title: R1 final disposition — accept the runtime epoch, no game adoption
status: Draft; no owner disposition; does not take effect
number: 0019
date: 2026-08-26
owner: Peter Permenter
issue: 174
candidate_commit: pending prerequisite merges and final rerun
candidate_tree: pending prerequisite merges and final rerun
runtime_contract: RUNTIME.md revision 2
runtime_contract_sha256: pending frozen candidate
gate_k_disposition: docs/decisions/0013-gate-k-disposition.md
round_two_termination: docs/decisions/0016-terminate-gate-k-round-two.md
---

# R1 final disposition — accept the runtime epoch, no game adoption

## Draft boundary

This record is prepared under issue #174 for owner review. It is deliberately
not yet an owner decision: the exact candidate commit and tree are not frozen,
the final `RUNTIME.md` hash is not recorded, and the prerequisite corrective
and evidence pull requests have not received their required non-author reruns.
Nothing in this draft takes effect, closes R1, or calls a candidate green.

Before this record can move out of draft:

- PR #169 must land issue #159's `nomos.effective_facts@2` spelling alignment;
- PR #170 must land issue #165's explicit arrival/player-cell invariant;
- PR #171 must land issue #167's content-owned route fixture and dynamic corpus
  tests;
- PR #173 must land issue #172's clean-checkout offline and complete budget
  receipt;
- each of those four branches must carry its own non-author rerun receipt; and
- one different author must rerun the complete proof on the combined candidate
  and record the commit, commands, environment, result, and reviewer.

The remainder is the proposed disposition and the evidence map that will be
made exact after those conditions hold.

## Proposed verdict

**Accept R1 as a completed, passing runtime epoch. Do not adopt Nomos into any
game project.**

Those are separate decisions. `RUNTIME.md` replaced the unsatisfiable Gate K
criterion for the R1 line and can be satisfied on its own terms. It also kept
`THESIS.md` section 22 criteria 2 through 5 in force. Those broader criteria are
not all satisfied: there is no adopting game, no approved target pack for one,
and no Gate 1 proof in its intended runtime. Passing R1 therefore accepts a
repository runtime baseline; it does not make the thesis apply to a project.

## Candidate and authority boundary

The final record will bind one commit, one Git tree, `RUNTIME.md` revision 2 and
its SHA-256, and the exact successful workflow runs at that commit. No local
change, unmerged branch, expired artifact without a preserved compact receipt,
or issue/PR assertion outside that boundary will contribute to the verdict.

Decision 0018 remains the authority for RUNTIME revision 2's four contract-text
repairs. This disposition repairs no criterion and changes no contract wording;
it evaluates the frozen candidate against the contract already in force.

## Proposed R1 adoption matrix

The final table will say `pass`, `fail`, or `incomplete`, never “mostly,” and a
failed row cannot be offset by another. The provisional results below become
results only at the frozen candidate after the pending rerun.

| # | Provisional result | Load-bearing evidence | Pending closure |
| ---: | --- | --- | --- |
| 1 | expected pass | R1-1 through R1-5 landed in dependency order with target-specific evidence and non-author receipts on PRs #130, #143, #147, #151, and #156. | PR #169 aligns the post-decision-0018 effective-facts identity and needs its own non-author receipt. |
| 2 | expected pass | PR #143 deleted `build-plan.mjs`; `kernel_divergences.rs` plants the false-blocker, sub-base-cost, and unequal-cost cases; the accepted rendering compiler copies kernel effective facts. Issue #132 is closed with those references. | Final source audit and combined-candidate rerun. |
| 3 | expected pass | R1-3's 69-row ownership audit, closed typed `nomos.presentation_source@2`, integer-only source, named sockets, and strict Rust decoder; R1-4/R1-5 closed its seven deferred rows. | PR #170 makes the remaining arrival/player-cell relationship explicit and mechanically enforced. |
| 4 | expected pass | PR #173's `r1-adoption-evidence` job builds and tests the clean workspace, captures content, builds wasm, stages/scans the public artifact, and loads its first frame with no default network route; every section 7 row has a numeric runner-bound value. | PR #173 non-author receipt, merge, and combined-candidate rerun. |
| 5 | expected pass | Ossuary Reach, Gloam Bastion, and Drowned Stair were added without renderer or compiler source edits; the six-area viewer ran unchanged. The owner judged the two cold-authored rooms compelling. | PR #171 removes the Rust test corpus pins exposed by the experiment and needs its non-author receipt. |

## First-target receipts

These receipts establish the five implementation slices; they do not replace
the pending final candidate rerun.

| Target | Merge | Non-author receipt | What it established |
| --- | --- | --- | --- |
| R1-1 | PR #130, `db3fbb6` | final head `42d7825`, Claude Fable 5 | kernel-owned effective movement/light facts, byte identity, path binding, comparison `20 scenarios compared, 0 differences` |
| R1-2 | PR #143, `5deac46` | head `2c4799f`, Claude Fable 5 | Rust rendering-plan compiler, no shadow resolver, four-area equivalence and unchanged drawn evidence |
| R1-3 | PR #147, `52296dc` | head `720d8cc`, Claude Fable 5 | typed and versioned presentation source, single ownership audit, integer-only authored transforms |
| R1-4 | PR #151, `c257fb9` | head `ab00340`, Claude Fable 5 | promoted isolated viewer, vendored/digest-pinned Three.js, scanned offline artifact, browser route |
| R1-5 | PR #156, `4155699` | head `6565597`, Claude Fable 5 | authoritative Rust play state and pursuit in wasm, browser/native session identity, reproducible runtime |

The original R1-1 receipt names `nomos.effective_facts@1`. Decision 0018 later
required the R1 string spelling; issue #159 therefore retires that byte identity
as `nomos.effective_facts@2` rather than silently treating the old receipt as
proof of new bytes. PR #169's comparison expands the corpus from twenty to
thirty scenarios and is a required input to the final candidate.

## Complete offline and budget evidence

PR #173 adds the one-command receipt required by `RUNTIME.md` section 1
criterion 4. Its implementation-head run `32905965046` checked out
`bdd2229219bfb3b9efdf6c64f0d865f3202a4d82` on GitHub `ubuntu24` x86_64 image
`20260823.283.1`, removed every default network route, retained loopback only,
and forced Cargo offline. The compact record is
`docs/evaluation/r1-adoption-evidence.md`; the uploaded artifact's archive
SHA-256 is
`8e17731c3db4fd2d9859430e8133fc3a3a11c7dfc0e7e63ef864d06837160c72`.

The observations recorded into section 7 are:

- 17.344 s clean release workspace build;
- 9.783 ms median and 9.989 ms p95 validation latency;
- 349.668 kernel replay commands/s;
- 1 206.731 six-area play replay commands/s, 63.476 ms median and 65.064 ms
  p95 per 77-command replay;
- 20 492 bytes across eight compiled-package files;
- 1 387 887 bytes across twenty-four public-artifact files;
- 422 432 bytes for the wasm play runtime, SHA-256
  `70addbe7662caab4af2d0147c09dc8e839dd282c617a99cd325ced026d0d3a0f`;
  and
- 27 771 ms from the cold content pipeline's start to the first completed WebGL
  render, with 2 056 ms from navigation to that frame.

Those are observations of one recorded runner, not performance promises. The
final candidate rerun must reproduce the proof's success; its timings need not
equal an earlier noisy observation digit for digit.

## Content-authoring evidence and its limit

The active six-area route is Cistern Walk, Ember Vault, Gloam Bastion, Drowned
Stair, Ossuary Reach, and North Gaol. Gloam Bastion and Drowned Stair were
independently cold-authored from the packet; both passed `gaol verify` without
an authoring diagnostic or forbidden-path touch, and the unchanged viewer
played each five-area branch. Their combined six-area route also needed no
renderer or compiler source edit.

The experiment exposed two harness debts rather than concealing them. Issue
#165 records the unstated equality between a destination's route entry and its
player actor's cell; PR #170 makes that relation a decoder invariant. Issue
#167 records crate tests that named the corpus and pinned route counters; PR
#171 makes tests discover the corpus and consumes a route/counter fixture that
`gaol accept` regenerates and `gaol verify` compares. Those repairs strengthen
the evidence for future content additions.

The owner verdict in `docs/review/cold-author-area-five.md` is **compelling**.
It is expressly an informal Gate 2/Gate 3 miniature, not a formal Gate 2 or
Gate 3 pass. This disposition preserves that limit.

## THESIS.md section 22 remains unsatisfied

`RUNTIME.md` replaces section 22 criterion 1 for R1 only. It says the remaining
criteria stand and that the thesis still applies to no game until they are
separately satisfied.

| THESIS.md §22 criterion | Result | Reason |
| ---: | --- | --- |
| 1. Gate K passes | historically failed; replaced for R1 only | Decision 0013 remains controlling; decision 0016 authorized no retry. An R1 pass cannot be read back as Gate K credit. |
| 2. Gate 0 target pack approved for the adopting game | not met | The owner found the six-area study compelling, but no adopting game or its ten-part target pack exists. |
| 3. Gate 1's three primitives proven end to end in the intended runtime | not met | The repository proves semantics, presentation, movement, pursuit, persistence, and diagnostics; the Gate 1 door/water/light matrix also requires systems such as audio and networking that are absent. |
| 4. Adopting project records its own authority decision | not met | Nomos has authority only for this repository. No other project is in scope or has adopted it. |
| 5. Adopting project accepts the measured runtime cost | not met | R1 now measures the costs, but no adopting project exists to accept them. Measurement is necessary and is not itself adoption. |

The conclusion is exact: R1 may pass while the thesis applies to no game.

## Known maintenance issues

The final candidate may retain three small open maintenance issues, none of
which changes an R1 acceptance result:

- #160 tracks an intermittent post-result smoke-process hang. The harness
  already closes its browser, CDP socket, server, and connections and exits
  under a hard deadline; the issue asks for ten-run timing, a planted-open-
  handle test, and more diagnostic shutdown timing. Current required browser
  lanes complete successfully, including the network-isolated R1 receipt.
- #134 is a locale pin missing from the frozen Gate K schema-ownership replay
  script. R1 CI uses `r1-schema-ownership.sh`, which already pins `LC_ALL=C`.
- #141 is the same locale class in historical Gate K evaluation-tree digest
  helpers. It affects portability of rerunning those archived checkers, not the
  R1 runtime's artifacts or current schema lane.

They remain fix-or-file compliant because each has evidence, acceptance, and a
clear disposition. This record neither closes them nor uses them to waive a red
required lane. If any required final-candidate lane is red, R1 is incomplete
regardless of this section.

Issue #145, the original schema-spelling umbrella, should close only after PR
#169 lands its last implementation piece; decision 0018 supplied the owner
rule, not the bytes.

## Preserved historical verdicts and non-claims

- Gate K remains **failed** under decision 0013. Criteria 17 and 18 remain
  failed, and no R1 evidence is a pass, waiver, or partial credit for them.
- Gate K round two remains **terminated incomplete** with no verdict under
  decision 0016. No checker, retry, protocol revision 7, or round three is
  authorized.
- `KERNEL.md` revision 7 stays frozen. This record changes neither it nor any
  historical candidate, tag, task receipt, or evidence packet.
- R1 is not Gate 0, Gate 1, Gate 2, or Gate 3. The public viewer is not a
  production-art claim, and its pixels are not deterministic across GPUs.
- Nothing here authorizes networking, audio, combat, production scaling, a
  later runtime epoch, or adoption into another repository.

## Proposed consequences

If the final matrix passes and the owner adopts the proposed verdict:

1. R1 closes as the accepted repository runtime baseline at the frozen
   candidate. Its five first targets and recorded dependencies remain governed
   by `RUNTIME.md` revision 2.
2. Future maintenance may repair bugs against that baseline through ordinary
   falsifiable issues and proof. A new accepted capability family, dependency
   policy, or runtime epoch requires its own owner decision.
3. Work toward a game remains exploratory until a concrete adopting project
   separately supplies Gate 0, Gate 1, its authority-tree decision, and its
   acceptance of measured cost.
4. No work reopens Gate K or its cold-agent rounds.

## Owner disposition

**Pending.** Recommended final text after the candidate is frozen and the
different-author receipt is recorded:

> **Accept R1; do not adopt Nomos into a game.** R1's five adoption criteria
> pass at the bound candidate, so the runtime epoch closes as the accepted
> baseline for this repository. `THESIS.md` section 22 criteria 2 through 5 do
> not all pass, so the thesis still applies to no game project. Gate K remains
> failed, round two remains terminated incomplete, and no later gate or epoch
> is authorized by this decision.
