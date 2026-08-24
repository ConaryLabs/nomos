# Diagnosis: why `input/failing.commands` produced `input/failing.run`

## Verdict

**Root semantic cause: the command script contains a duplicate, non-idempotent
`unlock` request. The `north_gate.access` machine is a one-shot lock; after the
first `unlock` commits, the machine is in state `closed`, and the compiled
machine has **no transition** for `unlock` out of `closed`. The second
identical request is therefore illegal, the runtime rejects it at commit time
with `EK0804`, and the run bundle publishes only the committed prefix (one
command, tick 1) plus the terminal rejection code.**

The terminal code `EK0804` is the *symptom*; the semantic cause is the state
machine's transition model (`unlock: locked -> closed` only) combined with the
script's duplicate line. The owning repair is **content-level** (command
script): delete the duplicate request line. The verified world and the `nomos`
binary require no change.

---

## 1. What the run shows

`input/failing.commands` is a 6-request command script (plus the schema header):

```text
schema nomos.command_script@1
unlock north_gate with credential/gaoler_key      (line 2)
unlock north_gate with credential/gaoler_key      (line 3, DUPLICATE)
open north_gate
unseal north_gate
ignite north_gate
extinguish brazier_02
```

`input/failing.run` is a **rejected** run bundle with:

| Evidence field | Value | Location |
|---|---|---|
| `status` | `"rejected"` | `failing.run/result.json` |
| `committed_command_count` | `1` | `result.json`, `forensics/failure.stdout.json` |
| `rejection_diagnostic.code` | `EK0804` | `result.json` |
| message | `` `north_gate.access.unlock` is illegal while the machine is `closed` `` | `forensics/failure.stdout.json` |
| `repairs` | `[]` | `forensics/failure.stdout.json` |
| process exit | `1` | `forensics/failure.exit.txt` |
| stderr | empty (0 bytes) | `forensics/failure.stderr.txt` |
| `result_digest` | `0989bf6abcafe8c3e7c00a824a54a6c4b5f5ebafcb856215584748ba5713e988` | `result.json` |

