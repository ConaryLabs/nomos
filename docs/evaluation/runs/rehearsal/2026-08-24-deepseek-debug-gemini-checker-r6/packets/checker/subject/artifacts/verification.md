# Verification record

Subject: `/workspace/input/failing.commands` → `/workspace/input/failing.run`
(one committed command, rejection `EK0804`). Repair: content-level command
script (`/workspace/output/repaired.commands`).

## Exact commands run (in order, all from `/workspace`, tool `/workspace/bin/nomos`)

1. Reproduce the failing run against the same verified world:

```bash
./bin/nomos run input/world --commands input/failing.commands --out output/probe/repro-failing2.run
```

Result (stdout, redirected to `output/probe/repro-failing2.stdout.json`):
`status: "rejected"`, `committed_command_count: 1`,
`diagnostics: [{code: "EK0804", message: "\`north_gate.access.unlock\` is illegal while the machine is \`closed\`", repairs: []}]`,
`first_state_hash: fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42`,
`final_state_hash: b8eac7726d570b7545d9b7e1a9f3ee838f93c095f2a294070f4976444d94952b`,
`result_digest: 0989bf6abcafe8c3e7c00a824a54a6c4b5f5ebafcb856215584748ba5713e988`
(byte-identical to the supplied `input/failing.run/result.json`), process exit
`1`, stderr empty.

2. Build the repaired command script (delete the duplicate second `unlock`
line; keep the same 5-request order and schema header):

```bash
printf 'schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nopen north_gate\nunseal north_gate\nignite north_gate\nextinguish brazier_02\n' > output/repaired.commands
```

`sha256sum output/repaired.commands` =
`31c84464a7e2d0d03397a92b32c76a21eccf39ecb94c0ca3605cf21fd4eb3719`.

File content:

```
schema nomos.command_script@1
unlock north_gate with credential/gaoler_key
open north_gate
unseal north_gate
ignite north_gate
extinguish brazier_02
```

Diff against the failing script (only one deleted line):

```diff
--- input/failing.commands
+++ output/repaired.commands
@@ -1,6 +1,5 @@
 schema nomos.command_script@1
 unlock north_gate with credential/gaoler_key
-unlock north_gate with credential/gaoler_key
 open north_gate
 unseal north_gate
 ignite north_gate
 extinguish brazier_02
```

3. Verify the repaired script into a new directory under `/workspace/output`:

```bash
./bin/nomos run input/world --commands output/repaired.commands --out output/verified.run
```

Result (stdout redirected to `output/probe/verified.stdout.json`):
`status: "completed"`, `committed_command_count: 5`, process exit `0`,
`first_state_hash: fa7247d115305ccbc6f570167919142e94e9ba423b43762be1774e057e50fe42`,
`final_state_hash: 3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc`,
`input_package_digest: fd2dca0e9cf8b352a474c414291d9a96c19758f518c1cf70fd4fa46293710440`,
`runtime_semantics_digest: 0cdb454be9876dccdc05369e91fb9bdea017973d77f7803d23a921a969a1ef3c`,
`result_digest: e13f4ce9339967e51a06353a1315fa82c0e5a45172073f48af02b1fe0d5a8e7a`.

Output tree: `output/verified.run/` containing exactly
`causal-receipts.json`, `command-log.json`, `final-state.json`,
`initial-state.json`, `result.json`, `state-hashes.json`.

4. Check the committed command log (5 rows, ordinals 0–4):

```bash
cat output/verified.run/command-log.json
```

Requests/result states: unlock/gaoler_key (`b8eac772...`), open
(`0545806c...`), unseal (`7843d53d...`), ignite (`2459a218...`),
extinguish (`3e06b963...`). Hash chain in `output/verified.run/state-hashes.json`
matches every row: ordinals 0–5 =
`fa7247d1…, b8eac772…, 0545806c…, 7843d53d…, 2459a218…, 3e06b963…`.

5. Inspect the final state (tick 5):

```bash
cat output/verified.run/final-state.json
```

Machines: `brazier_02.emission=extinguished`, `north_gate.access=open`,
`north_gate.combustion=burning`, `north_gate.integrity=destroyed`,
`north_gate.ward=unsealed`; `tick: 5`;
`state_hash: 3e06b9639cdef2ccd1c2795ae4b5a6581138de79ec04296d381b2334ab05bfdc`.

