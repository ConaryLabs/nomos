# Capture receipt

Captured on 2026-08-25 with:

```sh
experiments/executable-gaol/gaol capture
experiments/executable-gaol/gaol verify
```

- AreaCollection SHA-256: `4e926cd4e12fe7481a56041cfaa5ba26b1ef3cff17e2ffdf62cca20d5404f310`
- North Gaol RenderingPlan SHA-256: `35aede17d1bc0163cd649ef94c8d57dab558d6d7703e3ebbd45911e64ba359bd`
- Cistern Walk RenderingPlan SHA-256: `0f66208307d0f93c840f6accef0ccd09598a5445a64f8614a360681199360657`
- Ember Vault RenderingPlan SHA-256: `55c91ddcb354dde70ea58f80a17ee5663f0d7de60db8003fbea5c86acbc1c170`
- Ossuary Reach RenderingPlan SHA-256: `f7c3bc4465705945e36b6c2aa5f610985c95fe36d2b76872f9b0bf61d2d15a2c`
- cross-area contact-sheet.svg SHA-256: `be53d3b5f218ab3e17ca73b4b1016ab2b9a907dfd07aa7dea0d96385221ddb3e`
- cross-area contact-sheet.png SHA-256: `d0d2f33f6ed5f08b5559f61647e861858bdd91662e5c29cfd4a48cbb4c4b7999`
- verification: `EXECUTABLE_GAOL_VERIFY PASS`

Two consecutive clean captures produced all seven exact hashes.