So the boundary that failed is **runtime semantic rejection after commit 1**,
not CLI grammar, not I/O, and not package verification (all of the latter have
documented, distinct behavior: runtime.md states CLI input / environment /
publication failure "returns no partial output" — here a full rejected bundle
was published, the documented issue-#58 shape for a *runtime rejection*).

## 2. The committed prefix (what did succeed)

`failing.run/command-log.json` has exactly one committed row (ordinal 0):

- `request`: `unlock north_gate with credential/gaoler_key`
- `resolved_command`: `{action: unlock, argument: {credential: credential/gaoler_key, kind: credential}, namespace: north_gate.access}`
- `input_state_hash`: `fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42`
- `resulting_state_hash`: `b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b`

`failing.run/causal-receipts.json` records the effect of that commit:

```json
"transitions":[{"cause":{"action":"unlock","kind":"command"},
                "from":"locked","namespace":"north_gate.access",
                "phase":"local","to":"closed"}]
```

`failing.run/final-state.json` confirms the resulting machine state at tick 1:

```json
{"namespace":"north_gate.access","state":"closed"}
```

(`initial-state.json` had `"state":"locked"` at tick 0.)

`forensics/north-gate-tick-1.json` (the provided `explain-transition` over the
same bundle) independently renders the same receipt: `unlock` `locked -> closed`,
tick 1, `run_result_digest` `0989bf...`.

## 3. The semantic cause, exactly

The compiled transition table for the `north_gate.access` machine is in
`input/world/world-ir.json` and `input/world/simulation.json`:

```json
"machines":[{"initial":"locked",
             "namespace":"north_gate.access",
             "states":["locked","closed","open"],
             "transitions":[
               {"effect":{"kind":"set_state","state":"closed"},"source":"open",
                "trigger":{"input":{"kind":"none"},"kind":"command","name":"close"}},
               {"effect":{"kind":"set_state","state":"open"},"source":"closed",
                "trigger":{"input":{"kind":"none"},"kind":"command","name":"open"}},
               {"effect":{"kind":"set_state","state":"closed"},"source":"locked",
                "trigger":{"input":{"kind":"resolved_entity_credential"},
                           "kind":"command","name":"unlock"}}]}]
```

`simulation.json`'s command table agrees (it is the executable shape):

```json
{"action":"unlock","requirement":{"credential":"credential/gaoler_key","kind":"credential"},
 "source":"locked","target":"closed"}
```

The machine is a **one-shot lock**:

- `unlock` is legal **only** from `locked` → `closed` (consumes the lock step);
- `open` is legal from `closed` → `open`;
- `close` is legal from `open` → `closed`;
- no `unlock` transition exists from `closed` or from `open`.

After request 1, `north_gate.access` is `closed`. Request 2 is `unlock` again;
the runtime looks up an `unlock` transition applicable in `closed`, finds none,
and rejects with `EK0804` ("illegal while the machine is `closed`"). No command
after the rejection is attempted; the bundle intentionally binds only the
committed prefix and terminal code (issue #58 semantics, runtime.md).
The rejected request itself is deliberately not persisted — the evidence of
*which* line failed is the diagnostic message plus the fact that exactly the
prefix through the last legal transition was committed.

The script-level cause: line 3 duplicates line 2. The document canonical
fixture is a **five-request** script (`fixtures/gaol.commands` "five requests",
runtime.md; "proves `north_gate` at tick 4", explanations.md). `input/failing.commands`
has **six** request lines, i.e. exactly one extra request line — the duplicate
`unlock`. The remaining order (`unlock`, `open`, `unseal`, `ignite`,
`extinguish`) is the legal five-request sequence, so the single spurious line is
the duplicate at line 3 (the second `unlock`).

## 4. Evidence recap (what to cite)

| Claim | Evidence |
|---|---|
| World/package is valid and opens | `input/world/manifest.json` package digest `fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440`; `input/world` and `input/failing.run` carry the same `runtime_semantics_digest` `0cdb454be...`; `nomos inspect input/world` completes; `forensics/compile.stdout.json` status `completed` with the same `manifest_digest` |
| Machine semantics (`unlock` only from `locked`) | `input/world/world-ir.json` `north_gate.access.transitions`; `input/world/simulation.json` machine commands table |
| First commit succeeded, machine became `closed` | `failing.run/command-log.json` row 0; `failing.run/causal-receipts.json`; `failing.run/final-state.json`; `failing.run/state-hashes.json` rows 0–1 |
| Terminal rejection | `failing.run/result.json` (`EK0804`, `status: rejected`, count 1); `forensics/failure.stdout.json`; `forensics/failure.exit.txt` (`1`); empty `failure.stderr.txt` |
| Script has a duplicate line | `input/failing.commands` lines 2–3 are byte-identical; `wc -l` = 7 total (header + 6 requests) vs canonical 5 requests |
| Determinism (not a flaky/transient failure) | Re-running `bin/nomos run input/world --commands input/failing.commands` reproduces the identical `EK0804` bundle, identical `result_digest` `0989bf6...` and identical `final_state_hash` `b8eac7...` (see `output/probe/repro-failing2.*`) |

## 5. Plausible alternatives excluded

### A. Wrong / missing credential (EK0805 class, not EK0804)
Excluded. Evidence:
- The failing script's request 1 uses `credential/gaoler_key` and **committed**:
  `failing.run/command-log.json` row 0 shows the resolved command with the
  credential requirement satisfied; the world IR ownership receipt
  `entity_credential` for `north_gate` resolves to catalog value
  `credential/gaoler_key` (`world-ir.json`).
- Probe: a script beginning `unlock north_gate with credential/nonexistent`
  fails with a **different** code, `EK0805` ("command argument does not match
  the compiled input requirement"), with `committed_command_count: 0`
  (`output/probe/repro-badcred2.stdout.json`).
- The failing bundle shows EK0804 with `committed_command_count: 1`, i.e. the
  credential was accepted once; the rejection names the machine state, not the
  argument.

### B. Grammar / parse / resolution failure (EK0001 or EK05xx/EK06xx classes)
Excluded. Evidence:
- The same `unlock ... with credential/gaoler_key` line parsed and resolved at
  ordinal 0 into the typed command `{action: unlock, argument: {credential: ...},
  namespace: north_gate.access}` (`failing.run/command-log.json`).
- The schema header is present and valid (`schema nomos.command_script@1`,
  line 1). Probe: omitting the argument (`unlock north_gate`) yields `EK0805`
  with 0 commits (`output/probe/repro-badgrammar.stdout.json`), not EK0804.
- The command script schema identity `nomos.command_script@1` is in the packet
  manifest for `input/failing.commands`.

### C. Corrupt / mismatched world, state, or hash chain (EK0810, EK0822–EK0824, integrity failures)
Excluded. Evidence:
- The opener accepted the world and the persisted state; the first commit
  produced a valid hash chain (`state-hashes.json` ordinals 0–1 with
  `fa7247...` → `b8eac7...`), and `result.json` binds all five artifact digests
  (they match the actual files, verified independently).
- `runtime_semantics_digest` matches the compiled `input/world/simulation.json`
  (`0cdb454be...`), and the package digest matches the compile receipt.
- If the package or state were corrupt, the strict opener would reject before
  any commit and publish **no** bundle ("returns no partial output"); a
  published rejected bundle with a committed prefix is exactly the runtime
  rejection shape.
- The same verified world subsequently accepted the repaired five-request
  script to completion (Section 6), which a semantically mismatched world
  cannot do.

### D. Ordering alternative: "move `open` before `unlock`"
Excluded. Evidence: probe `open north_gate` on the initial (locked) machine is
itself rejected `EK0804` — `` `north_gate.access.open` is illegal while the
machine is `locked` `` — with `committed_command_count: 0`
(`output/probe/repro-openfirst.stdout.json`). `open` is legal only from
`closed`, which `unlock` produces. So `unlock` first is required and correct;
the failure is not that order.

### E. World/binary repair boundary ("make `unlock` idempotent", "add an `unlock` transition from `closed`")
Excluded. Evidence: this would require recompiling the world (editing source /
compiled IR), which the brief forbids, and it would contradict the documented
canonical fixture: `fixtures/gaol.commands` is a five-request script whose
`unlock` is a `locked -> closed` one-shot step and which proves `north_gate` at
tick 4 (runtime.md, explanations.md). The duplicate line is the only anomaly;
the five-request form completes without any world modification, so the owning
boundary is the command content, not the world.

## 6. Owning repair and verification

**Repair class: content-level (command script).** One line is spurious.
`output/repaired.commands` is the corrected script: `input/failing.commands` with
the duplicate second `unlock` line removed (diff is exactly `-unlock north_gate
with credential/gaoler_key` once; sha256 `31c84464a7e2d0d03397a92b32c76a21eccf39ecb94c0ca3605cf21fd4eb3719`).

```text
schema nomos.command_script@1
unlock north_gate with credential/gaoler_key
open north_gate
unseal north_gate
ignite north_gate
extinguish brazier_02
```

Verification (see `output/verification.md` for exact commands):

- `nomos run input/world --commands output/repaired.commands
  --out output/verified.run` → `status: "completed"`,
  `committed_command_count: 5`, exit 0, final state hash
  `3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc`.
- `output/verified.run/final-state.json` tick 5:
  `north_gate.access=open`, `north_gate.combustion=burning`,
  `north_gate.integrity=destroyed`, `north_gate.ward=unsealed`,
  `brazier_02.emission=extinguished`.
- Command log ordinals 0–4: unlock, open, unseal, ignite, extinguish.
- `nomos explain-transition output/verified.run north_gate --tick 4
  --world input/world` re-opens and re-executes the bundle and renders the
  tick-4 receipt: local `ignite` `cold -> burning` **and** causal
  `apply_damage` `intact -> destroyed` (the two-step tick-4 `north_gate`
  proof the canonical fixture is documented for), status `completed`.
- `nomos explain-transition output/verified.run brazier_02 --tick 5
  --world input/world` renders `extinguish` `lit -> extinguished`, with
  `brazier_02.emission#emits_light` removed and the light fact delta projected
  to diagnostics, persistence, and simulation.
- All five artifact digests in `output/verified.run/result.json` match the
  actual files by SHA-256, and `initial-state.json` is byte-identical
  (`9d6cf318...`) to the failing run's initial state — the same verified world
  and state that rejected the duplicate now accepts the repaired script.

## 7. Scope statement

No compiled world, source, projection, binary, or run evidence was modified.
All probes and scratch were kept in `/workspace/output`; only
`/workspace/bin/nomos` and ordinary shell inspection inside `/workspace` were
used. `output/repaired.commands` (+ diff), `output/verified.run/`, and this
diagnosis are the deliverables.
