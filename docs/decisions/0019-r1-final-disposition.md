---
title: R1 final disposition — accept the runtime epoch, no game adoption
status: Final proposal; owner disposition pending; does not take effect
number: 0019
date: 2026-08-26
owner: Peter Permenter
issue: 174
candidate_commit: 8ef33cedff69fdac0aad5094eb41d8375a2b0898
candidate_tree: f78f1a57ecf936f503e2bc8c8e852542b1172f75
runtime_contract: RUNTIME.md revision 3
runtime_contract_sha256: f13fef1b486b7258c5dfca76bb3db263f1a42e285450e9415dddeb20c789686c
r1_authority: docs/decisions/0017-post-gate-k-runtime-epoch.md
runtime_revision_3_authority: docs/decisions/0020-runtime-revision-3.md
gate_k_disposition: docs/decisions/0013-gate-k-disposition.md
round_two_termination: docs/decisions/0016-terminate-gate-k-round-two.md
---

# R1 final disposition — accept the runtime epoch, no game adoption

## Owner-review boundary

This record is complete for owner review but is not yet an owner decision.
Nothing in it takes effect or closes R1 until the owner records the explicit
disposition at the end.

The prerequisite corrections and evidence updates landed through PRs #169,
#170, #171, #173, #177, and #179, each with an exact-head non-author receipt.
The final different-author proof passed on the bound candidate and is recorded
below. No implementation, evidence, proof, or contract prerequisite remains;
only the owner's R1 disposition is pending.

## Proposed verdict

**Accept R1 as a completed, passing runtime epoch. No game adoption is
authorized by this decision.**

Those are separate decisions. `RUNTIME.md` replaced the unsatisfiable Gate K
criterion for the R1 line and can be satisfied on its own terms. It also kept
`THESIS.md` section 22 criteria 2 through 5 in force. Those broader criteria are
not all satisfied: there is no adopting game, no approved target pack for one,
and no Gate 1 proof in its intended runtime. Passing R1 therefore accepts a
repository runtime baseline; it does not make the thesis apply to a project.

## Candidate and authority boundary

This record binds commit `8ef33cedff69fdac0aad5094eb41d8375a2b0898`, Git
tree `f78f1a57ecf936f503e2bc8c8e852542b1172f75`, `RUNTIME.md` revision 3 and
SHA-256
`f13fef1b486b7258c5dfca76bb3db263f1a42e285450e9415dddeb20c789686c`, and
the exact successful workflow runs and final receipt below. No local change,
unmerged branch, expired artifact without a preserved compact receipt, or
unidentified issue/PR assertion contributes to the verdict. Historical slice
receipts contribute only where this record admits them by PR, exact head,
reviewer, command, and result.

Decision 0017 remains R1's authority, decision 0018 remains the authority for
revision 2's four repairs, and decision 0020 establishes revision 3's exact
comparison-count repair. This disposition repairs no criterion and changes no
contract wording; it evaluates the frozen candidate against revision 3 already
in force.

## R1 adoption matrix

Each row says `pass`, `fail`, or `incomplete`, never “mostly,” and no row offsets
another. All five pass at the bound candidate.

| # | Result | Load-bearing evidence |
| ---: | --- | --- |
| 1 | pass | R1-1 through R1-5 landed in dependency order with target-specific evidence and non-author receipts on PRs #130, #143, #147, #151, and #156. PR #169 and decision 0020 bind `nomos.effective_facts@2` and the normative `30 scenarios compared, 0 differences`; PR #177 carries the revision-3 non-author receipt. The final proof reran the full target set. |
| 2 | pass | PR #143 deleted `build-plan.mjs`; `kernel_divergences.rs` plants the false-blocker, sub-base-cost, and unequal-cost cases; the accepted rendering compiler consumes kernel effective facts. Issue #132 closed with those exact references, and the final source audit found no shadow resolver. |
| 3 | pass | R1-3's 69-row ownership audit, closed typed `nomos.presentation_source@2`, integer-only source, named sockets, and strict Rust decoder resolve the presentation boundary. PR #170 makes arrival/player-cell equality an `RP0202` decoder invariant; the final targeted refusal and 40-test source suite passed. |
| 4 | pass | PR #173 adds the no-default-route one-command proof; PR #179 binds section 7 and the compact record to the combined implementation candidate. Exact final-candidate run `32911714950`, job `98007062484`, rebuilt/tested offline and reproduced the recorded package, public-artifact, and wasm bytes. |
| 5 | pass | Area-addition commits `8f71e34`, `f62efa9`, `23c2cb2`, and `b790c54` contain no renderer/compiler source edit. PR #171 moves route/counter expectations into content, dynamically discovers the corpus, and refuses current area IDs in crate tests. The final proof regenerated the six-area fixture byte-identically: 77 commands, 65 moves, traversal cost 95. |

## First-target receipts

These receipts establish the five implementation slices. The final
different-author receipt below proves their combined result at the bound
candidate.

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

The prerequisite closure receipts are:

