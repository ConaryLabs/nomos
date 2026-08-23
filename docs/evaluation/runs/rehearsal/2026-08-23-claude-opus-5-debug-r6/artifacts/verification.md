# Verification log

All commands were run from `/workspace` with the packet binary
`/workspace/bin/nomos` (sha256 `4af70accf3d1680f6b0e78f860be5ac62c5ab11b470026a83f01eb5b95051fd1`,
commit `c800c98a67f2599b5522a84d42a7549600d53d1f`). Every write target is under
`/workspace/output`. Stdout that had to be captured was written to temporary
`*.stdout.json` / `explain-*.json` files under `/workspace/output`, transcribed
below, and then deleted; no other capture or suppression sink was used.

## 0. Package integrity

```
cd /workspace && ./bin/nomos inspect input/world/
# exit 0, "status":"completed",
# "manifest_digest":"fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440"

cd /workspace/input/world && sha256sum compiler-receipts.json diagnostics.json \
  navigation.json persistence.json schemas.json simulation.json world-ir.json
# all seven equal the digests in manifest.json / compiler-receipts.json;
# simulation.json = 0cdb454be9876dccdc05369e91fb9bdea017973d77f7803d23a921a969a1ef3c
#   = runtime_semantics_digest in result.json / initial-state.json / final-state.json

cd /workspace && sha256sum input/failing.commands
# 27a3f42e986e0cf8c2ee76821143ae337342ff0608bc6ad2f1ab5a81898fbc8f  (packet-manifest value)
```

## 1. Reproduce the reported failure

```
cd /workspace && ./bin/nomos run input/world/ --commands input/failing.commands \
  --out output/repro.run > output/repro.stdout.json    # temp capture, since deleted
# exit 1
```

Captured stdout (verbatim):

```json
{"artifacts":["output/repro.run/causal-receipts.json","output/repro.run/command-log.json","output/repro.run/final-state.json","output/repro.run/initial-state.json","output/repro.run/result.json","output/repro.run/state-hashes.json"],"command":"run","committed_command_count":1,"diagnostics":[{"code":"EK0804","message":"`north_gate.access.unlock` is illegal while the machine is `closed`","repairs":[]}],"final_state_hash":"b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b","first_state_hash":"fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42","output":"output/repro.run","result_digest":"0989bf6abcafe8c3e7c00a824a54a6c4b5f5ebafcb856215584748ba5713e988","status":"rejected"}
```

Byte-identity against the supplied run bundle:

```
cd /workspace && for f in causal-receipts.json command-log.json final-state.json \
  initial-state.json result.json state-hashes.json; do
  cmp -s input/failing.run/$f output/repro.run/$f && echo "IDENTICAL $f" || echo "DIFFERS $f"; done
# IDENTICAL for all six files (sha256 of both sets also equal)
```

## 2. Discriminating probes (alternative exclusion)

```
cd /workspace && ./bin/nomos command input/world/ --state input/failing.run/final-state.json \
  "unlock north_gate with credential/gaoler_key" --out output/probe-a.run
# exit 1, committed_command_count 0,
# EK0804 "`north_gate.access.unlock` is illegal while the machine is `closed`"
#   -> the duplicate line fails purely because of the post-commit state

cd /workspace && ./bin/nomos command input/world/ --state input/failing.run/initial-state.json \
  "unlock north_gate with credential/wrong_key" --out output/probe-b.run
# exit 1, EK0805 "command argument does not match the compiled input requirement"
#   -> credential failure has a different code; excludes alternative A1

cd /workspace && ./bin/nomos command input/world/ --state input/failing.run/initial-state.json \
  "unseal brazier_02" --out output/probe-c.run
# exit 1, EK0802 "entity `brazier_02` exposes no external command `unseal`"
#   -> unknown-verb failure has a different code; excludes part of A2

cd /workspace && ./bin/nomos command input/world/ --state input/failing.run/initial-state.json \
  "unlock gaol_door with credential/gaoler_key" --out output/probe-d.run
# exit 1, EK0801 "command entity `gaol_door` does not exist"
#   -> unknown-entity failure has a different code; excludes the rest of A2
```

