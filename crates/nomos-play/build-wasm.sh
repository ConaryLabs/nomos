#!/usr/bin/env bash
# Build the browser runtime, reproducibly.
#
# RUNTIME.md section 5 R1-5. Two things are load-bearing beyond the profile:
#
#   * The build runs from the workspace root, and `--remap-path-prefix` guards
#     the case where something makes it not. Even with `strip = true` and
#     `panic = "abort"`, `core::panic::Location` embeds the source path of every
#     `expect` and every slice index in the binary. Measured: from the root
#     those paths come out repository-relative (`crates/nomos-play/src/plan.rs`)
#     and the remap is a no-op, so the digest is the same with or without it;
#     from anywhere else they would be absolute, which would make the artifact
#     depend on where the checkout lives and would trip `build.mjs`'s scan rule
#     6. The `cd` is what makes it relative and the remap is the belt.
#
#   * `--profile wasm`, not `--release`. Cargo profiles are workspace-global and
#     `lto`, `panic`, and `strip` have no per-package override, so setting them
#     on `[profile.release]` would change every native release build in the
#     workspace. `Cargo.toml` records the measurement.
#
# Prints the staged path, the byte count, and the sha256, which is what the
# viewer's build receipt and RUNTIME.md section 7 record.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"

export RUSTFLAGS="--remap-path-prefix=${root}=/nomos${RUSTFLAGS:+ $RUSTFLAGS}"
cargo build --locked -p nomos-play --target wasm32-unknown-unknown --profile wasm "$@"

binary="$root/target/wasm32-unknown-unknown/wasm/nomos_play.wasm"
bytes=$(stat -c%s "$binary")
digest=$(sha256sum "$binary" | cut -d' ' -f1)

# Fail closed on a build-machine path rather than leaving it for the scan: a
# binary that leaks one is not publishable, and the reason is here.
if strings -n 6 "$binary" | grep -qE '/(home|work|tmp|root|Users|github|runner)/'; then
  echo "nomos_play.wasm carries a build-machine path; the remap did not take" >&2
  exit 1
fi

printf 'NOMOS_PLAY_WASM %s bytes=%s sha256=%s\n' "$binary" "$bytes" "$digest"
