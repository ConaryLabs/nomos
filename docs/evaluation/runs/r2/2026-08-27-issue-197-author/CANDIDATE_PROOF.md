# R2-2 clean candidate proof

Status: author proof passed; exact-head Luna Max non-author proof pending.

## Candidate binding

- Candidate commit: `d84ce9b21fc970b55d141e64058f5bb1144400e5`
- Candidate tree: `5787d85e795829092ba4ffdcd60faa167ce5c06b`
- Retained proof output:
  `/data/dev/src/nomos-r2-2-candidate-d84ce9b/`
- Host: `remi`, Linux `7.0.0-30-generic`, x86_64, 12 logical CPUs
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- Cargo: `cargo 1.98.0 (797e8a9bc 2026-08-05)`
- Node: `v22.22.1`
- Browser: `HeadlessChrome/152.0.7977.64`

The candidate worktree was clean at proof start and end. Generated build,
compiler, browser, and R1 regression outputs stayed beneath the worktree-local
`target/` or the retained proof directory above; no committed input changed.

## Workspace and R2 source proof

These commands passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
cargo build --release --locked -p nomos-observed-scene
docs/evaluation/r2-second-scene-packet.test.sh
docs/evaluation/r2-schema-ownership.sh
docs/evaluation/r2-source-provenance.sh
docs/evaluation/r2-source-provenance.test.sh
docs/evaluation/r2-adopter-neutrality.sh
docs/evaluation/r2-adopter-neutrality.test.sh
node docs/evaluation/r2-maximum.test.mjs
node docs/evaluation/r2-scene-signature.mjs fixtures/r2/scenes/scene_one.json fixtures/r2/scenes/scene_two.json
node --test apps/nomos-observed-viewer/test/*.test.mjs docs/evaluation/r2-scene-signature.test.mjs
```

Results:

- workspace boundary: clean; six kernel, two R1, and one R2 member;
- R2 schemas: exactly 2; ownership-register SHA-256
  `948dae85d6cbcedc0ffc55629bfbf633a8b934bb99e23a84a11bada43fe11531`;
- source provenance: 74 rows; register SHA-256
  `9b7873dc5dde438b6b6d7bd45779392729275e9f840e83d76df9bdbd236d39a9`;
- source-provenance plants: 7/7; adopter-neutrality plants: 5/5;
- adopter-neutrality: 52 files scanned against 5 forbidden patterns;
- exact maximum fixture: 98,421 bytes, SHA-256
  `fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909`;
- Node proof: 35 tests passed;
- release compiler SHA-256:
  `dde136c1f2abd66e68ec395ce2fcfb427eec62e17f150d1bf35776a9da41e264`;
- frozen catalog SHA-256:
  `6259520fbf318ae0393ea4ae69649864acb154db4034d081435416be2ffa9323`;
- frozen packet-manifest SHA-256:
  `d5708087cf7967a420667c56a7b02ed052b7058ed8545af06e6771170003c948`.

The release compiler reproduced both committed plans byte-for-byte. The packet
plant harness reconstructed the 42-file frozen packet and proved both closure
failures. The independent-author audit admitted exactly its eight authorized
additions and no implementation edit.

## Independent scenes and signatures

The independent Luna Max author received only the frozen packet. Its unchanged
source, plan, signature, receipt, and browser evidence are committed beneath
the paths named in `SECOND_AUTHOR_RECEIPT.md`.

| Evidence | Scene one | Scene two |
| --- | --- | --- |
| scene-source SHA-256 | `e4c04e7d6806aaba3e3ba9e0b94ba761442e00d5dd17a14581c29e1de22c41aa` | `21af593413804b9ece1df24f38f436ff2288de445589160b22506c98d53f227a` |
| compiled-plan SHA-256 | `717b91f3f35d815bfa9f9cc777b38f8a091f7a6339d786c57360e94ffe4c7699` | `1fd08cfb33d07f93a568e4bb337ebfbe8909a22a973d0f137139c92f0481e905` |
| full signature | `ef11771f3f8c210fdd8c9366e780ab720349a49dad88ae8dca969fcbe16c30d2` | `9afb46dc4d7ddb5b79cdcabd63b67d162b3c230aecfde9383e038128572f0f3d` |
| crop axis | `6bc0ba95810359c49e83d612501c8a9319e380e7f227ad72d0a9529c4e02d175` | `8c1ad689b143df6bdb2bc4c7e08f4b0f60a0bf114dc1a4931f641fa29662abeb` |
| terrain axis | `a7c4a7cca62253f9fb9aa62412297184a9413a4cc480a1ce39590aff2cd34c3f` | `752bc486adc6b77597a1849f23b9de1dff845bfcd6fbf08f7e5937e2eab8d9be` |
| actor axis | `a170f9a01b67fb0b4681aca6fda680bfa8695d423d8db9c6e382ee4b8c53d779` | `b1e10755b6f8b523ad084e5b8a29cf153be72ec52bec3eb01ebdf628aa3e3181` |
| action axis | `d9498c215739ed6ee5f3dce9d0ce21a19e35af26016f046e482c0ab785159705` | `28c07a530e8cf68e8aa6a3e4b2997aa9c4f89f178ce83210262e6d24773d47f1` |

The signature JSON SHA-256 is
`6c8e38c78896bf5d5a166427f5f9567044dc00314dc4e65124173c57ccab50c4`.
All four normalized axes differ independently.

## Offline distribution and browser proof

Two clean builds were byte-identical. Each contains exactly 14 regular files
and 805,600 bytes, below the 2,000,000-byte ceiling. Both build receipts have
SHA-256
`a52f2200e5027ae137c24ad6c8661b01fc29162088119b9fe60c739e543f7094`.
The distribution scan admitted no source scene, unmanifested file, external
origin, adopter payload, or dependency beyond the accepted Three.js bytes and
license.

The required-Chromium proof used 20 fresh profiles with cache disabled, ten
per scene. Every launch used WebGL2, completed exactly one frame, matched the
compiled consequence counts and integrity index, passed the negative resolver
control, and made zero external requests.

Raw scene-one samples in nanoseconds, in launch order:

```text
135485495 140677400 131944738 127644369 128138817
129247154 129132916 130308465 131239001 124350582
```

Median numerator/denominator: `259555619 / 2 ns`; p95: `140677400 ns`.

Raw scene-two samples in nanoseconds, in launch order:

```text
132612706 127326213 129765271 130730154 145075405
132309937 130687118 181408825 129959301 128881854
```

Median numerator/denominator: `261417272 / 2 ns`; p95: `181408825 ns`.
Combined median numerator/denominator: `260995583 / 2 ns`; combined p95:
`145075405 ns`. Each result is below the 5,000,000,000-nanosecond ceiling.

All raw browser/CDP/server closure rows are retained in `smoke/receipt.json`.
The worst after-result closure was `27.406838 ms`, below the 2,000-ms ceiling;
no process remained. The receipt SHA-256 is
`bed035371cc2a9a86897a633ade9a423408dc6e9d320252c2f10fc176107be32`.

The browser reproduced the exact committed pixels:

- scene one: 32,545 bytes, SHA-256
  `27833755cea790f04353c930a2158044d4aab05c87989ee5751dc8dff66f5fb6`;
- scene two: 38,964 bytes, SHA-256
  `8846a2cfa68bbd40fea458f8c58fb136f363e26c2e37f1969c05c3ec81dbfe64`;
- 2560 by 720 contact sheet: 70,660 bytes, SHA-256
  `b76edbd9dd03fce5a99c074200ee7311bf87d5d2e5829c800170c129d00bf576`.

The owner accepted the exact finite presentation direction before this slice
and preauthorized PR #198's merge only after every issue criterion is green.
Inspection of the exact recorded sheet confirms two coherent, readable members
of that one visual family. That disposition accepts the R2-2 contact sheet; it
is not a production-art, adopter-target, or final R2 verdict.

## Unchanged R1 regression

The complete documented R1 proof passed on the same candidate: executable-gaol
verification, wasm, native player, clean viewer build, all 104 Node tests,
required-Chromium smoke, and native replay.

- six areas; 65 moves; traversal cost 95; 23 loopback requests; zero external;
- wasm: 421,195 bytes, SHA-256
  `e8e03c125667ad937939f4a628b67df9ff813a88823fecd859784ed241673c97`;
- public distribution: 24 files, 1,386,650 bytes;
- native replay: 77 commands and receipts, chain head
  `43a1b2164f18bc54738d0402013419659576e2d866c3fca630321a2ca641f143`;
- R1 build-receipt SHA-256:
  `03f79d195f388d7aea105365cc0e8d28697d06ce031387ae10fac408175cd227`;
- R1 smoke-receipt SHA-256:
  `ee7923cfe0d1f44804e90f3207a73300d01f2e0bc81731d76ba15d7d2bc7216a`;
- session SHA-256:
  `9e05713fe5568fa76ff5e346e4f4de351972c774827a0e58318b910c7add1cea`;
- browser/CDP/server shutdown: 24 ms overall; pass.

## Limits and stop line

This proof does not claim the complete network-route-isolated R2 final proof,
adopter integration, game adoption, production art, a platform decision,
deployment, gameplay authority, or final R2 disposition. Chrome's resolver
rules denied external hosts, every request was recorded, and the negative
control passed. The separately authorized R2 final-evidence target must still
run `r2-complete-proof.sh` from a clean standalone checkout with external
network routes removed and stop for the owner's explicit R2 verdict.
