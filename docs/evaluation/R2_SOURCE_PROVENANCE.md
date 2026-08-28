---
title: R2 per-file source provenance register
status: R2 final-proof candidate implementation register
date: 2026-08-27
issue: 199
authority: R2.md revision 2 section 4
---

# R2 per-file source provenance register

This inventory covers every current regular file in the R2 carrier crate,
isolated browser application, R2 fixtures, R2-specific evaluation source, and
committed visual-evidence scope, plus the three R1 Three.js vendor files
referenced by the epoch contract. The independent scene and browser evidence
entered unchanged from the hash-frozen author packet.

`R2_SOURCE_PROVENANCE.md`, `r2-source-provenance.sh`, and producing receipts
beneath `docs/evaluation/runs/r2/` are control evidence. Per the contract's
self-binding rule they are bound by the candidate commit/tree and final receipt
rather than recursively inventorying themselves. Every other R2-specific
evaluation source and packet-manifest path is in the table and in the checker's
fail-closed expected-path calculation.

The `project_mit` disposition binds root `LICENSE`, SHA-256
`93b2b15b4b31c04481d1683e34fd1f796aacc2181899d9e661041e20d0f4ccbf`.
The `three_mit_preserved` disposition binds the exact accepted Three.js MIT
license bytes in the first inventory row.

## Inventory

