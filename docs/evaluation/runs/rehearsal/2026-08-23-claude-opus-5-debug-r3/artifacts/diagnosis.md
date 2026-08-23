# Diagnosis — cold-debug rehearsal (non-formal; not a Gate K attempt)

Subject: why `/workspace/input/failing.commands` produced `/workspace/input/failing.run`.

---

## 1. Verdict

**The command script is wrong, not the world and not the runtime.**

`input/failing.commands` issues `unlock north_gate with credential/gaoler_key` **twice in a
row** (lines 2 and 3). The first `unlock` is legal and commits, moving the namespace machine
`north_gate.access` from `locked` to `closed`. The second `unlock` is then evaluated against
the *post-commit* state. The compiled machine `north_gate.access` defines exactly one `unlock`
transition, with `source: "locked"`. No transition anywhere in the world re-enters `locked`,
so after tick 1 the guard can never be satisfied again. The runtime therefore commits the
one-command prefix, refuses command ordinal 1, and publishes a `rejected` run bundle.

- **Semantic cause:** a duplicated, non-idempotent, non-repeatable state-machine command —
  `unlock` is a one-shot edge out of an irreversible initial state.
- **Terminal code (symptom, not cause):** `EK0804` —
  `` `north_gate.access.unlock` is illegal while the machine is `closed` ``, exit `1`.
- **Owning boundary:** **command content** (`nomos.command_script@1` authoring). Repairable
  by deleting one line. See `output/repaired.commands` and `output/verification.md`.

---

## 2. Forensic evidence

### 2.1 The script itself

`input/failing.commands` (194 bytes, sha256 `27a3f42e…8fbc8f`, matching `packet-manifest.json`):

```text
schema nomos.command_script@1
unlock north_gate with credential/gaoler_key   <- ordinal 0, commits
unlock north_gate with credential/gaoler_key   <- ordinal 1, REJECTED (duplicate)
open north_gate
unseal north_gate
ignite north_gate
extinguish brazier_02
```

`cat -A` confirms plain `LF` line endings and no trailing whitespace, so this is a genuine
duplicated line and not a quoting or encoding artefact.

### 2.2 Result / diagnostic evidence

`input/failing.run/result.json`:

```json
"committed_command_count": 1,
"status": "rejected",
"rejection_diagnostic": { "code": "EK0804" },
"first_state_hash": "fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42",
"final_state_hash": "b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b",
"input_package_digest": "fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440",
"runtime_semantics_digest": "0cdb454be9876dccdc05369e91fb9bdea017973d77f7803d23a921a969a1ef3c"
```

`input/forensics/failure.stdout.json` supplies the human-readable message that names the
offending namespace **and the state it was in**:

```json
{"code":"EK0804","message":"`north_gate.access.unlock` is illegal while the machine is `closed`","repairs":[]}
```

