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
SYSTEM_PATH=/usr/sbin:/usr/bin:/sbin:/bin

# These are the only entries the unprivileged front half may create before
# handing the work directory to the root supervisor.  The supervisor validates
# this exact inventory through an opened directory descriptor before changing
# ownership or writing anything itself.  Keeping the list in one place also
# makes a later proof review able to distinguish the caller handoff from the
# root-created evidence set.
PRECREATED_WORK_FILES=(
  host-before-mount.txt host-before-mount.stderr host-before-mount.status
  host-before-loops.json host-before-loops.stderr host-before-loops.status
  host-before-mnt-ns host-before-net-ns host-before-pid-ns
  host-monitor-filesystem-before.statfs host-monitor-filesystem-before.statfs.stderr
  host-monitor-filesystem-before.statfs.status host-monitor-filesystem-before.findmnt
  host-monitor-filesystem-before.findmnt.stderr host-monitor-filesystem-before.findmnt.status
  host-monitor-filesystem-before.quota host-monitor-filesystem-before.quota.stderr
  host-monitor-filesystem-before.quota.status
  host-after-mount.txt host-after-mount.stderr host-after-mount.status
  host-after-loops.json host-after-loops.stderr host-after-loops.status
  host-after-mnt-ns host-after-net-ns host-after-pid-ns
  host-monitor-filesystem-after.statfs host-monitor-filesystem-after.statfs.stderr
  host-monitor-filesystem-after.statfs.status host-monitor-filesystem-after.findmnt
  host-monitor-filesystem-after.findmnt.stderr host-monitor-filesystem-after.findmnt.status
  host-monitor-filesystem-after.quota host-monitor-filesystem-after.quota.stderr
  host-monitor-filesystem-after.quota.status
  proof.stdout proof.stderr supervisor.stdout supervisor.stderr
)

fail() {
  printf 'R2 XFS wrapper: FAIL: %s\n' "$*" >&2
  exit 1
}

