# Verification log

All commands were run with CWD `/workspace`, using only `/workspace/bin/nomos` and ordinary
shell inspection. Every written path is inside `/workspace/output`. No source tree, repository
history, network, `/tmp`, `/home`, `/etc`, or `/workspace/..` path was read, written, or probed.

`bin/nomos` and `input/` were treated as read-only throughout; no compiled world or binary was
edited.

---

## 0. Scratch directory

```bash
mkdir -p /workspace/output/scratch
```

## 1. Baseline: the failing script reproduces the packet run byte-for-byte

```bash
cd /workspace
./bin/nomos run input/world/ --commands input/failing.commands --out output/scratch/repro.run \
  > output/scratch/repro.stdout.json 2> output/scratch/repro.stderr.txt; echo "exit=$?"
```

Result: `exit=1`, stderr empty, stdout reports
`"status":"rejected"`, `"committed_command_count":1`,
and the diagnostic

```json
{"code":"EK0804","message":"`north_gate.access.unlock` is illegal while the machine is `closed`","repairs":[]}
```

and
`"result_digest":"0989bf6abcafe8c3e7c00a824a54a6c4b5f5ebafcb856215584748ba5713e988"`.

Byte-identity of the six-file bundle against the packet:

```bash
cd /workspace
for f in causal-receipts.json command-log.json final-state.json initial-state.json \
         result.json state-hashes.json; do
  a=$(sha256sum input/failing.run/$f | cut -d' ' -f1)
  b=$(sha256sum output/scratch/repro.run/$f | cut -d' ' -f1)
  [ "$a" = "$b" ] && echo "MATCH $f $a" || echo "DIFF  $f $a $b"
done
```

Result: `MATCH` for all six files.

## 2. World package integrity

```bash
cd /workspace/input/world
python3 - <<'EOF'
import json,hashlib
m=json.load(open('manifest.json'))
for e in m['members']:
    d=hashlib.sha256(open(e['name'],'rb').read()).hexdigest()
    print(('MATCH' if d==e['sha256'] else 'DIFF '), e['name'], d)
EOF
```

Result: `MATCH` for all seven members.

```bash
cd /workspace
./bin/nomos inspect input/world/ > output/scratch/inspect.json; echo "exit=$?"
```

Result: `exit=0`; `north_gate.access` reports transitions
`open --close--> closed`, `closed --open--> open`, `locked --unlock(resolved_entity_credential)--> closed`.

## 3. Reachability of `locked` (why the duplicate can never be legalised)

```bash
cd /workspace
python3 - <<'EOF'
import json
s=json.load(open('input/world/simulation.json'))
for m in s['machines']:
    tgts={c['target'] for c in m['commands']} | {h['target'] for h in m['handlers']}
    unreach=[st for st in m['states'] if st not in tgts]
    print(m['namespace'],'initial=',m['initial'])
    print('   incoming-targets:',sorted(tgts),' no incoming transition:',sorted(unreach))
EOF
```

Result: for `north_gate.access`, incoming targets are `['closed','open']`; `locked` has **no**
incoming transition. `locked` is initial-only and irreversible.

## 4. Explanation evidence reproduces the packet forensic

```bash
cd /workspace
./bin/nomos explain-transition input/failing.run/ north_gate --tick 1 --world input/world/ \
  > output/scratch/explain-failing-t1.json 2> output/scratch/explain-failing-t1.err; echo "exit=$?"
python3 -c "
import json
a=json.load(open('output/scratch/explain-failing-t1.json'))
b=json.load(open('input/forensics/north-gate-tick-1.json'))
print('identical to packet forensic:', a==b)"
```

Result: `exit=0`, `identical to packet forensic: True`.

## 5. Excluded-alternative probes

### 5A. Wrong credential -> EK0805, not EK0804

```bash
cd /workspace
cat > output/scratch/alt-badcred.commands <<'EOF'
schema nomos.command_script@1
unlock north_gate with credential/rusty_nail
EOF
./bin/nomos run input/world/ --commands output/scratch/alt-badcred.commands \
  --out output/scratch/alt-badcred.run > output/scratch/alt-badcred.stdout.json 2>&1; echo "exit=$?"
```

Result: `exit=1`, `committed_command_count: 0`,
`EK0805 "command argument does not match the compiled input requirement"`.

### 5B. Missing credential argument -> EK0805, not EK0804

```bash
cd /workspace
cat > output/scratch/alt-noarg.commands <<'EOF'
schema nomos.command_script@1
unlock north_gate
EOF
./bin/nomos run input/world/ --commands output/scratch/alt-noarg.commands \
  --out output/scratch/alt-noarg.run > output/scratch/alt-noarg.stdout.json 2>&1; echo "exit=$?"
```

Result: `exit=1`, `committed_command_count: 0`, `EK0805`.

### 5C. Unknown entity -> EK0801, not EK0804

```bash
cd /workspace
cat > output/scratch/alt-unknown.commands <<'EOF'
schema nomos.command_script@1
unlock south_gate with credential/gaoler_key
EOF
./bin/nomos run input/world/ --commands output/scratch/alt-unknown.commands \
  --out output/scratch/alt-unknown.run > output/scratch/alt-unknown.stdout.json 2>&1; echo "exit=$?"
```

Result: `exit=1`, `committed_command_count: 0`,
`EK0801` with message: ``command entity `south_gate` does not exist``.

