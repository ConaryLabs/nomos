# Diagnosis — cold-debug rehearsal (non-formal)

Packet: `nomos.gate_k.packet_manifest@1`, candidate commit
`71093eb46805c6811100e4b552595048a11b5346`, binary
`bin/nomos` sha256 `4af70acc…51fd1`. Shape `debug`, writable path `output` only.
This is a **rehearsal**, not a formal Gate K attempt; operator intervention
budget was `none` and none was used.

---

## 1. Verdict in one line

`input/failing.commands` contains a **duplicated third line**
(`unlock north_gate with credential/gaoler_key` repeated at line 3). The compiled
`north_gate.access` machine defines `unlock` **only** from source state `locked`;
after the line-2 `unlock` commits, the machine is `closed`, so the line-3 repeat
has no compiled transition and the runtime rejects it with `EK0804`, publishing
the one-command committed prefix. **The world, the package, the runtime and the
run bundle are all correct.** The defect is entirely in command-script content.

## 2. The semantic cause, stated as semantics rather than as an error code

`world-ir.json` → entity `north_gate` → machine `north_gate.access`:

```
states: [locked, closed, open]
locked --command unlock(resolved_entity_credential)--> closed
closed --command open(none)-----------------------> open
open   --command close(none)----------------------> closed
```

