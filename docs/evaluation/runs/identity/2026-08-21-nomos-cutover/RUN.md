# Nomos identity-cutover author receipt

## Subject

- Issue: #31
- Decision: 0007, contract revision 6
- Branch: `feature/nomos-identity-31`
- Implementation commit: `7c0ca313cdeab46f30f926cb099852dc85641983`
- Author proof interval: 2026-08-21T20:54:24Z–20:54:25Z
- Worktree: clean before and after

Environment:

```text
Linux 7.1.8-arch1-3 x86_64
rustc 1.98.0 (88d9e12ae 2026-08-18)
cargo 1.98.0 (797e8a9bc 2026-08-05)
```

## Identity result

The active workspace contains `nomos-core`, `nomos-schema`,
`nomos-projection`, `nomos-compiler`, `nomos-sim`, `nomos-cli`, and isolated
`xtask`. The binary is `nomos`, the fixture is `fixtures/gaol.nomos`, active
schemas use `nomos.*`, and Cargo repository metadata names
`https://github.com/ConaryLabs/nomos`.

The active construction epoch is `nomos.world_ir.construction@1`. It is not a
version increment or compatibility claim over the closed prototype
`estate.world_ir.construction@1..3` epoch.

## Golden relationship

```text
prototype estate.world_ir.construction@3
  528a8730cecd6969120f92ba11e9889d4ea2741db916d9561b866e96d9658925
active nomos.world_ir.construction@1
  c1322a05806a2d66c634bf0cbdae0620a893a8db8f2088236e29cd8397a2db55

prototype estate.hash_domain_fixture@1
  09ef5bc23dd2e47109dec91aea083e4f883b3c0ff8e021f86dd127c06c94faf8
active nomos.hash_domain_fixture@1
  40a8cabed5fd8a036132acf952bdc0a0917199fca6c24e45e941e1227c0dfb12
```

The schema-bearing envelope hash changed. The fixture entity sub-hash stayed
`34c3f09f03cac779162de81789d4d842866c5219b12a76dd28d7ed4f3cfc6613`,
and the empty canonical object stayed
`44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a`.
That isolates the observed determinism change to identity-bearing bytes. The
complete parser, linker, ownership, transition, projection, resolver,
transaction, canonical, package, and mutation suites also pass unchanged.

The three prototype construction hash files remain committed beside an
explicit provenance README. No historical hash or completed receipt changed.

## Legacy-name audit

The repository was scanned with:

```bash
rg -n --hidden --glob '!.git/**' --glob '!target/**' \
  'signed-world|estate-|estate_|estate\.|\.estate|`estate '
```

Every remaining match belongs to one of these classified groups:

1. decision 0007, the current handoff, the active schema comment, and the
   construction-golden README explicitly describe the closed prototype epoch;
2. decisions 0001–0006, founding/checkpoint reviews, and completed evaluation
   runs are immutable historical evidence; or
3. completed SW-C/SW-D/SW-E entries in the handoff preserve the exact identity
   and commands that existed when those slices ran.

No active Rust path, crate or package name, binary, schema ID, fixture, source
extension, current command example, workflow, workspace rule, or Cargo metadata
retains the former identity. Open issues #24 and #17 received future-facing
Nomos disposition comments without rewriting their historical text.

## Proof

All commands passed:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

The documented `nomos-core` x86_64 debug/release determinism comparison had an
empty diff and produced:

```text
HASH hash_domain_fixture 40a8cabed5fd8a036132acf952bdc0a0917199fca6c24e45e941e1227c0dfb12
HASH hash_domain_fixture_empty_object 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a
HASH hash_domain_fixture_entities 34c3f09f03cac779162de81789d4d842866c5219b12a76dd28d7ed4f3cfc6613
```

## Pending external and non-author evidence

At this receipt point, the GitHub repository still has its historical
`ConaryLabs/signed-world` name. It will be renamed only after the code PR merges
so branch and PR recovery remain straightforward. PR CI, the GPT-5.6 Luna max
exact-head non-author rerun, repository rename, and post-merge CI remain pending
and cannot be inferred from this author receipt.
