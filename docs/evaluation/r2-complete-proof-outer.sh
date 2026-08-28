#!/usr/bin/env bash

# Source-only outer network/PID/mount confinement for r2-complete-proof.sh.
# The XFS wrapper owns filesystem and network-namespace provisioning and enters
# this layer only after dropping privilege. This layer verifies that isolation,
# adds PID/mount confinement, and exposes only the two writable checkout binds.
# shellcheck disable=SC2154 # Globals are established by the sourcing harness.

r2_discover_browser() {
  node --input-type=module - "$repo_root" <<'NODE'
import { pathToFileURL } from "node:url";
const { findChrome } = await import(
  pathToFileURL(`${process.argv[2]}/apps/nomos-viewer/smoke/chrome.mjs`).href,
);
const found = findChrome();
if (!found) process.exit(1);
process.stdout.write(found.binary);
NODE
}

r2_run_outer_proof() {
  local node_major active_toolchain installed_targets browser browser_version
  node_major=$(node -p 'Number(process.versions.node.split(".")[0])')
  [[ $node_major =~ ^[0-9]+$ && $node_major -ge 22 ]] ||
    fail 'Node 22 or newer is required'
  active_toolchain=$(rustup show active-toolchain)
  [[ $active_toolchain == 1.98.0-* ]] ||
    fail 'the active Rust toolchain is not 1.98.0'
  installed_targets=$(rustup target list --installed)
  grep -Fx 'wasm32-unknown-unknown' <<<"$installed_targets" >/dev/null ||
    fail 'the wasm32-unknown-unknown Rust target is not installed'
  browser=$(r2_discover_browser) ||
    fail 'Chrome/Chromium is not installed or cannot start'
  browser=$(realpath -e -- "$browser")
  [[ -f $browser && -x $browser && ! -L $browser ]] ||
    fail 'the discovered browser is not one executable regular file'
  browser_version=$("$browser" --version) ||
    fail 'the discovered browser cannot report its version'
  [[ -n $browser_version ]] || fail 'the discovered browser reported no version'

  bwrap --die-with-parent --new-session --unshare-pid \
    --ro-bind / / --dev /dev --proc /proc \
    "$(command -v bash)" -c : >/dev/null 2>&1 ||
    fail 'bubblewrap read-only root and PID confinement is unavailable'

  local caller_uid caller_gid caller_user caller_path rustup_home
  local isolated_netns host_netns host_pidns proof_token
  caller_uid=$(id -u)
  caller_gid=$(id -g)
  caller_user=$(id -un)
  caller_path=$PATH
  rustup_home=${RUSTUP_HOME:-$(rustup show home)}
  rustup_home=$(realpath -e -- "$rustup_home")
  isolated_netns=$(readlink /proc/self/ns/net)
  host_netns=${NOMOS_R2_HOST_NETNS:-}
  [[ $isolated_netns == net:\[*\] && $host_netns == net:\[*\] &&
     $isolated_netns != "$host_netns" ]] ||
    fail 'wrapper did not enter a fresh network namespace'
  host_pidns=$(readlink /proc/self/ns/pid)
  [[ $host_pidns == pid:\[*\] && $host_pidns == "${NOMOS_R2_HOST_PIDNS:-}" ]] ||
    fail 'wrapper changed or misreported the caller PID namespace'
  proof_token=${NOMOS_R2_PROOF_TOKEN:-}
  [[ $proof_token =~ ^[0-9a-f]{64}$ ]] ||
    fail 'wrapper proof process token is missing or malformed'
  [[ ${NOMOS_R2_EXTERNAL_POSITIVE:-} == connected ]] ||
    fail 'wrapper external-connect positive control marker is missing'
  mkdir -p "$repo_root/target"
  [[ $(stat -c %d "$repo_root/target") == "$(stat -c %d "$repo_root")" ]] ||
    fail 'target and checkout must share one filesystem'
  outer_control_stdout=$repo_root/target/.nomos-r2-network-$proof_token.stdout
  outer_control_stderr=$repo_root/target/.nomos-r2-network-$proof_token.stderr
  cleanup_outer_control() {
    local file
    for file in "${outer_control_stdout:-}" "${outer_control_stderr:-}"; do
      [[ -z $file || ! -e $file ]] || find "$file" -delete
    done
  }
  trap cleanup_outer_control EXIT
  trap 'cleanup_outer_control; exit 130' INT
  trap 'cleanup_outer_control; exit 143' TERM
  [[ -f $outer_control_stdout && ! -L $outer_control_stdout &&
     -f $outer_control_stderr && ! -L $outer_control_stderr &&
     $(stat -c %s "$outer_control_stdout") -eq 10 &&
     $(<"$outer_control_stdout") == connected &&
     ! -s $outer_control_stderr ]] ||
    fail 'wrapper external-connect positive control evidence is invalid'

  env -i \
    PATH="$caller_path" \
    HOME="$output_real/host/home" \
    TMPDIR="$output_real/host/tmp" \
    XDG_CACHE_HOME="$output_real/host/xdg-cache" \
    XDG_CONFIG_HOME="$output_real/host/xdg-config" \
    XDG_DATA_HOME="$output_real/host/xdg-data" \
    CARGO_HOME="$output_real/host/cargo-home" \
    CARGO_TARGET_TMPDIR="$repo_root/target/tmp" \
    RUSTUP_HOME="$rustup_home" \
    RUSTUP_NO_UPDATE_CHECK=1 \
    CARGO_NET_OFFLINE=true \
    CARGO_INCREMENTAL=0 \
    CARGO_TERM_COLOR=never \
    CHROME_BIN="$browser" \
    LC_ALL=C LANG=C \
    USER="$caller_user" LOGNAME="$caller_user" \
    NOMOS_R2_PROOF_INNER=1 \
    NOMOS_R2_HOST_NETNS="$host_netns" \
    NOMOS_R2_HOST_PIDNS="$host_pidns" \
    NOMOS_R2_CALLER_UID="$caller_uid" \
    NOMOS_R2_CALLER_GID="$caller_gid" \
    NOMOS_R2_EXPECTED_HEAD="$head" \
    NOMOS_R2_EXPECTED_TREE="$tree" \
    NOMOS_R2_OUTPUT_REAL="$output_real" \
    NOMOS_R2_OUTPUT_RELATIVE="$output_relative" \
    NOMOS_R2_PROOF_TOKEN="$proof_token" \
    NOMOS_R2_XFS_WRAPPER=1 \
    NOMOS_R2_XFS_UUID="$NOMOS_R2_XFS_UUID" \
    NOMOS_R2_XFS_FRAGMENT_SIZE="$NOMOS_R2_XFS_FRAGMENT_SIZE" \
    NOMOS_R2_XFS_DEVICE="$NOMOS_R2_XFS_DEVICE" \
    NOMOS_R2_XFS_MAJOR_MINOR="$NOMOS_R2_XFS_MAJOR_MINOR" \
    NOMOS_R2_EXTERNAL_POSITIVE=connected \
    GIT_OPTIONAL_LOCKS=0 \
    bwrap --die-with-parent --new-session --unshare-pid \
      --ro-bind / / \
      --ro-bind "$repo_root" "$repo_root" \
      --bind "$repo_root/target" "$repo_root/target" \
      --bind "$output_real" "$output_real" \
      --dev /dev --proc /proc \
      "$repo_root/docs/evaluation/r2-complete-proof.sh" --output "$output_real"
  cleanup_outer_control
  trap - EXIT INT TERM
}
