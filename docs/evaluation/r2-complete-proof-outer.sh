#!/usr/bin/env bash

# Source-only outer network/PID/mount confinement for r2-complete-proof.sh.
# The XFS wrapper owns filesystem provisioning and enters this layer only after
# dropping privilege. This layer proves an external route, creates the fresh
# network/PID/mount namespaces, and exposes only the two writable checkout binds.
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

r2_prepare_outer_preflight_log() {
  [[ $# -eq 1 && $1 == /* && $1 != *$'\n'* && $1 != *$'\r'* && $1 != *$'\t'* ]] || return 2
  local path=$1 parent actual basename expected descriptor_parent=false
  parent=${path%/*}
  [[ -n $parent ]] || parent=/
  basename=${path##*/}
  [[ -n $basename && $basename != . && $basename != .. && $basename != */* ]] || return 2
  if [[ $parent =~ ^/proc/self/fd/[0-9]+$ ]]; then
    descriptor_parent=true
  else
    [[ -d $parent && ! -L $parent ]] || return 2
  fi
  [[ -d $parent ]] || return 2
  actual=$(realpath -e -- "$parent") || return 2
  [[ $descriptor_parent == true || $actual == "$parent" ]] || return 2
  expected=$actual
  [[ $expected == / ]] || expected+=/
  expected+=$basename
  if [[ -e $path || -L $path ]]; then
    [[ -f $path && ! -L $path && ! -s $path ]] || return 2
    actual=$(realpath -e -- "$path") || return 2
    [[ $actual == "$expected" ]] || return 2
  else
    : >"$path" || return 1
  fi
}

r2_validate_outer_preflight_log() {
  [[ $# -eq 3 && -f $1 && ! -L $1 && $2 == net:\[*\] && $3 == pid:\[*\] ]] || return 2
  local path=$1 host_netns=$2 host_pidns=$3 parent actual expected descriptor_parent=false basename
  parent=${path%/*}
  [[ -n $parent ]] || parent=/
  basename=${path##*/}
  if [[ $parent =~ ^/proc/self/fd/[0-9]+$ ]]; then
    descriptor_parent=true
  else
    [[ -d $parent && ! -L $parent ]] || return 2
  fi
  actual=$(realpath -e -- "$parent") || return 2
  [[ $descriptor_parent == true || $actual == "$parent" ]] || return 2
  expected=$actual
  [[ $expected == / ]] || expected+=/
  expected+=$basename
  [[ $(realpath -e -- "$path") == "$expected" && $(wc -l <"$path") == 1 ]] || return 2
  jq -e --arg host_netns "$host_netns" --arg host_pidns "$host_pidns" '
    (keys | sort) == ["cap_ambient", "cap_bounding", "cap_effective", "cap_inheritable", "cap_permitted", "host_network_namespace", "host_pid_namespace", "network_namespace", "no_new_privs", "pid_namespace"] and
    .cap_ambient == "0000000000000000" and .cap_bounding == "0000000000000000" and
    .cap_effective == "0000000000000000" and .cap_inheritable == "0000000000000000" and
    .cap_permitted == "0000000000000000" and .no_new_privs == 1 and
    .host_network_namespace == $host_netns and .host_pid_namespace == $host_pidns and
    (.network_namespace | test("^net:\\[[0-9]+\\]$")) and
    (.pid_namespace | test("^pid:\\[[0-9]+\\]$")) and
    .network_namespace != $host_netns and .pid_namespace != $host_pidns
  ' "$path" >/dev/null
}

r2_compare_outer_positive() {
  [[ $# -eq 6 ]] || return 2
  local host_stdout=$1 host_stderr=$2 staged_stdout=$3 staged_stderr=$4 output_stdout=$5 output_stderr=$6
  local path
  for path in "$host_stdout" "$host_stderr" "$staged_stdout" "$staged_stderr" "$output_stdout" "$output_stderr"; do
    [[ -f $path && ! -L $path ]] || return 1
    if [[ ! $path =~ ^/proc/self/fd/[0-9]+/ ]]; then
      [[ $(realpath -e -- "$path") == "$path" ]] || return 1
    fi
  done
  cmp -s "$host_stdout" "$staged_stdout" && cmp -s "$host_stderr" "$staged_stderr" || return 1
  cmp -s "$host_stdout" "$output_stdout" && cmp -s "$host_stderr" "$output_stderr"
}

r2_run_outer_proof() {
  local node_major active_toolchain installed_targets browser browser_version
  local caller_uid caller_gid caller_user caller_path rustup_home
  local host_netns host_pidns proof_token outer_control_exit
  local host_control_stdout host_control_stderr
  local preflight_host_netns preflight_host_pidns outer_preflight_log
  local preflight_status inner_status positive_status
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

  caller_uid=$(id -u)
  caller_gid=$(id -g)
  caller_user=$(id -un)
  caller_path=$PATH
  host_netns=$(readlink /proc/self/ns/net)
  [[ $host_netns == net:\[*\] ]] ||
    fail 'could not identify the caller network namespace'
  host_pidns=$(readlink /proc/self/ns/pid)
  [[ $host_pidns == pid:\[*\] ]] ||
    fail 'could not identify the caller PID namespace'
  preflight_host_netns=$host_netns
  preflight_host_pidns=$host_pidns
  outer_preflight_log=${NOMOS_R2_OUTER_PREFLIGHT_LOG:-}
  if [[ -z $outer_preflight_log ]]; then
    outer_preflight_log=$(mktemp /tmp/nomos-r2-outer-preflight.XXXXXX) ||
      fail 'could not create an outer preflight host log'
  fi
  [[ $outer_preflight_log != "$repo_root" &&
     $outer_preflight_log != "$repo_root/"* ]] ||
    fail 'outer preflight host log must be outside the checkout'
  r2_prepare_outer_preflight_log "$outer_preflight_log" ||
    fail 'outer preflight host log is not a fresh canonical regular file'

  set +e
  # shellcheck disable=SC2016 # The confined child expands its own status fields.
  bwrap --die-with-parent --new-session --unshare-net --unshare-pid \
    --ro-bind / / --dev /dev --proc /proc \
    "$(command -v bash)" -ceu '
      cap_inheritable=$(awk "/^CapInh:/ {print \$2}" /proc/self/status)
      cap_permitted=$(awk "/^CapPrm:/ {print \$2}" /proc/self/status)
      cap_effective=$(awk "/^CapEff:/ {print \$2}" /proc/self/status)
      cap_bounding=$(awk "/^CapBnd:/ {print \$2}" /proc/self/status)
      cap_ambient=$(awk "/^CapAmb:/ {print \$2}" /proc/self/status)
      no_new_privs=$(awk "/^NoNewPrivs:/ {print \$2}" /proc/self/status)
      network_namespace=$(readlink /proc/self/ns/net)
      pid_namespace=$(readlink /proc/self/ns/pid)
      [[ $network_namespace != "$1" && $pid_namespace != "$2" ]]
      [[ $cap_inheritable == 0000000000000000 &&
         $cap_permitted == 0000000000000000 &&
         $cap_effective == 0000000000000000 &&
         $cap_bounding == 0000000000000000 &&
         $cap_ambient == 0000000000000000 && $no_new_privs == 1 ]]
      ip -j link show lo | jq -e ".[0].ifname == \"lo\" and (.[0].flags | index(\"UP\")) != null" >/dev/null
      jq -cn \
        --arg cap_inheritable "$cap_inheritable" \
        --arg cap_permitted "$cap_permitted" \
        --arg cap_effective "$cap_effective" \
        --arg cap_bounding "$cap_bounding" \
        --arg cap_ambient "$cap_ambient" \
        --arg no_new_privs "$no_new_privs" \
        --arg host_network_namespace "$1" \
        --arg host_pid_namespace "$2" \
        --arg network_namespace "$network_namespace" \
        --arg pid_namespace "$pid_namespace" \
        "{cap_inheritable:\$cap_inheritable,cap_permitted:\$cap_permitted,cap_effective:\$cap_effective,cap_bounding:\$cap_bounding,cap_ambient:\$cap_ambient,no_new_privs:(\$no_new_privs|tonumber),host_network_namespace:\$host_network_namespace,host_pid_namespace:\$host_pid_namespace,network_namespace:\$network_namespace,pid_namespace:\$pid_namespace}"
    ' r2-bwrap-preflight "$preflight_host_netns" "$preflight_host_pidns" \
    >"$outer_preflight_log" 2>/dev/null
  preflight_status=$?
  set -e
  [[ $preflight_status -eq 0 ]] ||
    fail 'unprivileged bubblewrap network/PID/read-only-root confinement is unavailable'
  r2_validate_outer_preflight_log "$outer_preflight_log" \
    "$preflight_host_netns" "$preflight_host_pidns" ||
    fail 'outer preflight evidence is missing, malformed, or does not prove confinement'

  rustup_home=${RUSTUP_HOME:-$(rustup show home)}
  rustup_home=$(realpath -e -- "$rustup_home")
  proof_token=$(printf '%s\n' \
    "$head:$caller_uid:$$:$(date +%s%N)" | sha256sum | cut -d' ' -f1)
  mkdir -p "$repo_root/target"
  [[ $(stat -c %d "$repo_root/target") == "$(stat -c %d "$repo_root")" ]] ||
    fail 'target and checkout must share one filesystem'
  outer_control_stdout=$repo_root/target/.nomos-r2-network-$proof_token.stdout
  outer_control_stderr=$repo_root/target/.nomos-r2-network-$proof_token.stderr
  host_control_stdout=${NOMOS_R2_OUTER_POSITIVE_STDOUT:-}
  host_control_stderr=${NOMOS_R2_OUTER_POSITIVE_STDERR:-}
  realpath -e -- "$host_control_stdout" >/dev/null || fail 'wrapper positive-control stdout path is missing'
  realpath -e -- "$host_control_stderr" >/dev/null || fail 'wrapper positive-control stderr path is missing'
  [[ -f $host_control_stdout && ! -L $host_control_stdout &&
     -f $host_control_stderr && ! -L $host_control_stderr &&
     $(stat -c '%u:%g' "$host_control_stdout") == "$caller_uid:$caller_gid" &&
     $(stat -c '%u:%g' "$host_control_stderr") == "$caller_uid:$caller_gid" &&
     ! -s $host_control_stdout && ! -s $host_control_stderr ]] ||
    fail 'wrapper positive-control logs are not fresh caller-owned regular files'
  [[ ! -e $outer_control_stdout && ! -L $outer_control_stdout &&
     ! -e $outer_control_stderr && ! -L $outer_control_stderr ]] ||
    fail 'network positive-control target is not fresh'
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
    >"$host_control_stdout" 2>"$host_control_stderr"
  outer_control_exit=$?
  set -e
  [[ $outer_control_exit -eq 0 &&
     -f $host_control_stdout && ! -L $host_control_stdout &&
     -f $host_control_stderr && ! -L $host_control_stderr &&
     $(stat -c %s "$host_control_stdout") -eq 10 &&
     $(<"$host_control_stdout") == connected &&
     ! -s $host_control_stderr ]] ||
    fail 'external-connect positive control did not connect'
  cp -- "$host_control_stdout" "$outer_control_stdout"
  cp -- "$host_control_stderr" "$outer_control_stderr"
  [[ -f $outer_control_stdout && ! -L $outer_control_stdout &&
     -f $outer_control_stderr && ! -L $outer_control_stderr ]] ||
    fail 'network positive-control staging evidence is invalid'

  set +e
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
    bwrap --die-with-parent --new-session --unshare-net --unshare-pid \
      --ro-bind / / \
      --ro-bind "$repo_root" "$repo_root" \
      --bind "$repo_root/target" "$repo_root/target" \
      --bind "$output_real" "$output_real" \
      --dev /dev --proc /proc \
      "$repo_root/docs/evaluation/r2-complete-proof.sh" --output "$output_real"
  inner_status=$?
  set -e
  if r2_compare_outer_positive \
    "$host_control_stdout" "$host_control_stderr" \
    "$outer_control_stdout" "$outer_control_stderr" \
    "$output_real/metadata/network-outer-positive.stdout" \
    "$output_real/metadata/network-outer-positive.stderr"; then
    positive_status=0
  else
    positive_status=1
  fi
  cleanup_outer_control
  trap - EXIT INT TERM
  [[ $positive_status -eq 0 ]] ||
    fail 'network positive-control evidence differs between host, staging, and inner output'
  [[ $inner_status -eq 0 ]] ||
    fail "isolated proof exited $inner_status"
}
