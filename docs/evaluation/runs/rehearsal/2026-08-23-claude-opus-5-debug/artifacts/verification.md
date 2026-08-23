# Verification record

All commands were run from `/workspace` with the packet binary `./bin/nomos`
(sha256 `4af70accf3d1680f6b0e78f860be5ac62c5ab11b470026a83f01eb5b95051fd1`).
Every write target is under `/workspace/output`; `input/` was never modified.

## 0. Packet integrity (before and after all experiments)

```sh
python3 - <<'EOF'
import json,hashlib
m=json.load(open('packet-manifest.json'))
bad=[f['path'] for f in m['files']
     if hashlib.sha256(open(f['path'],'rb').read()).hexdigest()!=f['sha256']]
print("mismatches:", bad if bad else "NONE")
EOF
```

Result both times: `files checked: 30` / `mismatches: NONE`.

## 1. World package opens and is the one that compiled

```sh
./bin/nomos inspect input/world
```

`status completed`, `manifest_digest fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440`
— equal to `input/forensics/compile.stdout.json` `manifest_digest` and to
`input/failing.run/result.json` `input_package_digest`.

```sh
./bin/nomos explain-entity input/world north_gate
```

Confirms `north_gate.access` = `locked -unlock-> closed`, `closed -open-> open`,
`open -close-> closed`, with **no** `unlock` edge out of `closed`.

## 2. Byte-identical reproduction of the reported failure

```sh
./bin/nomos run input/world --commands input/failing.commands --out output/repro-failing.run
```

exit `1`; `status rejected`; `EK0804`; `committed_command_count 1`;
`result_digest 0989bf6abcafe8c3e7c00a824a54a6c4b5f5ebafcb856215584748ba5713e988`.

```sh
for f in causal-receipts.json command-log.json final-state.json \
         initial-state.json result.json state-hashes.json; do
  cmp -s input/failing.run/$f output/repro-failing.run/$f \
    && echo "IDENTICAL $f" || echo "DIFFER $f"
done
```

All six: `IDENTICAL`.

## 3. Authenticity of the supplied run bundle

```sh
./bin/nomos explain-transition input/failing.run north_gate --tick 1 --world input/world \
  > output/refetch-failing-tick-1.json
cmp output/refetch-failing-tick-1.json input/forensics/north-gate-tick-1.json
```

exit `0`, `cmp` silent → byte-identical. (This command re-executes the whole
committed prefix under the strict opener, so it also proves the prefix reproduces.)

## 4. The repair

```sh
sed '3d' input/failing.commands > output/repaired.commands
```

`output/repaired.commands`, 149 bytes, sha256
`31c84464a7e2d0d03397a92b32c76a21eccf39ecb94c0ca3605cf21fd4eb3719`:

```
schema nomos.command_script@1
unlock north_gate with credential/gaoler_key
open north_gate
unseal north_gate
ignite north_gate
extinguish brazier_02
```

## 5. Verifying the repair into a new directory

```sh
./bin/nomos run input/world --commands output/repaired.commands --out output/repaired.run
```

exit `0`:

```json
{"command":"run","status":"completed","committed_command_count":5,
 "first_state_hash":"fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42",
 "final_state_hash":"3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc",
 "result_digest":"e13f4ce9339967e51a06353a1315fa82c0e5a45172073f48af02b1fe0d5a8e7a"}
```

Committed log (ordinal → resolved command):

```
0 unlock  -> north_gate.access      (credential credential/gaoler_key)
1 open    -> north_gate.access
2 unseal  -> north_gate.ward
3 ignite  -> north_gate.combustion
4 extinguish -> brazier_02.emission
```

`output/repaired.run/final-state.json`: `tick 5`,
`access=open`, `ward=unsealed`, `combustion=burning`, `integrity=destroyed`,
`brazier_02.emission=extinguished`.

**Prefix continuity:** `output/repaired.run/state-hashes.json` ordinal 1 =
`b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b` = the failing
run's `final_state_hash`.

### 5a. Determinism of the repaired run

```sh
./bin/nomos run input/world --commands output/repaired.commands --out output/repaired-recheck.run
for f in causal-receipts.json command-log.json final-state.json \
         initial-state.json result.json state-hashes.json; do
  cmp -s output/repaired.run/$f output/repaired-recheck.run/$f \
    && echo "IDENTICAL $f" || echo "DIFFER $f"
done
```

All six: `IDENTICAL`.

### 5b. Documented fixture behaviour restored (north_gate at tick 4)

```sh
./bin/nomos explain-transition output/repaired.run north_gate --tick 4 --world input/world \
  > output/repaired-north-gate-tick-4.json
```

exit `0`; receipt at tick 4 shows

