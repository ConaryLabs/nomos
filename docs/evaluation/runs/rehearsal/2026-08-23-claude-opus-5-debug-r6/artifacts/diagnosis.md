# Diagnosis — cold-debug rehearsal (non-formal)

Candidate commit: `c800c98a67f2599b5522a84d42a7549600d53d1f`
World package digest: `fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440`
Subject inputs: `/workspace/input/failing.commands` -> `/workspace/input/failing.run`

## 1. Verdict

The run was rejected because **`/workspace/input/failing.commands` contains the
`unlock north_gate with credential/gaoler_key` request twice (lines 2 and 3),
and the compiled `north_gate.access` machine defines `unlock` only as a
transition out of `locked`.** The first request legally moved
`north_gate.access` from `locked` to `closed` and was committed; the second,
identical request was then evaluated against the *new* state `closed`, where no
`unlock` transition exists, so the runtime rejected the whole script with the
stable state-legality code `EK0804` after committing the one-command prefix.

This is a **command-content defect**, not a world, compiler, runtime, evidence,
CLI, or environment defect. The owning boundary is the command script; the
repair is content-level and is supplied as `output/repaired.commands`.

## 2. Semantic mechanism

Compiled law (`input/world/world-ir.json`, entity `north_gate`, namespace
`north_gate.access`, source `fixtures/gaol.nomos` line 4, bytes 53..162):

| source state | trigger | effect |
|---|---|---|
| `locked` | command `unlock`, input `resolved_entity_credential` | set_state `closed` |
| `closed` | command `open`, input `none` | set_state `open` |
| `open`   | command `close`, input `none` | set_state `closed` |

`unlock` is **not** idempotent and has no `closed -> closed` self-transition, so
the second occurrence of the line has no legal transition to select. The machine
is a three-state lattice `locked | closed | open` with `initial: locked`
(also mirrored in `input/world/simulation.json` semantics digest
`0cdb454be9876dccdc05369e91fb9bdea017973d77f7803d23a921a969a1ef3c`).

