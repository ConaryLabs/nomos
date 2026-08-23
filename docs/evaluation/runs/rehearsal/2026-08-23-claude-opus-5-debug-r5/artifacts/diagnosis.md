# Diagnosis — `input/failing.commands` → `input/failing.run`

Rehearsal shape: cold-debug (non-formal). Candidate commit `c1b9f355fa32f8ba749b62aa8d15bd05e9c62808`,
binary `bin/nomos` (sha256 `4af70acc…51fd1`, matches `plan.json` and `packet-manifest.json`).

## 1. Terminal symptom

`input/failing.run/result.json`

```
"status": "rejected", "committed_command_count": 1,
"rejection_diagnostic": {"code": "EK0804"},
"first_state_hash":  fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42
"final_state_hash":  b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b
```

`input/forensics/failure.stdout.json` carries the human wording of the same stable code and
`input/forensics/failure.exit.txt` is `1` (documented semantic-rejection exit; `reference/explanations.md`).

```
EK0804  `north_gate.access.unlock` is illegal while the machine is `closed`
```

## 2. Actual semantic cause

`input/failing.commands` contains **the same request twice**:

```
1  schema nomos.command_script@1
2  unlock north_gate with credential/gaoler_key
3  unlock north_gate with credential/gaoler_key   <-- duplicate of line 2
4  open north_gate
5  unseal north_gate
6  ignite north_gate
7  extinguish brazier_02
```

`north_gate.access` is a three-state machine whose **only** `unlock` transition is guarded on the
source state `locked` (`input/world/simulation.json`, machine `north_gate.access`):

```json
{"action":"unlock","requirement":{"credential":"credential/gaoler_key","kind":"credential"},
 "source":"locked","target":"closed"}
```

and the machine exposes no action that returns `closed`/`open` to `locked` (`close` is
`open→closed`, `open` is `closed→open`; the same three transitions are shown by
`nomos explain-entity input/world north_gate`).

Therefore `unlock` is **not idempotent** in this world. Request ordinal 0 legally consumed the
only `locked → closed` edge; the duplicate at ordinal 1 was then applied to a machine already in
`closed`, for which no `unlock` transition exists, and the run stopped there.

Evidence chain that this is exactly what happened:

* `input/failing.run/command-log.json` holds **one** row (ordinal 0), request
  `{"action":"unlock","argument":{"kind":"catalog_value","value":"credential/gaoler_key"},"entity":"north_gate"}`,
  resolved to namespace command `north_gate.access.unlock`,
  input hash `fa7247d1…` → result hash `b8eac772…`.
* `input/failing.run/causal-receipts.json` (tick 1) records the single transition
  `north_gate.access: locked → closed`, phase `local`, cause `{"kind":"command","action":"unlock"}`.
* `input/failing.run/initial-state.json` has `north_gate.access = locked`;
  `input/failing.run/final-state.json` has `north_gate.access = closed` at tick 1 — i.e. the
  precondition the second `unlock` needed was consumed by the first one.
