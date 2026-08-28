#!/usr/bin/env bash

# Source-only outer network/PID/mount confinement for r2-complete-proof.sh.
# The XFS wrapper owns filesystem provisioning; this layer drops privilege,
# removes external routes, and exposes only the two authorized writable binds.
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

  sudo -n true >/dev/null 2>&1 ||
    fail 'passwordless sudo is required for network isolation'
  sudo -n unshare --net -- true >/dev/null 2>&1 ||
    fail 'sudo unshare --net is unavailable'
  bwrap --die-with-parent --new-session --unshare-pid \
    --ro-bind / / --dev /dev --proc /proc \
    "$(command -v bash)" -c : >/dev/null 2>&1 ||
    fail 'bubblewrap read-only root and PID confinement is unavailable'

  local caller_uid caller_gid caller_user caller_path rustup_home
  local host_netns host_pidns proof_token outer_control_exit
  caller_uid=$(id -u)
  caller_gid=$(id -g)
  caller_user=$(id -un)
  caller_path=$PATH
  rustup_home=${RUSTUP_HOME:-$(rustup show home)}
  rustup_home=$(realpath -e -- "$rustup_home")
  host_netns=$(readlink /proc/self/ns/net)
  [[ $host_netns == net:\[*\] ]] ||
    fail 'could not identify the caller network namespace'
  host_pidns=$(readlink /proc/self/ns/pid)
  [[ $host_pidns == pid:\[*\] ]] ||
    fail 'could not identify the caller PID namespace'
  proof_token=$(printf '%s\n' \
    "$head:$caller_uid:$$:$(date +%s%N)" | sha256sum | cut -d' ' -f1)
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
  set +e
  r2_network_probe 1.1.1.1 53 \
    >"$outer_control_stdout" 2>"$outer_control_stderr"
  outer_control_exit=$?
  set -e
  [[ $outer_control_exit -eq 0 &&
     $(stat -c %s "$outer_control_stdout") -eq 10 &&
     $(<"$outer_control_stdout") == connected &&
     ! -s $outer_control_stderr ]] ||
    fail 'external-connect positive control did not connect'

  sudo -n unshare --net -- bash -ceu '
    ip link set lo up
    exec setpriv \
      --reuid "$1" --regid "$2" --init-groups \
      --inh-caps=-all --ambient-caps=-all --bounding-set=-all --no-new-privs -- \
      env -i \
        PATH="$3" \
        HOME="$5/host/home" \
        TMPDIR="$5/host/tmp" \
        XDG_CACHE_HOME="$5/host/xdg-cache" \
        XDG_CONFIG_HOME="$5/host/xdg-config" \
        XDG_DATA_HOME="$5/host/xdg-data" \
        CARGO_HOME="$5/host/cargo-home" \
        CARGO_TARGET_TMPDIR="$4/target/tmp" \
        RUSTUP_HOME="$6" \
        RUSTUP_NO_UPDATE_CHECK=1 \
        CARGO_NET_OFFLINE=true \
        CARGO_INCREMENTAL=0 \
        CARGO_TERM_COLOR=never \
        CHROME_BIN="$7" \
        LC_ALL=C LANG=C \
        USER="$8" LOGNAME="$8" \
        NOMOS_R2_PROOF_INNER=1 \
        NOMOS_R2_HOST_NETNS="$9" \
        NOMOS_R2_HOST_PIDNS="${14}" \
        NOMOS_R2_CALLER_UID="$1" \
        NOMOS_R2_CALLER_GID="$2" \
        NOMOS_R2_EXPECTED_HEAD="${10}" \
        NOMOS_R2_EXPECTED_TREE="${11}" \
        NOMOS_R2_OUTPUT_REAL="$5" \
        NOMOS_R2_OUTPUT_RELATIVE="${12}" \
        NOMOS_R2_PROOF_TOKEN="${13}" \
        NOMOS_R2_XFS_WRAPPER=1 \
        NOMOS_R2_XFS_UUID="${15}" \
        NOMOS_R2_XFS_FRAGMENT_SIZE="${16}" \
        NOMOS_R2_XFS_DEVICE="${17}" \
        NOMOS_R2_XFS_MAJOR_MINOR="${18}" \
        NOMOS_R2_EXTERNAL_POSITIVE=connected \
        GIT_OPTIONAL_LOCKS=0 \
        bwrap --die-with-parent --new-session --unshare-pid \
          --ro-bind / / \
          --ro-bind "$4" "$4" \
          --bind "$4/target" "$4/target" \
          --bind "$5" "$5" \
          --dev /dev --proc /proc \
          "$4/docs/evaluation/r2-complete-proof.sh" --output "$5"
  ' r2-proof "$caller_uid" "$caller_gid" "$caller_path" "$repo_root" \
    "$output_real" "$rustup_home" "$browser" "$caller_user" "$host_netns" \
    "$head" "$tree" "$output_relative" "$proof_token" "$host_pidns" \
    "$NOMOS_R2_XFS_UUID" "$NOMOS_R2_XFS_FRAGMENT_SIZE" \
    "$NOMOS_R2_XFS_DEVICE" "$NOMOS_R2_XFS_MAJOR_MINOR"
  cleanup_outer_control
  trap - EXIT INT TERM
}
