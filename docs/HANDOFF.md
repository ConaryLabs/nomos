# Handoff — state of the repository

Updated 2026-08-23 for issue #79. Semantic implementation is complete through
SW-N. Combined issue #69/#70 evidence closure produced exact candidate
`gate-k-rc1` at `d8a0b85`. All four formal subject/checker sessions are complete,
and the owner dispositioned both formal attempts `fail`; Gate K is explicitly
unaccepted. Issue #79 repairs the finalizer without rewriting or retrying those
attempts. Its first implementation at `99c4436` passed CI but failed non-author
review; the replacement uses hash-bound structured command adjudication and
tightens candidate, subject/checker, and immutable-output bindings. A second
non-author audit rejected `a7d0d5a` because command records and the candidate
marker remained self-bound. A third audit rejected `5340158` because the
candidate packet and writable paths were still self-asserted, checker JSON
allowed duplicate keys, tool ends could precede starts, and numeric validation
accepted floats. A fourth audit rejected `0bf8155`: selected hashes did not
re-prove lifecycle/retry/accounting, qualification and launcher records could be
truncated, immutable packet members were not fully reopened, output could enter
a packet, and formal/candidate identity remained relabelable. The current repair
revalidates the complete Pi lifecycle and receipt chain, verifies every
immutable packet member, forbids packet/output overlap, and pins formal evidence
to the exact `gate-k-rc1` commit and binary. A fifth audit rejected `95a5430`:
event ordering and duplicate NDJSON keys remained open, writable packet output
was not compared with recorded artifacts, qualification receipts could still be
selectively truncated, and the invalidated candidate admitted fresh formal
attempts. A sixth audit of the subsequent repair found that transcript payloads,
qualification headers and ordering, packet isolation claims, timestamps, usage
arithmetic, and prelaunch attempt accounting were still incompletely
authenticated. The current repair validates the complete Pi event semantics and
pinned qualification environment, reopens every packet-boundary claim, rejects
invalid numeric/date values, and adds protocol revision 4's committed,
hash-chained formal-attempt ledger. The four frozen task receipts are imported
without pretending retroactive prelaunch proof exists. A seventh audit rejected
`7aef90d` because ledger closes did not authenticate the receipt/launcher,
runtime executable paths were insufficiently pinned, raw signature removal was
overbroad, optional usage and RFC 3339 ranges were loose, checker JSON admitted
non-finite values, and public packet schemas were not exact. The current repair
implements protocol revision 5 with receipt-backed closes, exact canonical
schemas, raw-stream digesting and signature-location checks, strict scalar
validation, and boundary schema `@3` runtime path-and-hash identities. Its fresh
exact-head eighth audit rejected `4108f06`: close still accepted a skeletal
launcher, legacy `@2` evidence was not restricted to the four frozen receipts,
signature removal was not schema-location-aware, Python boolean/integer equality
weakened exact scalar checks, and tooling metadata still named revision 3. The
current repair validates the complete task record and committed ledger HEAD,
admits legacy evidence only by exact frozen receipt hash, restricts sanitization
to explicit Pi message/result content locations, type-checks every fixed scalar,
and identifies protocol revision 5 consistently. The tenth audit then reproduced
one remaining scalar alias at `ac0f125`: Pi
session `version: 3.0` compared equal to integer `3`. The current repair requires
the exact integer type for Pi session versions and ledger sequence numbers and
adds adversarial regressions for both. A fresh exact-head audit remains.
This file orients a fresh session; it is rewritten at each slice boundary and
never accumulates history (git has that).

## Where things stand

- **Nomos is the active project identity:** contract revision 7
  and decision 0007 rename the runtime/project to Nomos while retaining The
  Signed World as the thesis name. Active crates use `nomos-*`, the binary is
  `nomos`, authoring source uses `.nomos`, schemas use `nomos.*`, and the fresh
  construction epoch began at `nomos.world_ir.construction@1` and its active
  light-aware shape is `nomos.world_ir.construction@3`. References to
  `signed-world`, `estate-*`, `.estate`, and `estate.*` below describe immutable
  prototype-era history unless explicitly called current.
  The mechanical implementation is commit `7c0ca31`; its full author proof,
  old-to-new golden relationship, and classified legacy-name audit are recorded
  under `docs/evaluation/runs/identity/2026-08-21-nomos-cutover/`. GPT-5.6 Luna
  max reran exact PR head `ec4e270` green; PR CI run `32526065565` passed. PR
  #32 merged as `6b803e1`, issue #31 closed, the repository and description were
  renamed, and post-merge CI run `32526851179` passed under
  `https://github.com/ConaryLabs/nomos`.

