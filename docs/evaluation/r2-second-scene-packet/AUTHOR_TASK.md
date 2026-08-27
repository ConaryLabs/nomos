# Independent R2 scene-two author task

You are the independent author for Nomos R2-2. Work only inside the delivered
packet and use only its files as content or design input. Do not inspect the
parent repository, another project, an external image, or an external content
payload. Host executables such as Node and Chromium are proof tools, not design
inputs.

First run `docs/evaluation/r2-second-scene-packet/verify.sh .`. Then author one
new generic canonical `nomos.observed_scene@1` document at
`fixtures/r2/scenes/scene_two.json`. It must satisfy `R2.md` revision 1 and:

- contain a cell shared by three layers with distinct terrain roles;
- contain both life states;
- contain one controlled-only, one hostile-only, and one protected-only actor;
- contain one actor on which all three positive booleans coexist;
- contain both action availabilities;
- give every actor a unique `(cell,life_state,controlled,hostile,protected)`
  tuple; and
- differ from scene one independently in crop, normalized terrain rows,
  normalized actor tuples, and normalized action rows.

Use the delivered release compiler to create
`fixtures/r2/plans/scene_two.json`. Do not edit that plan. Run the delivered
signature, build, and browser proof exactly as described by `PROOF_COMMANDS.md`.
The scene must compile and render without changing any manifested file.

Your only permitted packet additions are:

```text
fixtures/r2/scenes/scene_two.json
fixtures/r2/plans/scene_two.json
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/BROWSER_RECEIPT.json
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SCENE_SIGNATURES.json
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SECOND_AUTHOR_RECEIPT.md
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_1.png
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_2.png
```

The author receipt records your identity/model, delivered manifest digest,
detached commit/tree from `.nomos-packet-baseline`, every command, the exact
touched-file list, both full semantic signatures and all eight axis digests,
the plan/catalog/screenshot/contact-sheet digests, and an attestation that no
external project or payload was consulted or copied. End by running
`audit-author-output.sh .` and report any red result without modifying source.
