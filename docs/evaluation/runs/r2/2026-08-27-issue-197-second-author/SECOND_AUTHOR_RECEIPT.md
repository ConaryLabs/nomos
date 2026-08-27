# Independent R2 scene-two author receipt

## Author and packet

- Author identity: `/root/r2_scene_two_author` (independent second-scene author)
- Model: OpenAI Codex, GPT-5
- Date: 2026-08-27
- Packet root: `/data/dev/src/nomos-issue-197/target/r2-second-scene-packet-f64d374`
- Delivered packet manifest SHA-256: `d5708087cf7967a420667c56a7b02ed052b7058ed8545af06e6771170003c948`
- Detached packet baseline commit (from `.nomos-packet-baseline`): `f64d374ee001ef0e51b66e3b2b4078ad6d1d770e`
- Detached packet baseline tree (from `.nomos-packet-baseline`): `2cae2ec952ccd21cff9c6e9e91424922551b35e5`
- Required external proof output: `/data/dev/src/nomos-r2-second-author-proof-f64d374`
- Proof browser: `/data/dev/home/.cache/ms-playwright/chromium_headless_shell-1520797764/chrome-headless-shell-linux64/chrome-headless-shell`

## Result

The independently authored `scene_two.json` is a canonical generic
`nomos.observed_scene@1` document. It has a 7x5 crop, three terrain layers
sharing cell `(3,2)` with all three distinct roles, living and dead actors,
controlled-only, hostile-only, protected-only, and all-positive actors, unique
actor fact tuples, and enabled and disabled actions. The signature proof passes
and differs from scene one on crop, terrain, actors, and actions. The delivered
compiler, viewer tests, build, and one-sample browser smoke all pass.

## Touched files

The complete touched-file set is exactly the eight permitted additions below;
no manifested packet file was modified:

```text
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/BROWSER_RECEIPT.json
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SCENE_SIGNATURES.json
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SECOND_AUTHOR_RECEIPT.md
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_1.png
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_2.png
fixtures/r2/plans/scene_two.json
fixtures/r2/scenes/scene_two.json
```

## Produced-file digests

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `fixtures/r2/scenes/scene_two.json` | 1490 | `21af593413804b9ece1df24f38f436ff2288de445589160b22506c98d53f227a` |
| `fixtures/r2/plans/scene_two.json` | 2748 | `1fd08cfb33d07f93a568e4bb337ebfbe8909a22a973d0f137139c92f0481e905` |
| `fixtures/r2/plans/scene_one.json` | 2305 | `717b91f3f35d815bfa9f9cc777b38f8a091f7a6339d786c57360e94ffe4c7699` |
| `apps/nomos-observed-viewer/src/catalog.mjs` | 4352 | `6259520fbf318ae0393ea4ae69649864acb154db4034d081435416be2ffa9323` |
| `SCENE_SIGNATURES.json` | 7077 | `6c8e38c78896bf5d5a166427f5f9567044dc00314dc4e65124173c57ccab50c4` |
| `BROWSER_RECEIPT.json` | 5330 | `fc2565b7b8dc940e35a0dbb205e2453d046db10cdea3d1c568c085cf70ec390d` |
| `evidence/scene_1.png` | 32545 | `27833755cea790f04353c930a2158044d4aab05c87989ee5751dc8dff66f5fb6` |
| `evidence/scene_2.png` | 38964 | `8846a2cfa68bbd40fea458f8c58fb136f363e26c2e37f1969c05c3ec81dbfe64` |
| `evidence/contact-sheet.png` | 70660 | `b76edbd9dd03fce5a99c074200ee7311bf87d5d2e5829c800170c129d00bf576` |

## Full semantic signatures

These are the complete signature objects emitted by
`docs/evaluation/r2-scene-signature.mjs`, including every normalized axis.

### Scene one (`fixtures/r2/scenes/scene_one.json`)

```json
{"sha256":"ef11771f3f8c210fdd8c9366e780ab720349a49dad88ae8dca969fcbe16c30d2","normalized":{"actions":[{"availability":"disabled","target_actor_ordinal":4},{"availability":"enabled","target_actor_ordinal":2}],"actors":[{"cell":{"x":1,"y":1,"z":0},"controlled":true,"hostile":false,"life_state":"living","protected":false},{"cell":{"x":1,"y":4,"z":0},"controlled":false,"hostile":false,"life_state":"dead","protected":true},{"cell":{"x":2,"y":2,"z":0},"controlled":true,"hostile":true,"life_state":"living","protected":true},{"cell":{"x":3,"y":1,"z":0},"controlled":false,"hostile":true,"life_state":"living","protected":false},{"cell":{"x":4,"y":3,"z":0},"controlled":false,"hostile":false,"life_state":"dead","protected":false}],"crop":{"height":6,"width":6},"terrain":[{"cells":[{"x":0,"y":0},{"x":1,"y":0},{"x":2,"y":2}],"role":"calm_ground"},{"cells":[{"x":1,"y":1},{"x":2,"y":2},{"x":3,"y":3}],"role":"traversable_route"},{"cells":[{"x":2,"y":2},{"x":4,"y":2},{"x":4,"y":3}],"role":"structure_footprint"}]}}
```

### Scene two (`fixtures/r2/scenes/scene_two.json`)

