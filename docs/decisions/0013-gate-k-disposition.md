---
title: Gate K final disposition
status: Owner-authorized; Gate K failed
number: 0013
date: 2026-08-23
owner: Peter Permenter
issue: 73
candidate_tag: gate-k-rc1
candidate_commit: d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9
candidate_tree: 4013e6629ea274c6f2e2e2570306cb35b6d41505
contract_revision: 7
formal_protocol_revision: 3
current_protocol_revision: 5
evidence_index: docs/evaluation/GATE_K_FINAL_EVIDENCE_INDEX.json
---

# Gate K final disposition

## Decision authority

Peter Permenter reviewed the complete Gate K evidence matrix and records the
explicit owner verdict: **Gate K failed**.

This is not a judgment that the semantic implementation is useless. Criteria
1–16 passed their observable proofs, and the final different-author rerun
satisfied criterion 19. The one permitted formal cold-author attempt and the
one permitted formal cold-debug attempt nevertheless violated their
predeclared outside-workspace-path rubric. Criteria 17 and 18 therefore failed.
All nineteen criteria would have needed to pass for Gate K to be green.

No contract wording is changed, no failed attempt is relabelled, and no retry is
authorized.

## Candidate and evidence boundary

The evaluated semantic candidate is annotated tag `gate-k-rc1`, commit
`d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9`, Git tree
`4013e6629ea274c6f2e2e2570306cb35b6d41505`. It combines issue #69 mechanical
evidence and issue #70 formal evaluation tooling after the implementation
freeze at `eb86f25f5084a5da83cdd4f26e42e68089367a11`. Contract revision 7,
`KERNEL.md` SHA-256
`9c9bc478bb710666aec6572505d7a504630cfdafc652d1830c7ef6f555110a9e`,
and base fixture SHA-256
`a69582e7400921cb0fed84fde16469c21081363af4ebaba93411e59ae3ca4725`
identify the exact candidate inputs. The base fixture and contract wording are
unchanged by this disposition; `KERNEL.md` metadata and status prose are updated
only to point at this failed verdict, as issue #73 requires.

Exact-candidate workflows `32618725700` and `32618725710` passed verification,
the three-target ten-run matrix, measured budgets, and schema ownership. Both
formal subject/checker packets bind the same candidate commit.

After the formal sessions exposed fail-open evidence assembly, issue #79 and
PR #80 changed only protocol/evaluation authentication and documentation. They
did not change crates, fixtures, Cargo inputs, the CLI, semantic documentation,
`KERNEL.md`, either packet, or any raw task record. Decisions 0011 and 0012
explicitly preserve the four legacy task receipts byte-for-byte, admit exactly
that frozen inventory, and authorize no retry. PRs #77 and #78 attach the
deterministically finalized records to those immutable tasks. This is the
disclosed evidence-envelope boundary; it is not a second semantic release
candidate and does not upgrade either failed attempt to protocol revision 5.

No uncommitted candidate state, hidden patch, local-only fixture, or unrecorded
environment input contributes to the verdict. The formal debug mutation is
preserved separately and hashes to
`42dcb05bd8d5acee4ba90c79be48a749a365927078e4c56905111290f9a44ed9`.

## Acceptance matrix

This table has exactly the nineteen acceptance criteria in `KERNEL.md` section
11. “Pass” means the criterion's own observation succeeded; it does not offset
a failed row.

| # | Result | Load-bearing evidence |
| ---: | --- | --- |
| 1 | pass | `fixtures/gaol.nomos` at the frozen hash above; `compile_fixture::the_base_fixture_is_one_screen_and_names_exactly_the_contract_entities` in exact-candidate workflow `32618725700`. |
| 2 | pass | Compiler mutation/fixture tests distinguish entity and catalog namespaces; `compile_fixture::the_catalog_credential_never_becomes_a_fourth_entity`; full candidate workspace test receipt. |
| 3 | pass | Public `validate`/`compile`/`inspect` integration test `sw_h_cli::validate_compile_and_inspect_are_immutable_and_deterministic`; candidate package/World IR hashes in the mechanical receipt. |
| 4 | pass | Primitive expansion and transaction tests inspect separate `access`, `integrity`, `ward`, `combustion`, and `emission` machines; no product-state table appears in the package evidence. |
| 5 | pass | Exact-candidate run `32618725710`: ten fresh executions each on Linux x86_64 debug, x86_64 release, and aarch64 release; common semantic digest table `75fa9d29…8bfbe`; `transactions::ignite_stages_local_then_target_owned_fire_damage_exactly_once`. |
| 6 | pass | `transactions::unlock_open_close_unseal_and_extinguish_change_only_their_machine`, resolver tests, and the candidate causal-receipt/state artifacts show access changing while the ward remains the surviving blocker. |
| 7 | pass | Movement resolver/compiler mutation suites reject unresolved or malformed block/traverse facts; simulation/navigation plan bytes are required equal. |
| 8 | pass | Candidate `world-ir.json`, simulation, and navigation artifacts bind `flooded_section` with traversal cost 3; movement projection agreement tests passed. |
| 9 | pass | `sw_f::extinguish_commits_versioned_state_hash_and_typed_projection_receipt`, light projection mutation tests, and candidate run receipts observe the emission removal and persistence/diagnostics deltas. |
| 10 | pass | Candidate compile/run/replay artifacts plus `sw_g` semantic opener attacks and movement/light projection agreement tests prove all four projections derive and validate together. |
| 11 | pass | `nomos-compiler/tests/mutations.rs`, transition, movement, light, stable-IR, and semantic-opener mutation suites reject the section-9 ownership/cross-reference failures with stable diagnostics. |
| 12 | pass | Public CLI suites `sw_h`, `sw_j`, `sw_k`, and `sw_m` compare input bytes and reject overlap/existing destinations; candidate ordinary run/replay outputs are byte-identical. |
| 13 | pass | `sw_m::legacy_and_migrated_semantics_normalize_to_identical_runtime_v2_evidence` and `migrate_cli_is_deterministic_immutable_and_required_before_runtime_use`; exact candidate workspace proof. |
| 14 | pass | `sw_n::door_water_and_light_reports_freeze_distinct_semantic_causes`, tick-4/tick-7 transition reports, and verified-input mutation tests exercise public explanation commands. |
| 15 | pass | `cargo xtask boundary`; `docs/evaluation/SCHEMA_OWNERSHIP.md`; exact-candidate receipt reports 20 identities, zero duplicate meanings, and exact intentional compile/migration profiles. |
| 16 | pass | Predeclared `GATE_K_EVIDENCE_PLAN.md`, exact-candidate budget artifact, preserved raw samples, and measured build/disk/RSS/validate/command/replay values in `gate-k-rc1-mechanical-evidence.md`. |
| 17 | fail | One formal Gemini author subject produced the correct second door, and the DeepSeek checker reproduced it, but checker command ordinals 1 and 16 requested `/dev/null`. Frozen rubric: fail. Result SHA-256 `e6990dac…f07dc`; no retry. |
| 18 | fail | One formal DeepSeek debugger found the true seeded cause and repair, and Gemini independently confirmed them, but subject command ordinals 1, 48, and 65 requested `/dev/null`. Frozen rubric: fail. Result SHA-256 `f09c9214…8089c`; no retry. |
| 19 | pass | The final different-author receipt in `docs/evaluation/final/gate-k-rc1-final-non-author-proof.json` records the exact candidate/tag, clean state before/after, environment, ordered commands, exact-candidate mechanical revalidation, formal-record hash audit, outputs, and zero findings. |