* `input/failing.run/state-hashes.json` has exactly two rows (ordinals 0 and 1), so nothing after
  the duplicate ever executed. `open`, `unseal`, `ignite`, `extinguish` were never even resolved
  (`reference/runtime.md`: `execute_requests` "stops at the first rejection"; a rejected bundle
  "intentionally binds only the committed prefix and the terminal code … it does not persist the
  rejected request").
* `input/forensics/north-gate-tick-1.json` (`explain-transition`, package digest `fd2dca0e…`,
  run-result digest `0989bf6a…`) shows the *committed* tick‑1 step and unchanged effective facts —
  `north_gate` still `blocked` by `north_gate.portal#blocks_ground` and `north_gate.ward#blocks_ground`,
  because unlocking only reaches `closed`, not `open`.

The prefix commit is correct, documented behaviour, not a second defect.

## 3. Exact reproduction (fault is deterministic and content-owned)

```
$ ./bin/nomos run input/world --commands input/failing.commands --out <scratch>/repro.run   # exit 1
```

produced artifacts **byte-identical** to all six files of `input/failing.run`
(`cmp` on `causal-receipts.json`, `command-log.json`, `final-state.json`, `initial-state.json`,
`result.json`, `state-hashes.json`: identical), and stdout equal to
`input/forensics/failure.stdout.json` modulo the output path. Same `result_digest`
`0989bf6abcafe8c3e7c00a824a54a6c4b5f5ebafcb856215584748ba5713e988`.

## 4. Discriminating probes (`nomos command`, single request, verified world)

| # | input state | request | outcome |
|---|---|---|---|
| 1 | `final-state.json` (access = `closed`) | `unlock north_gate with credential/gaoler_key` | rejected **EK0804**, 0 commits — the failing code reproduced from *state alone* |
| 2 | `initial-state.json` (access = `locked`) | `unlock north_gate with credential/wrong_key` | rejected **EK0805** "command argument does not match the compiled input requirement" |
| 3 | `initial-state.json` | `unlock north_gate with credential/gaoler_key` | **completed**, 1 commit, → `b8eac772…` |
| 4 | `initial-state.json` | `lock north_gate` | rejected **EK0802** "entity `north_gate` exposes no external command `lock`" |

EK0804 is the *state-guard* code and is a pure function of (machine state = `closed`, action =
`unlock`); it is distinct from the credential code (EK0805) and the vocabulary code (EK0802).

## 5. Alternatives considered and excluded

1. **Wrong or missing credential on the `unlock` line.** Excluded: probe 2 shows a credential
   mismatch fails with **EK0805**, not EK0804; and probe 3 shows the *identical* line text succeeds
   from the initial state. The failing run's own row 0 already resolved and accepted
   `credential/gaoler_key` (`command-log.json`, `causal-receipts.json` requirement match).
2. **Malformed script: bad `schema` header, bad grammar, unknown action, or wrong entity.**
   Excluded: a header/grammar fault fails before any commit (`EK0001`/parse class,
   `reference/explanations.md`), an unknown external command yields **EK0802** (probe 4), and an
   absent entity yields the selection codes; here one command *committed* and the diagnostic names
   a resolved namespace command `north_gate.access.unlock`.
3. **Command ordering — e.g. `open`/`unseal`/`ignite` issued too early, or the `ignite` causal edge
   (`north_gate.combustion` → `apply_damage` → `north_gate.integrity`) invalidating a later step.**
   Excluded: the run stopped at ordinal 1; `state-hashes.json` has no ordinal ≥ 2 and no receipt for
   ticks 2–5 exists, so those commands never ran. The repaired run (§6) executes all four of them
   with no diagnostic, so their order is legal.
4. **A mutated / mismatched world package or wrong semantics binding.** Excluded: every one of the
   seven `manifest.json` members re-hashes to its recorded sha256; the package digest
   `fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440` is identical in
   `input/world/manifest.json`, the authoring-time `input/forensics/compile.stdout.json`
   (`manifest_digest`), `input/failing.run/result.json` (`input_package_digest`) and
   `input/forensics/north-gate-tick-1.json`; `runtime_semantics_digest` `0cdb454b…` equals
   sha256(`simulation.json`); `nomos inspect input/world` exits 0. A package/semantics mismatch
   would have failed closed at open time (`EK0810`/`EK0823` class), not with EK0804 after a commit.
5. **A runtime defect: non-atomic commit, lost commands, or nondeterminism.** Excluded: the run is
   bit-reproducible (§3); `reference/runtime.md` states that a rejected bundle binding only the
   committed prefix plus the terminal code is the required behaviour (issue #58); and
   `nomos explain-transition input/failing.run north_gate --tick 1 --world input/world`
   — which per decision 0009 strictly reopens the bundle and **re-executes every committed row**,
   requiring byte-identical states, logs, receipts and hashes — exits 0 and returns output equal to
   the packaged `input/forensics/north-gate-tick-1.json`. The evidence is internally consistent.
6. **World authoring defect (the `unlock` guard should have allowed `closed`).** Excluded as the
   owning boundary: the machine is coherent (`unlock` is a one-way `locked → closed` edge with no
   re-lock action), and removing the duplicate line reproduces exactly the documented accepted
   fixture behaviour — five commits, `north_gate` explained at tick 4, `brazier_02` at tick 5
   (`reference/explanations.md`, `reference/runtime.md` "five requests"). Nothing in the world needs
   to change, and the brief forbids editing the compiled world or binary in any case.

## 6. Owning boundary and repair

**Content-level (command script).** The defect is one duplicated request line in
`input/failing.commands`; the world package, the compiler artifacts and the runtime behaved to
specification. Repair = delete line 3 (the second `unlock north_gate with credential/gaoler_key`).

`output/repaired.commands` (sha256 `31c84464a7e2d0d03397a92b32c76a21eccf39ecb94c0ca3605cf21fd4eb3719`,
149 bytes) is exactly `input/failing.commands` minus that one line (`diff` = `3d2`).

Verified into `output/repaired.run` — see `output/verification.md`:

```
status completed, exit 0, committed_command_count 5, rejection_diagnostic null
first_state_hash fa7247d1…  (identical to the failing run)
ordinal 1 hash   b8eac772…  (identical to the failing run's committed prefix)
final_state_hash 3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc
```

`initial-state.json` is byte-identical to the failing run's, and command-log row 0 and the tick‑1
causal receipt are identical objects — proving the repair changes nothing except the removed
duplicate. Final machines: `access=open`, `ward=unsealed`, `combustion=burning`,
`integrity=destroyed` (via the tick‑4 causal fire-damage edge), `brazier_02.emission=extinguished`;
`north_gate` movement becomes traversable cost 1 at tick 4.