6. Independently re-open and re-execute the bundle via the read-only
explanation path (decision-0009 sequence), then render the target receipts:

```bash
./bin/nomos explain-transition output/verified.run north_gate --tick 4 --world input/world
./bin/nomos explain-transition output/verified.run brazier_02 --tick 5 --world input/world
```

- Tick 4 `north_gate`: `status: "completed"`, `package_digest fd2dca0e...`,
  `run_result_digest e13f4ce9...`, receipt transitions =
  `ignite` local `cold -> burning` and `apply_damage` causal
  `intact -> destroyed`; movement disposition of `north_gate` is traversable
  (cost 1) before/after.
- Tick 5 `brazier_02`: `status: "completed"`, receipt transition =
  `extinguish` local `lit -> extinguished`; claim removed
  `brazier_02.emission#emits_light`; `projection_deltas` update
  `emits_light` fact for diagnostics, persistence, and simulation.

7. Exclusion probes (all within `/workspace/output/probe`):

```bash
printf 'schema nomos.command_script@1\nunlock north_gate with credential/nonexistent\n' > output/probe/bad-cred2.commands
./bin/nomos run input/world --commands output/probe/bad-cred2.commands --out output/probe/repro-badcred2.run
# -> EK0805 "command argument does not match the compiled input requirement", committed 0 (not EK0804)

printf 'schema nomos.command_script@1\nunlock north_gate\n' > output/probe/bad-grammar2.commands
./bin/nomos run input/world --commands output/probe/bad-grammar2.commands --out output/probe/repro-badgrammar.run
# -> EK0805, committed 0 (not EK0804; argument-mismatch class, not grammar/parse)

printf 'schema nomos.command_script@1\nopen north_gate\n' > output/probe/open-first2.commands
./bin/nomos run input/world --commands output/probe/open-first2.commands --out output/probe/repro-openfirst.run
# -> EK0804 "`north_gate.access.open` is illegal while the machine is `locked`", committed 0
# (proves open-first ordering is also illegal; unlock must come first)
```

8. World/package integrity checks (read-only):

```bash
./bin/nomos inspect input/world
./bin/nomos explain-entity input/world brazier_02
```

Both complete; `inspect` reports `manifest_digest fd2dca0e...`, world IR
`nomos.world_ir@2`, and the machine definitions (incl. `north_gate.access`
initial `locked`; `unlock: locked -> closed`).

## Artifact digest cross-check (SHA-256, independent of `result.json`)

| `output/verified.run` file | SHA-256 |
|---|---|
| `causal-receipts.json` | `6ee9f7af8cf382ef1f3cc11c51ee608026310508ee9408129759e9e87c441d89` |
| `command-log.json` | `ef30d6a25026a162111e3e180a65a0e536f94b0b8ef0d73027bdbedd865ad6d9` |
| `final-state.json` | `fdeb5db8257ba41927de65fb50b08edc54cace49ae332ced327f4a8cad051a4d` |
| `initial-state.json` | `9d6cf31816a8a1f7887858ea9d8a69307b2d67b4c0cfa6489fb8b08658bcfa6b` |
| `state-hashes.json` | `1989700685bf430aca5f1394ef279c9e720011e4834fd48df36e8ed275595320` |

These exactly equal the `artifacts` binding rows in
`output/verified.run/result.json`. `initial-state.json` digest
`9d6cf31816...` is also the digest of the failing run's
`input/failing.run/initial-state.json`, confirming both runs start from the
same package-derived state.

## Conclusion of verification

- `output/repaired.commands` (sha256 `31c84464...`) = the failing command
  content with the single duplicate `unlock` request removed.
- `output/verified.run` was freshly created under `/workspace/output`; the
  run completed with 5/5 commands, tick 5, the expected final machine states,
  and a valid typed hash chain and receipt/causal evidence.
- The read-only explanation path re-opened and re-executed the bundle without
  modification and rendered the tick-4 `north_gate` (ignite + causal damage)
  and tick-5 `brazier_02` (extinguish) receipts.
- No world, source, projection, binary, or input run file was modified.