| Path | SHA-256 | Origin class | Producing receipt | License disposition |
| --- | --- | --- | --- | --- |
| `apps/nomos-observed-viewer/PUBLIC_FILES` | `7f53e4db82e1faed7d523be4165cfb024b57d12e2ebb9cf2b0f356fd27560da9` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/README.md` | `7195b279cbf644cb2535c40dabaea91eb99e138787c651e894acda827ed75cef` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/SOURCE_MANIFEST` | `84585a60921f3281920fdccc635be15ac61cfe3d9b8a9868cea352f7dd58361c` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/build.mjs` | `8f80ed101ed7c18b77880ffdd9152967e70b59742fd7fdddfecb84bf2d7c0c64` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/index.html` | `476f1e8cc14d189d82fa5cbade57afb18219cf099f08609f06949a9e7d380126` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/smoke/cdp.mjs` | `c1e1db651d3dec96a25c320ec6e0bde386ce906e95230309d24eaba97ab547d2` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/smoke/chrome.mjs` | `866a9dd013e40508f133f656ef8a0dd470410f72c57e06199f7888c1b24a595c` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/smoke/server.mjs` | `c7a8d36c247622593c1f2b1d009a6b708f59c98e8e5a7f28993d662bf80d518d` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/smoke/smoke.mjs` | `8eb17b1b172861a4a48db4297131de627b6ca88bd141cdf5fd0686e1a00cb41b` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/src/canonical.mjs` | `03565a2b7123c0ef59d8ba136a9f52d10c2dac989da829708058b9415bd4cf9d` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/src/catalog.mjs` | `6259520fbf318ae0393ea4ae69649864acb154db4034d081435416be2ffa9323` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/src/plan.mjs` | `0648f00d1929eb02b73d135656e0fdba45956fc4ef65ce95801b0b3b2e2ba200` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/src/render.mjs` | `758a2901c3218e5df99b81b0c09615a52afca2be4c6b1b61e4e25d5849eeacaa` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/src/runtime.mjs` | `04dfe115148fff0e024a14f20f0b4ada3511cefad699e9f7848d18825d97f946` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/src/ui.mjs` | `5fb2e5429fdd33c90e419f79344769ac9bea47da867eeb6a5808bf4c0e0bbebc` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/style.css` | `4c8be9d82c0d3a39ad72364bc09d6e37c414141775ebc20edd9f45c61e7fe12b` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/build.test.mjs` | `d70ff27d7b818bc094e5d023557485cce9f1974929231440ee314330c4b93534` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/canonical.test.mjs` | `b87994868e0d2d6c134b1875c996ca0e1a0e5433b69dce9d3d191613eb60850b` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/catalog.test.mjs` | `3e00fccfc751a13d611027b153ae953fdc210826b04087162cf95a4477454058` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/helpers.mjs` | `ffb6f369cb14d066098464d93b43a40c4a59a699124f2fb8351b4e04755da71c` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/manifests.test.mjs` | `ba12837b63edab3a0c707d685d20e79b0c2a37dcd8c7800e7d24de92f0bebb69` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/plan.test.mjs` | `ba6845a4747ae0040983c279a8e5b4ef08481125a78c8a7498880c989ce26b76` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/render.test.mjs` | `c9fcaed1864d59b98c51c866cba5fa4329b85c829684e6e8f540c0df873736dd` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/smoke.test.mjs` | `36cb0ec99afb41dddccdb6b81e21053b7a74604da4e73d3f84b3a83f36cc120b` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/three-stub.mjs` | `7a66b235f580e200608e7f149c27afbc1ee7783781cc3b14119ed6feb5ed4d1f` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-observed-viewer/test/ui.test.mjs` | `ce2034b0b7097a4c57b3954836b2d5bc069ac1e64440e274d313b2c0ad4c94cf` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `apps/nomos-viewer/vendor/three/LICENSE` | `8b378ebe60e2fe500158cb0ac71cb5e8b7d92953c2abcc63a0eb90499653b5bc` | `r1_vendor_reuse` | `apps/nomos-viewer/vendor/MANIFEST.json` | `three_mit_preserved` |
| `apps/nomos-viewer/vendor/three/three.core.min.js` | `05b2609338c76cd65daf74f3ac515bc9a5045e1b3b33edc07d8c9bd55250fa90` | `r1_vendor_reuse` | `apps/nomos-viewer/vendor/MANIFEST.json` | `three_mit_preserved` |
| `apps/nomos-viewer/vendor/three/three.module.min.js` | `86bcee248b64f44bcfc23c331ae74619061957d59cab040171dcb6fb5900beb6` | `r1_vendor_reuse` | `apps/nomos-viewer/vendor/MANIFEST.json` | `three_mit_preserved` |
| `crates/nomos-observed-scene/Cargo.toml` | `43e7697cacf8828829b713a6a64f9c3732b1ca65aeb008f903bfc91d8d15d3d9` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/bin/nomos-observed-scene.rs` | `43fbe7b67656e9a64dcc5717d0f808d502894895057a465920d1ac94ddde973f` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/command.rs` | `3e9905d00fee6aa7ba7527330f020f3907d5073bec175723bb7a1d669794c4f5` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/diagnostic.rs` | `08b806c01d9e40acef3597933c49c395afec6c12d1a1b04fc2e6a472b7d25005` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/input.rs` | `aaf9fb89d1cb2098273600df1c752bcc732facf53d33ec84ae6403646730408c` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/input/validate.rs` | `6dbeedf74ae9ba44a2ba141d8bc806202df3e47dd7fe31ab57f016fe3f309c2b` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/json.rs` | `10272042c80bebd66712e094ab1aa85cc600557b6dfe930290e8872383feb9bd` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/lib.rs` | `f92ae86940c706c8af123e532f8635972d552aed628fe5d5aa2f1e37519f11d6` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/plan.rs` | `5ac1e208c0560d4c606897bf5bec0eef320fb1e4b0e05728e38e756708bed4f1` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/plan/validate.rs` | `f663d34f3a0cd8cf72fa183293334e51944b172c8dc6bd4c04835f3a5f2fdd81` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/src/value.rs` | `d93eb83a36ed72a161172f5d1bf7b939fc03c352f2ee5a6bb672ada61142c7e5` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/command.rs` | `22a373c64f93fdb3ad4b86308ed4a1f285bc9c1504d37b478811622f2958d2d9` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/common/mod.rs` | `bca5caa334e58dd6cefe2f273d64e419886028d14125d96632357be7bd228b97` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/determinism.rs` | `521dda5312bcf97a8773ca35dbe96377a043f29b8c87997ba3b8d5079876f5f3` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/diagnostics.rs` | `b46abd05aa3166d397bdde33d47faaa601c6ca8261bedd007058f7a5af9dbf41` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/documents.rs` | `4de5fff383541196dd2667281a3a2f63969f136e0aae91b570b34a6ced56c30f` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/limits.rs` | `d21612736ed407e228fec433b779f39c8b29cab306d50c06c5d19eccc2aea0cd` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/precedence.rs` | `5211810f7634eedd576ad304b58eb167b382d067fd3a4fee537a0d0d33da0b99` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `crates/nomos-observed-scene/tests/validation.rs` | `b7186681c440cae348795e7ec663b605ffc18400be14894d296701fa2a2d5501` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/R2_SCHEMA_OWNERSHIP.md` | `948dae85d6cbcedc0ffc55629bfbf633a8b934bb99e23a84a11bada43fe11531` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/generate-r2-maximum.mjs` | `d0aa0b54d829c5a2fe2c64fba4d8c2180e6cd3b98000ef22b13096db149b80f2` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/measure-r2-compile.mjs` | `7c76a885d133968bcd5fda8cf9b71bb175986c4c5697224fe281f749cc0ba4a2` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-adopter-neutrality.sh` | `27f15ba8a211e36f383d49922444eced104dbf63fcdf918ff362fc01073ce3d8` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-adopter-neutrality.test.sh` | `c90458bd2b6977ed9f4f8cd6e173d19590170fb80cfff3c0e661ce034a471144` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-complete-proof-disk-plants.sh` | `3dadb69e6de4ed73d7a9286dadd7b2e453b7dbeb25dd6e3759c1263398b947f4` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-complete-proof-lib.sh` | `f307d5bf5dd4a71dc5c0e0a4cb63b8aef0e7cb1e69e8a48e56d7dad0af9e0fd3` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-complete-proof-process.mjs` | `3646a9593eb4d0803c68a8436c593b80e7d1fbae47b74b3a7612e92a116f7a94` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-complete-proof-process.test.mjs` | `9e0849f898ea3c35877901c25e73101d06e5c985aa9a2a877a5ef8fe8e3556c7` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-complete-proof-receipt.mjs` | `7644962ab4fea08fa3a36f541363a53a17c9c596d88470405155052f1c178c91` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-complete-proof-receipt.test.mjs` | `5c07ba1118d3bb388a380eb8073cc143ae77662a43149d87871cc30762d360ff` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-complete-proof.sh` | `d8d950df24ce0f7692a40ba76b9684b10a040252a8669c1e4c14c1504a72a6aa` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-complete-proof.test.sh` | `1ba06823e38132e3b081a619d541b77fcec83ae5f40e1a64bbbc58e24b63beba` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-disk-control-lib.sh` | `b901207a647a6d594ed2cfb104491f08ba7de4aa21a6721fbe6583eb99733f17` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-disk-history-plant.sh` | `5a4466394f63b9c49eda94b49d64db4e77632ca0055a58b90485bd619dbe5112` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-disk-lane-plants.sh` | `4036a58b3aa97bd70317f6bf541ee73221ffb6005ac6ee68c8b560efe225e634` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-disk-overload-plant.sh` | `7b069218b44366230f5ecb2c773272b132a548996587aff5c3f266112c529537` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-disk-sampler-lib.sh` | `c12f1523d961bcabc40e070930e0216025ab7dce6e7318b410740121610900aa` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-disk-slot-race-plants.sh` | `afffb3db28453b65c6d3bdc42aac6c10c05ccabdc9bd728fd4ae7c83ed2025ae` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-disk-terminal-order.test.sh` | `0a5cc815d741703b085e7c294791f5c1f936454d6ad96d37445d09d35dafc940` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-maximum.test.mjs` | `fefbb3428cce888f0561828471501170b12b89fff796fa1e8e38d250be0e87ba` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-procfs-read-plants.sh` | `b2edca1278092f34563480c523904445d9579e6527acb960af6cc25fa936a1b2` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-scene-signature.mjs` | `5aa86d8e5b2b86de4a83954c96d737231e3de43e4275485dff4d82e910eea8e2` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-scene-signature.test.mjs` | `11e092d4414c4fe5f6c25c256d0ac196d7f7a57bc792d5380a878636c8ea9538` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-schema-ownership-plants.sh` | `3d445c11ff19a5e6f74dcef6ae33acee68dc5de4a236fb3e96d53dc9aa38a8b4` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-schema-ownership.sh` | `b563a52ed9808ed66251cada59faa70de0101bc960e2e5375cf8efb094f77510` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-second-scene-packet.test.sh` | `2cdbfd40f1377e164ddf3accf24b26623fd63cfb5721479a28b3bb5d887154f4` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-second-scene-packet/AUTHOR_TASK.md` | `d4c3ac919fc5e25bed4c406a4b58e44a29cadede6996c2e7065843cb9c056b3c` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-second-scene-packet/MANIFEST.sha256` | `d5708087cf7967a420667c56a7b02ed052b7058ed8545af06e6771170003c948` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-second-scene-packet/PROOF_COMMANDS.md` | `968ee779f348c8b75301f456442c8c23042fead96c82b046223c56e7bbfe0f32` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-second-scene-packet/assemble.sh` | `645857754dd98e2024a7c154fe0d8c22251a43e574730017016011ba7136a91d` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-second-scene-packet/audit-author-output.sh` | `7952c431858ec71029e7e4ce59fd5aa119e6174214e45e579c4ee1df3b452188` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-second-scene-packet/verify.sh` | `e8cd7ebaf74ffdef5a5ed346e799427a16ae1157922c1e5704fd9465fb74e5d2` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/r2-source-provenance.test.sh` | `ad2655e93de94b203bc2417347aa02d5ffb3ed86e4c8f1203f0c6944f721523b` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SCENE_SIGNATURES.json` | `6c8e38c78896bf5d5a166427f5f9567044dc00314dc4e65124173c57ccab50c4` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SECOND_AUTHOR_RECEIPT.md` | `project_mit` |
| `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png` | `b76edbd9dd03fce5a99c074200ee7311bf87d5d2e5829c800170c129d00bf576` | `browser_produced` | `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/BROWSER_RECEIPT.json` | `project_mit` |
| `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_1.png` | `27833755cea790f04353c930a2158044d4aab05c87989ee5751dc8dff66f5fb6` | `browser_produced` | `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/BROWSER_RECEIPT.json` | `project_mit` |
| `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_2.png` | `8846a2cfa68bbd40fea458f8c58fb136f363e26c2e37f1969c05c3ec81dbfe64` | `browser_produced` | `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/BROWSER_RECEIPT.json` | `project_mit` |
| `fixtures/r2/maximum-observed-scene.json` | `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `fixtures/r2/plans/scene_one.json` | `717b91f3f35d815bfa9f9cc777b38f8a091f7a6339d786c57360e94ffe4c7699` | `compiler_produced` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/COMPILER_RECEIPT.md` | `project_mit` |
| `fixtures/r2/plans/scene_two.json` | `1fd08cfb33d07f93a568e4bb337ebfbe8909a22a973d0f137139c92f0481e905` | `compiler_produced` | `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SECOND_AUTHOR_RECEIPT.md` | `project_mit` |
| `fixtures/r2/scenes/scene_one.json` | `e4c04e7d6806aaba3e3ba9e0b94ba761442e00d5dd17a14581c29e1de22c41aa` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md` | `project_mit` |
| `fixtures/r2/scenes/scene_two.json` | `21af593413804b9ece1df24f38f436ff2288de445589160b22506c98d53f227a` | `r2_authored` | `docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SECOND_AUTHOR_RECEIPT.md` | `project_mit` |