The plans and the collection are `nomos.rendering_plan@2` and
`nomos.experiment.area_collection@2` as of R1-3 (issue #146), compiled from the
four `presentation.json` sources that replaced `area.json`. Every plan hash
moved, because every field name did; what did *not* move is the drawn output.

## Frame digests across R1-3

Thirty artifacts, captured immediately before and immediately after the switch.
**Twelve are byte-identical** — scenarios `03`, `04`, and `05` of all four
areas — and eighteen changed. The eighteen are exactly the artifacts that carry
a crescent or print the plan's identity, and each one is accounted for
byte-exactly:

| Artifact | Before | After |
| --- | --- | --- |
| `areas/cistern-walk/frames/01-baseline.svg` | `ddc79b8151c65ed1ecfeed316ebd2969f2b073d0a2ea44ec9bcad2d630051b4a` | `04b5b2864cd2098e7575d685f5a70fb4bfc09d0a9c13644d832d6b013f8b4e67` |
| `areas/cistern-walk/frames/02-breached-warded.svg` | `b72822b9d8eb05c1f9b7022af6786490e06120736ddb9408eacbfb71b450d8f9` | `b1136772219ea194310189f99dec58f686cb181f73b7e2d02e6a10801618743a` |
| `areas/cistern-walk/frames/contact-sheet.svg` | `dcca9c10f708b5693eeda86b54ee95aa992f01d45e010b64a93d8cf20e68089d` | `fb3f1b521c4036116ba2ebbaf7a75a0c3130cb43121105ab7d07c2b152a29c82` |
| `areas/cistern-walk/frames/forensic.svg` | `778865466c2e5e0d4a5f2d40a8a49abe844d26657d837313778f1a62a8278e00` | `a2c018d0dc37fd49fb6e84035d4b856572f682f5eb5a2fd8eb546ac5580525ad` |
| `areas/ember-vault/frames/01-baseline.svg` | `55dca6dae07725bef1f3d4293ee79e652d911d0df2f24861f29e1d9d660755b1` | `a375748faece16526a7f3c836575e8ba013369463638cdd3c11f6cb6af21fcb0` |
| `areas/ember-vault/frames/02-breached-warded.svg` | `28e0bafe52c0779ecfa5a014849ee63703e2663ab9a08aea6f1f9e2d54b111c5` | `39416a7ae019551aa8eb738470d88c850a88187261e00df6d4a541519162252b` |
| `areas/ember-vault/frames/contact-sheet.svg` | `f14c2c1d721916c496b10b45b677d43dcad7ec5b4949a6e7b6f68f852fca5b73` | `0da2a31415e8cff5187c4671527fa5f3ff24ebd9bf3987b33622408e372f211d` |
| `areas/ember-vault/frames/forensic.svg` | `3bab9cb145f503e5da93de979ebb10a1b84e7b8168a354284c25743b96295e49` | `a5049560d73bed6dc09320359c75c08ce8593000d53d012bb9f4809503c2e692` |
| `areas/north-gaol/frames/01-baseline.svg` | `610815fb3921d6dd787385e24f569c415338e0b7514fd5a27d1fa26393cda2cc` | `4ccd75bd01d718e8cd5bcb15db5176d684754e9626e3bc25d35b49d9db3ee8f4` |
| `areas/north-gaol/frames/02-breached-warded.svg` | `149598fb15379881f181740ee9bcef12a1f0e132e94d0a32f751116de7fea8c2` | `db577298be6e22c9bd97786344a4675197390e5cac2e43395701158c6a1be4bc` |
| `areas/north-gaol/frames/contact-sheet.svg` | `37acf9611dc738fc62b66f532287a3a1de2f47031c5122c0855edb3e5e1ef09d` | `931a96f13ff44725a500894cc2cd10c833295f0dfa8aeed30944c085ba331fdc` |
| `areas/north-gaol/frames/forensic.svg` | `cd953680da72de3084860133a5b4b64170b4ad80a076defb5dc3af8ea7e7d2ca` | `5c4af27608f3aacce4974e460fe48ab9baea34a71dd40749c61fd25a62aef3f6` |
| `areas/ossuary-reach/frames/01-baseline.svg` | `01e6f62de5215fec475a6218c80684620bf39d5124631aaf24203f235e4a8d11` | `744627cb03f4c24ba05330deedcb8b7925d9311ac6d6b539ba36007bf582d646` |
| `areas/ossuary-reach/frames/02-breached-warded.svg` | `e08a5bd20acce56f19aa99c5664af2bdccc144a4a6a268100278b60f420b8f2a` | `a83804ddf6c86e7f562d6a785ef5f5331756ae5247cef7f20202fea3dbf4259a` |
| `areas/ossuary-reach/frames/contact-sheet.svg` | `c54208b91906ecd24080f47b4a1a4d05bb32ead0d6a8e8ea5f922737e1219aec` | `d2130cf660c51d288ec679ad617d289fc7a9742db4ec6cf84736a3f20c3fe5c3` |
| `areas/ossuary-reach/frames/forensic.svg` | `99a4bc471c00341cb3f875033f1fde99f2c24dcd218b6f25e3e2034cf46e5529` | `7c47349bf19ed396c85b51de0447c0c3086cebfa27ac7ac9e67e828671c12aa6` |
| `frames/contact-sheet.svg` | `d30ec3fb140c5ddad8a6aacaf3b07a348f8195a8540b900fc63b5cb350a76795` | `be53d3b5f218ab3e17ca73b4b1016ab2b9a907dfd07aa7dea0d96385221ddb3e` |
| `frames/contact-sheet.png` | `af9c834a39e3045bca48d9209d27190978f80c7130de839ea70031de9a8b3eec` | `d0d2f33f6ed5f08b5559f61647e861858bdd91662e5c29cfd4a48cbb4c4b7999` |

Unchanged, byte for byte: `03-breached-unsealed.svg`,
`04-breached-unsealed-dark.svg`, and `05-open-dark.svg` in all four areas.

### The crescent-only proof

The four `visual/cyan_crescent` effects moved from hand-placed floats to the
gate's `ward` socket. In the SVG renderer a crescent is exactly three elements
— one `<path>` and two `<circle>`s — all generated from a single point:

| Area | Old anchor | Old point | Gate cell | New point |
| --- | --- | --- | --- | --- |
| north-gaol | `(3.6, 3.4, 0)` | `(479.6, 300)` | `north_gate (5, 0)` | `(734, 197.9)` |
| cistern-walk | `(4.9, 3.8, 0)` | `(522.8, 342.5)` | `sluice_gate (2, 0)` | `(590, 122.9)` |
| ember-vault | `(4, 3.8, 0)` | `(479.6, 320)` | `vault_gate (4, 0)` | `(686, 172.9)` |
| ossuary-reach | `(6.1, 1.2, 0)` | `(705.2, 307.5)` | `bone_gate (6, 0)` | `(782, 222.9)` |

For each changed artifact, rendering those three elements at the new point and
substituting the same three at the old point reproduces the before-bytes
exactly:

```text
17 artifacts checked, 0 not explained by the crescent substitution
```

The crescent is drawn only while the primary gate's ward is `sealed`, which is
scenarios `01` and `02`, so it appears once in each of the eight changed
frames, twice in each per-area contact sheet, and once per area in the
cross-area sheet. The four `forensic.svg` overlays render
`03-breached-unsealed`, whose ward is unsealed, and carry no crescent at all:
their only changed bytes are the plan identity the overlay prints in its own
provenance line, and substituting `nomos.rendering_plan@1` back for
`nomos.rendering_plan@2` reproduces each old digest exactly. The PNG is
`rsvg-convert` over the cross-area SVG and follows it.

The committed PNG and example plans are review conveniences; the command
regenerates them from the content, subsystem projections, and real runtime
states.
