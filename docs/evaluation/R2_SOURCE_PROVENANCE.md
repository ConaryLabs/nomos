---
title: R2 per-file source provenance register
status: R2-1 implementation register
date: 2026-08-27
issue: 195
authority: R2.md revision 1 section 4
---

# R2 per-file source provenance register

This inventory covers every current regular file in the R2 carrier crate,
R2 fixtures, and R2-1 evaluation-source scope, plus the three R1 Three.js
vendor files referenced by the epoch contract. The R2 browser application and
browser-produced visual-evidence scope do not exist in R2-1 and therefore add
no rows yet.

`R2_SOURCE_PROVENANCE.md`, `r2-source-provenance.sh`, and the producing packet
and receipts beneath `docs/evaluation/runs/r2/` are control evidence. Per the
contract's self-binding rule they are bound by the candidate commit/tree and
final receipt rather than recursively inventorying themselves. Every other
top-level R2-named evaluation source is in the table and in the checker's
fail-closed expected-path calculation.

The `project_mit` disposition binds root `LICENSE`, SHA-256
`93b2b15b4b31c04481d1683e34fd1f796aacc2181899d9e661041e20d0f4ccbf`.
The `three_mit_preserved` disposition binds the exact accepted Three.js MIT
license bytes in the first inventory row.

## Inventory

| Path | SHA-256 | Origin class | Producing receipt | License disposition |
| --- | --- | --- | --- | --- |
| `apps/nomos-viewer/vendor/three/LICENSE` | `8b378ebe60e2fe500158cb0ac71cb5e8b7d92953c2abcc63a0eb90499653b5bc` | `r1_vendor_reuse` | `apps/nomos-viewer/vendor/MANIFEST.json` | `three_mit_preserved` |
| `apps/nomos-viewer/vendor/three/three.core.min.js` | `05b2609338c76cd65daf74f3ac515bc9a5045e1b3b33edc07d8c9bd55250fa90` | `r1_vendor_reuse` | `apps/nomos-viewer/vendor/MANIFEST.json` | `three_mit_preserved` |
| `apps/nomos-viewer/vendor/three/three.module.min.js` | `86bcee248b64f44bcfc23c331ae74619061957d59cab040171dcb6fb5900beb6` | `r1_vendor_reuse` | `apps/nomos-viewer/vendor/MANIFEST.json` | `three_mit_preserved` |
| `crates/nomos-observed-scene/Cargo.toml` | `43e7697cacf8828829b713a6a64f9c3732b1ca65aeb008f903bfc91d8d15d3d9` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/bin/nomos-observed-scene.rs` | `43fbe7b67656e9a64dcc5717d0f808d502894895057a465920d1ac94ddde973f` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/command.rs` | `3e9905d00fee6aa7ba7527330f020f3907d5073bec175723bb7a1d669794c4f5` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/diagnostic.rs` | `08b806c01d9e40acef3597933c49c395afec6c12d1a1b04fc2e6a472b7d25005` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/input.rs` | `aaf9fb89d1cb2098273600df1c752bcc732facf53d33ec84ae6403646730408c` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/input/validate.rs` | `10b4b45acada8c4d9392109bf0e79cd960cdb6c01e18b8a9cea4de19297ca992` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/json.rs` | `10272042c80bebd66712e094ab1aa85cc600557b6dfe930290e8872383feb9bd` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/lib.rs` | `f92ae86940c706c8af123e532f8635972d552aed628fe5d5aa2f1e37519f11d6` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/plan.rs` | `c2d76105436e756cbbfa289ff2cee0fa5947d6230d32fc223e37b616fe21502a` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/plan/validate.rs` | `f663d34f3a0cd8cf72fa183293334e51944b172c8dc6bd4c04835f3a5f2fdd81` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/value.rs` | `d93eb83a36ed72a161172f5d1bf7b939fc03c352f2ee5a6bb672ada61142c7e5` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/command.rs` | `22a373c64f93fdb3ad4b86308ed4a1f285bc9c1504d37b478811622f2958d2d9` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/common/mod.rs` | `bca5caa334e58dd6cefe2f273d64e419886028d14125d96632357be7bd228b97` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/determinism.rs` | `521dda5312bcf97a8773ca35dbe96377a043f29b8c87997ba3b8d5079876f5f3` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/diagnostics.rs` | `b46abd05aa3166d397bdde33d47faaa601c6ca8261bedd007058f7a5af9dbf41` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/documents.rs` | `5ad72c665b744202d2ca93ed67fa30ee5e494b168a5ef73afd12ee754bea889c` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/limits.rs` | `d21612736ed407e228fec433b779f39c8b29cab306d50c06c5d19eccc2aea0cd` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/precedence.rs` | `5d119f87393593e567b48434ff2d4ddea60effa03074783a5232b214b1cfb25c` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/validation.rs` | `b7186681c440cae348795e7ec663b605ffc18400be14894d296701fa2a2d5501` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/R2_SCHEMA_OWNERSHIP.md` | `948dae85d6cbcedc0ffc55629bfbf633a8b934bb99e23a84a11bada43fe11531` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/generate-r2-maximum.mjs` | `d0aa0b54d829c5a2fe2c64fba4d8c2180e6cd3b98000ef22b13096db149b80f2` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/measure-r2-compile.mjs` | `7c76a885d133968bcd5fda8cf9b71bb175986c4c5697224fe281f749cc0ba4a2` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-maximum.test.mjs` | `fefbb3428cce888f0561828471501170b12b89fff796fa1e8e38d250be0e87ba` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-schema-ownership.sh` | `b563a52ed9808ed66251cada59faa70de0101bc960e2e5375cf8efb094f77510` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-source-provenance.test.sh` | `5787801136d6d571808af5df14b93f64e255aca927a0d694c4da94ca7db884e8` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `fixtures/r2/maximum-observed-scene.json` | `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `fixtures/r2/plans/scene_one.json` | `717b91f3f35d815bfa9f9cc777b38f8a091f7a6339d786c57360e94ffe4c7699` | `compiler_produced` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/COMPILER_RECEIPT.md` | `project_mit` |
| `fixtures/r2/scenes/scene_one.json` | `e4c04e7d6806aaba3e3ba9e0b94ba761442e00d5dd17a14581c29e1de22c41aa` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