| Repair or evidence update | Merge | Exact reviewed head | Non-author reviewer |
| --- | --- | --- | --- |
| Effective-facts schema alignment, #159 | PR #169, `1e65f48` | `956186b` | `/root/review_169` |
| Route-entry/player-cell invariant, #165 | PR #170, `068ad3e` | `72bf17d` | `/root/review_170` |
| Dynamic corpus and route fixture, #167 | PR #171, `db5d065` | `381431d` | `/root/review_171` |
| Complete offline and budget lane, #172 | PR #173, `bf9e11b` | `02e8b04` | `/root/review_173` |
| Combined-candidate budget refresh, #178 | PR #179, `197832e` | `1b9400e` | `/root/review_179` |
| Runtime revision 3, #176 | PR #177, `8ef33ce` | `342b0c3` | `/root/review_177_draft` |

## Complete offline and budget evidence

PR #173 adds the one-command receipt required by `RUNTIME.md` section 1
criterion 4. PR #179 refreshes that receipt after the corrective implementation
merges and binds section 7 to their combined candidate
`bf9e11b25a37591401033d76b94ac875a1cb92c1`, tree
`df7b1a9c023f5c9b4943b61f39c13f6b67668ead`. Run `32908589982`, job
`97997912940`, used GitHub `ubuntu24` x86_64 image `20260816.277.1`, removed
every default network route, retained loopback only, and forced Cargo offline.
The compact record is `docs/evaluation/r1-adoption-evidence.md`; artifact
`9585756215` has archive SHA-256
`8180c7ee3e267e6ff9b371a982189a6161a3c308a092d59d215bc535aadf104d`.

The observations recorded into section 7 are:

- 22.225 s clean release workspace build;
- 15.692 ms median and 15.905 ms p95 validation latency;
- 226.913 kernel replay commands/s;
- 932.278 six-area play replay commands/s, 82.273 ms median and 83.616 ms
  p95 per 77-command replay;
- 20 492 bytes across eight compiled-package files;
- 1 386 650 bytes across twenty-four public-artifact files;
- 421 195 bytes for the wasm play runtime, SHA-256
  `e8e03c125667ad937939f4a628b67df9ff813a88823fecd859784ed241673c97`;
  and
- 27 740 ms from the cold content pipeline's start to the first completed WebGL
  render, with 2 821 ms from navigation to that frame.

Those are observations of one recorded runner, not performance promises. The
final candidate rerun reproduced the exact package, public-artifact, and wasm
sizes and wasm digest. Its noisy timing observations were 22.559 s build,
14.217/14.327 ms validation, 249.336 kernel commands/s, 1 034.014 play
commands/s with 74.445/75.045 ms replay latency, and 24 022/2 381 ms content
pipeline/navigation-to-frame.

## Final different-author proof

Reviewer `/root/final_r1_proof` (Luna/max) independently checked the exact bound
candidate on Linux x86_64 with Rust/Cargo 1.98.0, Node 26.7.0, and headless
Chrome 151.0.7922.34. The worktree was clean before and after, and the reviewer
made no source edit, commit, push, merge, or GitHub mutation.

The reviewer set `PROOF_TARGET` to the fresh disposable directory
`/work/signed-dev/r1-final-proof-target.yOEgJP` and `PROOF_DOWNLOAD` to
`/work/signed-dev/r1-final-proof-download.065tc6`. These were the exact proof
commands; the area-ID loop follows separately:

```text
CARGO_TARGET_DIR="$PROOF_TARGET" cargo fmt --all -- --check
CARGO_TARGET_DIR="$PROOF_TARGET" cargo clippy --workspace --all-targets --locked -- -D warnings
CARGO_TARGET_DIR="$PROOF_TARGET" cargo test --workspace --locked
CARGO_TARGET_DIR="$PROOF_TARGET" cargo xtask boundary
docs/evaluation/r1-schema-ownership.sh
CARGO_TARGET_DIR="$PROOF_TARGET" experiments/executable-gaol/compare-effective-facts.sh
CARGO_TARGET_DIR="$PROOF_TARGET" experiments/executable-gaol/gaol verify
CARGO_TARGET_DIR="$PROOF_TARGET" cargo test -p nomos-render-plan --test schema_binding the_gate_k_object_spelling_is_refused_for_an_r1_document --locked
CARGO_TARGET_DIR="$PROOF_TARGET" cargo test -p nomos-play r1_schema_binding_refuses_the_gate_k_object_spelling --locked
CARGO_TARGET_DIR="$PROOF_TARGET" cargo test -p nomos-render-plan --test source a_route_entry_different_from_the_player_cell_is_refused --locked
CARGO_TARGET_DIR="$PROOF_TARGET" cargo test -p nomos-play --test corpus --locked
CARGO_TARGET_DIR="$PROOF_TARGET" cargo test -p nomos-play --test session --locked
CARGO_TARGET_DIR="$PROOF_TARGET" cargo test -p nomos-render-plan --test collection --locked
node experiments/executable-gaol/src/route-expectations.mjs target/executable-gaol/areas.json target/executable-gaol/areas "$PROOF_TARGET/route-expectations-independent.json"
cmp "$PROOF_TARGET/route-expectations-independent.json" experiments/executable-gaol/route-expectations.json
CARGO_TARGET_DIR="$PROOF_TARGET" crates/nomos-play/build-wasm.sh --offline
CARGO_TARGET_DIR="$PROOF_TARGET" cargo build --locked -p nomos-play --release
node apps/nomos-viewer/build.mjs --from target/executable-gaol --wasm target/wasm32-unknown-unknown/wasm/nomos_play.wasm --out apps/nomos-viewer/dist --receipt "$PROOF_DOWNLOAD/viewer-build-receipt.json"
CHROME_BIN=/work/signed-dev/.cache/ms-playwright/chromium_headless_shell-1234/chrome-headless-shell-linux64/chrome-headless-shell node --test apps/nomos-viewer/test/*.test.mjs
CHROME_BIN=/work/signed-dev/.cache/ms-playwright/chromium_headless_shell-1234/chrome-headless-shell-linux64/chrome-headless-shell node apps/nomos-viewer/smoke/smoke.mjs --dist apps/nomos-viewer/dist --out target/nomos-viewer-smoke --require-chrome
target/release/nomos-play replay target/executable-gaol/areas --session target/nomos-viewer-smoke/session.json
```

