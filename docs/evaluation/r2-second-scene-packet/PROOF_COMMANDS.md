# R2 scene-two packet proof commands

Run from the packet root. `NOMOS_R2_PROOF_OUT` must name a new path outside the
packet. `CHROME_BIN` may name a preinstalled Chromium-family proof tool.

```text
docs/evaluation/r2-second-scene-packet/verify.sh .
bin/nomos-observed-scene compile --input fixtures/r2/scenes/scene_two.json --out fixtures/r2/plans/scene_two.json
node docs/evaluation/r2-scene-signature.mjs fixtures/r2/scenes/scene_one.json fixtures/r2/scenes/scene_two.json
node --test apps/nomos-observed-viewer/test/*.test.mjs docs/evaluation/r2-scene-signature.test.mjs
node apps/nomos-observed-viewer/build.mjs --plan fixtures/r2/plans/scene_one.json --plan fixtures/r2/plans/scene_two.json --out "$NOMOS_R2_PROOF_OUT/dist" --receipt "$NOMOS_R2_PROOF_OUT/build.json"
node apps/nomos-observed-viewer/smoke/smoke.mjs --dist "$NOMOS_R2_PROOF_OUT/dist" --out "$NOMOS_R2_PROOF_OUT/smoke" --samples 1
docs/evaluation/r2-second-scene-packet/audit-author-output.sh .
```

Capture the signature command's JSON as `SCENE_SIGNATURES.json`. Copy the
smoke receipt and three PNG files to the exact permitted evidence paths before
the final audit. The one-sample smoke proves independent renderability; the
primary author performs and retains the contract's final ten samples per scene.