pinned_supervisor=false
if [[ ${1:-} == --pinned-supervise ]]; then
  pinned_supervisor=true
  [[ $# -ge 3 && $2 == /* && $3 =~ ^/proc/[0-9]+/fd/[0-9]+$ ]] ||
    fail 'pinned supervisor handoff is malformed'
  [[ $(/usr/bin/id -u) == 0 ]] || fail 'private supervisor is root-only'
  script_source=${BASH_SOURCE[0]}
  [[ $script_source =~ ^/proc/[0-9]+/fd/[0-9]+$ && -r $script_source ]] ||
    fail 'private supervisor did not execute a pinned wrapper descriptor'
  script_directory=$2
  workdir_helper=$3
  receipt_helper=$script_directory/r2-complete-proof-xfs-receipt.mjs
  shift 3
else
  script_source=${BASH_SOURCE[0]}
  script_directory=$(builtin cd -- "$(/usr/bin/dirname -- "$script_source")" && /usr/bin/pwd -P) ||
    fail 'wrapper directory is not canonical'
  self_path=$(/usr/bin/realpath -e -- "$script_source") || fail 'wrapper path is not canonical'
  expected_self=$script_directory/r2-complete-proof-xfs.sh
  [[ $self_path == "$expected_self" && -f $self_path && ! -L $self_path ]] ||
    fail 'wrapper path does not identify this exact regular file'
  receipt_helper=$script_directory/r2-complete-proof-xfs-receipt.mjs
  [[ -f $receipt_helper && ! -L $receipt_helper ]] ||
    fail 'wrapper receipt helper is missing or symlinked'
  workdir_helper=$script_directory/r2-complete-proof-xfs-workdir.sh
  [[ -f $workdir_helper && ! -L $workdir_helper ]] ||
    fail 'wrapper work-directory helper is missing or symlinked'
  exec {public_helper_fd}<"$script_directory" || fail 'wrapper directory could not be pinned'
  public_helper_fd_path=/proc/self/fd/$public_helper_fd
  [[ $(/usr/bin/readlink -e -- "$public_helper_fd_path") == "$script_directory" ]] ||
    fail 'wrapper directory changed while pinning'
  public_helper_identity=$(/usr/bin/stat -Lc '%d:%i' -- "$public_helper_fd_path") ||
    fail 'wrapper directory identity is unavailable'
  [[ $public_helper_identity =~ ^[0-9]+:[0-9]+$ ]] || fail 'wrapper directory identity is malformed'
  exec {public_self_fd}<"$self_path" || fail 'wrapper file could not be pinned'
  exec {public_workdir_helper_fd}<"$workdir_helper" || fail 'work-directory helper file could not be pinned'
  public_self_fd_path=/proc/self/fd/$public_self_fd
  public_workdir_helper_fd_path=/proc/self/fd/$public_workdir_helper_fd
  [[ $(/usr/bin/stat -Lc '%d:%i' -- "$public_self_fd_path") == $(/usr/bin/stat -Lc '%d:%i' -- "$self_path") &&
     $(/usr/bin/stat -Lc '%d:%i' -- "$public_workdir_helper_fd_path") == $(/usr/bin/stat -Lc '%d:%i' -- "$workdir_helper") ]] ||
    fail 'pinned supervisor source identity differs'
fi

run_capture() {
  local stdout_path=$1
  local stderr_path=$2
  shift 2
  local started ended identity_before=''
  # The sidecar's before identity is sampled before either output stream is
  # opened or the child is exec'd.  The supervisor owns the pinned directory
  # at this point, so a failed sample is a fail-closed handoff rather than a
  # ledger row with an invented pre-execution identity.
  if [[ ${work_pinned:-false} == true && -n ${wrapper_command_ledger:-} ]]; then
    r2_work_path_identity_ok || fail 'work directory changed before captured command'
    identity_before=$(r2_observe_work_identity) || fail 'work identity could not be observed before captured command'
    [[ $identity_before == "$work_identity" ]] || fail 'work identity differs before captured command'
  fi
  : >"$stdout_path"
  : >"$stderr_path"
  started=$(/usr/bin/date +%s%N)
  set +e
  "$@" >"$stdout_path" 2>"$stderr_path"
  RUN_STATUS=$?
  set -e
  ended=$(/usr/bin/date +%s%N)
  if [[ -n ${wrapper_command_ledger:-} ]]; then
    local command_id=${stdout_path##*/}
    command_id=${command_id%.stdout}
    local command_uid command_gid
    command_uid=$(/usr/bin/id -u)
    command_gid=$(/usr/bin/id -g)
    local -a recorded_argv=("$@")
    if [[ ${recorded_argv[0]:-} == run_as_user ]]; then
      command_uid=$caller_uid
      command_gid=$caller_gid
      recorded_argv=("${recorded_argv[@]:1}")
    fi
    record_wrapper_command "$command_id" "$started" "$ended" "$RUN_STATUS" \
      "$stdout_path" "$stderr_path" "$command_uid" "$command_gid" "$work" "$identity_before" \
      "${recorded_argv[@]}"
  fi
}

# shellcheck source=docs/evaluation/r2-complete-proof-xfs-workdir.sh
if [[ $pinned_supervisor == true ]]; then
  source "$workdir_helper"
else
  source "$public_workdir_helper_fd_path"
fi

write_fallback_facts() {
  local source=$1
  local runtime_work=$2
  local supervisor_status=$3
  local facts=$runtime_work/supervisor-facts.json
  local display_work display_image display_fs display_proof_stdout display_proof_stderr
  local display_mount_before display_mount_after display_loops_before display_loops_after
  display_work=$(r2_display_public_work_path "$runtime_work") || return 1
  display_image=$display_work/filesystem.xfs
  display_fs=$display_work/fs
  display_proof_stdout=$display_work/proof.stdout
  display_proof_stderr=$display_work/proof.stderr
  display_mount_before=$display_work/host-before-mount.txt
  display_mount_after=$display_work/host-after-mount.txt
  display_loops_before=$display_work/host-before-loops.json
  display_loops_after=$display_work/host-after-loops.json
  [[ -e $facts || -L $facts ]] && return 0
  jq -n \
    --arg source "$source" --arg work "$display_work" --arg head "${SOURCE_HEAD:-}" \
    --arg tree "${SOURCE_TREE:-}" --argjson supervisor_status "$supervisor_status" \
    --arg image "$display_image" --arg fs "$display_fs" \
    --arg proof_stdout "$display_proof_stdout" --arg proof_stderr "$display_proof_stderr" \
    --arg mount_before "$display_mount_before" \
    --arg mount_after "$display_mount_after" \
    --arg loops_before "$display_loops_before" \
    --arg loops_after "$display_loops_after" \
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
  export PATH=$SYSTEM_PATH
  [[ $(/usr/bin/id -u) == 0 ]] || fail 'private supervisor is root-only'
  [[ $# -eq 11 ]] || fail 'private supervisor argument shape is invalid'
  # Bash runs EXIT after unwinding function locals. These values intentionally
  # live for the private supervisor process so its EXIT trap can always detach
  # the exact loop and emit truthful failure facts after any intermediate exit.
  source=$1 work=$2 expected_head=$3 expected_tree=$4 caller_uid=$5 caller_gid=$6
  caller_user=$7 caller_path=$8 rustup_home=$9 caller_browser=${10} work_identity=${11}
  work_real=$work
  work_fd=''
  work_fd_path=''
  work_original_mode=''
  work_pinned=false
  declare -A work_file_modes=()
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
  inner_start_ns='' inner_end_ns='' inner_identity_before=''
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
  inventory_path=$work/export/inventory.json
  outer_preflight_log=$work/outer-preflight.json
  proof_script=$checkout/docs/evaluation/r2-complete-proof.sh
  tool_register=$work/wrapper-tools.tsv tool_register_json=$work/wrapper-tools.json
  wrapper_command_ledger=$work/wrapper-commands.ndjson
  wrapper_execution_ledger=$work/wrapper-execution.ndjson
  tool_register_json_value='{}'

  caller_environment=()

  # Every repository read, including the source revalidation, runs as the
  # original caller.  This avoids changing Git's global safe.directory policy
  # merely because the supervisor is privileged.
  run_as_user() {
    setpriv --reuid="$caller_uid" --regid="$caller_gid" --init-groups \
      --inh-caps=-all --ambient-caps=-all --bounding-set=-all --no-new-privs -- \
      env -i "${caller_environment[@]}" "$@"
  }

  build_caller_environment() {
    # This is deliberately built after r2_pin_work_directory.  All work-tree
    # paths therefore resolve through the opened directory descriptor, not a
    # caller-renamable parent path or a canonical spelling.
    caller_environment=(
      PATH="$caller_path"
      HOME="$user_env/home" TMPDIR="$user_env/tmp"
      XDG_CACHE_HOME="$user_env/xdg-cache" XDG_CONFIG_HOME="$user_env/xdg-config"
      XDG_DATA_HOME="$user_env/xdg-data" CARGO_HOME="$user_env/cargo-home"
      CARGO_TARGET_TMPDIR="$user_env/tmp/cargo-target" RUSTUP_HOME="$rustup_home"
      NOMOS_R2_OUTER_PREFLIGHT_LOG="$outer_preflight_log"
      RUSTUP_NO_UPDATE_CHECK=1 CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0
      CARGO_TERM_COLOR=never LC_ALL=C LANG=C USER="$caller_user" LOGNAME="$caller_user"
      GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_GLOBAL=/dev/null
      GIT_TERMINAL_PROMPT=0 GIT_OPTIONAL_LOCKS=0
    )
    [[ -n $caller_browser ]] && caller_environment+=(CHROME_BIN="$caller_browser")
  }

  write_facts() {
    # Facts are receipt-facing and must retain the canonical handoff spelling,
    # but only after the descriptor still resolves to the original inode.  A
    # rename/replacement therefore makes facts fail closed instead of binding
    # a root operation to an attacker-selected path.
    r2_work_path_identity_ok || return 1
    local display_work display_fs display_checkout display_output display_image
    local display_image_stat display_image_filefrag display_image_fallocate_stdout display_image_fallocate_stderr
    local display_image_sync_stdout display_image_sync_stderr display_xfs_info display_setup_statfs
    local display_checkout_statfs display_close_statfs display_proof_script display_proof_stdout display_proof_stderr
    local display_command_ledger display_execution_ledger display_export_destination display_inventory_path display_export_sha
    local display_receipt_helper
    local display_host_before display_host_after display_inner_manifest display_outer_preflight
    display_work=$(r2_display_work_path "$work") || return 1
    display_fs=$(r2_display_work_path "$fs") || return 1
    display_checkout=$(r2_display_work_path "$checkout") || return 1
    display_output=$(r2_display_work_path "$output") || return 1
    display_image=$(r2_display_work_path "$image") || return 1
    display_image_stat=$(r2_display_work_path "$image_stat") || return 1
    display_image_filefrag=$(r2_display_work_path "$image_filefrag") || return 1
    display_image_fallocate_stdout=$(r2_display_work_path "$image_fallocate_stdout") || return 1
    display_image_fallocate_stderr=$(r2_display_work_path "$image_fallocate_stderr") || return 1
    display_image_sync_stdout=$(r2_display_work_path "$image_sync_stdout") || return 1
    display_image_sync_stderr=$(r2_display_work_path "$image_sync_stderr") || return 1
    display_xfs_info=$(r2_display_work_path "$xfs_info_file") || return 1
    display_setup_statfs=$(r2_display_work_path "$setup_statfs") || return 1
    display_checkout_statfs=$(r2_display_work_path "$checkout_statfs") || return 1
    display_close_statfs=$(r2_display_work_path "$close_statfs") || return 1
    display_proof_script=$(r2_display_work_path "$proof_script") || return 1
    display_proof_stdout=$(r2_display_work_path "$work/proof.stdout") || return 1
    display_proof_stderr=$(r2_display_work_path "$work/proof.stderr") || return 1
    display_command_ledger=$(r2_display_work_path "$wrapper_command_ledger") || return 1
    display_execution_ledger=$(r2_display_work_path "$wrapper_execution_ledger") || return 1
    display_export_destination=$(r2_display_work_path "$export_destination") || return 1
    display_inventory_path=$(r2_display_work_path "$inventory_path") || return 1
    display_export_sha=$(r2_display_work_path "$work/export.sha256") || return 1
    display_host_before=$(r2_display_work_path "$work/host-filesystem-before") || return 1
    display_host_after=$(r2_display_work_path "$work/host-filesystem-after") || return 1
    display_outer_preflight=$(r2_display_work_path "$outer_preflight_log") || return 1
    if [[ $receipt_helper == "$work_fd_path"* ]]; then
      display_receipt_helper=$(r2_display_work_path "$receipt_helper") || return 1
    else
      display_receipt_helper=$receipt_helper
    fi
    display_inner_manifest=''
    if [[ -n ${inner_evidence_manifest_path:-} ]]; then
      display_inner_manifest=$(r2_display_work_path "$inner_evidence_manifest_path") || return 1
    fi
    local tmp=$facts.tmp
    local inner_pass=false export_equal=false
    [[ $inner_status -eq 0 ]] && inner_pass=true
    [[ -n $source_inventory_sha256 && $source_inventory_sha256 == "$export_inventory_sha256" && $export_status -eq 0 ]] && export_equal=true
    jq -n \
      --arg source "$source" --arg head "$expected_head" --arg tree "$expected_tree" \
      --argjson source_status "${source_status:-125}" --argjson candidate_clean "$candidate_clean" \
      --arg image "$display_image" --arg image_stat "$display_image_stat" --arg image_filefrag "$display_image_filefrag" \
      --arg image_fallocate_stdout "$display_image_fallocate_stdout" --arg image_fallocate_stderr "$display_image_fallocate_stderr" \
      --arg image_sync_stdout "$display_image_sync_stdout" --arg image_sync_stderr "$display_image_sync_stderr" \
      --argjson image_status "$image_status" --argjson image_sync_status "$image_sync_status" \
      --arg image_logical "$image_logical" --arg image_allocated "$image_allocated" \
      --argjson loop_attached "$loop_attached" --arg loop_device "$loop_device" --arg major_minor "$major_minor" \
      --arg loop_size "$loop_size" --argjson loop_attach_status "$loop_attach_status" \
      --argjson mkfs_status "$mkfs_status" --arg xfs_info "$display_xfs_info" \
      --arg uuid "$uuid" --argjson mount_status "$mount_status" --argjson mounted "$mounted" \
      --arg fs "$display_fs" --arg mount_options "$mount_options" --arg mount_propagation "$mount_propagation" \
      --arg fragment_size "$fragment_size" --argjson capacity_ok "$capacity_ok" \
      --arg setup_statfs "$display_setup_statfs" --arg checkout_statfs "$display_checkout_statfs" \
      --arg close_statfs "$display_close_statfs" --arg checkout "$display_checkout" --arg output "$display_output" \
      --arg proof_script "$display_proof_script" --arg receipt_helper "$display_receipt_helper" \
      --arg work "$work_real" --arg display_work "$display_work" \
      --argjson inner_status "$inner_status" \
      --argjson inner_pass "$inner_pass" --arg inner_start_ns "$inner_start_ns" --arg inner_end_ns "$inner_end_ns" \
      --arg proof_stdout "$display_proof_stdout" --arg proof_stderr "$display_proof_stderr" \
      --arg command_ledger "$display_command_ledger" \
      --arg execution_ledger "$display_execution_ledger" \
      --arg outer_preflight "$display_outer_preflight" \
      --arg caller_uid "$caller_uid" --arg caller_gid "$caller_gid" --arg caller_user "$caller_user" \
      --argjson wrapper_tools "$tool_register_json_value" \
      --argjson export_status "$export_status" --argjson export_equal "$export_equal" \
      --arg export_destination "$display_export_destination" --arg inventory_path "$display_inventory_path" \
      --arg export_sha "$display_export_sha" \
      --arg source_inventory_sha256 "$source_inventory_sha256" --arg export_inventory_sha256 "$export_inventory_sha256" \
      --arg inner_evidence_manifest_path "$display_inner_manifest" \
      --argjson supervisor_exit "$supervisor_exit" --argjson setup_failed "$setup_failed" \
      --argjson unmounted "$unmounted" --argjson loop_detached "$loop_detached" --argjson no_holder "$no_holder" \
      --argjson mount_absent "$mount_absent" --argjson image_unattached "$image_unattached" \
      --argjson fuser_status "$fuser_status" --argjson sync_before_umount_status "$sync_before_umount_status" \
      --argjson umount_status "$umount_status" --argjson detach_status "$detach_status" \
      --arg loop_path "$loop_device" --arg host_before "$display_host_before" \
      --arg host_after "$display_host_after" \
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
        invocation:{argv:["/usr/bin/bash",$proof_script,"--output",$output],cwd:$checkout,uid:($caller_uid|tonumber),gid:($caller_gid|tonumber),user:$caller_user,status:$inner_status,inner_pass:$inner_pass,start_ns:($inner_start_ns|nz),end_ns:($inner_end_ns|nz),stdout_path:$proof_stdout,stderr_path:$proof_stderr,command_ledger_path:$command_ledger,execution_ledger_path:$execution_ledger,outer_preflight_path:$outer_preflight},
        export:{source:$output,destination:$export_destination,status:$export_status,equal:$export_equal,source_inventory_sha256:($source_inventory_sha256|nz),export_inventory_sha256:($export_inventory_sha256|nz),inventory_path:$inventory_path,inventory_digest_path:$export_sha,inner_evidence_manifest_path:($inner_evidence_manifest_path|nz)},
        teardown:{unmounted:$unmounted,loop_detached:$loop_detached,no_holder:$no_holder,mount_absent:$mount_absent,image_unattached:$image_unattached,fuser_status:$fuser_status,umount_status:$umount_status,detach_status:$detach_status,supervisor_status:$supervisor_exit,host_monitor:{clean:false,mountpoint:$fs,proof_loop_device:($loop_path|nz),new_loop_devices:[],mount_namespace:""}},
        host_monitor:{clean:false,mountpoint:$fs,proof_loop_device:($loop_path|nz),new_loop_devices:[],mount_namespace:""},
        operations:{
          fallocate:{argv:["/usr/bin/fallocate","--posix","--length","8589934592",$image],cwd:$display_work,status:$image_status,stdout_path:$image_fallocate_stdout,stderr_path:$image_fallocate_stderr},
          image_sync:{argv:["/usr/bin/sync","-f",$image],cwd:$display_work,status:$image_sync_status,stdout_path:$image_sync_stdout,stderr_path:$image_sync_stderr},
          loop_attach:{argv:["/usr/sbin/losetup","--find","--show",$image],cwd:$display_work,status:$loop_attach_status,stdout_path:($display_work+"/loop-attach.stdout"),stderr_path:($display_work+"/loop-attach.stderr")},
          mkfs_xfs:{argv:["/usr/sbin/mkfs.xfs","-f","-l","internal",($loop_path|nz)],cwd:$display_work,status:$mkfs_status,stdout_path:($display_work+"/mkfs-xfs.stdout"),stderr_path:($display_work+"/mkfs-xfs.stderr")},
          mount:{argv:["/usr/bin/mount","-t","xfs","-o","rw,nodev,nosuid",($loop_path|nz),$display_fs],cwd:$display_work,status:$mount_status,stdout_path:($display_work+"/mount.stdout"),stderr_path:($display_work+"/mount.stderr")},
          proof:{argv:["/usr/bin/bash",$proof_script,"--output",$display_output],cwd:$display_checkout,status:$inner_status,stdout_path:$proof_stdout,stderr_path:$proof_stderr},
          export:{argv:["/usr/bin/node",$receipt_helper,"copy","--source",$display_output,"--destination",$display_export_destination,"--output",$display_inventory_path],cwd:$display_work,status:$export_status,stdout_path:($display_work+"/export.stdout"),stderr_path:($display_work+"/export.stderr")},
          sync_before_umount:{argv:["/usr/bin/sync","-f",$display_fs],cwd:$display_work,status:$sync_before_umount_status,stdout_path:($display_work+"/sync-before-umount.stdout"),stderr_path:($display_work+"/sync-before-umount.stderr")},
          umount:{argv:["/usr/bin/umount",$display_fs],cwd:$display_work,status:$umount_status,stdout_path:($display_work+"/umount.stdout"),stderr_path:($display_work+"/umount.stderr")},
          loop_detach:{argv:["/usr/sbin/losetup","--detach",($loop_path|nz)],cwd:$display_work,status:$detach_status,stdout_path:($display_work+"/loop-detach.stdout"),stderr_path:($display_work+"/loop-detach.stderr")}
        },
        tool_register:$wrapper_tools}' >"$tmp"
    chmod 0644 "$tmp"
    mv -f -- "$tmp" "$facts"
    chmod 0644 "$facts"
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
      printf '%s\n' "$mount_cleanup_status" >"$work/mount-cleanup.status"
      if [[ $mount_cleanup_status -eq 1 && ! -s $work/mount-cleanup.txt &&
            ! -s $work/mount-cleanup.stderr ]]; then
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
    if ! r2_restore_work_access; then supervisor_exit=1; fi
    exit "$supervisor_exit"
  }
  trap 'exit 130' INT
  trap 'exit 143' TERM
  trap supervisor_cleanup EXIT

  # This is the only privileged operation before work pinning and does not
  # touch caller-supplied paths.  Keep every privileged lookup on a fixed
  # system path.  The caller's PATH is passed only inside run_as_user after the
  # identity/capability drop.
  /usr/bin/mount --make-rprivate /
  r2_pin_work_directory
  r2_display_work_path "$work" >/dev/null || fail 'work directory changed immediately after pinning'
  r2_display_work_path "$fs" >/dev/null || fail 'work directory changed before filesystem setup'
  r2_display_work_path "$checkout" >/dev/null || fail 'work directory changed before checkout setup'
  r2_display_work_path "$output" >/dev/null || fail 'work directory changed before proof output setup'
  r2_display_work_path "$proof_script" >/dev/null || fail 'work directory changed before proof setup'
  r2_display_work_path "$export_destination" >/dev/null || fail 'work directory changed before export setup'
  r2_display_work_path "$inventory_path" >/dev/null || fail 'work directory changed before inventory setup'
  : >"$outer_preflight_log"
  chown "$caller_uid:$caller_gid" -- "$outer_preflight_log"
  chmod 0600 -- "$outer_preflight_log"
  build_caller_environment
  cd -- "$work"
  mkdir -p -- "$user_env/home" "$user_env/tmp/cargo-target" "$user_env/xdg-cache" \
    "$user_env/xdg-config" "$user_env/xdg-data" "$user_env/cargo-home"
  chown -R "$caller_uid:$caller_gid" "$user_env"
  : >"$wrapper_command_ledger"
  : >"$wrapper_execution_ledger"
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
  r2_capture_host_filesystem "$work" "$work/host-filesystem-before"
  # The exact fallocate/sync/filefrag commands are caller-identity operations.
  # Create the empty regular file through the pinned descriptor, then pass its
  # canonical spelling only after revalidating the descriptor identity.  All
  # supervisor-owned streams and later losetup arguments remain descriptor
  # derived.
  : >"$image"
  [[ -f $image && ! -L $image ]] || fail 'backing image was not created as one regular file'
  [[ $(/usr/bin/stat -Lc '%h' -- "$image") == 1 ]] || fail 'backing image has an unexpected hardlink'
  /usr/bin/chown --no-dereference "$caller_uid:$caller_gid" -- "$image"
  /usr/bin/chmod 0600 -- "$image"
  r2_display_work_path "$image" >/dev/null || fail 'work directory changed before caller image operation'
  run_capture "$image_fallocate_stdout" "$image_fallocate_stderr" \
    run_as_user /usr/bin/fallocate --posix --length 8589934592 "$image"
  image_status=$RUN_STATUS
  [[ $image_status -eq 0 ]] || fail 'exact backing-image preallocation failed'
  r2_display_work_path "$image" >/dev/null || fail 'work directory changed before caller image sync'
  run_capture "$image_sync_stdout" "$image_sync_stderr" run_as_user /usr/bin/sync -f "$image"
  image_sync_status=$RUN_STATUS
  [[ $image_sync_status -eq 0 ]] || fail 'backing-image sync failed'
  read -r image_logical image_blocks image_block_size < <(/usr/bin/stat -c '%s %b %B' "$image")
  [[ $image_block_size == 512 ]] || fail 'st_blocks fundamental unit is not 512 bytes'
  image_allocated=$((image_blocks * 512))
  printf '%s\n' "logical_bytes=$image_logical" "st_blocks=$image_blocks" "allocated_bytes=$image_allocated" "block_size=$image_block_size" >"$image_stat"
  [[ $image_logical == "$IMAGE_BYTES" && $image_allocated -ge $IMAGE_BYTES ]] || fail 'backing image is not exact and fully allocated'
  r2_display_work_path "$image" >/dev/null || fail 'work directory changed before caller extent inspection'
  run_capture "$image_filefrag" "$work/image.filefrag.stderr" run_as_user /usr/sbin/filefrag -v "$image"
  [[ $RUN_STATUS -eq 0 ]] || fail 'filefrag extent evidence failed'
  # The postallocation snapshot follows sync, exact-size/stat-block, and
  # extent evidence, so the two records cannot be confused in the receipt.
  r2_capture_host_filesystem "$work" "$work/host-filesystem-after"

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
  mount_record=$(/usr/bin/findmnt --json -T "$fs" -o TARGET,SOURCE,FSTYPE,OPTIONS,PROPAGATION) || fail 'mounted XFS record missing'
  mount_target=$(/usr/bin/jq -er 'if (.filesystems | length) == 1 then .filesystems[0].target else empty end' <<<"$mount_record") ||
    fail 'mounted XFS target record is malformed'
  mount_source=$(/usr/bin/jq -er 'if (.filesystems | length) == 1 then .filesystems[0].source else empty end' <<<"$mount_record") ||
    fail 'mounted XFS source record is malformed'
  mount_type=$(/usr/bin/jq -er 'if (.filesystems | length) == 1 then .filesystems[0].fstype else empty end' <<<"$mount_record") ||
    fail 'mounted XFS type record is malformed'
  mount_options=$(/usr/bin/jq -er 'if (.filesystems | length) == 1 then .filesystems[0].options else empty end' <<<"$mount_record") ||
    fail 'mounted XFS options record is malformed'
  mount_propagation=$(/usr/bin/jq -er 'if (.filesystems | length) == 1 then .filesystems[0].propagation else empty end' <<<"$mount_record") ||
    fail 'mounted XFS propagation record is malformed'
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
  r2_display_work_path "$checkout" >/dev/null || fail 'work directory changed before caller clone'
  run_capture "$work/clone.stdout" "$work/clone.stderr" run_as_user /usr/bin/git \
    -c protocol.file.allow=always clone --no-local --no-hardlinks --no-checkout \
    --config core.hooksPath=/dev/null "$source" "$checkout"
  clone_status=$RUN_STATUS
  [[ $clone_status -eq 0 ]] || fail 'standalone clone failed'
  [[ -d $checkout/.git && ! -L $checkout/.git ]] || fail 'clone did not produce local Git metadata'
  r2_display_work_path "$checkout" >/dev/null || fail 'work directory changed before caller clone configuration'
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
  r2_display_work_path "$checkout" >/dev/null || fail 'work directory changed before caller archive'
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
  r2_display_work_path "$archive" >/dev/null || fail 'work directory changed before caller archive extraction'
  run_as_user /usr/bin/tar --extract --file "$archive" --directory "$checkout" \
    --no-same-owner --no-same-permissions --no-overwrite-dir
  rm -f -- "$archive"
  r2_display_work_path "$checkout" >/dev/null || fail 'work directory changed before caller checkout validation'
  [[ $(run_as_user /usr/bin/git -C "$checkout" rev-parse --verify 'HEAD^{commit}') == "$expected_head" &&
     $(run_as_user /usr/bin/git -C "$checkout" rev-parse --verify 'HEAD^{tree}') == "$expected_tree" ]] ||
    fail 'cloned checkout identity differs'
  [[ -z $(run_as_user /usr/bin/git -C "$checkout" status --porcelain=v1 --untracked-files=all) ]] ||
    fail 'archive checkout is not clean'
  run_as_user /usr/bin/git -C "$checkout" fsck --connectivity-only --no-dangling >/dev/null 2>&1 ||
    fail 'cloned checkout object graph is incomplete'
  [[ ! -e $checkout/target && ! -L $checkout/target ]] || fail 'clone already contains target'
  r2_display_work_path "$checkout" >/dev/null || fail 'work directory changed before target creation'
  r2_display_work_path "$output" >/dev/null || fail 'work directory changed before output creation'
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
  receipt_helper=$checkout/docs/evaluation/r2-complete-proof-xfs-receipt.mjs
  [[ -f $receipt_helper && ! -L $receipt_helper ]] || fail 'cloned receipt helper is missing or unsafe'

  # The unprivileged inner harness owns the network positive control and the
  # subsequent Bubblewrap network namespace. Precreate two caller-owned host
  # logs so a red before inner evidence assembly still preserves that control.
  outer_positive_stdout=$work/network-outer-positive.stdout
  outer_positive_stderr=$work/network-outer-positive.stderr
  : >"$outer_positive_stdout"
  : >"$outer_positive_stderr"
  chown "$caller_uid:$caller_gid" "$outer_positive_stdout" "$outer_positive_stderr"
  chmod 0600 "$outer_positive_stdout" "$outer_positive_stderr"
  r2_work_path_identity_ok || fail 'work directory changed before proof handoff'
  r2_display_work_path "$checkout" >/dev/null || fail 'work directory changed before proof checkout handoff'
  r2_display_work_path "$output" >/dev/null || fail 'work directory changed before proof output handoff'
  r2_display_work_path "$proof_script" >/dev/null || fail 'work directory changed before proof script handoff'
  inner_identity_before=$(r2_observe_work_identity) || fail 'work identity could not be observed before proof handoff'
  [[ $inner_identity_before == "$work_identity" ]] || fail 'work identity differs before proof handoff'

  # The inner proof owns its exact reservation, setup/shutdown du crosschecks,
  # persistent kernel-accounting sampler, and final no-write proof.  The
  # wrapper records only the fixed-capacity checkpoints around that child.
  inner_start_ns=$(date +%s%N)
  set +e
  (cd -- "$checkout" && run_as_user /usr/bin/env \
    NOMOS_R2_XFS_WRAPPER=1 NOMOS_R2_XFS_UUID="$uuid" \
    NOMOS_R2_XFS_FRAGMENT_SIZE="$fragment_size" NOMOS_R2_XFS_DEVICE="$loop_device" \
    NOMOS_R2_XFS_MAJOR_MINOR="$major_minor" \
    NOMOS_R2_OUTER_PREFLIGHT_LOG="$outer_preflight_log" \
    NOMOS_R2_OUTER_POSITIVE_STDOUT="$outer_positive_stdout" \
    NOMOS_R2_OUTER_POSITIVE_STDERR="$outer_positive_stderr" \
    /usr/bin/bash "$proof_script" --output "$output") \
    >"$work/proof.stdout" 2>"$work/proof.stderr"
  inner_status=$?
  set -e
  inner_end_ns=$(date +%s%N)
  record_wrapper_command inner-proof "$inner_start_ns" "$inner_end_ns" "$inner_status" \
    "$work/proof.stdout" "$work/proof.stderr" "$caller_uid" "$caller_gid" "$checkout" "$inner_identity_before" \
    /usr/bin/env NOMOS_R2_XFS_WRAPPER=1 NOMOS_R2_XFS_UUID="$uuid" \
    NOMOS_R2_XFS_FRAGMENT_SIZE="$fragment_size" NOMOS_R2_XFS_DEVICE="$loop_device" \
    NOMOS_R2_XFS_MAJOR_MINOR="$major_minor" \
    NOMOS_R2_OUTER_PREFLIGHT_LOG="$outer_preflight_log" \
    NOMOS_R2_OUTER_POSITIVE_STDOUT="$outer_positive_stdout" \
    NOMOS_R2_OUTER_POSITIVE_STDERR="$outer_positive_stderr" \
    /usr/bin/bash "$proof_script" --output "$output"
  record_fs_statfs "$fs" "$fragment_size" "$close_statfs" || fail 'close statfs checkpoint failed'
  [[ $(find "$fs" -mindepth 1 -maxdepth 1 -printf '%f\n') == checkout ]] ||
    fail 'checkout is not the XFS sole top-level entry after proof closure'

  export_root=$work/export
  export_parent=$export_root/target
  export_destination=$export_parent/r2-complete-proof
  mkdir -p -- "$export_parent"
  chown -R --no-dereference "$caller_uid:$caller_gid" -- "$export_root"
  # The copy helper emits its inventory inside the caller-owned export tree.
  # The pinned top-level directory remains mode 0711 throughout supervision;
  # root never reopens caller create/unlink authority while facts are pending.
  chmod 0700 -- "$export_root"
  r2_work_path_identity_ok || fail 'work directory changed before evidence export'
  r2_display_work_path "$output" >/dev/null || fail 'work directory changed before export source handoff'
  r2_display_work_path "$export_destination" >/dev/null || fail 'work directory changed before export destination handoff'
  r2_display_work_path "$inventory_path" >/dev/null || fail 'work directory changed before export inventory handoff'
  run_capture "$work/export.stdout" "$work/export.stderr" run_as_user /usr/bin/node "$receipt_helper" copy \
    --source "$output" --destination "$export_destination" --output "$inventory_path"
  export_status=$RUN_STATUS
  if [[ $export_status -eq 0 ]]; then
    source_inventory_sha256=$(jq -r '.source_inventory_sha256' "$inventory_path")
    export_inventory_sha256=$(jq -r '.export_inventory_sha256' "$inventory_path")
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
    mount_check_status=0
    mount_absent=false
  else
    mount_check_status=$?
    [[ $mount_check_status -eq 1 && ! -s $work/supervisor-mount-after.txt &&
       ! -s $work/supervisor-mount-after.stderr ]] && mount_absent=true || mount_absent=false
  fi
  printf '%s\n' "$mount_check_status" >"$work/supervisor-mount-after.status"
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

if [[ $pinned_supervisor == true ]]; then
  supervisor "$@"
fi
[[ ${1:-} != --supervise ]] || fail 'private supervisor requires the pinned descriptor handoff'

[[ $# -eq 4 && $1 == --source && $3 == --work ]] ||
  fail 'usage: r2-complete-proof-xfs.sh --source CLEAN --work EMPTY'
[[ ${BASH_SOURCE[0]} == "$0" ]] || fail 'wrapper must be executed, not sourced'

source_argument=$2
work_argument=$4
source=$(canonical_existing "$source_argument" source)
work=$(canonical_existing "$work_argument" work)
work_real=$work
[[ -d $source && ! -L $source ]] || fail 'source must be a real directory'
[[ -d $work && ! -L $work ]] || fail 'work must be a real directory'
[[ -r $work && -w $work && -x $work ]] || fail 'work directory is not caller-readable, writable, and searchable'
validate_source "$source"
assert_source_head=$SOURCE_HEAD
assert_source_tree=$SOURCE_TREE
r2_public_open_source_directory
if [[ $source == "$work" || $source == "$work/"* || $work == "$source/"* ]]; then
  fail 'source and work paths overlap'
fi
r2_validate_private_parent_targets "$source" "$work" "$script_directory" ||
  fail 'source, work, and helper directories must not be direct children of the filesystem root'
[[ -z $(/usr/bin/find "$work" -mindepth 1 -print -quit) ]] || fail 'work directory must be empty'
require_public_tools

caller_uid=$(/usr/bin/id -u)
caller_gid=$(/usr/bin/id -g)
caller_user=$(/usr/bin/id -un)
[[ $caller_uid =~ ^[0-9]+$ && $caller_gid =~ ^[0-9]+$ &&
   -n $caller_user && $caller_user != *$'\r'* && $caller_user != *$'\n'* &&
   $caller_user != *$'\t'* ]] || fail 'caller identity is malformed'
caller_rustup=$(type -P rustup) || fail 'rustup executable is missing from caller PATH'
caller_rustup=$(canonical_existing "$caller_rustup" rustup)
caller_rustup_bin=$(/usr/bin/dirname -- "$caller_rustup")
caller_rustup_bin=$(canonical_existing "$caller_rustup_bin" rustup bin)
caller_path=$caller_rustup_bin:$SYSTEM_PATH
rustup_home=${RUSTUP_HOME:-$("$caller_rustup" show home)}
rustup_home=$(canonical_existing "$rustup_home" RUSTUP_HOME)
caller_browser=${CHROME_BIN:-}
if [[ -n $caller_browser ]]; then caller_browser=$(canonical_existing "$caller_browser" CHROME_BIN); fi
[[ $self_path == "$source/docs/evaluation/r2-complete-proof-xfs.sh" &&
   $workdir_helper == "$source/docs/evaluation/r2-complete-proof-xfs-workdir.sh" &&
   $receipt_helper == "$source/docs/evaluation/r2-complete-proof-xfs-receipt.mjs" ]] ||
  fail 'wrapper and helper files must come from the source candidate'
r2_public_open_receipt_modules
work_identity=$(/usr/bin/stat -c '%d:%i' -- "$work")
[[ $work_identity =~ ^[0-9]+:[0-9]+$ ]] || fail 'work directory identity is malformed'
r2_public_open_work_directory
work=$public_work_fd_path
export PATH=$SYSTEM_PATH
host_mount_namespace=$(/usr/bin/readlink /proc/self/ns/mnt)
host_net_namespace=$(/usr/bin/readlink /proc/self/ns/net)
host_pid_namespace=$(/usr/bin/readlink /proc/self/ns/pid)
[[ $host_mount_namespace =~ ^mnt:\[[0-9]+\]$ && $host_net_namespace =~ ^net:\[[0-9]+\]$ &&
  $host_pid_namespace =~ ^pid:\[[0-9]+\]$ ]] || fail 'host namespace identities are malformed'

# Precreate host-side evidence paths only after the EMPTY precondition has
# been proved.  None is beneath the eventual XFS mount.
for file in \
  "${PRECREATED_WORK_FILES[@]}"; do
  : >"$work/$file"
  chmod 0644 -- "$work/$file"
done
r2_capture_mount_state "$work/fs" "$work/host-before-mount.txt" "$work/host-before-mount.stderr"
before_mount_status=$RUN_STATUS
printf '%s\n' "$before_mount_status" >"$work/host-before-mount.status"
[[ $before_mount_status -eq 1 && ! -s $work/host-before-mount.txt &&
   ! -s $work/host-before-mount.stderr ]] ||
  fail 'work/fs mount-state precondition could not be proved absent'
printf '%s\n' "$host_mount_namespace" >"$work/host-before-mnt-ns"
printf '%s\n' "$host_net_namespace" >"$work/host-before-net-ns"
printf '%s\n' "$host_pid_namespace" >"$work/host-before-pid-ns"
set +e
/usr/sbin/losetup --list --json >"$work/host-before-loops.json" 2>"$work/host-before-loops.stderr"
before_loop_status=$?
set -e
printf '%s\n' "$before_loop_status" >"$work/host-before-loops.status"
[[ $before_loop_status -eq 0 ]] || fail 'unprivileged host loop monitor is unavailable'
r2_capture_host_filesystem "$work" "$work/host-monitor-filesystem-before"

launcher_pid=$BASHPID
pinned_supervisor_path=/proc/$launcher_pid/fd/$public_self_fd
pinned_workdir_helper_path=/proc/$launcher_pid/fd/$public_workdir_helper_fd
[[ -r $pinned_supervisor_path && -r $pinned_workdir_helper_path ]] ||
  fail 'pinned supervisor handoff descriptors are unreadable'
set +e
# shellcheck disable=SC2024
/usr/bin/sudo -n /usr/bin/unshare --mount --propagation private --fork --kill-child=TERM \
  /usr/bin/bash "$pinned_supervisor_path" --pinned-supervise \
  "$script_directory" "$pinned_workdir_helper_path" \
  "$source" "$work_real" "$assert_source_head" "$assert_source_tree" \
  "$caller_uid" "$caller_gid" "$caller_user" "$caller_path" "$rustup_home" \
  "$caller_browser" "$work_identity" \
  >"$work/supervisor.stdout" 2>"$work/supervisor.stderr"
supervisor_status=$?
set -e

r2_public_work_identity_ok || fail 'work directory changed during supervision'
r2_capture_mount_state "$work/fs" "$work/host-after-mount.txt" "$work/host-after-mount.stderr"
after_mount_status=$RUN_STATUS
printf '%s\n' "$after_mount_status" >"$work/host-after-mount.status"
printf '%s\n' "$(/usr/bin/readlink /proc/self/ns/mnt)" >"$work/host-after-mnt-ns"
printf '%s\n' "$(/usr/bin/readlink /proc/self/ns/net)" >"$work/host-after-net-ns"
printf '%s\n' "$(/usr/bin/readlink /proc/self/ns/pid)" >"$work/host-after-pid-ns"
set +e
/usr/sbin/losetup --list --json >"$work/host-after-loops.json" 2>"$work/host-after-loops.stderr"
after_loop_status=$?
set -e
printf '%s\n' "$after_loop_status" >"$work/host-after-loops.status"
if [[ $after_loop_status -ne 0 ]]; then : >"$work/host-after-loops.json"; fi
# Preserve the supervisor's exact postallocation record; these later host
# snapshots are separate teardown-time observations and must not overwrite it.
r2_capture_host_filesystem "$work" "$work/host-monitor-filesystem-after"

host_monitor=$work/host-monitor.json
facts=$work/supervisor-facts.json
proof_loop_device=
if [[ -f $facts && ! -L $facts ]]; then
  proof_loop_device=$(jq -r '.loop_device.path // empty' "$facts" 2>/dev/null || true)
  [[ $proof_loop_device =~ ^/dev/loop[0-9]+$ ]] || proof_loop_device=
fi
if [[ -f $work/filesystem.xfs && ! -L $work/filesystem.xfs && $after_loop_status -eq 0 ]]; then
  set +e
  display_work=$(r2_display_public_work_path "$work")
  if [[ -z $display_work ]]; then
    set -e
    fail 'work directory changed before host receipt binding'
  fi
  host_check_args=(host-check \
    --before-mount "$display_work/host-before-mount.txt" --after-mount "$display_work/host-after-mount.txt" \
    --before-mount-stderr "$display_work/host-before-mount.stderr" \
    --after-mount-stderr "$display_work/host-after-mount.stderr" \
    --before-mount-status "$display_work/host-before-mount.status" \
    --after-mount-status "$display_work/host-after-mount.status" \
    --before-loops-status "$display_work/host-before-loops.status" \
    --after-loops-status "$display_work/host-after-loops.status" \
    --before-loops-stderr "$display_work/host-before-loops.stderr" \
    --after-loops-stderr "$display_work/host-after-loops.stderr" \
    --before-loops "$display_work/host-before-loops.json" --after-loops "$display_work/host-after-loops.json" \
    --image "$display_work/filesystem.xfs" --mountpoint "$display_work/fs" \
    --mount-ns-before "$display_work/host-before-mnt-ns" --mount-ns-after "$display_work/host-after-mnt-ns")
  [[ -n $proof_loop_device ]] && host_check_args+=(--proof-loop-device "$proof_loop_device")
  r2_public_pinned_exec /usr/bin/node "$receipt_helper" "${host_check_args[@]}" \
    >"$work/host-check.stdout" 2>"$work/host-check.stderr"
  host_check_status=$?
  set -e
else
  host_check_status=1
fi
if [[ ${host_check_status:-1} -eq 0 ]]; then
  cp -- "$work/host-check.stdout" "$host_monitor"
else
  display_work=$(r2_display_public_work_path "$work") || fail 'work directory changed before fallback host receipt binding'
  jq -n --arg before "$display_work/host-before-mount.txt" --arg after "$display_work/host-after-mount.txt" \
    --arg before_loops "$display_work/host-before-loops.json" --arg after_loops "$display_work/host-after-loops.json" \
    --argjson supervisor_status "$supervisor_status" --argjson host_check_status "${host_check_status:-1}" \
    '{clean:false,before_mount:$before,after_mount:$after,before_loops:$before_loops,after_loops:$after_loops,
      supervisor_status:$supervisor_status,host_check_status:$host_check_status}' >"$host_monitor"
fi

write_fallback_facts "$source" "$work" "$supervisor_status"
display_work=$(r2_display_public_work_path "$work") || fail 'work directory changed before receipt binding'
display_facts=$display_work/supervisor-facts.json
display_host_monitor=$display_work/host-monitor.json
display_receipt=$display_work/wrapper-receipt.json
set +e
r2_public_pinned_exec /usr/bin/node "$receipt_helper" receipt --facts "$display_facts" \
  --host-monitor "$display_host_monitor" --output "$display_receipt" \
  >"$work/receipt.stdout" 2>"$work/receipt.stderr"
receipt_status=$?
set -e
if [[ $receipt_status -eq 0 && $supervisor_status -eq 0 ]]; then
  r2_public_close_work_directory
  printf 'R2 XFS wrapper: PASS\n'
  exit 0
fi
printf 'R2 XFS wrapper: RED (receipt=%s supervisor=%s)\n' "$receipt_status" "$supervisor_status" >&2
r2_public_close_work_directory
exit 1