`input/forensics/failure.exit.txt` = `1` (semantic rejection per `reference/explanations.md`:
"Malformed CLI grammar remains `EK0001` with exit `2`; semantic rejection is exit `1`;
environment/I/O failure remains exit `3`"). `failure.stderr.txt` is empty (0 bytes) — no
environment or I/O fault.

The message is the whole diagnosis in miniature: the machine is `closed`, and `unlock`
requires `locked`.

### 2.3 Command-log / state-hash evidence (exactly one command committed)

`input/failing.run/command-log.json` has a **single** row:

| ordinal | request | resolved_command | in → out state hash |
|---|---|---|---|
| 0 | `unlock north_gate` arg `catalog_value credential/gaoler_key` | `north_gate.access.unlock` arg `credential credential/gaoler_key` | `fa7247d1…` → `b8eac772…` |

`input/failing.run/state-hashes.json` has exactly two snapshots (ordinal 0 = initial,
ordinal 1 = after the single committed command). The absent ordinal-2 row is direct
evidence that ordinal 1 of the script never committed.

### 2.4 Causal-receipt evidence (the state actually changed under the first unlock)

`input/failing.run/causal-receipts.json`, single receipt at `tick: 1`:

```json
"transitions": [{"namespace":"north_gate.access","phase":"local",
                 "from":"locked","to":"closed",
                 "cause":{"kind":"command","action":"unlock"}}]
```

`projection_deltas: []` and identical `effective_facts_before`/`effective_facts_after`
(north_gate still `blocked` by `north_gate.portal#blocks_ground` and
`north_gate.ward#blocks_ground`) — the unlock changed the *access machine* without changing
any movement/light fact. This matters: the run did not stall because nothing happened; it
stalled because something *did* happen and consumed the only `unlock` edge.

### 2.5 State evidence (before vs after)

`initial-state.json` → `north_gate.access: "locked"`, `tick: 0`, hash `fa7247d1…`.
`final-state.json`  → `north_gate.access: "closed"`, `tick: 1`, hash `b8eac772…`.

The second `unlock` was therefore evaluated against `closed`. Exactly as the diagnostic says.

### 2.6 World / package evidence (the machine really has only one unlock edge)

`input/world/simulation.json` → `machines[namespace = "north_gate.access"]`:

```json
"initial": "locked",
"states": ["closed","locked","open"],
"commands": [
  {"action":"close",  "source":"open",   "target":"closed", "requirement":{"kind":"none"}},
  {"action":"open",   "source":"closed", "target":"open",   "requirement":{"kind":"none"}},
  {"action":"unlock", "source":"locked", "target":"closed",
   "requirement":{"kind":"credential","credential":"credential/gaoler_key"}}
],
"handlers": []
```

`nomos inspect input/world/` independently reports the same three transitions from the stable
IR, with `unlock` carrying `trigger.input.kind = "resolved_entity_credential"`.

**Reachability check (computed from `simulation.json`, recorded in
`output/scratch/*` session output):** across *all five* machines, the set of states that are
the `target` of some command or handler is
`{extinguished, closed, open, burning, destroyed, unsealed}`. `locked` is **not** among them.
`locked` is an initial-only, irreversible state. Consequently:

> **No ordering, insertion, or credential change can make a second `unlock` legal in any run
> of this world.** The only content-level repair is deletion.

Package integrity was independently confirmed: every member of `input/world/manifest.json`
re-hashes to its recorded sha256, and `manifest.package_digest`
(`fd2dca0e…710440`) equals both `compile.stdout.json.manifest_digest` and
`result.json.input_package_digest`. `simulation.json`'s sha256 (`0cdb454b…1ef3c`) equals the
`runtime_semantics_digest` in `result.json`, `initial-state.json`, and `final-state.json`.

### 2.7 Explanation evidence

```text
bin/nomos explain-transition input/failing.run/ north_gate --tick 1 --world input/world/
```

exits `0` and returns JSON **byte-equal to the packet's own
`input/forensics/north-gate-tick-1.json`** (verified by structural comparison). Per
`reference/explanations.md`, `explain-transition` re-executes every committed request and
requires byte-identical states, logs, receipts and hashes before it will render. Its success
proves the committed prefix is internally consistent and honestly recorded — the bundle is
not corrupt, it is a truthful record of a run that was stopped by its own second command.

It also gives the source mapping `fixtures/gaol.nomos` line 4, bytes 53–162 — the `north_gate`
declaration that owns the access machine.

### 2.8 Exact reproduction

```text
bin/nomos run input/world/ --commands input/failing.commands --out output/scratch/repro.run
```

exits `1` and produces a run bundle whose **six files are byte-identical (sha256) to
`input/failing.run/`**, including `result.json` digest `0989bf6a…5713e988`, and emits the same
`EK0804` diagnostic and the same `committed_command_count: 1`. The failure is fully
deterministic and fully explained by the supplied inputs.

---

## 3. Excluded alternatives

Each was tested against the actual binary and world; probe artefacts are under
`output/scratch/`.

### A. "The credential is wrong / missing." — EXCLUDED

If `unlock`'s credential requirement were unsatisfied, the code would be **`EK0805`**, not
`EK0804`, and **zero** commands would commit:

```text
unlock north_gate with credential/rusty_nail  -> EK0805 "command argument does not match the
                                                 compiled input requirement", committed 0
unlock north_gate                             -> EK0805, committed 0
```

Observed instead: `committed_command_count: 1` and `EK0804`. Moreover the *identical*
credential string succeeded at ordinal 0 (`command-log.json` row 0 resolves
`{"kind":"credential","credential":"credential/gaoler_key"}`). The credential is correct.

### B. "The entity or action name is wrong / unresolvable." — EXCLUDED

Unresolvable entities produce **`EK0801`** with zero commits:

```text
unlock south_gate with credential/gaoler_key  -> EK0801 "command entity `south_gate` does not
                                                 exist", committed 0
```

Also, `north_gate` is present in `simulation.json`, `persistence.json`, `diagnostics.json`,
`navigation.json` and `initial-state.json`, owning `north_gate.access`, `.combustion`,
`.integrity`, `.ward`; and resolution to `north_gate.access` demonstrably succeeded at ordinal
0. Name resolution is not implicated.

### C. "The compiled world / run bundle is corrupt or was tampered with." — EXCLUDED

- All seven world members re-hash to their `manifest.json` sha256 values (§2.6).
- All packet files match `packet-manifest.json`.
- `nomos inspect` opens the world strictly, exit `0`.
- `nomos explain-transition` strictly re-executes the committed prefix and matches
  byte-for-byte, exit `0` (§2.7) — a corrupt bundle would fail that opener.
- `nomos run` re-derives a byte-identical bundle from the same inputs (§2.8).

### D. "A runtime/binary regression rejects a legal command." — EXCLUDED

The rejection is *correct* against the compiled semantics: `unlock.source == "locked"`, the
machine was `closed`. The guard is also generic rather than an `unlock`-specific bug — the
same code fires on a different namespace under the same shape:

```text
unlock north_gate with credential/gaoler_key; extinguish brazier_02; extinguish brazier_02
  -> committed 2, EK0804 "`brazier_02.emission.extinguish` is illegal while the machine is
     `extinguished`"
```

Independently, the corrected script runs to `completed` with `committed_command_count: 5` on
this same binary and world (§4), so the runtime is not rejecting legal input.

### E. "Repeating a command text is itself forbidden (duplicate detection)." — EXCLUDED

A six-command script that repeats `open north_gate` twice (legal because `close` returns the
machine to `closed`) commits **all six** and completes:

```text
unlock; open; close; open; unseal; extinguish  -> status completed, committed 6
```

So the runtime has no duplicate-line rule. The rejection is purely about the *source state* of
the transition, which is why `unlock` (whose source state is unreachable after tick 1) fails
while `open` (whose source state is re-enterable via `close`) does not.

### F. "The script is too long, or the run budget/tick limit was hit." — EXCLUDED

The failing script has 6 commands; the probe in (E) also has 6 and completes. The repaired
script's 5 commands complete. No budget or arity limit is in play.

### G. "A later command (`open` / `unseal` / `ignite` / `extinguish`) is the real problem." — EXCLUDED

None of them ever executed (`command-log.json` has one row; `state-hashes.json` has two
snapshots). And in the repaired run all four commit cleanly, including the causal edge
`north_gate.combustion → burning` firing handler `apply_damage {damage, fire, 2}` which drives
`north_gate.integrity: intact → destroyed`. The tail of the script was always sound.

### H. "The publisher wrongly emitted a bundle for a rejected run." — EXCLUDED

Emitting a six-file bundle with `status: "rejected"` and the committed prefix is the designed
contract (`reference/runtime.md`: contiguous zero-based committed rows; ordinal 0 for the
initial snapshot plus one hash per committed command). The bundle is self-consistent and
re-verifiable (§2.7), so it is evidence *about* the failure, not a second failure.

---

## 4. Repair classification and result

**Classification: content-level repair, owned by the command script.** The world source,
compiled package, and `bin/nomos` are all correct and were not modified.

`output/repaired.commands` is `input/failing.commands` with the duplicated line 3 deleted
(unified diff: `3d2 < unlock north_gate with credential/gaoler_key`). Nothing else changed —
same schema header, same order, same six lines minus one.

Verified result (full commands in `output/verification.md`):

```text
bin/nomos run input/world/ --commands output/repaired.commands --out output/repaired.run
exit 0   status "completed"   committed_command_count 5
final_state_hash 3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc
```

Prefix fidelity: the repaired run's `command-log.json` row 0 is **structurally identical** to
the failing run's only row (same request, same resolved command, same
`fa7247d1…` → `b8eac772…` hashes), and `initial-state.json` is byte-identical between the two
runs. The repair preserves everything the original run legitimately achieved and only removes
the impossible step.

Final machine states: `access: open`, `ward: unsealed`, `combustion: burning`,
`integrity: destroyed`, `emission: extinguished`, tick 5 — i.e. the script's evident intent
(open and neutralise the gate, douse the brazier) is fully realised.

---

## 5. One-line summary

`unlock` is a one-shot transition out of the irreversible initial state `locked`; the script
issued it twice, the first consumed it (receipt tick 1, `locked → closed`, hash `b8eac772…`),
and the second was correctly refused with `EK0804` after a one-command commit — a command-content
defect repaired by deleting the duplicate line.