There is **no** transition whose `source` is `closed` and whose trigger is
`unlock`. `unlock` in this world is a *one-way, non-idempotent* act: it consumes
the `locked` state and lands in `closed`, and `closed` is the same state a
`close` would produce. The failing script implicitly assumes one of two false
models — either that `unlock` is idempotent (safe to repeat / "make sure it is
unlocked"), or that `unlock` yields a distinct `unlocked` state from which a
second `unlock` is still meaningful. Neither is true of the compiled machine.

So the script does not describe a path through the state graph. Its intended
5-step path exists; the injected 6th step demands the non-existent edge
`closed --unlock--> ?`.

The terminal code `EK0804` is only the *name* of that fact. The cause is
"repeating a state-consuming command against a machine that has already left the
required source state", and it would be the same cause with a different message
for `ignite`, `unseal` or `extinguish` (demonstrated in §4, alternative F).

## 3. Forensic evidence

### 3.1 Command content (the defect itself)

`input/failing.commands` (194 bytes, sha256 `27a3f42e…8fbc8f`, matches the packet
manifest, so it is the exact script that was run):

```
1  schema nomos.command_script@1
2  unlock north_gate with credential/gaoler_key
3  unlock north_gate with credential/gaoler_key   <-- byte-identical duplicate
4  open north_gate
5  unseal north_gate
6  ignite north_gate
7  extinguish brazier_02
```

Lines 2 and 3 are byte-identical (`cat -A` shows no trailing-whitespace or
line-ending difference). Six requests; five distinct.

Corroboration that five was the intended count: `reference/explanations.md` §
"Transition explanation" states the accepted `fixtures/gaol.commands` "prove
`north_gate` at tick 4". A 5-command script whose 4th command is `ignite`
produces the last `north_gate` transition at tick 4 — which is exactly what the
repaired script produces (§5). The failing script is the accepted fixture with
one line duplicated.

### 3.2 Result and diagnostic

`input/failing.run/result.json` (`nomos.run_result@1`):

- `"status": "rejected"`, `"rejection_diagnostic": {"code": "EK0804"}`
- `"committed_command_count": 1`
- `"first_state_hash": "fa7247d1…50fe42"`, `"final_state_hash": "b8eac772…94952b"`

`input/forensics/failure.stdout.json` carries the human wording that
`result.json` deliberately does not:

> `EK0804` — ``` `north_gate.access.unlock` is illegal while the machine is `closed` ```

with `"repairs": []`, exit code `1` (`failure.exit.txt`), empty stderr. Per
`reference/runtime.md`, `EK08xx` is runtime rejection, exit `1` is semantic
rejection, and "a rejected bundle intentionally binds only the committed prefix
and the terminal code … it does not persist the rejected request". The absence
of the rejected line from the bundle is therefore expected, not evidence loss.

### 3.3 Command log — the rejection is at ordinal 1, not ordinal 0

`command-log.json` holds exactly one row, ordinal `0`:

```
request        : unlock / north_gate / catalog_value credential/gaoler_key
resolved_command: unlock / north_gate.access / credential credential/gaoler_key
input_state_hash    fa7247d1…50fe42   ->   resulting_state_hash b8eac772…94952b
```

This is decisive on two points at once: **resolution and credential checking
already succeeded** for this exact text (so nothing about the line is malformed),
and the *second* instance of the same text is the one that died.

### 3.4 Causal receipt — the state that made the repeat illegal

`causal-receipts.json` receipt `tick 1` records the single transition:

```
namespace north_gate.access, phase local, cause command `unlock`, from "locked" to "closed"
```

`effective_facts_before` and `effective_facts_after` are identical
(`north_gate` still `blocked` for reasons `north_gate.portal#blocks_ground` and
`north_gate.ward#blocks_ground`), `projection_deltas: []`. That is the compiled
truth: unlocking alone changes no movement or light fact — the gate is still
blocked by the portal claim (`access != open`) and by the ward claim
(`ward == sealed`). This confirms the run stopped before `open` and `unseal`.

### 3.5 State hashes and persisted state

`state-hashes.json` has ordinals `0` and `1` only — initial plus one commit.
`final-state.json` (`nomos.persisted_runtime_state@2`) shows `tick: 1` and

```
brazier_02.emission = lit        (extinguish never ran)
north_gate.access   = closed     <-- the state that rejects a second `unlock`
north_gate.combustion = cold     (ignite never ran)
north_gate.integrity  = intact
north_gate.ward       = sealed   (unseal never ran)
```

Every machine except `north_gate.access` is still at its `world-ir.json`
`initial` value. The tail of the script demonstrably never executed, which is
consistent only with a fail-closed stop at request index 1.

### 3.6 Package identity — the run used exactly the packet's world

- `input/forensics/compile.stdout.json`: `manifest_digest fd2dca0e…710440`
- `input/world/manifest.json`: `package_digest fd2dca0e…710440`
- `result.json`: `input_package_digest fd2dca0e…710440`
- `./bin/nomos inspect input/world` → `manifest_digest fd2dca0e…710440`,
  `status completed` (the strict opener verified all seven members)
- `runtime_semantics_digest 0cdb454b…1ef3c` in `result.json`, in both persisted
  states, and as the manifest sha256 of `simulation.json` — all equal.

Independent check: all 30 packet files hash exactly to `packet-manifest.json`,
before and after every experiment below.

### 3.7 Reproduction — byte-identical

```
./bin/nomos run input/world --commands input/failing.commands --out output/repro-failing.run
```

exits `1` with the same `EK0804`, `committed_command_count 1`,
`result_digest 0989bf6a…13e988`, and **all six artifacts compare byte-identical**
to `input/failing.run/`. The failure is deterministic and fully explained by the
package plus the script; nothing environmental is involved.

### 3.8 The run bundle is authentic, not forged

```
./bin/nomos explain-transition input/failing.run north_gate --tick 1 --world input/world
```

exits `0` and is byte-identical to `input/forensics/north-gate-tick-1.json`.
Per `reference/explanations.md`, this command strictly re-opens the world,
strictly opens the six-file bundle, **re-executes every committed request** and
requires byte-identical states, logs, receipts and hashes. The committed prefix
therefore genuinely reproduces; the evidence is trustworthy.

## 4. Excluded alternatives

Each alternative was tested against the same world; each produces a
*different* stable code and/or a different committed count, so none can explain
the observed `EK0804` + `committed_command_count: 1`.

| # | Hypothesis | Test | Observed | Why excluded |
|---|---|---|---|---|
| A | Bad/unresolvable credential (`with credential/…` wrong) | `unlock north_gate with credential/wrong_key` | `EK0805` "command argument does not match the compiled input requirement", committed `0` | Different code, and ordinal 0 of the real run *committed* with `credential/gaoler_key` resolved to `{kind: credential}` (§3.3) |
| B | Malformed script / wrong schema header | header `nomos.command_script@2` | `EK0814` "command script has the wrong or missing schema header", **no bundle written** | Different code; a header fault produces no artifacts at all, yet the failing run published six |
| C | Unknown / misspelled entity | `unlock south_gate …` | `EK0801` "command entity `south_gate` does not exist", committed `0` | Different code; `north_gate` resolved fine at ordinal 0 |
| D | Action not offered by the entity (resolution fault) | `open brazier_02` | `EK0802` "entity `brazier_02` exposes no external command `open`", committed `0` | Different code; `unlock` *is* an external command of `north_gate` |
| E | Wrong ordering of the tail (e.g. `unseal` must precede `open`, or `ignite` requires `open`) | reordered 5-command script `unlock, unseal, open, ignite, extinguish` | `completed`, committed `5`, final hash `3e06b963…05bfdc` — **identical** to the repaired script's final hash | These four commands are order-independent here; ordering was never a constraint, so ordering cannot be the cause |
| F | The problem is specific to `unlock`, or the runtime mis-implements `unlock` | duplicate `ignite` instead: `unlock, open, unseal, ignite, ignite, extinguish` | `EK0804` "`north_gate.combustion.ignite` is illegal while the machine is `burning`", committed `4` | Same class of failure at a different namespace — the cause is *repetition of a state-consuming command*, not a defect peculiar to `unlock` |
| G | Runtime bug: `EK0804` raised spuriously for a legal command | `nomos command` with the *same text* from two different states | from `initial-state.json` (`locked`) → **completed**, hash `b8eac772…94952b`; from `final-state.json` (`closed`) → **`EK0804`**, committed `0` | Identical command text, identical world; only the input state differs. The rejection tracks the state exactly as `world-ir.json` prescribes — correct behaviour, not a bug |
| H | World/package corruption or drift between compile and run | `inspect` + digest chain + manifest hashing (§3.6) | every digest agrees; all 30 packet files match the manifest | The world is intact and is the same one that compiled |
| I | Forged or damaged run bundle | `explain-transition` re-execution (§3.8) | byte-identical to the packet forensic file, exit `0` | Bundle re-executes exactly; evidence is authentic |
| J | Nondeterminism / environment | re-run of `failing.commands` (§3.7) and two runs of the repaired script | byte-identical artifacts both times | Fully deterministic |

Note on D vs. the real failure: `EK0802` is *resolution* ("this entity has no
such command"), while `EK0804` is *legality* ("the command exists but not from
this state"). The failing run is squarely the second, which is only reachable
after resolution has already succeeded — further confirming the script's grammar,
entity, action and argument are all fine.

## 5. Owning boundary and repair classification

**Owning boundary: command-script content (`nomos.command_script@1` input).**
Not the source world, not the compiler, not the runtime, not the CLI.

Justification:
- The compiled machine is *deliberately* non-idempotent for `unlock`: a lock is
  consumed once. Adding `closed --unlock--> closed` would be a semantic change to
  a **verified, frozen** world, and `brief.txt` forbids editing the compiled world
  or the binary. Nothing in the evidence suggests the world is wrong.
- The runtime behaved exactly as `reference/runtime.md` specifies: stop at the
  first rejection, commit only the legal prefix atomically, publish the terminal
  code, leave the input package byte-identical.
- The script is the only artifact that is both wrong and mine to change.

**Repair: delete the duplicated line 3.** Written to
`output/repaired.commands` (149 bytes, sha256 `31c84464…eb3719`), verified into
`output/repaired.run` — `status completed`, `committed_command_count 5`,
final state hash `3e06b963…05bfdc`, tick `5`, final machines
`access=open, ward=unsealed, combustion=burning, integrity=destroyed,
emission=extinguished`. Exact commands in `output/verification.md`.

Two extra confirmations that the repair is *minimal and correct*:

1. **Prefix continuity.** `output/repaired.run/state-hashes.json` ordinal 1 is
   `b8eac772…94952b` — byte-identical to the failing run's `final_state_hash`.
   The repaired run reproduces the failing run's committed prefix exactly and
   then continues, proving nothing but the extra line was ever at fault.
2. **Documented fixture behaviour restored.** `explain-transition
   output/repaired.run north_gate --tick 4 --world input/world` succeeds and
   shows the local step `north_gate.combustion cold -> burning` plus the causal
   step `north_gate.integrity intact -> destroyed` (handler `apply_damage`,
   fire 2), matching the `on_enter(burning)` interaction in `world-ir.json` and
   `reference/explanations.md`'s claim that the accepted fixture "prove[s]
   `north_gate` at tick 4".

## 6. Scope, limits, and a disclosed deviation

- Only `bin/nomos` and ordinary shell inspection inside `/workspace` were used.
  No source, no repository history, no network, no package installation.
- All writes are under `/workspace/output`. The `input/` tree is byte-identical
  to `packet-manifest.json` after every experiment.
- **Disclosed deviation:** while sanity-checking the sandbox I ran
  `ls /workspace/..`, which listed directory names one level above the workspace
  before I stopped. No file outside `/workspace` was opened and nothing from that
  listing informs this diagnosis. It should not have been run; I did not repeat it.
- I did not see `fixtures/gaol.nomos` or `fixtures/gaol.commands`; the claim that
  the failing script is the accepted fixture plus one duplicated line is an
  inference from `reference/explanations.md` and the tick-4 reproduction, not from
  reading the fixture. The diagnosis does not depend on it.
- `EK08xx` code meanings beyond those the binary emitted here are not asserted.
