#!/usr/bin/env bash
# R2 revision-3 filesystem-accounting wrapper.
#
# Public interface (and the only supported host entry point):
#   r2-complete-proof-xfs.sh --source CLEAN --work EMPTY
#
# Provisioning is deliberately kept in the private --supervise branch.  The
# host process remains unprivileged and performs the before/after leak check;
# the supervisor is the sole process allowed to use mount/loop infrastructure.
set -Eeuo pipefail
export LC_ALL=C

IMAGE_BYTES=8589934592

fail() {
  printf 'R2 XFS wrapper: FAIL: %s\n' "$*" >&2
  exit 1
}

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
self_path=$(realpath -e -- "${BASH_SOURCE[0]}") || fail 'wrapper path is not canonical'
receipt_helper=$script_directory/r2-complete-proof-xfs-receipt.mjs

run_capture() {
  local stdout_path=$1
  local stderr_path=$2
  shift 2
  local started ended
  : >"$stdout_path"
  : >"$stderr_path"
  started=$(date +%s%N)
  set +e
  "$@" >"$stdout_path" 2>"$stderr_path"
  RUN_STATUS=$?
  set -e
  ended=$(date +%s%N)
  if [[ -n ${wrapper_command_ledger:-} ]]; then
    local command_id=${stdout_path##*/}
    command_id=${command_id%.stdout}
    local command_uid command_gid
    command_uid=$(id -u)
    command_gid=$(id -g)
    local -a recorded_argv=("$@")
    if [[ ${recorded_argv[0]:-} == run_as_user ]]; then
      command_uid=$caller_uid
      command_gid=$caller_gid
      recorded_argv=("${recorded_argv[@]:1}")
    fi
    record_wrapper_command "$command_id" "$started" "$ended" "$RUN_STATUS" \
      "$stdout_path" "$stderr_path" "$command_uid" "$command_gid" "$(pwd -P)" \
      "${recorded_argv[@]}"
  fi
}

record_wrapper_command() {
  local command_id=$1 started=$2 ended=$3 status=$4 stdout_path=$5 stderr_path=$6
  local command_uid=$7 command_gid=$8 cwd=$9
  shift 9
  local -a argv=("$@")
  local argv_json
  argv_json=$(jq -cn --args '$ARGS.positional' -- "${argv[@]}")
  jq -cn \
    --arg id "$command_id" --arg started_ns "$started" --arg ended_ns "$ended" \
    --arg stdout_path "$stdout_path" --arg stderr_path "$stderr_path" \
    --arg uid "$command_uid" --arg gid "$command_gid" --arg cwd "$cwd" \
    --argjson status "$status" --argjson argv "$argv_json" \
    '{id:$id,started_ns:$started_ns,ended_ns:$ended_ns,status:$status,uid:($uid|tonumber),gid:($gid|tonumber),cwd:$cwd,argv:$argv,stdout_path:$stdout_path,stderr_path:$stderr_path}' \
    >>"$wrapper_command_ledger"
}

record_wrapper_tools() {
  local register=$1
  local temporary=$register.tmp
  local -a tools=(
    bash git realpath find stat date mkdir readlink id node jq sudo unshare findmnt losetup
    mount umount blockdev mkfs.xfs xfs_info xfs_quota filefrag fuser fallocate sync setpriv
    bwrap tar ionice du blkid chown cp rm env sha256sum
  )
  printf 'name\tpath\tversion_argv\tversion_status\tsha256\tversion\n' >"$temporary"
  local tool path version status digest version_text
  local -a version_argv
  for tool in "${tools[@]}"; do
    path=$(command -v "$tool") || fail "tool disappeared while recording: $tool"
    path=$(realpath -e -- "$path") || fail "tool path is not canonical: $tool"
    [[ -f $path && -x $path && ! -L $path ]] || fail "tool is not one executable file: $tool"
    case $tool in
      mkfs.xfs|xfs_info|xfs_quota|filefrag) version_argv=(-V) ;;
      *) version_argv=(--version) ;;
    esac
    set +e
    version=$("$path" "${version_argv[@]}" 2>&1)
    status=$?
    set -e
    [[ -n $version ]] || fail "tool reported no version: $tool"
    version_text=${version//$'\n'/\\n}
    version_text=${version_text//$'\r'/\\r}
    version_text=${version_text//$'\t'/\\t}
    digest=$(sha256sum "$path" | awk '{print $1}')
    [[ $digest =~ ^[0-9a-f]{64}$ ]] || fail "tool digest is malformed: $tool"
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$tool" "$path" "${version_argv[*]}" "$status" "$digest" "$version_text" >>"$temporary"
  done
  chmod 0644 "$temporary"
  mv -f -- "$temporary" "$register"
}

record_wrapper_user_tool() {
  local register=$1 tool=$2
  local temporary=$register.tmp
  local path version status digest version_text
  [[ -f $register && ! -L $register && ! -e $temporary && ! -L $temporary ]] ||
    fail 'wrapper tool register is not in a clean append state'
  cp -- "$register" "$temporary"
  # Resolve and execute caller-selected development tools only after the
  # identity/capability drop.  The privileged process merely hashes the
  # resulting executable path for provenance.
  # shellcheck disable=SC2016 # The caller-side shell expands its positional parameter.
  path=$(run_as_user /bin/sh -c 'command -v -- "$1"' _ "$tool") ||
    fail "caller tool disappeared while recording: $tool"
  path=$(realpath -e -- "$path") || fail "caller tool path is not canonical: $tool"
  [[ -f $path && -x $path && ! -L $path ]] || fail "caller tool is not executable: $tool"
  set +e
  version=$(run_as_user "$tool" --version 2>&1)
  status=$?
  set -e
  [[ -n $version ]] || fail "caller tool reported no version: $tool"
  version_text=${version//$'\n'/\\n}
  version_text=${version_text//$'\r'/\\r}
  version_text=${version_text//$'\t'/\\t}
  digest=$(sha256sum "$path" | awk '{print $1}')
  [[ $digest =~ ^[0-9a-f]{64}$ ]] || fail "caller tool digest is malformed: $tool"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$tool" "$path" '--version' "$status" "$digest" "$version_text" >>"$temporary"
  mv -f -- "$temporary" "$register"
}

record_wrapper_tool_json() {
  local register=$1 strict_json=$2
  # The receipt's closed tool register binds exactly the executables used by
  # its operation records.  Keep the wider TSV as auditable host evidence,
  # while emitting this small JSON projection for the strict pass validator.
  jq -R -s '
    split("\n") | .[1:-1] | map(split("\t"))
    | map({key: .[0], value: {path: .[1], version_argv: .[2], version_status: (.[3] | tonumber), sha256: .[4], version: .[5]}})
    | from_entries
  ' "$register" >"$strict_json.tmp"
  chmod 0644 "$strict_json.tmp"
  mv -f -- "$strict_json.tmp" "$strict_json"
  jq -e '
    (keys | sort) == ["bash", "blkid", "blockdev", "bwrap", "cargo", "chown", "cp", "date", "du", "env", "fallocate", "filefrag", "find", "findmnt", "fuser", "git", "id", "ionice", "jq", "losetup", "mkdir", "mkfs.xfs", "mount", "node", "readlink", "realpath", "rm", "rustc", "rustup", "setpriv", "sha256sum", "stat", "sudo", "sync", "tar", "umount", "unshare", "xfs_info", "xfs_quota"]
    and all(.[]; (.path | startswith("/")) and (.version_argv | type == "string" and length > 0)
      and (.version_status | type == "number" and floor == .)
      and (.sha256 | test("^[0-9a-f]{64}$")) and (.version | type == "string" and length > 0))
  ' "$strict_json" >/dev/null || fail 'strict wrapper tool register is incomplete'
}

canonical_existing() {
  local value=$1
  local label=$2
  [[ $value == /* && $value != *$'\n'* && $value != *$'\r'* && $value != *$'\t'* ]] ||
    fail "$label must be one absolute safe path"
  local actual
  actual=$(realpath -e -- "$value") || fail "$label does not exist"
  [[ $actual == "$value" ]] || fail "$label is not canonical or traverses a symlink"
  printf '%s\n' "$actual"
}

validate_source() {
  local source=$1
  [[ -d $source && ! -L $source ]] || fail 'source must be one real directory'
  [[ -d $source/.git && ! -L $source/.git ]] || fail 'source must have a local .git directory'
  local git_dir common_dir
  git_dir=$(/usr/bin/git -C "$source" rev-parse --absolute-git-dir) || fail 'source is not a Git checkout'
  common_dir=$(/usr/bin/git -C "$source" rev-parse --git-common-dir) || fail 'source has no Git common directory'
  git_dir=$(realpath -e -- "$git_dir") || fail 'source Git directory is absent'
  common_dir=$(realpath -e -- "$source/$common_dir") || fail 'source Git common directory is absent'
  [[ $git_dir == "$source/.git" && $common_dir == "$source/.git" ]] ||
    fail 'source is not a standalone checkout'
  [[ $(/usr/bin/git -C "$source" rev-parse --is-shallow-repository) == false ]] ||
    fail 'source must be a full non-shallow checkout'
  if /usr/bin/git -C "$source" symbolic-ref -q HEAD >/dev/null 2>&1; then
    fail 'source HEAD must be detached'
  fi
  [[ -z $(/usr/bin/git -C "$source" status --porcelain=v1 --untracked-files=all) ]] ||
    fail 'source checkout is not clean'
  [[ -z ${GIT_ALTERNATE_OBJECT_DIRECTORIES:-} ]] || fail 'source uses Git object alternates'
  [[ ! -s $source/.git/objects/info/alternates ]] || fail 'source uses Git object alternates'
  [[ -z $(find "$source/.git/objects" -type f -links +1 -print -quit) ]] ||
    fail 'source Git objects contain hardlinks'
  if /usr/bin/git -C "$source" config --get-regexp '^(extensions\.partialclone|remote\..*\.promisor)$' >/dev/null 2>&1; then
    fail 'source is a partial or promisor checkout'
  fi
  /usr/bin/git -C "$source" fsck --connectivity-only --no-dangling >/dev/null 2>&1 ||
    fail 'source object graph is incomplete'
  SOURCE_HEAD=$(/usr/bin/git -C "$source" rev-parse --verify 'HEAD^{commit}') || fail 'source HEAD is invalid'
  SOURCE_TREE=$(/usr/bin/git -C "$source" rev-parse --verify 'HEAD^{tree}') || fail 'source tree is invalid'
  [[ $SOURCE_HEAD =~ ^[0-9a-f]{40}$ && $SOURCE_TREE =~ ^[0-9a-f]{40}$ ]] ||
    fail 'source identity is not a full lowercase Git ID'
}

require_public_tools() {
  local tool
  for tool in git realpath find stat date mkdir readlink id node jq sudo unshare findmnt losetup rustup cargo rustc sha256sum; do
    command -v "$tool" >/dev/null 2>&1 || fail "required host executable is missing: $tool"
  done
  for tool in mount umount blockdev mkfs.xfs xfs_info xfs_quota filefrag fuser fallocate sync setpriv bwrap tar ionice du blkid; do
    command -v "$tool" >/dev/null 2>&1 || fail "required supervisor executable is missing: $tool"
  done
  [[ -f $receipt_helper && ! -L $receipt_helper ]] || fail 'wrapper receipt helper is missing or symlinked'
}

capture_mount_state() {
  local path=$1
  local stdout_path=$2
  local stderr_path=$3
  run_capture "$stdout_path" "$stderr_path" /usr/bin/findmnt -rn --mountpoint "$path" \
    -o TARGET,SOURCE,FSTYPE,OPTIONS,PROPAGATION
}

capture_host_filesystem() {
  local root=$1
  local prefix=$2
  local mount_target=''
  /usr/bin/stat -f -c 'f_frsize=%S f_blocks=%b f_bfree=%f f_bavail=%a f_type=%T f_fsid=%i' \
    "$root" >"$prefix.statfs" 2>"$prefix.statfs.stderr" || true
  /usr/bin/findmnt -T "$root" -rn -o TARGET,SOURCE,FSTYPE,OPTIONS,PROPAGATION \
    >"$prefix.findmnt" 2>"$prefix.findmnt.stderr" || true
  if [[ -s $prefix.findmnt ]]; then
    IFS=$' \t' read -r mount_target _ <"$prefix.findmnt" || true
  fi
  [[ -n $mount_target ]] || mount_target=$root
  # Query the mountpoint, not an arbitrary descendant.  `state` is silent on
  # a no-quota XFS mount; verbose state remains canonical, status-zero output
  # and proves the quota mode without making stderr part of the receipt.
  run_capture "$prefix.quota" "$prefix.quota.stderr" /usr/sbin/xfs_quota -x -c 'state -v' "$mount_target"
  printf '%s\n' "$RUN_STATUS" >"$prefix.quota.status"
}

write_fallback_facts() {
  local source=$1
  local work=$2
  local supervisor_status=$3
  local facts=$work/supervisor-facts.json
  [[ -e $facts || -L $facts ]] && return 0
  jq -n \
    --arg source "$source" --arg work "$work" --arg head "${SOURCE_HEAD:-}" \
    --arg tree "${SOURCE_TREE:-}" --argjson supervisor_status "$supervisor_status" \
    --arg image "$work/filesystem.xfs" --arg fs "$work/fs" \
    --arg proof_stdout "$work/proof.stdout" --arg proof_stderr "$work/proof.stderr" \
    --arg mount_before "$work/host-before-mount.txt" \
    --arg mount_after "$work/host-after-mount.txt" \
    --arg loops_before "$work/host-before-loops.json" \
    --arg loops_after "$work/host-after-loops.json" \
    '{
      setup_failed:true, inner_pass:false,
      candidate:{source:$source,commit:$head,tree:$tree,clean:false,source_status:$supervisor_status},
      image:{path:$image}, loop_device:{}, filesystem:{}, mount:{path:$fs},
      invocation:{argv:[],cwd:null,uid:null,gid:null,status:$supervisor_status,inner_pass:false,
        stdout_path:$proof_stdout,stderr_path:$proof_stderr},
      export:{},
      teardown:{unmounted:false,loop_detached:false,no_holder:false,
        mount_absent:false,image_unattached:false,
        host_monitor:{clean:false,before_mount:$mount_before,after_mount:$mount_after,
          before_loops:$loops_before,after_loops:$loops_after}}
    }' >"$facts"
  chmod 0644 "$facts"
}

record_fs_statfs() {
  local fs=$1
  local fragment=$2
  local output=$3
  : >"$output.stderr"
  set +e
  /usr/bin/node --input-type=module - "$fs" "$fragment" "$IMAGE_BYTES" >"$output" 2>"$output.stderr" <<'NODE'
import { statfsSync } from "node:fs";
const [root, fragmentText, limitText] = process.argv.slice(2);
const fragment = BigInt(fragmentText);
const limit = BigInt(limitText);
const stat = statfsSync(root, { bigint: true });
if (stat.type !== 1481003842n || stat.bsize !== fragment || fragment === 0n) process.exit(2);
if (stat.bavail > stat.bfree || stat.bfree > stat.blocks) process.exit(3);
const capacity = stat.blocks * fragment;
const allocated = (stat.blocks - stat.bfree) * fragment;
if (capacity > limit) process.exit(4);
process.stdout.write(JSON.stringify({
  f_type: stat.type.toString(), f_bsize: stat.bsize.toString(), f_frsize: fragment.toString(),
  f_blocks: stat.blocks.toString(), f_bfree: stat.bfree.toString(), f_bavail: stat.bavail.toString(),
  capacity_bytes: capacity.toString(), allocated_bytes: allocated.toString(),
  allocated_mib: ((allocated + 1048575n) / 1048576n).toString(),
}) + "\n");
NODE
  RUN_STATUS=$?
  set -e
  [[ $RUN_STATUS -eq 0 ]] || return 1
  jq -e '(.f_type == "1481003842" and .f_bsize == .f_frsize and
    (.f_bavail | tonumber) <= (.f_bfree | tonumber) and
    (.f_bfree | tonumber) <= (.f_blocks | tonumber) and
    (.capacity_bytes | tonumber) <= 8589934592)' "$output" >/dev/null
}

supervisor() {
  # Establish the infrastructure path before even the root check; no
  # privileged lookup may consult the caller-controlled PATH.
  export PATH=/usr/sbin:/usr/bin:/sbin:/bin
  [[ $(/usr/bin/id -u) == 0 ]] || fail 'private supervisor is root-only'
  [[ $# -eq 12 ]] || fail 'private supervisor argument shape is invalid'
  # Bash runs EXIT after unwinding function locals. These values intentionally
  # live for the private supervisor process so its EXIT trap can always detach
  # the exact loop and emit truthful failure facts after any intermediate exit.
  source=$1 work=$2 expected_head=$3 expected_tree=$4 caller_uid=$5 caller_gid=$6
  caller_user=$7 caller_path=$8 rustup_home=$9 caller_browser=${10}
  host_netns=${11} host_pidns=${12}
  fs=$work/fs checkout=$work/fs/checkout output=$work/fs/checkout/target/r2-complete-proof
  image=$work/filesystem.xfs facts=$work/supervisor-facts.json
  loop_device='' major_minor='' uuid='' fragment_size=''
  mounted=false loop_attached=false unmounted=false loop_detached=false
  mount_absent=false image_unattached=false no_holder=false
  candidate_clean=false setup_failed=true capacity_ok=false
  image_status=125 image_sync_status=125 image_logical=0 image_allocated=0
  loop_size=0 loop_attach_status=125 mkfs_status=125 mount_status=125
  inner_status=125
  export_status=125 source_inventory_sha256='' export_inventory_sha256=''
  source_evidence_manifest_path=$output/EVIDENCE.sha256 inner_evidence_manifest_path=''
  setup_statfs=$work/statfs-mounted.json checkout_statfs=$work/statfs-checkout.json close_statfs=$work/statfs-close.json
  inner_start_ns='' inner_end_ns=''
  fuser_status=125 loop_fuser_status=125 sync_before_umount_status=125 umount_status=125 detach_status=125
  mount_check_status=125
  image_filefrag=$work/image.filefrag xfs_info_file=$work/xfs-info.txt
  xfs_uuid_status=125 archive=$work/checkout.tar
  archive_status=125 clone_status=125
  image_stat=$work/image.stat image_fallocate_stdout=$work/image-fallocate.stdout
  image_fallocate_stderr=$work/image-fallocate.stderr image_sync_stdout=$work/image-sync.stdout
  image_sync_stderr=$work/image-sync.stderr
  supervisor_exit=1
  mount_options='' mount_propagation='' mount_target='' mount_source='' mount_type=''
  user_env=$work/user-env
  source_status=125 filter_status=125
  image_blocks=0 image_block_size=0
  export_root=$work/export export_parent=$work/export/target
  export_destination=$work/export/target/r2-complete-proof
  proof_script=$checkout/docs/evaluation/r2-complete-proof.sh
  tool_register=$work/wrapper-tools.tsv tool_register_json=$work/wrapper-tools.json
  wrapper_command_ledger=$work/wrapper-commands.ndjson
  tool_register_json_value='{}'

  # Every repository read, including the source revalidation, runs as the
  # original caller.  This avoids changing Git's global safe.directory policy
  # merely because the supervisor is privileged.
  run_as_user() {
    local -a environment=(
      PATH="$caller_path"
      HOME="$user_env/home" TMPDIR="$user_env/tmp"
      XDG_CACHE_HOME="$user_env/xdg-cache" XDG_CONFIG_HOME="$user_env/xdg-config"
      XDG_DATA_HOME="$user_env/xdg-data" CARGO_HOME="$user_env/cargo-home"
      CARGO_TARGET_TMPDIR="$user_env/tmp/cargo-target" RUSTUP_HOME="$rustup_home"
      RUSTUP_NO_UPDATE_CHECK=1 CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0
      CARGO_TERM_COLOR=never LC_ALL=C LANG=C USER="$caller_user" LOGNAME="$caller_user"
      GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_GLOBAL=/dev/null
      GIT_TERMINAL_PROMPT=0 GIT_OPTIONAL_LOCKS=0
    )
    [[ -n $caller_browser ]] && environment+=(CHROME_BIN="$caller_browser")
    setpriv --reuid="$caller_uid" --regid="$caller_gid" --init-groups \
      --inh-caps=-all --ambient-caps=-all --bounding-set=-all --no-new-privs -- \
      env -i "${environment[@]}" "$@"
  }

  write_facts() {
    local tmp=$facts.tmp
    local inner_pass=false export_equal=false
    [[ $inner_status -eq 0 ]] && inner_pass=true
    [[ -n $source_inventory_sha256 && $source_inventory_sha256 == "$export_inventory_sha256" && $export_status -eq 0 ]] && export_equal=true
    jq -n \
      --arg source "$source" --arg head "$expected_head" --arg tree "$expected_tree" \
      --argjson source_status "${source_status:-125}" --argjson candidate_clean "$candidate_clean" \
      --arg image "$image" --arg image_stat "$image_stat" --arg image_filefrag "$image_filefrag" \
      --arg image_fallocate_stdout "$image_fallocate_stdout" --arg image_fallocate_stderr "$image_fallocate_stderr" \
      --arg image_sync_stdout "$image_sync_stdout" --arg image_sync_stderr "$image_sync_stderr" \
      --argjson image_status "$image_status" --argjson image_sync_status "$image_sync_status" \
      --arg image_logical "$image_logical" --arg image_allocated "$image_allocated" \
      --argjson loop_attached "$loop_attached" --arg loop_device "$loop_device" --arg major_minor "$major_minor" \
      --arg loop_size "$loop_size" --argjson loop_attach_status "$loop_attach_status" \
      --argjson mkfs_status "$mkfs_status" --arg xfs_info "$xfs_info_file" \
      --arg uuid "$uuid" --argjson mount_status "$mount_status" --argjson mounted "$mounted" \
      --arg fs "$fs" --arg mount_options "$mount_options" --arg mount_propagation "$mount_propagation" \
      --arg fragment_size "$fragment_size" --argjson capacity_ok "$capacity_ok" \
      --arg setup_statfs "$setup_statfs" --arg checkout_statfs "$checkout_statfs" \
      --arg close_statfs "$close_statfs" --arg checkout "$checkout" --arg output "$output" \
      --arg proof_script "$proof_script" --arg receipt_helper "$receipt_helper" --arg work "$work" \
      --argjson inner_status "$inner_status" \
      --argjson inner_pass "$inner_pass" --arg inner_start_ns "$inner_start_ns" --arg inner_end_ns "$inner_end_ns" \
      --arg proof_stdout "$work/proof.stdout" --arg proof_stderr "$work/proof.stderr" \
      --arg caller_uid "$caller_uid" --arg caller_gid "$caller_gid" --arg caller_user "$caller_user" \
      --argjson wrapper_tools "$tool_register_json_value" \
      --argjson export_status "$export_status" --argjson export_equal "$export_equal" \
      --arg export_destination "$export_destination" --arg inventory_path "$work/export.json" \
      --arg export_sha "$work/export.sha256" \
      --arg source_inventory_sha256 "$source_inventory_sha256" --arg export_inventory_sha256 "$export_inventory_sha256" \
      --arg inner_evidence_manifest_path "${inner_evidence_manifest_path:-}" \
      --argjson supervisor_exit "$supervisor_exit" --argjson setup_failed "$setup_failed" \
      --argjson unmounted "$unmounted" --argjson loop_detached "$loop_detached" --argjson no_holder "$no_holder" \
      --argjson mount_absent "$mount_absent" --argjson image_unattached "$image_unattached" \
      --argjson fuser_status "$fuser_status" --argjson sync_before_umount_status "$sync_before_umount_status" \
      --argjson umount_status "$umount_status" --argjson detach_status "$detach_status" \
      --arg loop_path "$loop_device" --arg host_before "$work/host-filesystem-before" \
      --arg host_after "$work/host-filesystem-after" \
      'def nz: if . == "" then null else . end;
       {setup_failed:$setup_failed,inner_pass:$inner_pass,
        candidate:{source:$source,commit:$head,tree:$tree,clean:$candidate_clean,source_status:$source_status},
        image:{path:$image,stat_path:$image_stat,filefrag_path:$image_filefrag,
          fallocate_stdout:$image_fallocate_stdout,fallocate_stderr:$image_fallocate_stderr,
          sync_stdout:$image_sync_stdout,sync_stderr:$image_sync_stderr,status:$image_status,
          sync_status:$image_sync_status,logical_bytes:$image_logical,allocated_bytes:$image_allocated,
          expected_bytes:"8589934592"},
        loop_device:{path:($loop_device|nz),major_minor:($major_minor|nz),size_bytes:($loop_size|nz),attached:$loop_attached},
        filesystem:{type:"xfs",uuid:($uuid|nz),fragment_size:($fragment_size|nz),capacity_limit_bytes:"8589934592",capacity_ok:$capacity_ok,
          mounted_statfs_path:$setup_statfs,checkout_statfs_path:$checkout_statfs,close_statfs_path:$close_statfs,
          host_filesystem_before_path:$host_before,host_filesystem_after_path:$host_after,xfs_info_path:$xfs_info},
        mount:{path:$fs,source:($loop_device|nz),options:($mount_options|nz),propagation:($mount_propagation|nz),status:$mount_status,mounted:$mounted,unmounted:$unmounted,mount_absent:$mount_absent},
        invocation:{argv:["/usr/bin/bash",$proof_script,"--output",$output],cwd:$checkout,uid:($caller_uid|tonumber),gid:($caller_gid|tonumber),user:$caller_user,status:$inner_status,inner_pass:$inner_pass,start_ns:($inner_start_ns|nz),end_ns:($inner_end_ns|nz),stdout_path:$proof_stdout,stderr_path:$proof_stderr},
        export:{source:$output,destination:$export_destination,status:$export_status,equal:$export_equal,source_inventory_sha256:($source_inventory_sha256|nz),export_inventory_sha256:($export_inventory_sha256|nz),inventory_path:$inventory_path,inventory_digest_path:$export_sha,inner_evidence_manifest_path:($inner_evidence_manifest_path|nz)},
        teardown:{unmounted:$unmounted,loop_detached:$loop_detached,no_holder:$no_holder,mount_absent:$mount_absent,image_unattached:$image_unattached,fuser_status:$fuser_status,umount_status:$umount_status,detach_status:$detach_status,supervisor_status:$supervisor_exit,host_monitor:{clean:false,mountpoint:$fs,proof_loop_device:($loop_path|nz),new_loop_devices:[],mount_namespace:""}},
        host_monitor:{clean:false,mountpoint:$fs,proof_loop_device:($loop_path|nz),new_loop_devices:[],mount_namespace:""},
        operations:{
          fallocate:{argv:["/usr/bin/fallocate","--posix","--length","8589934592",$image],cwd:$work,status:$image_status,stdout_path:$image_fallocate_stdout,stderr_path:$image_fallocate_stderr},
          image_sync:{argv:["/usr/bin/sync","-f",$image],cwd:$work,status:$image_sync_status,stdout_path:$image_sync_stdout,stderr_path:$image_sync_stderr},
          loop_attach:{argv:["/usr/sbin/losetup","--find","--show",$image],cwd:$work,status:$loop_attach_status,stdout_path:($work+"/loop-attach.stdout"),stderr_path:($work+"/loop-attach.stderr")},
          mkfs_xfs:{argv:["/usr/sbin/mkfs.xfs","-f","-l","internal",($loop_path|nz)],cwd:$work,status:$mkfs_status,stdout_path:($work+"/mkfs-xfs.stdout"),stderr_path:($work+"/mkfs-xfs.stderr")},
          mount:{argv:["/usr/bin/mount","-t","xfs","-o","rw,nodev,nosuid",($loop_path|nz),$fs],cwd:$work,status:$mount_status,stdout_path:($work+"/mount.stdout"),stderr_path:($work+"/mount.stderr")},
          proof:{argv:["/usr/bin/bash",$proof_script,"--output",$output],cwd:$checkout,status:$inner_status,stdout_path:$proof_stdout,stderr_path:$proof_stderr},
          export:{argv:["/usr/bin/node",$receipt_helper,"copy","--source",$output,"--destination",$export_destination,"--output",$inventory_path],cwd:$work,status:$export_status,stdout_path:($work+"/export.stdout"),stderr_path:($work+"/export.stderr")},
          sync_before_umount:{argv:["/usr/bin/sync","-f",$fs],cwd:$work,status:$sync_before_umount_status,stdout_path:($work+"/sync-before-umount.stdout"),stderr_path:($work+"/sync-before-umount.stderr")},
          umount:{argv:["/usr/bin/umount",$fs],cwd:$work,status:$umount_status,stdout_path:($work+"/umount.stdout"),stderr_path:($work+"/umount.stderr")},
          loop_detach:{argv:["/usr/sbin/losetup","--detach",($loop_path|nz)],cwd:$work,status:$detach_status,stdout_path:($work+"/loop-detach.stdout"),stderr_path:($work+"/loop-detach.stderr")}
        },
        tool_register:$wrapper_tools}' >"$tmp"
    chmod 0644 "$tmp"
    chown "$caller_uid:$caller_gid" "$tmp" 2>/dev/null || true
    mv -f -- "$tmp" "$facts"
    chmod 0644 "$facts"
    chown "$caller_uid:$caller_gid" "$facts" 2>/dev/null || true
  }

  # EXIT trap invokes this function indirectly.
  # shellcheck disable=SC2329
  supervisor_cleanup() {
    local rc=$?
    set +e
    cd -- "$work" 2>/dev/null || true
    if [[ $mounted == true && $unmounted != true ]]; then
      /usr/bin/fuser -m "$fs" >"$work/fuser-cleanup.stdout" 2>"$work/fuser-cleanup.stderr"
      fuser_status=$?
      if [[ $fuser_status -eq 1 && ! -s $work/fuser-cleanup.stdout &&
            ! -s $work/fuser-cleanup.stderr ]]; then
        no_holder=true
      fi
      /usr/bin/sync -f "$fs" >"$work/sync-cleanup.stdout" 2>"$work/sync-cleanup.stderr" || true
      /usr/bin/umount "$fs" >"$work/umount-cleanup.stdout" 2>"$work/umount-cleanup.stderr"
      umount_status=$?
      if [[ $umount_status -eq 0 ]]; then unmounted=true; fi
    fi
    if [[ $mounted != true ]]; then
      mount_absent=true
    else
      /usr/bin/findmnt -rn --mountpoint "$fs" >"$work/mount-cleanup.txt" 2>"$work/mount-cleanup.stderr"
      local mount_cleanup_status=$?
      if [[ $mount_cleanup_status -eq 1 && ! -s $work/mount-cleanup.txt ]]; then
        mount_absent=true
      else
        mount_absent=false
      fi
    fi
    if [[ $loop_attached == true && $loop_detached != true && $mount_absent == true ]]; then
      /usr/sbin/losetup --detach "$loop_device" >"$work/detach-cleanup.stdout" 2>"$work/detach-cleanup.stderr"
      detach_status=$?
      if [[ $detach_status -eq 0 ]]; then loop_detached=true; loop_attached=false; fi
      /usr/bin/fuser "$loop_device" >"$work/loop-fuser-cleanup.stdout" 2>"$work/loop-fuser-cleanup.stderr"
      local loop_fuser_status=$?
      run_capture "$work/loop-associated-cleanup.stdout" "$work/loop-associated-cleanup.stderr" \
        /usr/sbin/losetup --associated "$image"
      if [[ $loop_fuser_status -eq 1 && ! -s $work/loop-fuser-cleanup.stdout &&
            ! -s $work/loop-fuser-cleanup.stderr && $RUN_STATUS -eq 0 &&
            ! -s $work/loop-associated-cleanup.stdout && ! -s $work/loop-associated-cleanup.stderr ]]; then
        image_unattached=true
      fi
    fi
    supervisor_exit=$rc
    write_facts >/dev/null 2>&1 || true
    exit "$rc"
  }
  trap 'exit 130' INT
  trap 'exit 143' TERM
  trap supervisor_cleanup EXIT

  # This must be the first privileged operation inside the fresh namespace.
  # Keep every privileged lookup on a fixed system path.  The caller's PATH
  # is passed only inside run_as_user after the identity/capability drop.
  /usr/bin/mount --make-rprivate /
  mkdir -p -- "$work"
  cd -- "$work"
  [[ $(realpath -e -- "$source") == "$source" && $(realpath -e -- "$work") == "$work" ]] || fail 'supervisor paths changed'
  mkdir -p -- "$user_env/home" "$user_env/tmp/cargo-target" "$user_env/xdg-cache" \
    "$user_env/xdg-config" "$user_env/xdg-data" "$user_env/cargo-home"
  chown -R "$caller_uid:$caller_gid" "$user_env"
  : >"$wrapper_command_ledger"
  record_wrapper_tools "$tool_register"
  record_wrapper_user_tool "$tool_register" rustup
  record_wrapper_user_tool "$tool_register" cargo
  record_wrapper_user_tool "$tool_register" rustc
  record_wrapper_tool_json "$tool_register" "$tool_register_json"
  tool_register_json_value=$(<"$tool_register_json")
  [[ $(run_as_user /usr/bin/git -C "$source" rev-parse --verify 'HEAD^{commit}') == "$expected_head" &&
     $(run_as_user /usr/bin/git -C "$source" rev-parse --verify 'HEAD^{tree}') == "$expected_tree" ]] || fail 'source identity changed before provisioning'
  if run_as_user /usr/bin/git -C "$source" symbolic-ref -q HEAD >/dev/null 2>&1; then fail 'source HEAD became attached'; fi
  [[ -z $(run_as_user /usr/bin/git -C "$source" status --porcelain=v1 --untracked-files=all) ]] || fail 'source became dirty'
  source_status=0

  [[ ! -e $image && ! -L $image ]] || fail 'backing image path is not fresh'
  # This is the true preallocation snapshot: no image bytes have been
  # allocated yet, and fallocate is the very next provisioning operation.
  capture_host_filesystem "$work" "$work/host-filesystem-before"
  run_capture "$image_fallocate_stdout" "$image_fallocate_stderr" \
    /usr/bin/fallocate --posix --length 8589934592 "$image"
  image_status=$RUN_STATUS
  [[ $image_status -eq 0 ]] || fail 'exact backing-image preallocation failed'
  run_capture "$image_sync_stdout" "$image_sync_stderr" /usr/bin/sync -f "$image"
  image_sync_status=$RUN_STATUS
  [[ $image_sync_status -eq 0 ]] || fail 'backing-image sync failed'
  read -r image_logical image_blocks image_block_size < <(/usr/bin/stat -c '%s %b %B' "$image")
  [[ $image_block_size == 512 ]] || fail 'st_blocks fundamental unit is not 512 bytes'
  image_allocated=$((image_blocks * 512))
  printf '%s\n' "logical_bytes=$image_logical" "st_blocks=$image_blocks" "allocated_bytes=$image_allocated" "block_size=$image_block_size" >"$image_stat"
  [[ $image_logical == "$IMAGE_BYTES" && $image_allocated -ge $IMAGE_BYTES ]] || fail 'backing image is not exact and fully allocated'
  run_capture "$image_filefrag" "$work/image.filefrag.stderr" /usr/sbin/filefrag -v "$image"
  [[ $RUN_STATUS -eq 0 ]] || fail 'filefrag extent evidence failed'
  # The postallocation snapshot follows sync, exact-size/stat-block, and
  # extent evidence, so the two records cannot be confused in the receipt.
  capture_host_filesystem "$work" "$work/host-filesystem-after"

  run_capture "$work/loop-attach.stdout" "$work/loop-attach.stderr" /usr/sbin/losetup --find --show "$image"
  loop_attach_status=$RUN_STATUS
  [[ $loop_attach_status -eq 0 ]] || fail 'loop attachment failed'
  loop_device=$(tr -d '\r\n' <"$work/loop-attach.stdout")
  [[ $loop_device =~ ^/dev/loop[0-9]+$ ]] || fail 'loop attachment returned an invalid device'
  loop_attached=true
  run_capture "$work/loop-size.stdout" "$work/loop-size.stderr" /usr/sbin/blockdev --getsize64 "$loop_device"
  [[ $RUN_STATUS -eq 0 ]] || fail 'loop size query failed'
  loop_size=$(tr -d '\r\n' <"$work/loop-size.stdout")
  [[ $loop_size == "$IMAGE_BYTES" ]] || fail 'loop device size is not exact'
  loop_hex_major=$(/usr/bin/stat -c '%t' "$loop_device")
  loop_hex_minor=$(/usr/bin/stat -c '%T' "$loop_device")
  major_decimal=$((16#$loop_hex_major))
  minor_decimal=$((16#$loop_hex_minor))
  major_minor=$major_decimal:$minor_decimal
  [[ $major_minor =~ ^[0-9]+:[0-9]+$ ]] || fail 'loop major:minor is malformed'

  run_capture "$work/mkfs-xfs.stdout" "$work/mkfs-xfs.stderr" /usr/sbin/mkfs.xfs -f -l internal "$loop_device"
  mkfs_status=$RUN_STATUS
  [[ $mkfs_status -eq 0 ]] || fail 'XFS formatting failed'
  run_capture "$xfs_info_file" "$work/xfs-info.stderr" /usr/sbin/xfs_info "$loop_device"
  [[ $RUN_STATUS -eq 0 ]] || fail 'XFS identity inspection failed'
  grep -Eq '(^|[[:space:]])log[[:space:]]*=internal([[:space:]]|$)' "$xfs_info_file" || fail 'XFS log is not internal'
  grep -Eq '(^|[[:space:]])realtime[[:space:]]*=none([[:space:]]|$)' "$xfs_info_file" || fail 'XFS realtime device is not none'
  run_capture "$work/blkid.stdout" "$work/blkid.stderr" /usr/sbin/blkid -p -s TYPE -o value "$loop_device"
  [[ $RUN_STATUS -eq 0 && $(<"$work/blkid.stdout") == xfs ]] || fail 'loop device is not XFS'
  run_capture "$work/xfs-uuid.stdout" "$work/xfs-uuid.stderr" /usr/sbin/blkid -p -s UUID -o value "$loop_device"
  xfs_uuid_status=$RUN_STATUS
  uuid=$(tr -d '\r\n' <"$work/xfs-uuid.stdout" | tr '[:upper:]' '[:lower:]')
  [[ $xfs_uuid_status -eq 0 && $uuid =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] || fail 'XFS UUID is missing or malformed'

  mkdir -p -- "$fs"
  run_capture "$work/mount.stdout" "$work/mount.stderr" /usr/bin/mount -t xfs -o rw,nodev,nosuid "$loop_device" "$fs"
  mount_status=$RUN_STATUS
  [[ $mount_status -eq 0 ]] || fail 'XFS mount failed'
  mounted=true
  mount_record=$(/usr/bin/findmnt -T "$fs" -rn -o TARGET,SOURCE,FSTYPE,OPTIONS,PROPAGATION) || fail 'mounted XFS record missing'
  read -r mount_target mount_source mount_type mount_options mount_propagation <<<"$mount_record"
  [[ $mount_target == "$fs" && $mount_source == "$loop_device" && $mount_type == xfs ]] || fail 'mounted XFS identity differs'
  [[ ,$mount_options, == *,rw,* && ,$mount_options, == *,nodev,* && ,$mount_options, == *,nosuid,* ]] || fail 'XFS mount options lack rw,nodev,nosuid'
  [[ $mount_propagation == private ]] || fail 'XFS mount is not private'
  fragment_size=$(/usr/bin/stat -f -c '%S' "$fs")
  [[ $fragment_size =~ ^[1-9][0-9]*$ ]] || fail 'XFS fragment size is malformed'
  record_fs_statfs "$fs" "$fragment_size" "$setup_statfs" || fail 'mounted XFS statfs is invalid'
  capacity_ok=true
  # The checkout destination is intentionally nonexistent.  Give its parent
  # XFS root to the original identity so git clone can create it without any
  # privileged repository operation.
  chown "$caller_uid:$caller_gid" "$fs"

  # Clone, archive, and output creation are all performed with the original
  # identity.  The environment is allowlisted and Git hooks/config filters
  # are disabled before any checkout material is materialized.
  [[ ! -e $checkout && ! -L $checkout ]] || fail 'checkout destination is not fresh'
  run_capture "$work/clone.stdout" "$work/clone.stderr" run_as_user /usr/bin/git \
    -c protocol.file.allow=always clone --no-local --no-hardlinks --no-checkout \
    --config core.hooksPath=/dev/null "$source" "$checkout"
  clone_status=$RUN_STATUS
  [[ $clone_status -eq 0 ]] || fail 'standalone clone failed'
  [[ -d $checkout/.git && ! -L $checkout/.git ]] || fail 'clone did not produce local Git metadata'
  run_as_user /usr/bin/git -C "$checkout" config --local core.hooksPath /dev/null
  run_as_user /usr/bin/git -C "$checkout" config --local core.autocrlf false
  set +e
  run_as_user /usr/bin/git -C "$checkout" config --local --get-regexp '^filter\..*\.(clean|smudge|process)$' \
    >"$work/clone-filters.txt" 2>"$work/clone-filters.stderr"
  filter_status=$?
  set -e
  [[ $filter_status -eq 1 && ! -s $work/clone-filters.txt ]] || fail 'clone contains active checkout filters'
  run_as_user /usr/bin/git -C "$checkout" update-ref --no-deref HEAD "$expected_head"
  run_as_user /usr/bin/git -C "$checkout" read-tree "$expected_head"

  # `git archive` emits blob bytes and does not run smudge/clean filters.  A
  # member-list check precedes extraction and rejects path traversal or names
  # which cannot be represented by the canonical receipt ledger.
  : >"$archive"
  chown "$caller_uid:$caller_gid" "$archive"
  set +e
  run_as_user /usr/bin/git -C "$checkout" archive --format=tar "$expected_head" \
    >"$archive" 2>"$work/archive.stderr"
  archive_status=$?
  set -e
  [[ $archive_status -eq 0 ]] || fail 'checkout archive generation failed'
  tar -tf "$archive" >"$work/archive.members" || fail 'checkout archive listing failed'
  while IFS= read -r member; do
    [[ -n $member && $member != /* && $member != *$'\t'* && $member != *$'\r'* && $member != *$'\n'* ]] ||
      fail 'checkout archive contains an unsafe member path'
    member=${member%/}
    [[ $member != . && $member != .. && $member != */../* && $member != ../* && $member != */.. ]] ||
      fail 'checkout archive contains parent traversal'
  done <"$work/archive.members"
  run_as_user /usr/bin/tar --extract --file "$archive" --directory "$checkout" \
    --no-same-owner --no-same-permissions --no-overwrite-dir
  rm -f -- "$archive"
  [[ $(run_as_user /usr/bin/git -C "$checkout" rev-parse --verify 'HEAD^{commit}') == "$expected_head" &&
     $(run_as_user /usr/bin/git -C "$checkout" rev-parse --verify 'HEAD^{tree}') == "$expected_tree" ]] ||
    fail 'cloned checkout identity differs'
  [[ -z $(run_as_user /usr/bin/git -C "$checkout" status --porcelain=v1 --untracked-files=all) ]] ||
    fail 'archive checkout is not clean'
  run_as_user /usr/bin/git -C "$checkout" fsck --connectivity-only --no-dangling >/dev/null 2>&1 ||
    fail 'cloned checkout object graph is incomplete'
  [[ ! -e $checkout/target && ! -L $checkout/target ]] || fail 'clone already contains target'
  run_as_user /usr/bin/mkdir -- "$checkout/target"
  run_as_user /usr/bin/mkdir -- "$output"
  [[ -d $output && ! -L $output ]] || fail 'proof output is not a real directory'
  run_as_user /usr/bin/git -C "$checkout" check-ignore -q --no-index -- target/r2-complete-proof ||
    fail 'proof output is not ignored by the checkout'
  [[ $(find "$fs" -mindepth 1 -maxdepth 1 -printf '%f\n') == checkout ]] ||
    fail 'checkout is not the XFS sole top-level entry'
  record_fs_statfs "$fs" "$fragment_size" "$checkout_statfs" || fail 'XFS capacity changed above ceiling after checkout'
  candidate_clean=true
  setup_failed=false

  # The inner proof owns its exact reservation, setup/shutdown du crosschecks,
  # persistent kernel-accounting sampler, and final no-write proof.  The
  # wrapper records only the fixed-capacity checkpoints around that child.
  inner_start_ns=$(date +%s%N)
  set +e
  (cd -- "$checkout" && run_as_user /usr/bin/env \
    NOMOS_R2_XFS_WRAPPER=1 NOMOS_R2_XFS_UUID="$uuid" \
    NOMOS_R2_XFS_FRAGMENT_SIZE="$fragment_size" NOMOS_R2_XFS_DEVICE="$loop_device" \
    NOMOS_R2_XFS_MAJOR_MINOR="$major_minor" NOMOS_R2_HOST_NETNS="$host_netns" \
    NOMOS_R2_HOST_PIDNS="$host_pidns" \
    /usr/bin/bash "$checkout/docs/evaluation/r2-complete-proof.sh" --output "$output") \
    >"$work/proof.stdout" 2>"$work/proof.stderr"
  inner_status=$?
  set -e
  inner_end_ns=$(date +%s%N)
  record_wrapper_command inner-proof "$inner_start_ns" "$inner_end_ns" "$inner_status" \
    "$work/proof.stdout" "$work/proof.stderr" "$caller_uid" "$caller_gid" "$checkout" \
    env NOMOS_R2_XFS_WRAPPER=1 NOMOS_R2_XFS_UUID="$uuid" \
    NOMOS_R2_XFS_FRAGMENT_SIZE="$fragment_size" NOMOS_R2_XFS_DEVICE="$loop_device" \
    NOMOS_R2_XFS_MAJOR_MINOR="$major_minor" NOMOS_R2_HOST_NETNS="$host_netns" \
    NOMOS_R2_HOST_PIDNS="$host_pidns" /usr/bin/bash "$proof_script" --output "$output"
  record_fs_statfs "$fs" "$fragment_size" "$close_statfs" || fail 'close statfs checkpoint failed'
  [[ $(find "$fs" -mindepth 1 -maxdepth 1 -printf '%f\n') == checkout ]] ||
    fail 'checkout is not the XFS sole top-level entry after proof closure'

  export_root=$work/export
  export_parent=$export_root/target
  export_destination=$export_parent/r2-complete-proof
  run_as_user /usr/bin/mkdir -p -- "$export_parent"
  run_capture "$work/export.stdout" "$work/export.stderr" run_as_user /usr/bin/node "$receipt_helper" copy \
    --source "$output" --destination "$export_destination" --output "$work/export.json"
  export_status=$RUN_STATUS
  if [[ $export_status -eq 0 ]]; then
    source_inventory_sha256=$(jq -r '.source_inventory_sha256' "$work/export.json")
    export_inventory_sha256=$(jq -r '.export_inventory_sha256' "$work/export.json")
    printf 'source\t%s\nexport\t%s\n' "$source_inventory_sha256" "$export_inventory_sha256" >"$work/export.sha256"
  fi
  if [[ -L $source_evidence_manifest_path ]]; then fail 'inner EVIDENCE.sha256 is a symlink'; fi
  if [[ -e $source_evidence_manifest_path && ! -f $source_evidence_manifest_path ]]; then
    fail 'inner EVIDENCE.sha256 is not a regular file'
  fi
  if [[ -f $source_evidence_manifest_path ]]; then
    if [[ $export_status -eq 0 && -f $export_destination/EVIDENCE.sha256 &&
          ! -L $export_destination/EVIDENCE.sha256 ]]; then
      # Receipt assembly happens after unmount; bind the retained exported
      # spelling, not the now-hidden source spelling, into the facts.
      inner_evidence_manifest_path=$export_destination/EVIDENCE.sha256
    elif [[ $export_status -eq 0 ]]; then
      export_status=1
    fi
  else
    inner_evidence_manifest_path=''
  fi

  cd -- "$work"
  run_capture "$work/fuser.stdout" "$work/fuser.stderr" /usr/bin/fuser -m "$fs"
  fuser_status=$RUN_STATUS
  [[ $fuser_status -eq 1 && ! -s $work/fuser.stdout && ! -s $work/fuser.stderr ]] && no_holder=true || no_holder=false
  [[ $no_holder == true ]] || fail 'a process still holds the XFS mount'
  run_capture "$work/sync-before-umount.stdout" "$work/sync-before-umount.stderr" /usr/bin/sync -f "$fs"
  sync_before_umount_status=$RUN_STATUS
  [[ $sync_before_umount_status -eq 0 ]] || fail 'ordinary mount sync failed'
  run_capture "$work/umount.stdout" "$work/umount.stderr" /usr/bin/umount "$fs"
  umount_status=$RUN_STATUS
  [[ $umount_status -eq 0 ]] || fail 'ordinary XFS umount failed'
  unmounted=true
  if /usr/bin/findmnt -rn --mountpoint "$fs" >"$work/supervisor-mount-after.txt" 2>"$work/supervisor-mount-after.stderr"; then
    mount_absent=false
  else
    mount_check_status=$?
    [[ $mount_check_status -eq 1 && ! -s $work/supervisor-mount-after.txt ]] && mount_absent=true || mount_absent=false
  fi
  [[ $mount_absent == true ]] || fail 'XFS mount remains after ordinary umount'
  run_capture "$work/loop-detach.stdout" "$work/loop-detach.stderr" /usr/sbin/losetup --detach "$loop_device"
  detach_status=$RUN_STATUS
  [[ $detach_status -eq 0 ]] || fail 'loop detach failed'
  loop_detached=true
  run_capture "$work/loop-fuser.stdout" "$work/loop-fuser.stderr" /usr/bin/fuser "$loop_device"
  loop_fuser_status=$RUN_STATUS
  if [[ $loop_fuser_status -eq 1 && ! -s $work/loop-fuser.stdout && ! -s $work/loop-fuser.stderr ]]; then
    image_unattached=true
  else
    image_unattached=false
  fi
  run_capture "$work/loop-associated.stdout" "$work/loop-associated.stderr" /usr/sbin/losetup --associated "$image"
  [[ $RUN_STATUS -eq 0 && ! -s $work/loop-associated.stdout && ! -s $work/loop-associated.stderr ]] || image_unattached=false
  [[ $image_unattached == true ]] || fail 'detached loop still has a holder or image attachment'
  loop_attached=false
  supervisor_exit=0
  [[ $inner_status -eq 0 && $export_status -eq 0 && $unmounted == true &&
     $mount_absent == true && $loop_detached == true && $no_holder == true &&
     $image_unattached == true ]] || supervisor_exit=1
  write_facts
  exit "$supervisor_exit"
}

if [[ ${1:-} == --supervise ]]; then
  shift
  supervisor "$@"
fi

[[ $# -eq 4 && $1 == --source && $3 == --work ]] ||
  fail 'usage: r2-complete-proof-xfs.sh --source CLEAN --work EMPTY'
[[ ${BASH_SOURCE[0]} == "$0" ]] || fail 'wrapper must be executed, not sourced'

source_argument=$2
work_argument=$4
source=$(canonical_existing "$source_argument" source)
work=$(canonical_existing "$work_argument" work)
[[ -d $source && ! -L $source ]] || fail 'source must be a real directory'
[[ -d $work && ! -L $work ]] || fail 'work must be a real directory'
validate_source "$source"
assert_source_head=$SOURCE_HEAD
assert_source_tree=$SOURCE_TREE
if [[ $source == "$work" || $source == "$work/"* || $work == "$source/"* ]]; then
  fail 'source and work paths overlap'
fi
[[ -z $(find "$work" -mindepth 1 -print -quit) ]] || fail 'work directory must be empty'
require_public_tools

caller_uid=$(id -u)
caller_gid=$(id -g)
caller_user=$(id -un)
caller_path=$PATH
[[ -n $caller_path && $caller_path != *$'\r'* &&
   $caller_path != *$'\n'* && $caller_path != *$'\t'* ]] || fail 'caller PATH is not safe'
rustup_home=${RUSTUP_HOME:-$(rustup show home)}
rustup_home=$(canonical_existing "$rustup_home" RUSTUP_HOME)
caller_browser=${CHROME_BIN:-}
if [[ -n $caller_browser ]]; then caller_browser=$(canonical_existing "$caller_browser" CHROME_BIN); fi
host_mount_namespace=$(readlink /proc/self/ns/mnt)
host_net_namespace=$(readlink /proc/self/ns/net)
host_pid_namespace=$(readlink /proc/self/ns/pid)
[[ $host_mount_namespace =~ ^mnt:\[[0-9]+\]$ && $host_net_namespace =~ ^net:\[[0-9]+\]$ &&
  $host_pid_namespace =~ ^pid:\[[0-9]+\]$ ]] || fail 'host namespace identities are malformed'

# Precreate host-side evidence paths only after the EMPTY precondition has
# been proved.  None is beneath the eventual XFS mount.
for file in \
  host-before-mount.txt host-before-mount.stderr host-before-loops.json \
  host-before-loops.stderr host-before-mnt-ns host-before-net-ns host-before-pid-ns \
  host-after-mount.txt host-after-mount.stderr host-after-loops.json host-after-loops.stderr \
  host-after-mnt-ns host-after-net-ns host-after-pid-ns proof.stdout proof.stderr \
  supervisor.stdout supervisor.stderr; do
  : >"$work/$file"
done
capture_mount_state "$work/fs" "$work/host-before-mount.txt" "$work/host-before-mount.stderr"
[[ ! -s $work/host-before-mount.txt ]] || fail 'work/fs is already mounted'
printf '%s\n' "$host_mount_namespace" >"$work/host-before-mnt-ns"
printf '%s\n' "$host_net_namespace" >"$work/host-before-net-ns"
printf '%s\n' "$host_pid_namespace" >"$work/host-before-pid-ns"
set +e
/usr/sbin/losetup --list --json >"$work/host-before-loops.json" 2>"$work/host-before-loops.stderr"
before_loop_status=$?
set -e
[[ $before_loop_status -eq 0 ]] || fail 'unprivileged host loop monitor is unavailable'
capture_host_filesystem "$work" "$work/host-monitor-filesystem-before"

set +e
# shellcheck disable=SC2024
/usr/bin/sudo -n /usr/bin/unshare --mount --propagation private --fork --kill-child=TERM "$self_path" --supervise \
  "$source" "$work" "$assert_source_head" "$assert_source_tree" \
  "$caller_uid" "$caller_gid" "$caller_user" "$caller_path" "$rustup_home" \
  "$caller_browser" "$host_net_namespace" "$host_pid_namespace" \
  >"$work/supervisor.stdout" 2>"$work/supervisor.stderr"
supervisor_status=$?
set -e

capture_mount_state "$work/fs" "$work/host-after-mount.txt" "$work/host-after-mount.stderr"
printf '%s\n' "$(readlink /proc/self/ns/mnt)" >"$work/host-after-mnt-ns"
printf '%s\n' "$(readlink /proc/self/ns/net)" >"$work/host-after-net-ns"
printf '%s\n' "$(readlink /proc/self/ns/pid)" >"$work/host-after-pid-ns"
set +e
/usr/sbin/losetup --list --json >"$work/host-after-loops.json" 2>"$work/host-after-loops.stderr"
after_loop_status=$?
set -e
if [[ $after_loop_status -ne 0 ]]; then : >"$work/host-after-loops.json"; fi
# Preserve the supervisor's exact postallocation record; these later host
# snapshots are separate teardown-time observations and must not overwrite it.
capture_host_filesystem "$work" "$work/host-monitor-filesystem-after"

host_monitor=$work/host-monitor.json
facts=$work/supervisor-facts.json
proof_loop_device=
if [[ -f $facts && ! -L $facts ]]; then
  proof_loop_device=$(jq -r '.loop_device.path // empty' "$facts" 2>/dev/null || true)
  [[ $proof_loop_device =~ ^/dev/loop[0-9]+$ ]] || proof_loop_device=
fi
if [[ -f $work/filesystem.xfs && ! -L $work/filesystem.xfs && $after_loop_status -eq 0 ]]; then
  set +e
  host_check_args=(host-check \
    --before-mount "$work/host-before-mount.txt" --after-mount "$work/host-after-mount.txt" \
    --before-loops "$work/host-before-loops.json" --after-loops "$work/host-after-loops.json" \
    --image "$work/filesystem.xfs" --mountpoint "$work/fs" \
    --mount-ns-before "$work/host-before-mnt-ns" --mount-ns-after "$work/host-after-mnt-ns")
  [[ -n $proof_loop_device ]] && host_check_args+=(--proof-loop-device "$proof_loop_device")
  /usr/bin/node "$receipt_helper" "${host_check_args[@]}" \
    >"$work/host-check.stdout" 2>"$work/host-check.stderr"
  host_check_status=$?
  set -e
else
  host_check_status=1
fi
if [[ ${host_check_status:-1} -eq 0 ]]; then
  cp -- "$work/host-check.stdout" "$host_monitor"
else
  jq -n --arg before "$work/host-before-mount.txt" --arg after "$work/host-after-mount.txt" \
    --arg before_loops "$work/host-before-loops.json" --arg after_loops "$work/host-after-loops.json" \
    --argjson supervisor_status "$supervisor_status" --argjson host_check_status "${host_check_status:-1}" \
    '{clean:false,before_mount:$before,after_mount:$after,before_loops:$before_loops,after_loops:$after_loops,
      supervisor_status:$supervisor_status,host_check_status:$host_check_status}' >"$host_monitor"
fi

write_fallback_facts "$source" "$work" "$supervisor_status"
receipt=$work/wrapper-receipt.json
set +e
/usr/bin/node "$receipt_helper" receipt --facts "$facts" --host-monitor "$host_monitor" --output "$receipt" \
  >"$work/receipt.stdout" 2>"$work/receipt.stderr"
receipt_status=$?
set -e
if [[ $receipt_status -eq 0 && $supervisor_status -eq 0 ]]; then
  printf 'R2 XFS wrapper: PASS\n'
  exit 0
fi
printf 'R2 XFS wrapper: RED (receipt=%s supervisor=%s)\n' "$receipt_status" "$supervisor_status" >&2
exit 1