### 5D. Repeating a command text is NOT forbidden (six commands, `open` twice) -> completed

```bash
cd /workspace
printf 'schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nopen north_gate\nclose north_gate\nopen north_gate\nunseal north_gate\nextinguish brazier_02\n' \
  > output/scratch/alt-sixlegal.commands
./bin/nomos run input/world/ --commands output/scratch/alt-sixlegal.commands \
  --out output/scratch/alt-sixlegal.run
```

Result: `status completed`, `committed_command_count 6`. Excludes both "duplicate detection"
and "script too long".

### 5E. EK0804 is a generic state-legality guard, not an `unlock` bug

```bash
cd /workspace
printf 'schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nextinguish brazier_02\nextinguish brazier_02\n' \
  > output/scratch/alt-dupextinguish.commands
./bin/nomos run input/world/ --commands output/scratch/alt-dupextinguish.commands \
  --out output/scratch/alt-dupext.run > output/scratch/alt-dupext.stdout.json 2>&1; echo "exit=$?"
```

Result: `exit=1`, `committed_command_count: 2`,
`EK0804` with message: ``` `brazier_02.emission.extinguish` is illegal while the machine is `extinguished` ```.

---

## 6. The repair

```bash
cd /workspace
printf 'schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nopen north_gate\nunseal north_gate\nignite north_gate\nextinguish brazier_02\n' \
  > output/repaired.commands
diff input/failing.commands output/repaired.commands
```

Diff (exit status 1, i.e. a difference exists, as intended):

```text
3d2
< unlock north_gate with credential/gaoler_key
```

Exactly one line removed; nothing else altered.

## 7. Verification of the repair into a NEW directory

```bash
cd /workspace
./bin/nomos run input/world/ --commands output/repaired.commands --out output/repaired.run \
  > output/repaired.stdout.json 2> output/repaired.stderr.txt; echo "exit=$?"
```

Result: `exit=0`, stderr empty.

```json
{"command":"run","status":"completed","committed_command_count":5,
 "first_state_hash":"fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42",
 "final_state_hash":"3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc",
 "result_digest":"e13f4ce9339967e51a06353a1315fa82c0e5a45172073f48af02b1fe0d5a8e7a",
 "output":"output/repaired.run"}
```

### 7.1 All five commands committed, in order

```bash
cd /workspace
python3 -c "
import json
r=json.load(open('output/repaired.run/command-log.json'))
for row in r['rows']:
    print(row['ordinal'], row['input_state_hash'][:12],'->',row['resulting_state_hash'][:12],
          json.dumps(row['resolved_command']))"
```

```text
0 fa7247d11530 -> b8eac7726d57 north_gate.access.unlock (credential/gaoler_key)
1 b8eac7726d57 -> 0545806ca388 north_gate.access.open
2 0545806ca388 -> 7843d53de4c8 north_gate.ward.unseal
3 7843d53de4c8 -> 2459a218d9d9 north_gate.combustion.ignite
4 2459a218d9d9 -> 3e06b9639cde brazier_02.emission.extinguish
```

`state-hashes.json` contains ordinals 0..5 (initial + one per committed command).

### 7.2 Committed prefix preserved exactly

```bash
cd /workspace
python3 -c "
import json
f=json.load(open('input/failing.run/command-log.json'))['rows'][0]
r=json.load(open('output/repaired.run/command-log.json'))['rows'][0]
print('committed prefix row identical:', f==r)"
cmp -s input/failing.run/initial-state.json output/repaired.run/initial-state.json \
  && echo "initial-state.json byte-identical" || echo "initial-state differs"
```

Result: `committed prefix row identical: True`; `initial-state.json byte-identical`.

### 7.3 Final semantics are the script's evident intent

```bash
cd /workspace
python3 -c "
import json
d=json.load(open('output/repaired.run/final-state.json'))
print(json.dumps(d['state']['machines'])); print('tick',d['state']['tick'],'hash',d['state_hash'])"
```

```text
brazier_02.emission=extinguished, north_gate.access=open, north_gate.combustion=burning,
north_gate.integrity=destroyed, north_gate.ward=unsealed   tick 5
hash 3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc
```

### 7.4 Independent re-check of the repaired run through the strict explanation opener

```bash
cd /workspace
./bin/nomos explain-transition output/repaired.run/ north_gate --tick 4 --world input/world/ \
  > output/scratch/explain-repaired-t4.json 2>&1; echo "exit=$?"
```

Result: `exit=0`. The tick-4 receipt shows the local step
`north_gate.combustion cold -> burning (command ignite)` followed by the causal step
`north_gate.integrity intact -> destroyed (event apply_damage {damage, fire, 2})`, and
`effective_facts_after` shows `north_gate` movement changed from `blocked` to
`traversable cost 1` with no remaining blocker reasons. Since `explain-transition` re-executes
every committed request and demands byte-identical states, logs, receipts and hashes before
rendering, `exit=0` is an independent confirmation that the repaired bundle is sound.

---

## 8. Output inventory

- `output/diagnosis.md` — diagnosis, evidence, excluded alternatives.
- `output/repaired.commands` — corrected `nomos.command_script@1` (duplicate line removed).
- `output/repaired.run/` — verified run bundle, `status: completed`, 5 commands.
- `output/repaired.stdout.json`, `output/repaired.stderr.txt` — verification run output.
- `output/verification.md` — this file.
- `output/scratch/` — reproduction and excluded-alternative probe artefacts.