`docs/runtime.md` (packet copy `reference/runtime.md`, "Filesystem execution and
run bundles"): `execute_requests` "stops at the first rejection"; "only
successful commits enter the log, the final state is the last committed
snapshot, and `result.json` records the stable rejection code." The published
bundle therefore *correctly* shows a one-command prefix plus a terminal code.

## 3. Forensic evidence

**a. Command script (package member `input/failing.commands`, sha256
`27a3f42e…8fbc8f`, 194 bytes, 7 lines).** Lines 2 and 3 are byte-identical:

```
1 schema nomos.command_script@1
2 unlock north_gate with credential/gaoler_key
3 unlock north_gate with credential/gaoler_key   <-- duplicate, the defect
4 open north_gate
5 unseal north_gate
6 ignite north_gate
7 extinguish brazier_02
```

**b. Terminal diagnostic (`input/forensics/failure.stdout.json`, exit
`input/forensics/failure.exit.txt` = `1`, stderr empty).**

```
{"code":"EK0804","message":"`north_gate.access.unlock` is illegal while the machine is `closed`","repairs":[]}
"committed_command_count":1, "status":"rejected"
```

The message names the *resolved namespace command* `north_gate.access.unlock`
and the *observed machine state* `closed` — i.e. resolution succeeded and the
failure is state legality, at exactly the state the previous command produced.
Exit `1` is documented semantic rejection (`reference/explanations.md`: "Malformed
CLI grammar remains `EK0001` with exit `2`; semantic rejection is exit `1`;
environment/I/O failure remains exit `3`"), and stderr is 0 bytes, so this is not
a CLI, environment, or I/O failure.

**c. Committed prefix and state hashes.**
- `input/failing.run/initial-state.json`: `north_gate.access = locked`, tick 0,
  hash `fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42`.
- `input/failing.run/final-state.json`: `north_gate.access = closed`, tick 1,
  hash `b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b`.
- `input/failing.run/state-hashes.json`: exactly two rows (ordinal 0 = `fa7247…`,
  ordinal 1 = `b8eac7…`), i.e. one committed command.
- `input/failing.run/command-log.json`: exactly one row, ordinal 0, request
  `{action: unlock, argument: {kind: catalog_value, value: credential/gaoler_key},
  entity: north_gate}` resolved to
  `{namespace: north_gate.access, action: unlock, argument: {kind: credential,
  credential: credential/gaoler_key}}`, input hash `fa7247…` -> result hash
  `b8eac7…`.
- `input/failing.run/result.json`: `status: rejected`,
  `committed_command_count: 1`, `rejection_diagnostic.code: EK0804`,
  `input_package_digest fd2dca…710440`, `runtime_semantics_digest 0cdb45…1ef3c`.

The hash chain is intact and consistent; nothing about the evidence is
malformed. The rejected request is (by design, `reference/runtime.md`: "a
rejected bundle … does not persist the rejected request") absent from the log,
which is why the log shows one `unlock` while the script contains two.

**d. Causal receipt / transition explanation.**
`input/failing.run/causal-receipts.json` and
`input/forensics/north-gate-tick-1.json` both record the single committed
transition:

```
{"cause":{"action":"unlock","kind":"command"},"namespace":"north_gate.access",
 "phase":"local","from":"locked","to":"closed"}
```

with `state_hash b8eac7…` and `tick 1`. `effective_facts_before` ==
`effective_facts_after` (north_gate still `blocked` via
`north_gate.portal#blocks_ground` and `north_gate.ward#blocks_ground`), and
`projection_deltas: []` — the commit was legal but semantically "quiet"; the
gate stays blocked because `unlock` only reaches `closed`, not `open`. That
receipt is the proof that the state seen by line 3 was `closed`, exactly the
state named in `EK0804`.

**e. Reproduction (independent check with the packet binary).**
`bin/nomos run input/world/ --commands input/failing.commands --out
output/repro.run` re-produced the failure with **byte-identical** artifacts:
all six files `cmp`-equal to `input/failing.run/*`, same
`result_digest 0989bf6abcafe8c3e7c00a824a54a6c4b5f5ebafcb856215584748ba5713e988`,
same `EK0804`, same `committed_command_count: 1`, exit 1. The failure is
deterministic and fully explained by the packet inputs.

**f. Package integrity (world is not the defect).**
`sha256sum` of all seven `input/world/*` members equals the digests recorded in
`input/world/manifest.json` and `input/world/compiler-receipts.json`;
`manifest.package_digest` = `fd2dca…710440` = the `manifest_digest` in
`input/forensics/compile.stdout.json` = `input_package_digest` in
`result.json`; `simulation.json` sha256 = `0cdb45…1ef3c` =
`runtime_semantics_digest` in `result.json`, `initial-state.json` and
`final-state.json`. `bin/nomos inspect input/world/` exits 0 with
`status: completed`. `compile.stdout.json` shows `status: completed` with no
diagnostics; `input/world/diagnostics.json` carries no error entries. The
compiled world is verified, self-consistent, and the same world the failing run
used.

## 4. Excluded alternatives

**A1 — Wrong or missing credential / authority failure.** *Excluded.* The
resolved command in both the log and the receipt carries
`{kind: credential, credential: credential/gaoler_key}`, i.e. the credential
resolved and the *first* unlock committed with it. The catalog contains exactly
`credential/gaoler_key` (`world-ir.json: catalog_values`) and `north_gate` is
bound to it (`credential: "credential/gaoler_key"`). Direct probe: replaying
`unlock north_gate with credential/wrong_key` against the *initial* state yields
a different stable code, `EK0805` "command argument does not match the compiled
input requirement", never `EK0804`. Credential failure and state-legality
failure are distinct codes; the observed code is the latter.

**A2 — Unknown entity, unknown verb, or a mis-typed later line
(`unseal`/`ignite`/`extinguish` unsupported).** *Excluded.* Those failures have
their own stable codes, proven by direct probe against this world:
`unlock gaol_door …` -> `EK0801` "command entity `gaol_door` does not exist";
`unseal brazier_02` -> `EK0802` "entity `brazier_02` exposes no external command
`unseal`". Neither code appears. Moreover `world-ir.json` defines `unseal`
(`north_gate.ward: sealed -> unsealed`), `ignite` (`north_gate.combustion:
cold -> burning`) and `extinguish` (`brazier_02.emission: lit -> extinguished`),
and the repaired script executes all of them to completion (§5), so lines 4–7
are sound. The defect is confined to line 3.

**A3 — Wrong command ordering (e.g. `open` before `unlock`, or `ignite`
prerequisites unmet).** *Excluded.* The relative order in the script is already
legal: `locked --unlock-> closed --open-> open`, ward `sealed --unseal->
unsealed`, combustion `cold --ignite-> burning`. Rejection happened at ordinal 1,
before any ordering interaction could matter, and the repaired script — same
order, one line deleted — completes all five commands. No reordering was needed
or performed.

**A4 — World/compiler defect (the `access` machine "should" accept `unlock`
while `closed`).** *Excluded, and out of scope for repair.* The three
transitions in `world-ir.json` are internally coherent (`unlock` is the
credentialled entry into the lattice; `open`/`close` are the free pair), the
compiler reports `status: completed` with the `projection_agreement` and
`exact_package_member_set` invariants satisfied, all projections agree, and every
member digest matches. Making `unlock` idempotent would be a *source/world*
change that alters sealed semantics and the package digest; the brief forbids
editing the compiled world, and nothing in the evidence indicates the world is
wrong. The script, not the law, asserted an illegal thing.

**A5 — Runtime commit / publisher / evidence bug (partial commit, lost
commands, broken chain).** *Excluded.* The prefix-commit behaviour is the
documented contract (`reference/runtime.md`: stop at first rejection, publish
only committed rows, record the stable code, do not persist the rejected
request). The bundle is internally consistent: `state-hashes` rows = committed
count + 1; `command-log` row 0 input/result hashes equal `state-hashes` ordinals
0/1; `final-state.json` hash equals the last row; `result.json` artifact digests
equal the on-disk sha256 of all five bound files; `causal-receipts` tick 1
matches. `explain-transition` re-opened and strictly re-executed the run bundle
successfully (exit 0), which per `reference/explanations.md` requires
byte-identical re-execution. Nothing is missing or corrupt.

**A6 — Malformed script header / schema or state-version mismatch.**
*Excluded.* The script header is exactly `schema nomos.command_script@1`
(packet `schemaIdentity` for that file), and header/decoding failures would be
raised before any commit — yet one command committed. The persisted states are
`nomos.persisted_runtime_state@2` and both carry
`runtime_semantics_digest 0cdb45…1ef3c`, identical to this world's
`simulation.json` digest, so there is no cross-semantics or v1/v2 state reuse.

**A7 — Non-determinism, stale state, or a mutated input tree.** *Excluded.* The
re-run reproduced all six artifacts byte-for-byte (§3e); `input/failing.commands`
still hashes to the packet-manifest value `27a3f42e…8fbc8f`; every world member
hashes to its manifest value; the run began from the package-derived initial
state (`first_state_hash fa7247…`, tick 0, all machines at their `initial`
states).

## 5. Repair and its boundary

Owning boundary: **command-script content** (author of `failing.commands`).
Minimal repair: delete the duplicated line 3. Nothing else changes — no
reordering, no substitution, no world or binary edit.

`output/repaired.commands` (149 bytes, sha256
`31c84464a7e2d0d03397a92b32c76a21eccf39ecb94c0ca3605cf21fd4eb3719`) differs from
the input by exactly `3d2` (one deleted line, 45 bytes).

Verified result (`bin/nomos run input/world/ --commands output/repaired.commands
--out output/repaired.run`, exit 0):

- `status: completed`, `committed_command_count: 5`, no diagnostics;
- `first_state_hash fa7247…` (unchanged) and ordinal 1 =
  `b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b`, i.e. the
  previously committed prefix is preserved bit-for-bit;
- final state tick 5, hash
  `3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc`, machines
  `north_gate.access = open`, `north_gate.ward = unsealed`,
  `north_gate.combustion = burning`, `north_gate.integrity = destroyed`,
  `brazier_02.emission = extinguished`;
- tick 4 (`ignite`) shows the intended causal chain: local
  `north_gate.combustion cold -> burning` followed by causal
  `north_gate.integrity intact -> destroyed` via handler `apply_damage`
  (`{kind: damage, channel: fire, amount: 2}`), matching the compiled
  `on_enter burning` interaction;
- `explain-transition output/repaired.run north_gate --tick 1 --world
  input/world/` renders a receipt **identical** to
  `input/forensics/north-gate-tick-1.json` in every field except
  `run_result_digest` — direct proof that the repair changed only the rejected
  duplicate and preserved the committed semantics.

Corroboration that five commands is the intended shape: `reference/explanations.md`
records that the accepted `fixtures/gaol.commands` contains **five** requests and
"prove[s] `north_gate` at tick 4" — exactly the repaired script's length and
exactly the tick at which the repaired run explains `north_gate`'s ignite/damage
transition.

Exact verification commands and their outputs: `output/verification.md`.
