# Verification record

All commands were run from `/workspace` with `./bin/nomos`
(sha256 `4af70accf3d1680f6b0e78f860be5ac62c5ab11b470026a83f01eb5b95051fd1`).
`<scratch>` = `output/scratch`, a temporary directory under the writable subtree that was removed
after the results below were recorded. Only `output/repaired.commands`, `output/repaired.run/`,
`output/repaired.stdout.json`, `output/diagnosis.md` and this file remain.

## A. Package integrity of the supplied world

```
sha256sum input/world/*.json
python3 -c "<re-hash every manifest.json member and compare name/sha256/size>"
./bin/nomos inspect input/world > <scratch>/inspect.json      # exit 0
```

Result: all seven members MATCH their `manifest.json` rows; `inspect` exits 0.
Package digest `fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440`.

## B. Bit-exact reproduction of the failure

```
./bin/nomos run input/world --commands input/failing.commands --out <scratch>/repro.run \
  > <scratch>/repro.stdout.json                                # exit 1
cmp <scratch>/repro.run/<file> input/failing.run/<file>        # for all six bundle files
```

Result: exit 1; EK0804 `north_gate.access.unlock` is illegal while the machine is `closed`;
`committed_command_count` 1; `result_digest` `0989bf6abcafe8c3e7c00a824a54a6c4b5f5ebafcb856215584748ba5713e988`;
all six artifacts IDENTICAL to `input/failing.run`; stdout equal to
`input/forensics/failure.stdout.json` modulo the `artifacts`/`output` paths.

## C. Discriminating single-command probes

```
./bin/nomos command input/world --state input/failing.run/final-state.json \
  "unlock north_gate with credential/gaoler_key" --out <scratch>/probe1.run   # exit 1  EK0804
./bin/nomos command input/world --state input/failing.run/initial-state.json \
  "unlock north_gate with credential/wrong_key"  --out <scratch>/probe2.run   # exit 1  EK0805
./bin/nomos command input/world --state input/failing.run/initial-state.json \
  "unlock north_gate with credential/gaoler_key" --out <scratch>/probe3.run   # exit 0  completed
./bin/nomos command input/world --state input/failing.run/initial-state.json \
  "lock north_gate"                              --out <scratch>/probe4.run   # exit 1  EK0802
```

## D. Independent re-execution of the supplied failing bundle

```
./bin/nomos explain-transition input/failing.run north_gate --tick 1 --world input/world \
  > <scratch>/failing-tick-1.check.json                        # exit 0
```

Result: exit 0 and output equal to the packaged `input/forensics/north-gate-tick-1.json`.
Per decision 0009 this strictly reopens the bundle and re-executes every committed row, requiring
byte-identical states, logs, receipts and hashes.

## E. Building the repair

```
awk 'NR!=3' input/failing.commands > output/repaired.commands
diff input/failing.commands output/repaired.commands           # 3d2 (one deleted line)
sha256sum output/repaired.commands
```

`output/repaired.commands` = `31c84464a7e2d0d03397a92b32c76a21eccf39ecb94c0ca3605cf21fd4eb3719`, 149 bytes.

## F. Verifying the repair into a new directory

```
./bin/nomos run input/world --commands output/repaired.commands --out output/repaired.run \
  > output/repaired.stdout.json                                # exit 0
```

Result (`output/repaired.run/result.json`, `output/repaired.stdout.json`):

```
status                    completed
rejection_diagnostic      null
committed_command_count   5
input_package_digest      fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440
runtime_semantics_digest  0cdb454be9876dccdc05369e91fb9bdea017973d77f7803d23a921a969a1ef3c
first_state_hash          fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42
final_state_hash          3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc
result_digest             e13f4ce9339967e51a06353a1315fa82c0e5a45172073f48af02b1fe0d5a8e7a
```

State-hash chain: `fa7247d1…`, `b8eac772…`, `0545806c…`, `7843d53d…`, `2459a218…`, `3e06b963…`.
Ordinal 0 and ordinal 1 equal the failing run's two hashes, so the previously committed prefix is
preserved exactly.

Committed receipts: tick 1 `north_gate.access unlock locked→closed`; tick 2 `open closed→open`;
tick 3 `north_gate.ward unseal sealed→unsealed`; tick 4 `north_gate.combustion ignite cold→burning`
plus causal `north_gate.integrity intact→destroyed`; tick 5 `brazier_02.emission extinguish
lit→extinguished` with light-fact deltas to simulation, persistence and diagnostics.

Prefix-identity checks (python `json.load` comparison):

```
command-log row 0 identical to input/failing.run  : True
tick-1 causal receipt identical                   : True
initial-state.json bytes identical                : True
```

## G. Independent re-execution of the repaired bundle

```
./bin/nomos explain-transition output/repaired.run north_gate --tick 4 --world input/world \
  > <scratch>/repaired-north-gate-tick-4.json                  # exit 0
./bin/nomos explain-transition output/repaired.run brazier_02 --tick 5 --world input/world \
  > <scratch>/repaired-brazier-tick-5.json                     # exit 0
```

Both exit 0, i.e. the whole repaired bundle strictly reopens and re-executes byte-identically.
They reproduce the reference behaviour documented in `reference/explanations.md`: `north_gate` at
tick 4 (movement becomes traversable cost 1) and `brazier_02` at tick 5 (light fact goes false).

## H. Cleanup

```
rm -r output/scratch
```

No file was read, written, or probed outside `/workspace`; nothing outside `output/` was modified.