## Evidence ledger

The machine-readable content-addressed index is
`docs/evaluation/GATE_K_FINAL_EVIDENCE_INDEX.json`. The load-bearing closure
sequence is:

- issue #68 / PR #74 — implementation freeze, merge
  `eb86f25f5084a5da83cdd4f26e42e68089367a11`;
- issue #69 / PR #75 — mechanical matrix, budgets, and schema ownership, merge
  `8c32286dc779b76ce8e30f3b1b7817a551f41ba9`;
- issue #70 / PR #76 — packet/checker tooling and non-formal rehearsals, merge
  and candidate `d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9`;
- candidate workflows `32618725700` and `32618725710` — both pass;
- issue #79 / PR #80 — fail-closed evidence authentication, merge
  `f7e541c8eb809bdc6411ef763b0c65377d0ba573`, post-merge workflows
  `32647910061` and `32647910069` pass;
- issue #71 / PR #77 — immutable author/checker record, merge
  `898f5a20f4813d2b48ca0ca445f2940bae0bbec4`;
- issue #72 / PR #78 — immutable debug/checker record, merge
  `6b990f24b47bda6e92c5e9caf6905b4790d5db11`;
- final evidence-main workflows `32649651879` and `32649651810` — both pass;
- issue #73 — this owner disposition and final different-author candidate
  proof.

Qualification probes, Opus rehearsals, founding/architecture reviews, ordinary
slice audits, and superseded audit attempts remain preserved but do not count
as criteria 17 or 18. Only the four frozen formal ledger entries do.

## Formal outcomes

The author task receipt hashes are
`732af45918ebc27c02675f6c75c32e7718407545c9fa3a39de327d3591d382a8`
and `2e8c97d5a939ddd6fa9b33769f6e24b80fc242b1420c2660eef7f9742d542db3`.
Every one of the subject's 37 commands and checker's 42 commands was reviewed.
The two checker outside-path findings are hash-bound in `adjudication.json`.

The debug task receipt hashes are
`2820d2f46b2d895abc22b6677f4f3ba908199cdb9d057aee181b477eaeb82390`
and `0053d3df610e7e31322a2cfd9dfc641e160d3e5c64582df387d34cd4ddd37d37`.
Every one of the subject's 78 commands and checker's 41 commands was reviewed.
The three subject outside-path findings and hidden mutation are hash-bound in
the final record.

The formal subjects did the semantic work correctly. That fact is retained as
diagnostic evidence, not used to waive their protocol failures.

## Limits and non-goals

- Gate K proves no visual quality, renderer, gameplay quality, production
  scale, security/authenticity, networking, audio, or long-term save policy.
- The budget values are observations of the recorded runner, not portable
  performance guarantees.
- GitHub workflow archives have finite retention. Their archive hashes, compact
  receipts, semantic digest table, raw budget samples, formal task records, and
  final audit receipts are preserved in the repository.
- Protocol revisions 4 and 5 harden future evidence authentication; they do not
  retroactively change revision-3 model behavior or authorize another attempt.
- `gate-k-rc1` remains a failed release candidate, not an acceptance tag.

## Consequence

Gate K is closed as failed. Criteria 17 and 18 remain failed, formal retry is
not authorized, and no more Gate K semantic feature work is active. Gate 0,
renderer implementation, and later gates are not authorized by this result.
Any future continuation requires a new owner decision and a separately
falsifiable issue; it may not rewrite this result or weaken revision 7 after the
fact.

Decision 0005's temporary zero-third-party-dependency constraint ends with this
Gate K disposition. That admits no dependency automatically. Any later
experiment must state and authorize its own dependency policy.