(Each probe wrote its stdout to a temporary `output/probe-*.stdout.json`, which
was transcribed above and deleted. The probe run bundles remain in
`/workspace/output` as artifacts.)

## 3. Write the repair

```
cd /workspace && printf 'schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nopen north_gate\nunseal north_gate\nignite north_gate\nextinguish brazier_02\n' > output/repaired.commands

cd /workspace && diff input/failing.commands output/repaired.commands
# 3d2
# < unlock north_gate with credential/gaoler_key
# (exactly one deleted line, 194 -> 149 bytes)

cd /workspace && sha256sum output/repaired.commands
# 31c84464a7e2d0d03397a92b32c76a21eccf39ecb94c0ca3605cf21fd4eb3719
```

## 4. Verify the repair into a new directory

```
cd /workspace && ./bin/nomos run input/world/ --commands output/repaired.commands \
  --out output/repaired.run > output/repaired.stdout.json   # temp capture, since deleted
# exit 0
```

Captured stdout (verbatim):

```json
{"artifacts":["output/repaired.run/causal-receipts.json","output/repaired.run/command-log.json","output/repaired.run/final-state.json","output/repaired.run/initial-state.json","output/repaired.run/result.json","output/repaired.run/state-hashes.json"],"command":"run","committed_command_count":5,"final_state_hash":"3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc","first_state_hash":"fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42","output":"output/repaired.run","result_digest":"e13f4ce9339967e51a06353a1315fa82c0e5a45172073f48af02b1fe0d5a8e7a","status":"completed"}
```

Bundle contents checked with ordinary inspection:

```
cd /workspace && python3 -m json.tool output/repaired.run/state-hashes.json
# ordinal 0 fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42  (unchanged initial)
# ordinal 1 b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b  (= failing run's committed prefix)
# ordinal 2 0545806ca38848850f9e9ca674fa4891066ed57b72fa7a957fd66332cae7314b
# ordinal 3 7843d53de4c8bb543a8ea11603d5ada260b549b18a5d5076e74e0e29b81a6e79
# ordinal 4 2459a218d9d99015aff8a7c5cee88513f61542eabd63d2871fcd3caf973ed545
# ordinal 5 3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc

# final-state.json: tick 5; brazier_02.emission=extinguished, north_gate.access=open,
#   north_gate.combustion=burning, north_gate.integrity=destroyed, north_gate.ward=unsealed
# command-log.json: 5 rows, unlock/open/unseal/ignite (north_gate) then extinguish (brazier_02),
#   each resolving to exactly one namespace command
```

## 5. Cross-check the repair against the supplied forensic receipt

```
cd /workspace && ./bin/nomos explain-transition output/repaired.run north_gate \
  --tick 1 --world input/world/ > output/explain-t1.json   # temp capture, since deleted
# exit 0
# receipt object compared field-by-field with input/forensics/north-gate-tick-1.json:
#   identical; the only differing top-level key is run_result_digest
#   (0989bf6a…5713e988 for the rejected run vs e13f4ce9…0d5a8e7a for the repaired run)

cd /workspace && ./bin/nomos explain-transition output/repaired.run north_gate \
  --tick 4 --world input/world/ > output/explain-t4.json   # temp capture, since deleted
# exit 0; transitions:
#   local  north_gate.combustion cold -> burning        cause command `ignite`
#   causal north_gate.integrity  intact -> destroyed    cause event `apply_damage`
#          payload {kind: damage, channel: fire, amount: 2}
```

`explain-transition` strictly re-opens the world and re-executes every committed
row of the run bundle (`reference/explanations.md`, decision-0009 sequence), so
its exit-0 result is an independent confirmation that `output/repaired.run` is a
self-consistent, hash-verified bundle.

## 6. Residual files under /workspace/output

`diagnosis.md`, `repaired.commands`, `verification.md`, the verification bundle
`repaired.run/`, the reproduction bundle `repro.run/`, and the four probe
bundles `probe-a.run/` … `probe-d.run/`. All temporary stdout captures were
deleted after transcription. No file outside `/workspace/output` was written and
no path outside `/workspace` was read, written, or probed.