```json
{"sha256":"9afb46dc4d7ddb5b79cdcabd63b67d162b3c230aecfde9383e038128572f0f3d","normalized":{"actions":[{"availability":"disabled","target_actor_ordinal":4},{"availability":"enabled","target_actor_ordinal":3},{"availability":"enabled","target_actor_ordinal":5}],"actors":[{"cell":{"x":0,"y":1,"z":0},"controlled":true,"hostile":false,"life_state":"dead","protected":false},{"cell":{"x":1,"y":3,"z":0},"controlled":false,"hostile":false,"life_state":"living","protected":false},{"cell":{"x":2,"y":4,"z":0},"controlled":false,"hostile":false,"life_state":"dead","protected":true},{"cell":{"x":3,"y":2,"z":0},"controlled":true,"hostile":true,"life_state":"living","protected":true},{"cell":{"x":4,"y":0,"z":0},"controlled":false,"hostile":false,"life_state":"dead","protected":false},{"cell":{"x":6,"y":1,"z":0},"controlled":false,"hostile":true,"life_state":"living","protected":false}],"crop":{"height":5,"width":7},"terrain":[{"cells":[{"x":0,"y":0},{"x":1,"y":0},{"x":3,"y":2},{"x":5,"y":4}],"role":"calm_ground"},{"cells":[{"x":1,"y":1},{"x":3,"y":2},{"x":3,"y":3},{"x":5,"y":3}],"role":"structure_footprint"},{"cells":[{"x":2,"y":1},{"x":3,"y":2},{"x":4,"y":2},{"x":6,"y":4}],"role":"traversable_route"}]}}
```

## All eight axis digests

| Axis | Scene one | Scene two | Different |
| --- | --- | --- | :---: |
| `crop` | `6bc0ba95810359c49e83d612501c8a9319e380e7f227ad72d0a9529c4e02d175` | `8c1ad689b143df6bdb2bc4c7e08f4b0f60a0bf114dc1a4931f641fa29662abeb` | yes |
| `terrain` | `a7c4a7cca62253f9fb9aa62412297184a9413a4cc480a1ce39590aff2cd34c3f` | `752bc486adc6b77597a1849f23b9de1dff845bfcd6fbf08f7e5937e2eab8d9be` | yes |
| `actors` | `a170f9a01b67fb0b4681aca6fda680bfa8695d423d8db9c6e382ee4b8c53d779` | `b1e10755b6f8b523ad084e5b8a29cf153be72ec52bec3eb01ebdf628aa3e3181` | yes |
| `actions` | `d9498c215739ed6ee5f3dce9d0ce21a19e35af26016f046e482c0ab785159705` | `28c07a530e8cf68e8aa6a3e4b2997aa9c4f89f178ce83210262e6d24773d47f1` | yes |

Full semantic signature SHA-256 values are scene one
`ef11771f3f8c210fdd8c9366e780ab720349a49dad88ae8dca969fcbe16c30d2` and
scene two `9afb46dc4d7ddb5b79cdcabd63b67d162b3c230aecfde9383e038128572f0f3d`.

## Proof commands and results

The exact mandated proof sequence was run from the packet root:

```text
docs/evaluation/r2-second-scene-packet/verify.sh .
bin/nomos-observed-scene compile --input fixtures/r2/scenes/scene_two.json --out fixtures/r2/plans/scene_two.json
node docs/evaluation/r2-scene-signature.mjs fixtures/r2/scenes/scene_one.json fixtures/r2/scenes/scene_two.json
node --test apps/nomos-observed-viewer/test/*.test.mjs docs/evaluation/r2-scene-signature.test.mjs
export NOMOS_R2_PROOF_OUT=/data/dev/src/nomos-r2-second-author-proof-f64d374; node apps/nomos-observed-viewer/build.mjs --plan fixtures/r2/plans/scene_one.json --plan fixtures/r2/plans/scene_two.json --out "$NOMOS_R2_PROOF_OUT/dist" --receipt "$NOMOS_R2_PROOF_OUT/build.json"
export NOMOS_R2_PROOF_OUT=/data/dev/src/nomos-r2-second-author-proof-f64d374; export CHROME_BIN=/data/dev/home/.cache/ms-playwright/chromium_headless_shell-1520797764/chrome-headless-shell-linux64/chrome-headless-shell; node apps/nomos-observed-viewer/smoke/smoke.mjs --dist "$NOMOS_R2_PROOF_OUT/dist" --out "$NOMOS_R2_PROOF_OUT/smoke" --samples 1
cp /data/dev/src/nomos-r2-second-author-proof-f64d374/smoke/receipt.json docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/BROWSER_RECEIPT.json
cp /data/dev/src/nomos-r2-second-author-proof-f64d374/smoke/contact-sheet.png docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png
cp /data/dev/src/nomos-r2-second-author-proof-f64d374/smoke/scene_1.png docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_1.png
cp /data/dev/src/nomos-r2-second-author-proof-f64d374/smoke/scene_2.png docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_2.png
docs/evaluation/r2-second-scene-packet/audit-author-output.sh .
```

Results: packet verification `PASS` (42 manifested files; manifest digest as
above); final compile exit 0 with no stdout/stderr; signature outcome `pass`
with all four axes different; 35 Node tests passed and 0 failed; build
`NOMOS_OBSERVED_BUILD PASS files=14 bytes=805600`; browser smoke
`NOMOS_OBSERVED_SMOKE PASS scenes=2 samples=2 external=0`.

The first compile attempt reported `OS0103` because the new source had a
trailing LF from file creation; removing that formatting byte from the new
file yielded canonical input. A first build invocation used an inline
environment assignment that expanded too late and reported `EACCES` for
`/dist`; exporting the required variable and rerunning passed. These attempts
did not modify manifested files. The final audit was run after all eight
permitted additions were present.

## Input provenance attestation

I authored scene two from the frozen packet only. I consulted no parent
repository, adopter repository, external project, adopter payload, target
frame, palette, image, prose, schema, coordinate set, mechanic, or other
external content/design payload, and copied none. Node, the delivered compiler,
and the preinstalled Chromium executable were used only as proof tools. No
packet manifest input was edited.