```
local : north_gate.combustion  cold   -> burning   (cause: command ignite)
causal: north_gate.integrity   intact -> destroyed (cause: event apply_damage, fire 2)
```

matching the `on_enter(burning)` interaction in `world-ir.json` and
`reference/explanations.md` ("prove `north_gate` at tick 4").

## 6. Alternative-exclusion experiments

Scripts under `output/alt/`, bundles under `output/alt/*.run`.

```sh
mkdir -p output/alt

# A — wrong credential
printf 'schema nomos.command_script@1\nunlock north_gate with credential/wrong_key\n' \
  > output/alt/alt-a-wrong-credential.commands
./bin/nomos run input/world --commands output/alt/alt-a-wrong-credential.commands --out output/alt/alt-a.run
# -> exit 1, EK0805 "command argument does not match the compiled input requirement", committed 0

# B — wrong schema header
printf 'schema nomos.command_script@2\nunlock north_gate with credential/gaoler_key\n' \
  > output/alt/alt-b-bad-header.commands
./bin/nomos run input/world --commands output/alt/alt-b-bad-header.commands --out output/alt/alt-b.run
# -> exit 1, EK0814 "command script has the wrong or missing schema header", NO bundle written

# C — unknown entity
printf 'schema nomos.command_script@1\nunlock south_gate with credential/gaoler_key\n' \
  > output/alt/alt-c-unknown-entity.commands
./bin/nomos run input/world --commands output/alt/alt-c-unknown-entity.commands --out output/alt/alt-c.run
# -> exit 1, EK0801 "command entity `south_gate` does not exist", committed 0

# D — action not exposed by the entity
printf 'schema nomos.command_script@1\nopen brazier_02\n' \
  > output/alt/alt-f-wrong-action.commands
./bin/nomos run input/world --commands output/alt/alt-f-wrong-action.commands --out output/alt/alt-f.run
# -> exit 1, EK0802 "entity `brazier_02` exposes no external command `open`", committed 0

# E — ordering hypothesis: same five commands, different order
printf 'schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nunseal north_gate\nopen north_gate\nignite north_gate\nextinguish brazier_02\n' \
  > output/alt/alt-e-reordered.commands
./bin/nomos run input/world --commands output/alt/alt-e-reordered.commands --out output/alt/alt-e.run
# -> exit 0, completed, committed 5, final hash 3e06b963…05bfdc (same as repaired.run)

# F — duplication is generically fatal, not specific to `unlock`
printf 'schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nopen north_gate\nunseal north_gate\nignite north_gate\nignite north_gate\nextinguish brazier_02\n' \
  > output/alt/alt-g-dup-ignite.commands
./bin/nomos run input/world --commands output/alt/alt-g-dup-ignite.commands --out output/alt/alt-g.run
# -> exit 1, EK0804 "`north_gate.combustion.ignite` is illegal while the machine is `burning`", committed 4
```

### 6a. The decisive isolation — same text, two states

```sh
# from the failing run's committed final state (north_gate.access = closed)
./bin/nomos command input/world --state input/failing.run/final-state.json \
  "unlock north_gate with credential/gaoler_key" --out output/alt/alt-d1.run
# -> exit 1, EK0804 "`north_gate.access.unlock` is illegal while the machine is `closed`", committed 0

# the identical line from the initial state (north_gate.access = locked)
./bin/nomos command input/world --state input/failing.run/initial-state.json \
  "unlock north_gate with credential/gaoler_key" --out output/alt/alt-d2.run
# -> exit 0, completed, committed 1, final hash b8eac772…94952b

# the next scripted line from the failing run's final state
./bin/nomos command input/world --state input/failing.run/final-state.json \
  "open north_gate" --out output/alt/alt-d3.run
# -> exit 0, completed, committed 1, final hash 0545806c…7314b
```

Identical world, identical command text; only the input state differs. The
rejection tracks machine state exactly as the compiled `north_gate.access`
transition table prescribes, and the very next scripted command is legal from
that same state — i.e. the script's tail was fine and only the duplicate was not.

## 7. Artifacts produced under `output/`

```
diagnosis.md                        this diagnosis
verification.md                     this record
repaired.commands                   the repair (sha256 31c84464…eb3719)
repaired.run/                       verified run of the repair (completed, 5 commands)
repaired-recheck.run/               determinism re-run, byte-identical
repaired-north-gate-tick-4.json     tick-4 transition explanation of the repair
repro-failing.run/                  byte-identical reproduction of the reported failure
refetch-failing-tick-1.json         re-derived tick-1 explanation of the supplied bundle
alt/                                alternative-exclusion scripts and bundles (A–G)
```