```text
ids=$(find experiments/executable-gaol/areas -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | LC_ALL=C sort)
count=$(printf '%s\n' "$ids" | sed '/^$/d' | wc -l)
test "$count" -eq 6
matches=0
while IFS= read -r id; do
  [ -n "$id" ] || continue
  found=$(rg -n --fixed-strings "$id" crates/*/tests 2>/dev/null || true)
  [ -z "$found" ] || matches=$((matches + 1))
done <<EOF
$ids
EOF
test "$matches" -eq 0
```

The literal-`target/debug` harnesses used a temporary `target` symlink to that
fresh external target. The independent area scan found six directory IDs and
zero occurrences in `crates/*/tests`; route regeneration was byte-identical.

The ordered local proof passed formatting, clippy with warnings denied, all
workspace tests, the dependency boundary, schema ownership at 20 Gate K and 10
R1 identities, 30 effective-facts scenarios with zero differences, six-area
`gaol verify`, targeted schema refusals and `RP0202`, dynamic corpus/session/
collection tests, the no-area-ID test scan, byte-identical route-fixture
regeneration, wasm and native play builds, 102 viewer Node tests, browser smoke
with zero external requests, and native replay. It reproduced 6 areas, 77
commands, 65 moves, traversal cost 95, and native/browser chain head
`43a1b2164f18bc54738d0402013419659576e2d866c3fca630321a2ca641f143`.

The exact-head GitHub receipts are:

- `verify`: run `32911714874`, job `98007062214`, success;
- `gate-k-evidence`: run `32911714880`, jobs `98007062164`, `98007062309`,
  `98007062310`, `98007062375`, `98007062384`, and `98007187884`, all
  success; and
- `nomos viewer`: run `32911714950`, tests job `98007062606` and R1 job
  `98007062484`, both success.

Artifact `9586787444` is 1 494 683 archive bytes with SHA-256
`c3e62456301ebce3466717c6bfc117c05c205b7ea6fa06f179e481b541e77388`.
The reviewer evaluated each of the five `RUNTIME.md` section 1 criteria
separately and returned five passes. The overall independent verdict was safe
to call R1 proof green; owner disposition remained expressly outside the
reviewer's authority.

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
#171 removes current area IDs and route counters from code-side tests, makes
tests discover the corpus, and consumes a route/counter fixture that `gaol
accept` regenerates and `gaol verify` compares. Those repairs strengthen the
evidence for future content additions.

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
| 3. Gate 1's three primitives proven end to end in the intended runtime | not met | R1 proves its specified subset: semantics, presentation, movement, pursuit, persistence, and diagnostics in this repository runtime. Gate 1's intended adopting-game proof also requires the target's door/water/light matrix and systems such as audio, networking, and replication; no adopting runtime exists. |
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

Issue #145, the original schema-spelling umbrella, closed after PR #169 landed
its last implementation piece; decision 0018 supplied the owner rule, not the
bytes.

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
   by `RUNTIME.md` revision 3.
2. Future maintenance may repair bugs against that baseline through ordinary
   falsifiable issues and proof. A new accepted capability family, dependency
   policy, or runtime epoch requires its own owner decision.
3. Work toward a game remains exploratory until a concrete adopting project
   separately supplies Gate 0, Gate 1, its authority-tree decision, and its
   acceptance of measured cost.
4. No Gate K retry, round, or reopening is authorized. Any future attempt
   requires a new owner decision.

## Owner disposition

**Pending.** The candidate and evidence are complete. Recommended owner text:

> **Accept R1; do not adopt Nomos into a game.** R1's five adoption criteria
> pass at the bound candidate, so the runtime epoch closes as the accepted
> baseline for this repository. `THESIS.md` section 22 criteria 2 through 5 do
> not all pass, so the thesis still applies to no game project. Gate K remains
> failed, round two remains terminated incomplete, and no later gate or epoch
> is authorized by this decision.