- **Contract revision 3 is owner-authorized:** decision 0003 pins the canonical
  escape profile, ASCII identifier and field-name grammar, isolated `xtask`,
  and the honest evidence boundary for semantic schema ownership.
- **Contract revision 4 is merged (#16):** issue #15 and decision 0004 correct
  the incomplete SW-C linker snapshot from `estate.world_ir@1` to separately
  versioned `estate.world_ir.construction@1`. The merge commit is `2603a4e`;
  the full-byte golden guard, Opus 5 review history, exact-head DeepSeek reruns,
  PR CI, and post-merge CI run `32507602324` are green. Local receipts live
  under `docs/evaluation/runs/contract/` and `docs/evaluation/runs/ci/`. Issue
  #15 is closed.
- **Rust 1.98.0 is current (#19/#20):** PR #20 advanced the live toolchain pin
  and workspace MSRV, passed author proof and CI, then received a GPT-5.6 Luna
  max non-author rerun against merge commit `feacad0`. The original SW-B
  receipts correctly remain on Rust 1.97.1. The workspace still has seven local
  lockfile entries and no third-party crates.
- **The dependency policy is explicit (#23):** owner-authorized decision 0005
  keeps Gate K at zero third-party dependencies for offline reproducibility and
  a small audit surface. It is temporary through Gate K, not inherited law for
  later renderer, signing, network, asset, or platform work.
- **SW-B is merged (#3):** six kernel crates plus isolated `xtask`, deterministic
  core primitives, package foundations, boundary enforcement, and green CI on
  main. Its original evidence used Rust 1.97.1.
- **SW-C is implemented by PR #6:** source-language decision 0002, the
  exact `fixtures/gaol.estate`, source AST and Canonical World IR construction
  schemas, typed lattice bindings, parser, distinct typed symbol tables, the
  sealed three-primitive expansion catalog, ownership linker/receipts, and
  mutation tests. The implementation commit is `be5576d`.
- **Non-author disposition:** Peter explicitly authorized merging PR #6 before
  its non-author rerun. DeepSeek V4 Pro then reran the complete proof through
  direct Reasonix at max effort against merge commit `4ec25e5`; all four
  commands passed with a clean tree before and after. The durable receipt is
  under `docs/evaluation/runs/gate-k/2026-08-21-deepseek-v4-pro-sw-c-rerun/`.
  SW-C now satisfies the repository's non-author rule. This was not a formal
  cold-agent run and does not upgrade whole-Gate-K status.
- **Issue #5 is disposed by PR #6.** Acceptance 3 remains only partially
  covered: IR expansion is proved, but the active observable `nomos inspect`
  command still requires complete packages and projections.
- **Issue #4 is closed:** revision 3 merged in PR #13.
- **The whole-kernel cold roster is predeclared:** Gemini 3.7 Flash High is the
  formal cold author; DeepSeek V4 Flash Vision Exp is the formal cold debugger;
  each independently checks the other's output. Issue #49 changes their
  transport to Pi without changing either family or role, and advances the
  exact DeepSeek-family model by owner direction. The plan and invalidation
  rules are in `docs/evaluation/GATE_K_COLD_AGENT_PLAN.md`.
- **The `agy` print path is repaired (#17):** official client 1.1.17 was
  restored after the host reinstall. The old argument order passed `--model` as
  the seven-byte print prompt; a prompt-first invocation now proves the exact
  Gemini 3.7 Flash High model and a completed `pwd` tool event in the target
  worktree. The fail-closed harness and exact receipt are under
  `docs/evaluation/`. The three 2026-08-21 attempts retain zero evidentiary
  value.
- **The `agy` formal route is ineligible on 1.1.17 (#45):** a fresh project and
  conversation with sandboxing, slash commands disabled, and a custom
  three-tool main agent still reported 57 tools and no machine-readable project,
  context-source, or memory disclosure. The fail-closed guard therefore exits
  `1` with `AGY_FORMAL_BOUNDARY BLOCKED`. The exact receipt is under
  `docs/evaluation/runs/tooling/2026-08-22-agy-formal-boundary-falsification/`.
  No Gemini formal attempt may launch until a future client/route passes that
  guard or the owner approves a new plan. This is tooling falsification, not a
  formal run, roster substitution, or protocol change.
- **Pi qualifies all three non-Codex lanes (#49):** Pi 0.84.2 now drives
  `antigravity/gemini-3.7-flash` at high,
  `deepseek/deepseek-v4-flash-vision-exp` at max, and supplemental
  `anthropic/claude-opus-5` at high through one common fail-closed launcher.
  The Gemini lane explicitly loads the separately pinned `pi-antigravity`
  0.4.0 provider; package discovery remains off. The new DeepSeek model was
  released after Pi 0.84.2, so that lane supplies a repository-owned,
  hash-pinned declarative model catalog while retaining Pi's built-in DeepSeek
  transport; it loads no executable DeepSeek plugin. All three authenticated
  author probes passed with fresh ephemeral sessions, the exact provider/model,
  only the repository-owned isolated `bash`, no model-requested tool call, and
  no credential leakage. Bubblewrap exposes only the target checkout as
  read-write, clears the child environment, and unshares the network. The
  offline matrix rejects every issue #49 negative case. Sanitized receipts are
  under
  `docs/evaluation/runs/tooling/2026-08-22-pi-provider-qualification/`.
  Because committing a receipt changes the head it names, final exact-head
  author and non-author disposition is recorded externally in PR #50. GPT-5.6
  Luna reran exact head `64cd3c7` green; PR #50 merged as `1ea9b44`, issue #49
  closed, and post-merge CI run `32545891569` passed.
- **Cold-agent protocol revision 2 is owner-authorized (#47):** decision 0008
  changes the active default tool wording from the published `estate` CLI to
  the published `nomos` CLI. Tool scope, packet boundaries, budgets, rubric,
  roster, attempt accounting, existing verdicts, and immutable prototype-era
  receipts do not change. PR #51 merged as `795791e`, issue #47 closed, its
  exact-head Luna rerun and CI passed, and no formal cold-agent attempt was
  launched by it.
- **Cold-agent protocol revision 3 is owner-authorized (#70):** decision 0010
  removes token, turn, validation/compile-cycle, and diagnostic-cycle ceilings
  after Opus rehearsals showed they added termination and shell-parsing failure
  modes. Tokens, turns, tool calls, and exact commands remain recorded. Fresh
  sessions, no coaching, no retry after model failure, packet isolation,
  eligibility, and rubrics remain unchanged. A non-formal Opus author
  subject/checker pair and debug subject/checker pair completed against exact clean
  candidate `71093eb46805c6811100e4b552595048a11b5346`; complete receipts live
  under `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author/`
  and `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-debug/`.
  The author pair recorded 17/15 assistant turns, 25/23 tool calls, and
  278,826/637,121 provider-reported tokens. The debug pair recorded 20/17
  assistant turns, 32/26 tool calls, and 750,160/719,023 provider-reported
  tokens. All four sessions were distinct, neither rehearsal was formal, and
  the Gemini and DeepSeek formal attempt counts remain zero. A GPT-5.6
  non-author audit of exact PR head `97a9c7c` then found three blocking harness
  defects: copied input trees were insufficiently allowlisted, packet-run
  `/tmp` remained writable, and durable finalization omitted checker
  reproduction artifacts. The branch repairs all three and extends the
  falsification suite. The first repaired-boundary author rerun then attempted
  two forbidden `/tmp` paths; Bubblewrap denied them, but its checker wrongly
  treated the attempts as minor and passed. The prompts and rubric now state
  explicitly that a denied model-requested outside access is still a rejection.
  Replacement r3 author and debug pairs passed against exact clean candidate
  `0072f9970cbc88c8936f3741b8cf9f48495a8c13`. The author pair recorded 15/17
  turns, 21/25 tool calls, and 241,568/519,793 provider-reported tokens; the
  debug pair recorded 23/19 turns, 37/25 tool calls, and 675,963/651,191
  provider-reported tokens. All four task sessions are distinct, record no
  intervention or retry, state `formalAttempt: false`, and have recomputable
  complete subject/checker artifact trees. The durable r3 receipts are under
  `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author-r3/` and
  `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-debug-r3/`. No
  superseded rehearsal transport is reclassified.
  A replacement non-author audit at evidence commit `18525aa` then found that
  packet-run `--dev /dev` exposed writable `/dev/null` and `/dev/shm`, the r3
  author/checker used `/dev/null` contrary to the exact rubric, and checker
  packet inputs were not bound to the subject receipt. The repair removes the
  packet-run device mount, remounts `/proc` read-only, proves both properties,
  and changes checker construction to accept and verify one complete subject
  record. R3 remains preserved but invalidated; both pairs require r4 reruns.
  The first repaired author pair at `0b807bf` proved the new boundary and
  receipt binding, but the subject still typed one reflexive `2>/dev/null`.
  The sandbox denied it, the subject disclosed it, and the checker correctly
  returned `reject`. That r4 failure is preserved under
  `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author-r4-rejected/`.
  Prompts now name the legal workspace-local output sink explicitly. Clean r5
  author and debug pairs passed against exact candidate `c1b9f35`. The author
  pair recorded 16/19 turns, 23/28 tool calls, and 265,474/564,830
  provider-reported tokens; the debug pair recorded 16/18 turns, 26/23 tool
  calls, and 529,484/523,550 tokens. All four sessions are distinct, record no
  intervention or retry, and state `formalAttempt: false`. Their durable
  records are under
  `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author-r5/` and
  `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-debug-r5/`.
  A non-author audit of evidence head `d317122` found the r5 sessions and
  receipt chains internally clean but invalidated the harness: copied trees,
  packet manifests, artifact digests, and finalization bound only regular files
  while preserving empty directories. A directory name could therefore carry
  an expected answer without changing a digest. Construction, verification,
  recording, and finalization now reject empty descendant directories, with
  adversarial coverage at every boundary. Clean r6 author and debug pairs then
  passed against exact repaired candidate `c800c98`. The author pair recorded
  14/20 turns, 21/32 tool calls, and 208,252/770,315 provider-reported tokens;
  the debug pair recorded 19/15 turns, 30/19 tool calls, and
  611,607/393,092 tokens. All four sessions are distinct, record no
  intervention or retry, state `formalAttempt: false`, and contain no empty
  artifact directory. Their durable records are under
  `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-author-r6/` and
  `docs/evaluation/runs/rehearsal/2026-08-23-claude-opus-5-debug-r6/`.
- **CI uses `actions/checkout@v7` (#11):** PR and post-merge verification passed
  without the Node 20 compatibility annotation.
- **The GPT Pro architecture checkpoint is owner-disposed (#25):** review of
  clean `main` at `feacad0` found the project on target, endorsed SW-D's scope,
  and filed #21–#24. This was architecture fuzzing, not a formal Gate K run.
- **SW-D is merged (#14/#27):** construction IR advances to
  `estate.world_ir.construction@2` with typed transitions and one phased causal
  edge. The compiler emits `estate.projection.simulation@1`, rejects invalid
  references and cycles, and no longer claims unimplemented projection
  artifacts. `estate-sim` initializes projected machines and atomically prepares
  local-then-causal state changes without seeing source or IR. GPT-5.6 Luna max
  reviewed and reran exact PR head `ce90aa5`; PR CI run `32517611393` and
  post-merge run `32518205450` passed. Merge commit `5f5e730` is clean on main.
- **Issue #21 closed in SW-D's first isolated commit (`a3be521`):** canonical
  object fields, stable keyed arrays, package members/manifest rows,
  machine/claim identities, transitions, and interactions fail closed instead
  of retaining a final duplicate.
- **SW-E is merged (#28/#29):** construction IR advances to
  `estate.world_ir.construction@3` with explicit movement composition,
  coherence, connectivity, and resolver subjects. Simulation advances to `@2`,
  navigation begins at `@1`, and both receive one byte-identical typed resolver
  plan. `estate-sim` evaluates typed
  claim activation after complete local/causal settlement and exposes immutable
  before/after facts. The exact fixture proves two initial gate blockers, ward
  survival after opening or destruction, base cost `1` after unsealing, and
  water cost `3`. GPT-5.6 Luna max found and verified fixes for stale projection
  documentation and invalid-connectivity validation, then reran exact head
  `6dda1d0` green. PR #29 and post-merge CI run `32521857686` passed; merge
  commit `dacfaef` is clean on main. Issue #28 is closed.
- **Contract revision 5 is merged (#22/#30):** decision 0006 replaces the
  unmanifested package `receipts/` subtree with canonical hashed
  `compiler-receipts.json`; runtime causal receipts remain only in run outputs.
  It also defines same-filesystem staged publication and exact filesystem and
  manifest verification. The first GPT-5.6 Luna max pass on
  `c70b5bf` found a trailing-separator root-symlink bypass and an unstated
  path-based reader race boundary. Both are repaired: roots are lexically
  normalized before entry checks, the regression test covers both spellings,
  and revision 5 now explicitly requires a caller-owned quiescent package tree.
  The replacement Luna rerun and PR CI passed exact head `5f65978`; post-merge
  CI run `32525028043` passed merge commit `0eb50b7`. Issue #22 is closed.
- **Typed forensic provenance is merged (#24/#34):** construction IR advances
  to `nomos.world_ir.construction@2`; its exact fixture golden is
  `1a977d4f5f5bcbb11e1ae701e6cc1f3d06688707ee98c04a143d10454aeb126a`,
  while Nomos construction-v1 remains preserved as historical evidence. Fact
  identity, resolved values, projection consumers, derivation producers/passes,
  and causal inputs are typed. The IR rejects dangling roots and edges, values
  that contradict actual entity records, unknown or fact-incompatible passes,
  non-canonical owners, empty/duplicate derivations, and missing typed inputs.
  Canonical ordering no longer depends on human-readable `Display` output.
  Luna max rejected exact heads `9061f68` and `1426451` with real semantic
  findings; after repair it reviewed and reran final head `7ea49d6` green on
  Rust 1.98.0. PR CI run `32530844878` passed, PR #34 merged as `1f04fce`, issue
  #24 closed, and post-merge CI run `32531098740` passed. Structured canonical
  data and readable explanations are separate outputs. At that slice boundary
  `nomos explain-*` remained later work; SW-N now consumes the typed record.
- **SW-F is merged (#35/#38):** construction IR advances to
  `nomos.world_ir.construction@3` and simulation to
  `nomos.projection.simulation@3`; persistence and diagnostics begin at `@1`.
  The three light consumers receive one byte-identical compiler-owned union
  plan. Runtime commands resolve light after complete local/causal settlement,
  commit a new immutable `nomos.runtime_state@1` snapshot, hash its exact
  canonical envelope with SHA-256, and emit typed `nomos.causal_receipt@1`
  evidence. Rejections emit no commit evidence. GPT-5.6 Luna max found no
  semantic or test defects and reran exact head `e1b20845` green with a clean
  tree before and after. PR CI run `32533855605` passed; PR #38 merged as
  `b5ea3b2`, issue #35 closed, and post-merge CI run `32535564970` passed.
- **SW-G is merged (#40/#42):** stable `nomos.world_ir@1` is a distinct
  authoritative artifact promoted from preserved construction evidence. All
  four public projection compilers consume stable IR. The compiler assembles
  the exact seven semantic members, `nomos-core` alone emits `manifest.json`,
  and the semantic opener verifies schemas, ownership, compiler receipts,
  initialization material, and cross-projection agreement. Reopened package
  bytes reproduce the exact initial `nomos.runtime_state@1` snapshot without
  source or construction IR. Stable-IR fixture digest
  `555017cf5e13a33b4bb5b18bae14b7577fd1fc38abf89b1f6f475874600fa493`
  and complete-package manifest digest
  `f1af0cc92ea44fd09ba93815bb99cc6c24517b56888f39be33a9d47b1299bab7`
  are frozen. The implementation head is `f200b4e`; its author proof and
  focused release SW-G suite passed. GPT-5.6 Luna max independently audited and
  reran the exact head with no findings and a clean tree before and after. PR
  CI run `32537565881` passed; PR #42 merged as `0863750`, issue #40 closed,
  and post-merge CI run `32537995524` passed.
- **SW-G semantic open is repaired (#54/#55):** package opening now performs
  complete strict `StableWorldIr` reconstruction, re-derives compiler-owned
  primitive/resolver/movement/relation/provenance invariants, regenerates all
  four typed projections, and returns one `OpenedCompiledWorld`. The partial
  initialization decoder and alternate runtime path are deleted. Refreshed
  receipt-and-manifest mutation tests fail closed across nested commands,
  handlers, claims, capabilities, provenance, ordering, and IR/projection
  disagreement. Exact repair head `d43150a736cd91c7307da22c16992c720e8827e6`
  passed PR CI and a clean non-author rerun; PR #55 merged as `15661d2`, and
  post-merge CI run `32550461102` passed.
- **SW-H is merged (#52/#53):** the `nomos` binary now
  exposes exact dependency-free `validate`, `compile`, and `inspect` command
  grammars. Non-help results are one canonical JSON value; source, argv,
  package, and host failures use the fixed four exit classes. Compilation uses
  the existing staged immutable package boundary. Inspection opens and
  semantically validates the package as an `OpenedCompiledWorld`; its
  compiler-owned report prints the three primitives' capabilities, independent
  machines, claims, and source mappings without reading source or exposing
  `nomos-schema` to the CLI.
  The end-to-end subprocess suite proves all four exits, determinism,
  no-artifact validation, tamper/extra-entry refusal, symlink policy, staging
  cleanup, and existing-output preservation. Exact head `8288e71` passed the
  complete author proof, PR CI run `32550761498`, and a clean non-author rerun
  with no findings. PR #53 merged as `54e5dd2`, issue #52 closed, and post-merge
  CI run `32551021136` passed.
- **SW-I is merged (#56/#57):** strict typed decoders now
  reconstruct `nomos.runtime_state@1` and every nested value in
  `nomos.causal_receipt@1`. `nomos.persisted_runtime_state@1` binds the unchanged
  inner state hash to exact simulation semantics; `nomos.command_script@1`
  resolves requests through entity-owned external machines; and the distinct
  `nomos.command_log@1`, `nomos.causal_receipt_sequence@1`,
  `nomos.state_hash_sequence@1`, and `nomos.run_result@1` types enforce receipt
  digests, contiguous ordinals and ticks, row-by-row state hashes, exact
  non-result artifact coverage, status/diagnostic consistency, and typed
  artifact hashes. A first non-author audit rejected head `943828c` because
  three artifact digests were caller-supplied rather than verified, receipt
  ticks were not anchored to the persisted input tick, and current-state
  validation omitted the plan's namespace-ownership-set check. Repair commit
  `1d5da43` closes all three findings: `RunResult` now derives and revalidates
  all five hashes from typed artifacts, receipt sequences anchor to the actual
  initial tick, and state validation checks ownership. Refreshed-integrity
  mutations cover every artifact binding. A second audit of `845cc1f` confirmed
  the implementation repairs and all four proof commands but rejected the test
  evidence because it showed only tick-zero success. The following repair adds
  a successful command/evidence chain beginning from the fixture's persisted
  tick-5 state, plus a wrong-zero-anchor rejection. The four pre-SW-I receipt
  digests and state hashes were compared against untouched checkpoint `94f60eb`
  and remain frozen as regression goldens. Final head `e890e69` passed the full
  author proof, PR CI run `32554756895`, and a clean GPT-5.6 Luna max non-author
  exact-head audit/rerun. PR #57 merged as `3d4a19a`, issue #56 closed, and
  post-merge CI run `32555728168` passed. SW-I added no runtime CLI command or
  run-directory publisher.
- **SW-J is merged and green (#58):** `nomos run` and `nomos command` publish and
  strictly reopen the exact six-file run bundle through verified staging and one
  rename. Opening re-executes committed requests and requires byte-identical
  state, log, receipt, and hash evidence; output overlap cannot mutate a package
  or verified input-state run bundle, including through symlinked ancestors.
  Final head `ba2061d` passed the full author proof, PR CI run `32559105492`, and
  a finding-free GPT-5.6 Luna max non-author exact-head audit/rerun. PR #59
  merged as `593bca2`, issue #58 closed, post-merge CI run `32559562200` passed,
  and the feature/review worktrees and branches were removed.
- **SW-K is merged and green (#60/#61):** strict canonical
  `nomos.replay_log@1`, binds it to the exact package, simulation semantics,
  package-derived initial state, committed command evidence, and final state,
  and exposes `nomos replay`. The checked-in `fixtures/gaol.replay` derives from
  the five-command accepted execution. Replay re-resolves and commits every
  request, requires the regenerated command log and final state hash to match,
  then publishes through the unchanged SW-J run-bundle boundary. Current tests
  cover exact fixture derivation, strict decoding, deterministic byte equality
  with an ordinary run, wrong input identities, semantically false expected
  evidence, immutable output collisions/overlap, and all CLI failure classes.
  Final head `6ce23e0` passed the full author proof, PR CI run `32560201236`,
  and a clean GPT-5.6 Luna max non-author exact-head audit/rerun. PR #61 merged
  as `c0e77a6`, issue #60 closed, post-merge CI run `32560842827` passed, and
  the feature/review worktrees and branches were removed.
- **SW-M is merged and green (#63/#64):** the accepted implementation
  preserves the exact accepted stable-v1 package, advances new compilation to
  tagged stable `nomos.world_ir@2`, and normalizes active runtime and persisted
  state at v2.
  `nomos migrate <v1-world/> --to 2 --out <v2-world/>` strictly validates all
  legacy package meaning, regenerates every active artifact, and publishes
  immutably. Focused tests prove deterministic output, direct-v1 refusal,
  strict v1/v2 mutation rejection, and identical normalized five-command state,
  command, receipt, movement, light, projection-delta, and hash-chain evidence.
  Final head `946b98b` passed the full author proof, PR CI run `32564268648`,
  and a clean GPT-5.6 Luna max non-author exact-head audit/rerun after two prior
  audits found and drove repairs to exported path-alias and typed-decoder
  boundaries. PR #64 merged as `0b9d948`, issue #63 closed, post-merge CI run
  `32573036770` passed, and the feature/review worktrees and branches were
  removed.
- **Contract revision 7 is merged and green (#62/#66):** decision 0009 requires
  `explain-transition` to receive `--world <world/>`, strictly verify that world,
  and strictly open and re-execute the run against it before explaining a
  receipt. The tick-7 brazier example uses separate seven-command evidence, so
  the accepted five-command command/replay fixture remains unchanged. This
  repair adds no run member, package locator, persisted schema, or weaker
  standalone forensic path. Exact head `98e71d2` passed author proof, PR CI run
  `32585274094`, and a finding-free GPT-5.6 Luna high non-author rerun after a
  superseded audit found three stale references. PR #66 merged as `3d67c4f`,
  issue #62 closed, post-merge CI run `32585743441` passed, and the feature and
  review worktrees and branches were removed.
- **SW-N is merged and green (#65/#67):** `nomos explain-entity` renders linked
  source, primitive expansion, independent machines, claim templates, active
  initial claims, effective facts, typed ownership/derivation records,
  consumers, and schema identities from a strictly opened world.
  `nomos explain-transition`
  strictly opens that world, then strictly opens and re-executes the unchanged
  six-file run before selecting a tick and rendering the unresolved request,
  resolved command, causes, ordered steps, claim changes, complete effective
  facts, projection deltas, source mapping, and state hash. The primary
  five-command evidence proves `north_gate` at tick 4;
  `fixtures/gaol-seven.commands` separately proves `brazier_02` at tick 7.
  Focused subprocess tests freeze exact output hashes and rejection behavior.
  No schema, package/run member, package locator, dependency, or runtime path is
  added. Implementation head `f6a7eb3bd9222459ccfb5d37f20ddd5fd522a993`
  passed the exact-head author proof and PR CI run `32586792897`. A fresh
  GPT-5.6 Luna high non-author audit reran all four proof commands on that exact
  head with a clean detached tree and no findings; its durable receipt is
  [PR #67 comment 5381611726](https://github.com/ConaryLabs/nomos/pull/67#issuecomment-5381611726).
  PR #67 merged as
  `05ba8e1007fe507288bae0c7f4019ba7d750f0f8`, issue #65 closed, and post-merge
  CI run `32587114490` passed.
- **The post-SW-N implementation candidate is frozen (#68/#74):** the accepted
  semantic implementation boundary is merge commit
  `eb86f25f5084a5da83cdd4f26e42e68089367a11`. README and handoff status,
  candidate invalidation rules, the no-new-feature boundary, and the Gate K
  closure order are explicit. PR #74 and post-merge CI run `32602527283`
  passed, issue #68 closed, and its feature branch was removed. This is an
  implementation-complete boundary, not an RC tag or Gate K pass.
- **Mechanical evidence closure is merged (#69/#75):** the predeclared method
  lives in `docs/evaluation/GATE_K_EVIDENCE_PLAN.md`; the dedicated workflow
  runs ten public compile/run/replay processes on native Linux x86_64 debug,
  x86_64 release, and aarch64 release lanes, then fails on any within-lane or
  cross-target byte difference. It also records clean release build time,
  sampled peak/final target disk, process RSS, and three warmup plus twenty
  measured validate/command/replay samples with raw data and distribution
  summaries. `docs/evaluation/SCHEMA_OWNERSHIP.md` records the finding-free
  source review of the exact twenty canonical identities and explicitly
  dispositions the shared compiler receipt profiles. The final exact-head
  workflow and non-author disposition are recorded externally on PR #75
  because committing their receipt would create a different head. PR #75
  merged as `8c32286dc779b76ce8e30f3b1b7817a551f41ba9`; no formal cold-agent
  attempt is part of this slice.

## What is next

Commit the sixth-audit repair, complete its exact-head non-author rerun, and
finish CI for #79. The externally assembled immutable #71 and #72 structured
results still have SHA-256
`e6990dacde903f527d1cb46784a54d938a7e130f1193e51bb830a4a2284f07dc` and
`f09c9214329f7f8bd7d4d4b31476a0f24c825add2f5bb434b7bf780f64d8089c`.
After #79 merges, attach/finalize those owner-disposed failures and merge their
evidence PRs #77 and #78. Issue #73 then maps all nineteen acceptance criteria
and records the final Gate K disposition. No formal retry or new semantic
feature slice is authorized.

`gate-k-rc1` remains the exact historical subject of the completed formal
attempts. Because #79 changes evaluation-harness code after the tag, it is
invalid for any later exact-head evidence. A future formal launch would require
a new candidate, combined-head proof, and explicit owner authorization; issue
#79 itself does not authorize one.

The old `agy` Gemini route remains falsified evidence. Issue #49 qualifies Pi as
the replacement transport without spending a formal attempt or changing the
roster. Neither the historical `agy` result nor the Pi qualification probe is
formal Gate K evidence.

## How to prove the current branch

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
cargo run --locked --bin nomos -- --help
cargo run --locked --bin nomos -- validate fixtures/gaol.nomos
cargo run --locked --bin nomos -- compile fixtures/gaol.nomos --out target/tmp/gate-k-proof/gaol.world
cargo run --locked --bin nomos -- migrate fixtures/gaol-v1.world --to 2 --out target/tmp/gate-k-proof/gaol-migrated.world
cargo run --locked --bin nomos -- inspect target/tmp/gate-k-proof/gaol-migrated.world
cargo run --locked --bin nomos -- run target/tmp/gate-k-proof/gaol.world --commands fixtures/gaol.commands --out target/tmp/gate-k-proof/gaol.run
cargo run --locked --bin nomos -- run target/tmp/gate-k-proof/gaol.world --commands fixtures/gaol-seven.commands --out target/tmp/gate-k-proof/gaol-seven.run
cargo run --locked --bin nomos -- explain-entity target/tmp/gate-k-proof/gaol.world north_gate
cargo run --locked --bin nomos -- explain-entity target/tmp/gate-k-proof/gaol.world flooded_section
cargo run --locked --bin nomos -- explain-entity target/tmp/gate-k-proof/gaol.world brazier_02
cargo run --locked --bin nomos -- explain-transition target/tmp/gate-k-proof/gaol.run north_gate --tick 4 --world target/tmp/gate-k-proof/gaol.world
cargo run --locked --bin nomos -- explain-transition target/tmp/gate-k-proof/gaol-seven.run brazier_02 --tick 7 --world target/tmp/gate-k-proof/gaol.world
cargo run --locked --bin nomos -- command target/tmp/gate-k-proof/gaol.world --state target/tmp/gate-k-proof/gaol.run/final-state.json "close north_gate" --out target/tmp/gate-k-proof/after-close.run
cargo run --locked --bin nomos -- replay target/tmp/gate-k-proof/gaol.world --log fixtures/gaol.replay --out target/tmp/gate-k-proof/replay.run
docs/evaluation/test-agy-print-preflight.sh
docs/evaluation/test-agy-formal-boundary-preflight.sh
docs/evaluation/test-pi-cold-agent-preflight.sh
docs/evaluation/test-gate-k-eval-tooling.sh
```

The authenticated formal-boundary command is an expected negative proof on
1.1.17: `docs/evaluation/agy-formal-boundary-preflight.sh` must exit `1` and
print `AGY_FORMAL_BOUNDARY BLOCKED`. A zero exit would require inspection and
owner disposition before any formal launch.

The issue #54 repair author proof, PR CI, non-author exact-head rerun, and
post-merge CI passed as recorded above and externally in PR #55. SW-H's
refreshed author proof, PR CI, non-author rerun, and post-merge CI passed as
recorded above and externally in PR #53. SW-G's original
author proof, Luna max exact-head rerun, PR CI, and post-merge CI also passed all
four commands; the focused release SW-G test passed. SW-F, SW-D, and
issue #24 have the same author/non-author/CI chain as recorded above. Earlier
SW-C, revision-4, and Rust-1.98 maintenance reruns also passed.
Issue #69 closes the final exact-head disposition for Linux aarch64 release,
ten runs per target, measured Gate K budgets, and the final explicit schema-
ownership source review at `gate-k-rc1`. The formal cold-agent sessions are now
performed, but neither criterion 17 nor 18 passes.

## Remaining evidence points

Issues #69 and #70 supply the merged mechanical evidence, evaluation harness,
and non-formal rehearsals behind `gate-k-rc1`. Draft PR #77 preserves the
Gemini author and DeepSeek checker; draft PR #78 preserves the DeepSeek debugger
and Gemini checker. The semantic authoring and diagnosis were correct, but
recorded outside-workspace requests make both formal attempts fail under the
frozen rubric. Issue #79 is the only active tooling repair before #73.
