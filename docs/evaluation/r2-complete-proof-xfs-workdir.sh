#!/usr/bin/env bash

# Source-only work-directory boundary for r2-complete-proof-xfs.sh.
#
# The public half creates a fixed handoff inventory.  The private supervisor
# opens and validates that directory and then uses the opened descriptor as
# the root of every privileged work path.  A canonical path is used only for
# identity checks and receipt-facing strings; it is never a substitute for
# the descriptor after root has taken ownership.
# shellcheck disable=SC2154 # Supervisor state is deliberately dynamic-scoped into these helpers.
# shellcheck disable=SC2034 # Tool-register paths and versions are consumed by the wrapper receipt.
# shellcheck disable=SC2016 # Confined child scripts expand only inside their own process.

canonical_existing() {
  local value=$1 label=$2 actual
  [[ $value == /* && $value != *$'\n'* && $value != *$'\r'* && $value != *$'\t'* ]] ||
    fail "$label must be one absolute safe path"
  actual=$(/usr/bin/realpath -e -- "$value") || fail "$label does not exist"
  [[ $actual == "$value" ]] || fail "$label is not canonical or traverses a symlink"
  printf '%s\n' "$actual"
}

validate_source() {
  local source=$1 git_dir common_dir
  [[ -d $source && ! -L $source ]] || fail 'source must be one real directory'
  [[ -d $source/.git && ! -L $source/.git ]] || fail 'source must have a local .git directory'
  git_dir=$(/usr/bin/git -C "$source" rev-parse --absolute-git-dir) || fail 'source is not a Git checkout'
  common_dir=$(/usr/bin/git -C "$source" rev-parse --git-common-dir) || fail 'source has no Git common directory'
  git_dir=$(/usr/bin/realpath -e -- "$git_dir") || fail 'source Git directory is absent'
  common_dir=$(/usr/bin/realpath -e -- "$source/$common_dir") || fail 'source Git common directory is absent'
  [[ $git_dir == "$source/.git" && $common_dir == "$source/.git" ]] || fail 'source is not a standalone checkout'
  [[ $(/usr/bin/git -C "$source" rev-parse --is-shallow-repository) == false ]] || fail 'source must be a full non-shallow checkout'
  /usr/bin/git -C "$source" symbolic-ref -q HEAD >/dev/null 2>&1 && fail 'source HEAD must be detached'
  [[ -z $(/usr/bin/git -C "$source" status --porcelain=v1 --untracked-files=all) ]] || fail 'source checkout is not clean'
  [[ -z ${GIT_ALTERNATE_OBJECT_DIRECTORIES:-} && ! -s $source/.git/objects/info/alternates ]] || fail 'source uses Git object alternates'
  [[ -z $(/usr/bin/find "$source/.git/objects" -type f -links +1 -print -quit) ]] || fail 'source Git objects contain hardlinks'
  /usr/bin/git -C "$source" config --get-regexp '^(extensions\.partialclone|remote\..*\.promisor)$' >/dev/null 2>&1 &&
    fail 'source is a partial or promisor checkout'
  /usr/bin/git -C "$source" fsck --connectivity-only --no-dangling >/dev/null 2>&1 || fail 'source object graph is incomplete'
  SOURCE_HEAD=$(/usr/bin/git -C "$source" rev-parse --verify 'HEAD^{commit}') || fail 'source HEAD is invalid'
  SOURCE_TREE=$(/usr/bin/git -C "$source" rev-parse --verify 'HEAD^{tree}') || fail 'source tree is invalid'
  [[ $SOURCE_HEAD =~ ^[0-9a-f]{40}$ && $SOURCE_TREE =~ ^[0-9a-f]{40}$ ]] || fail 'source identity is not a full lowercase Git ID'
}

R2_INNER_REQUIRED_TOOLS=(
  git realpath readlink find grep awk sed sort cmp cut sha256sum stat date du jq
  /usr/bin/time /usr/bin/fallocate /usr/bin/sync /usr/bin/unlink
  ar basename bash bwrap cargo cc chmod cp diff dirname env getconf
  head id install ionice ip ld ln mkdir mktemp mv node paste ps rm rustc rustup seq setpriv
  setsid sh sleep strings sudo tar taskset timeout touch tr uname unshare wc
)
R2_WRAPPER_REQUIRED_TOOLS=(
  bash sh git realpath find stat date mkdir readlink id node jq sudo unshare findmnt losetup
  mount umount blockdev mkfs.xfs xfs_info xfs_quota filefrag fuser fallocate sync setpriv
  bwrap tar ionice du blkid chown cp rm env sha256sum awk grep tr chmod mv dirname pwd cut
  rustup cargo rustc
)

require_public_tools() {
  local tool
  for tool in "${R2_INNER_REQUIRED_TOOLS[@]}" "${R2_WRAPPER_REQUIRED_TOOLS[@]}"; do
    type -P -- "$tool" >/dev/null 2>&1 || fail "required executable is missing: $tool"
  done
  [[ -f $receipt_helper && ! -L $receipt_helper ]] || fail 'wrapper receipt helper is missing or symlinked'
}

r2_is_precreated_work_name() {
  local candidate=$1 name
  for name in "${PRECREATED_WORK_FILES[@]}"; do
    [[ $candidate == "$name" ]] && return 0
  done
  return 1
}

r2_validate_precreated_work_inventory() {
  local root=$1 expected_uid=$2 expected_gid=$3
  local entry name links owner group mode
  local -a entries=()
  while IFS= read -r -d '' entry; do
    entries+=("$entry")
  done < <(/usr/bin/find -- "$root" -mindepth 1 -maxdepth 1 -print0)
  [[ ${#entries[@]} -eq ${#PRECREATED_WORK_FILES[@]} ]] ||
    fail 'work precreated inventory has unexpected entry count'
  for entry in "${entries[@]}"; do
    [[ $entry == "$root/"* ]] || fail 'work inventory escaped its directory descriptor'
    name=${entry#"$root/"}
    [[ $name != */* && $name != *$'\n'* && $name != *$'\r'* && $name != *$'\t'* ]] ||
      fail 'work precreated inventory contains an unsafe name'
    r2_is_precreated_work_name "$name" ||
      fail "work precreated inventory contains unexpected entry: $name"
    [[ -f $entry && ! -L $entry ]] ||
      fail "work precreated entry is not one regular file: $name"
    read -r links owner group mode < <(/usr/bin/stat -Lc '%h %u %g %a' -- "$entry") ||
      fail "work precreated entry cannot be stated: $name"
    [[ $links == 1 && $owner == "$expected_uid" && $group == "$expected_gid" &&
       $mode == 644 ]] ||
      fail "work precreated entry ownership, links, or mode differ: $name"
  done
  # A second explicit pass prevents a missing or swapped expected name from
  # being hidden by the same-sized unexpected inventory.
  for name in "${PRECREATED_WORK_FILES[@]}"; do
    entry=$root/$name
    [[ -f $entry && ! -L $entry ]] ||
      fail "work precreated entry is missing: $name"
    links=$(/usr/bin/stat -Lc '%h' -- "$entry") ||
      fail "work precreated entry cannot be restated: $name"
    [[ $links == 1 ]] || fail "work precreated entry is hardlinked: $name"
  done
}

r2_capture_mount_state() {
  local path=$1 stdout_path=$2 stderr_path=$3
  run_capture "$stdout_path" "$stderr_path" /usr/bin/findmnt -rn --mountpoint "$path" \
    -o TARGET,SOURCE,FSTYPE,OPTIONS,PROPAGATION
}

r2_capture_host_filesystem() {
  local root=$1 prefix=$2
  local statfs_status findmnt_status quota_status mount_target=''
  set +e
  /usr/bin/stat -f -c 'f_frsize=%S f_blocks=%b f_bfree=%f f_bavail=%a f_type=%T f_fsid=%i' \
    "$root" >"$prefix.statfs" 2>"$prefix.statfs.stderr"
  statfs_status=$?
  # JSON is the binding representation: unlike the default findmnt table it
  # cannot split a target containing spaces or escaped characters on IFS.
  /usr/bin/findmnt --json -T "$root" -o TARGET,SOURCE,FSTYPE,OPTIONS,PROPAGATION \
    >"$prefix.findmnt" 2>"$prefix.findmnt.stderr"
  findmnt_status=$?
  set -e
  printf '%s\n' "$statfs_status" >"$prefix.statfs.status"
  printf '%s\n' "$findmnt_status" >"$prefix.findmnt.status"
  if [[ $findmnt_status -eq 0 ]]; then
    mount_target=$(/usr/bin/jq -er \
      'if (.filesystems | length) == 1 then .filesystems[0].target else empty end' \
      "$prefix.findmnt") || return 1
  fi
  [[ -n $mount_target ]] || return 1
  run_capture "$prefix.quota" "$prefix.quota.stderr" \
    /usr/sbin/xfs_quota -x -c 'state -v' "$mount_target"
  quota_status=$RUN_STATUS
  printf '%s\n' "$quota_status" >"$prefix.quota.status"
  [[ $statfs_status -eq 0 && $findmnt_status -eq 0 && $quota_status -eq 0 ]] || return 1
}

r2_work_path_identity_ok() {
  local current_type current_owner current_group current_device current_inode
  local current_path
  [[ -n ${work_real:-} && -n ${work_fd_path:-} && ${work_identity:-} =~ ^[0-9]+:[0-9]+$ ]] || return 1
  current_path=$(/usr/bin/realpath -e -- "$work_fd_path") || return 1
  [[ $current_path == "$work_real" ]] || return 1
  read -r current_type current_owner current_group _ _ current_device current_inode \
    < <(/usr/bin/stat -Lc '%F %u %g %a %h %d %i' -- "$work_fd_path") || return 1
  [[ $current_type == directory && $current_owner == 0 && $current_group == 0 &&
     "$current_device:$current_inode" == "$work_identity" ]]
}

r2_observe_work_identity() {
  local current_type current_device current_inode
  read -r current_type current_device current_inode \
    < <(/usr/bin/stat -Lc '%F %d %i' -- "$work_fd_path") || return 1
  [[ $current_type == directory ]] || return 1
  printf '%s:%s\n' "$current_device" "$current_inode"
}

r2_pin_source_directory() {
  local handoff=$1 launcher source_type source_device source_inode fd_target
  [[ $handoff =~ ^/proc/([1-9][0-9]*)/fd/[1-9][0-9]*$ ]] ||
    fail 'supervisor source handoff is malformed'
  launcher=${BASH_REMATCH[1]}
  [[ $script_source =~ ^/proc/$launcher/fd/[1-9][0-9]*$ &&
     ${source_identity:-} =~ ^[0-9]+:[0-9]+$ ]] ||
    fail 'supervisor source handoff is not from the pinned launcher'
  exec {source_fd}<"$handoff" || fail 'supervisor could not open source handoff'
  source_fd_path=/proc/self/fd/$source_fd
  fd_target=$(/usr/bin/readlink -e -- "$source_fd_path") ||
    fail 'supervisor source descriptor is unreadable'
  read -r source_type source_device source_inode \
    < <(/usr/bin/stat -Lc '%F %d %i' -- "$source_fd_path") ||
    fail 'supervisor source descriptor cannot be stated'
  [[ $fd_target == "$source" && $source_type == directory &&
     "$source_device:$source_inode" == "$source_identity" ]] ||
    fail 'supervisor source directory identity differs'
}

r2_source_path_identity_ok() {
  local current_path source_type source_device source_inode
  [[ -n ${source:-} && -n ${source_fd_path:-} &&
     ${source_identity:-} =~ ^[0-9]+:[0-9]+$ ]] || return 1
  current_path=$(/usr/bin/readlink -e -- "$source_fd_path") || return 1
  read -r source_type source_device source_inode \
    < <(/usr/bin/stat -Lc '%F %d %i' -- "$source_fd_path") || return 1
  [[ $current_path == "$source" && $source_type == directory &&
     "$source_device:$source_inode" == "$source_identity" ]]
}

r2_display_work_path() {
  local path=$1 suffix
  r2_work_path_identity_ok || return 1
  if [[ $path == "$work_fd_path"* ]]; then
    suffix=${path#"$work_fd_path"}
    [[ -z $suffix || $suffix == /* ]] || return 1
    printf '%s%s\n' "$work_real" "$suffix"
  else
    printf '%s\n' "$path"
  fi
}

r2_display_work_argument() {
  local argument=$1 key value mapped suffix
  if [[ $argument == "$work_fd_path"* ]]; then
    r2_display_work_path "$argument"
    return
  fi
  if [[ -n ${source_fd_path:-} && $argument == "$source_fd_path"* ]]; then
    r2_source_path_identity_ok || return 1
    suffix=${argument#"$source_fd_path"}
    [[ -z $suffix || $suffix == /* ]] || return 1
    printf '%s%s\n' "$source" "$suffix"
    return
  fi
  # Inner-proof environment assignments carry descriptor-derived files after
  # the equals sign. Bind those values without changing unrelated arguments.
  if [[ $argument == *=* ]]; then
    key=${argument%%=*}
    value=${argument#*=}
    if [[ $value == "$work_fd_path"* ]]; then
      mapped=$(r2_display_work_path "$value") || return 1
      printf '%s=%s\n' "$key" "$mapped"
      return
    fi
    if [[ -n ${source_fd_path:-} && $value == "$source_fd_path"* ]]; then
      r2_source_path_identity_ok || return 1
      suffix=${value#"$source_fd_path"}
      [[ -z $suffix || $suffix == /* ]] || return 1
      printf '%s=%s%s\n' "$key" "$source" "$suffix"
      return
    fi
  fi
  printf '%s\n' "$argument"
}

record_wrapper_command() {
  local command_id=$1 started=$2 ended=$3 status=$4 stdout_path=$5 stderr_path=$6
  local command_uid=$7 command_gid=$8 cwd=$9 identity_before=${10}
  shift 10
  local -a argv=("$@") canonical_argv=()
  local argument mapped argv_json canonical_argv_json
  local canonical_stdout canonical_stderr canonical_cwd
  local identity_after semantic_record execution_record
  [[ $identity_before =~ ^[0-9]+:[0-9]+$ ]] ||
    fail 'captured command has no valid pre-execution work identity'
  r2_work_path_identity_ok || fail 'work directory changed before command ledger binding'
  canonical_stdout=$(r2_display_work_path "$stdout_path") || fail 'command stdout path escaped pinned work'
  canonical_stderr=$(r2_display_work_path "$stderr_path") || fail 'command stderr path escaped pinned work'
  canonical_cwd=$(r2_display_work_path "$cwd") || fail 'command cwd escaped pinned work'
  for argument in "${argv[@]}"; do
    mapped=$(r2_display_work_argument "$argument") || fail 'command argv escaped pinned work'
    canonical_argv+=("$mapped")
  done
  r2_work_path_identity_ok || fail 'work directory changed after command ledger binding'
  identity_after=$(r2_observe_work_identity) || fail 'work identity could not be observed after command execution'
  [[ $identity_before == "$work_identity" && $identity_after == "$work_identity" ]] ||
    fail 'work identity changed across command execution'
  argv_json=$(/usr/bin/jq -cn --args '$ARGS.positional' -- "${argv[@]}")
  canonical_argv_json=$(/usr/bin/jq -cn --args '$ARGS.positional' -- "${canonical_argv[@]}")
  semantic_record=$(/usr/bin/jq -cn \
    --arg id "$command_id" --arg started_ns "$started" --arg ended_ns "$ended" \
    --arg stdout_path "$canonical_stdout" --arg stderr_path "$canonical_stderr" \
    --arg uid "$command_uid" --arg gid "$command_gid" --arg cwd "$canonical_cwd" \
    --argjson status "$status" --argjson argv "$canonical_argv_json" \
    '{id:$id,started_ns:$started_ns,ended_ns:$ended_ns,status:$status,uid:($uid|tonumber),gid:($gid|tonumber),cwd:$cwd,argv:$argv,stdout_path:$stdout_path,stderr_path:$stderr_path}')
  execution_record=$(/usr/bin/jq -cn \
    --arg id "$command_id" --arg started_ns "$started" --arg ended_ns "$ended" \
    --arg uid "$command_uid" --arg gid "$command_gid" \
    --argjson status "$status" --argjson actual_argv "$argv_json" \
    --arg actual_cwd "$cwd" --arg actual_stdout_path "$stdout_path" --arg actual_stderr_path "$stderr_path" \
    --argjson bound_argv "$canonical_argv_json" --arg bound_cwd "$canonical_cwd" \
    --arg bound_stdout_path "$canonical_stdout" --arg bound_stderr_path "$canonical_stderr" \
    --arg canonical_work_path "$work_real" --arg work_identity "$work_identity" \
    --arg work_identity_before "$identity_before" --arg work_identity_after "$identity_after" \
    '{id:$id,started_ns:$started_ns,ended_ns:$ended_ns,status:$status,uid:($uid|tonumber),gid:($gid|tonumber),
      actual_argv:$actual_argv,actual_cwd:$actual_cwd,actual_stdout_path:$actual_stdout_path,actual_stderr_path:$actual_stderr_path,
      bound_argv:$bound_argv,bound_cwd:$bound_cwd,bound_stdout_path:$bound_stdout_path,bound_stderr_path:$bound_stderr_path,
      canonical_work_path:$canonical_work_path,work_identity:$work_identity,work_identity_before:$work_identity_before,work_identity_after:$work_identity_after}')
  printf '%s\n' "$semantic_record" >>"$wrapper_command_ledger"
  printf '%s\n' "$execution_record" >>"$wrapper_execution_ledger"
}

r2_restore_work_access() {
  local result=0 name path mode
  [[ ${work_pinned:-false} == true ]] || return 0
  set +e
  # The directory is still root-owned and mode 0711 while all entries are
  # restored, so the caller cannot create or unlink a sibling during this
  # sequence.  --no-dereference makes an unexpected symlink fail closed.
  /usr/bin/chmod 0711 -- "$work_fd_path" || result=1
  # The directory path is /proc/self/fd/N, whose final component is itself a
  # procfs symlink; chown must follow that descriptor link to reach the inode.
  /usr/bin/chown 0:0 -- "$work_fd_path" || result=1
  for name in "${PRECREATED_WORK_FILES[@]}"; do
    path=$work_fd_path/$name
    if [[ -f $path && ! -L $path ]]; then
      mode=${work_file_modes[$name]:-0644}
      /usr/bin/chown --no-dereference "$caller_uid:$caller_gid" -- "$path" || result=1
      /usr/bin/chmod "$mode" -- "$path" || result=1
    else
      result=1
    fi
  done
  /usr/bin/chown "$caller_uid:$caller_gid" -- "$work_fd_path" || result=1
  /usr/bin/chmod "$work_original_mode" -- "$work_fd_path" || result=1
  exec {work_fd}<&-
  work_pinned=false
  return "$result"
}

r2_pin_work_directory() {
  local fd_target work_type work_owner work_group work_mode work_links
  local work_device work_inode name path
  [[ $work_identity =~ ^[0-9]+:[0-9]+$ ]] ||
    fail 'supervisor work identity is malformed'
  exec {work_fd}<"$work_real" || fail 'supervisor could not open work directory'
  work_fd_path=/proc/self/fd/$work_fd
  fd_target=$(/usr/bin/readlink -e -- "$work_fd_path") ||
    fail 'supervisor work descriptor is unreadable'
  [[ $fd_target == "$work_real" ]] || fail 'supervisor work path changed before pinning'
  read -r work_type work_owner work_group work_mode work_links work_device work_inode \
    < <(/usr/bin/stat -Lc '%F %u %g %a %h %d %i' -- "$work_fd_path") ||
    fail 'supervisor work descriptor cannot be stated'
  [[ $work_type == directory && $work_owner == "$caller_uid" && $work_group == "$caller_gid" &&
     $work_links == 2 && $work_mode =~ ^[0-7]{3,4}$ &&
     "$work_device:$work_inode" == "$work_identity" ]] ||
    fail 'supervisor work directory identity or ownership differs'
  work_original_mode=$work_mode
  r2_validate_precreated_work_inventory "$work_fd_path" "$caller_uid" "$caller_gid"
  for name in "${PRECREATED_WORK_FILES[@]}"; do
    path=$work_fd_path/$name
    work_file_modes["$name"]=$(/usr/bin/stat -Lc '%a' -- "$path")
  done

  # Lock the inode through the descriptor.  Chmod precedes chown so a caller
  # loses write authority before the second inventory validation and before
  # any entry is changed.
  work_pinned=true
  /usr/bin/chmod 0711 -- "$work_fd_path" || fail 'supervisor could not lock work directory mode'
  /usr/bin/chown 0:0 -- "$work_fd_path" ||
    fail 'supervisor could not take work ownership'
  r2_validate_precreated_work_inventory "$work_fd_path" "$caller_uid" "$caller_gid"

  # Every supervisor operation now stays below the opened descriptor.  A
  # caller may rename the parent, but cannot redirect these paths to a
  # replacement directory.  Receipt-facing strings use r2_display_work_path
  # only after the canonical handoff identity is checked.
  work=$work_fd_path
  fs=$work/fs checkout=$work/fs/checkout output=$work/fs/checkout/target/r2-complete-proof
  image=$work/filesystem.xfs facts=$work/supervisor-facts.json
  setup_statfs=$work/statfs-mounted.json checkout_statfs=$work/statfs-checkout.json close_statfs=$work/statfs-close.json
  image_filefrag=$work/image.filefrag xfs_info_file=$work/xfs-info.txt
  archive=$work/checkout.tar
  image_stat=$work/image.stat image_fallocate_stdout=$work/image-fallocate.stdout
  image_fallocate_stderr=$work/image-fallocate.stderr image_sync_stdout=$work/image-sync.stdout
  image_sync_stderr=$work/image-sync.stderr
  user_env=$work/user-env
  export_root=$work/export export_parent=$work/export/target
  export_destination=$work/export/target/r2-complete-proof
  inventory_path=$work/export/inventory.json
  outer_preflight_log=$work/outer-preflight.json
  tool_register=$work/wrapper-tools.tsv tool_register_json=$work/wrapper-tools.json
  wrapper_command_ledger=$work/wrapper-commands.ndjson
  wrapper_execution_ledger=$work/wrapper-execution.ndjson
  proof_script=$checkout/docs/evaluation/r2-complete-proof.sh
  source_evidence_manifest_path=$output/EVIDENCE.sha256
  for name in "${PRECREATED_WORK_FILES[@]}"; do
    path=$work_fd_path/$name
    [[ -f $path && ! -L $path ]] || fail "precreated entry is unsafe: $name"
    /usr/bin/chown --no-dereference 0:0 -- "$path" ||
      fail "supervisor could not lock precreated entry: $name"
    /usr/bin/chmod 0644 -- "$path" ||
      fail "supervisor could not normalize precreated entry: $name"
  done
}

# The unprivileged front half keeps its own descriptor open across sudo.  This
# is separate from the root supervisor descriptor: after sudo exits, every
# host-side read/write still goes through this descriptor and the canonical
# spelling is accepted only when it resolves to the same directory identity.
r2_public_open_work_directory() {
  local fd_target work_type work_owner work_group work_device work_inode
  [[ ${work_identity:-} =~ ^[0-9]+:[0-9]+$ ]] || fail 'public work identity is malformed'
  exec {public_work_fd}<"$work_real" || fail 'public process could not open work directory'
  public_work_fd_path=/proc/self/fd/$public_work_fd
  fd_target=$(/usr/bin/readlink -e -- "$public_work_fd_path") ||
    fail 'public work descriptor is unreadable'
  [[ $fd_target == "$work_real" ]] || fail 'public work path changed before handoff'
  read -r work_type work_owner work_group _ _ work_device work_inode \
    < <(/usr/bin/stat -Lc '%F %u %g %a %h %d %i' -- "$public_work_fd_path") ||
    fail 'public work descriptor cannot be stated'
  [[ $work_type == directory && $work_owner == "$caller_uid" &&
     $work_group == "$caller_gid" && "$work_device:$work_inode" == "$work_identity" ]] ||
    fail 'public work descriptor identity or ownership differs'
}

r2_public_open_source_directory() {
  local fd_target source_type source_device source_inode source_head source_tree
  exec {public_source_fd}<"$source" || fail 'public process could not open source directory'
  public_source_fd_path=/proc/self/fd/$public_source_fd
  fd_target=$(/usr/bin/readlink -e -- "$public_source_fd_path") ||
    fail 'public source descriptor is unreadable'
  [[ $fd_target == "$source" ]] || fail 'public source path changed while pinning'
  read -r source_type source_device source_inode \
    < <(/usr/bin/stat -Lc '%F %d %i' -- "$public_source_fd_path") ||
    fail 'public source descriptor cannot be stated'
  [[ $source_type == directory ]] || fail 'public source descriptor is not a directory'
  public_source_identity=$source_device:$source_inode
  source_head=$(/usr/bin/git -C "$public_source_fd_path" rev-parse --verify 'HEAD^{commit}') ||
    fail 'public source descriptor has no candidate commit'
  source_tree=$(/usr/bin/git -C "$public_source_fd_path" rev-parse --verify 'HEAD^{tree}') ||
    fail 'public source descriptor has no candidate tree'
  [[ $source_head == "$assert_source_head" && $source_tree == "$assert_source_tree" ]] ||
    fail 'public source descriptor identity differs from the candidate'
}

r2_public_open_receipt_modules() {
  local name path fd identity blob actual_blob
  [[ $script_directory == "$source/docs/evaluation" &&
     ${assert_source_tree:-} =~ ^[0-9a-f]{40}$ ]] ||
    fail 'receipt modules are not rooted in the asserted source candidate'
  public_receipt_module_names=(
    r2-complete-proof-xfs-receipt.mjs
    r2-complete-proof-xfs-evidence.mjs
    r2-complete-proof-xfs-ledger.mjs
    r2-filesystem-accounting.mjs
  )
  declare -gA public_receipt_module_fds=()
  declare -gA public_receipt_module_identities=()
  declare -gA public_receipt_module_blobs=()
  for name in "${public_receipt_module_names[@]}"; do
    path=$script_directory/$name
    [[ -f $path && ! -L $path ]] || fail "receipt module is missing or unsafe: $name"
    exec {fd}<"$path" || fail "receipt module could not be pinned: $name"
    identity=$(/usr/bin/stat -Lc '%d:%i' -- "/proc/self/fd/$fd") ||
      fail "receipt module identity is unavailable: $name"
    [[ $(/usr/bin/realpath -e -- "/proc/self/fd/$fd") == "$path" && $identity =~ ^[0-9]+:[0-9]+$ ]] ||
      fail "receipt module changed while pinning: $name"
    blob=$(/usr/bin/git -C "$public_source_fd_path" rev-parse --verify \
      "$assert_source_tree:docs/evaluation/$name") ||
      fail "receipt module is absent from the asserted candidate tree: $name"
    [[ $blob =~ ^[0-9a-f]{40}$ &&
       $(/usr/bin/git -C "$public_source_fd_path" cat-file -t "$blob") == blob ]] ||
      fail "receipt module candidate object is not one Git blob: $name"
    actual_blob=$(/usr/bin/git hash-object --no-filters -- "/proc/self/fd/$fd") ||
      fail "receipt module could not be hashed: $name"
    [[ $actual_blob == "$blob" ]] || fail "receipt module differs from the asserted candidate: $name"
    public_receipt_module_fds["$name"]=$fd
    public_receipt_module_identities["$name"]=$identity
    public_receipt_module_blobs["$name"]=$blob
  done
}

r2_public_receipt_module_identities_ok() {
  local name fd path identity actual_blob
  [[ ${#public_receipt_module_names[@]} -eq 4 ]] || return 1
  for name in "${public_receipt_module_names[@]}"; do
    fd=${public_receipt_module_fds[$name]:-}
    path=$script_directory/$name
    identity=$(/usr/bin/stat -Lc '%d:%i' -- "/proc/self/fd/$fd") || return 1
    actual_blob=$(/usr/bin/git hash-object --no-filters -- "/proc/self/fd/$fd") || return 1
    [[ $(/usr/bin/realpath -e -- "/proc/self/fd/$fd") == "$path" &&
       $identity == "${public_receipt_module_identities[$name]:-}" &&
       $actual_blob == "${public_receipt_module_blobs[$name]:-}" ]] || return 1
  done
}

r2_public_work_identity_ok() {
  local current_type current_owner current_group current_device current_inode current_path
  [[ -n ${work_real:-} && -n ${public_work_fd_path:-} &&
     ${work_identity:-} =~ ^[0-9]+:[0-9]+$ ]] || return 1
  current_path=$(/usr/bin/realpath -e -- "$public_work_fd_path") || return 1
  [[ $current_path == "$work_real" ]] || return 1
  read -r current_type current_owner current_group _ _ current_device current_inode \
    < <(/usr/bin/stat -Lc '%F %u %g %a %h %d %i' -- "$public_work_fd_path") || return 1
  [[ $current_type == directory && $current_owner == "$caller_uid" &&
     $current_group == "$caller_gid" && "$current_device:$current_inode" == "$work_identity" ]]
}

r2_public_directory_identity_ok() {
  local fd_path=$1 expected_path=$2 expected_identity=$3
  local current_path current_type current_device current_inode
  [[ -n $fd_path && -n $expected_path && $expected_identity =~ ^[0-9]+:[0-9]+$ ]] || return 1
  current_path=$(/usr/bin/realpath -e -- "$fd_path") || return 1
  [[ $current_path == "$expected_path" ]] || return 1
  read -r current_type current_device current_inode \
    < <(/usr/bin/stat -Lc '%F %d %i' -- "$fd_path") || return 1
  [[ $current_type == directory && "$current_device:$current_inode" == "$expected_identity" ]]
}

r2_public_auxiliary_identities_ok() {
  r2_public_directory_identity_ok "${public_source_fd_path:-}" "$source" \
    "${public_source_identity:-}" &&
    r2_public_directory_identity_ok "${public_helper_fd_path:-}" "$script_directory" \
      "${public_helper_identity:-}" && r2_public_receipt_module_identities_ok
}

r2_validate_private_parent_targets() {
  local target parent
  [[ $# -gt 0 ]] || return 2
  for target in "$@"; do
    [[ $target == /* ]] || return 2
    parent=${target%/*}
    [[ -n $parent ]] || parent=/
    [[ $parent != / ]] || return 1
  done
}

# Run a receipt-side command with private parents for each retained canonical
# directory.  Mounting a descriptor directly on a host-backed dentry is not
# sufficient: another namespace can rename that dentry, moving the mountpoint.
# The tmpfs parents below create child-only target dentries first, after which
# the work, candidate, and helper descriptors are mounted at their canonical
# spellings.  Host renames can no longer redirect a receipt read.
r2_public_pinned_exec() {
  local status candidate_parent existing covered target_private module_snapshot_fd fd
  local sandbox_path=${SYSTEM_PATH:-/usr/sbin:/usr/bin:/sbin:/bin}
  local work_parent=${work_real%/*} source_parent=${source%/*}
  local helper_parent=${script_directory%/*}
  local -a private_parents=() next_parents=() sandbox_args=()
  local -a module_snapshot_fds=() module_guard_args=()
  [[ $# -gt 0 ]] || return 125
  r2_validate_private_parent_targets "$work_real" "$source" "$script_directory" || return 125
  r2_public_work_identity_ok && r2_public_auxiliary_identities_ok || return 125
  for candidate_parent in "$work_parent" "$source_parent" "$helper_parent"; do
    [[ -n $candidate_parent ]] || candidate_parent=/
    [[ $candidate_parent != / ]] || continue
    covered=false
    for existing in "${private_parents[@]}"; do
      if [[ $candidate_parent == "$existing" || $candidate_parent == "$existing/"* ]]; then
        covered=true
        break
      fi
    done
    [[ $covered == false ]] || continue
    next_parents=()
    for existing in "${private_parents[@]}"; do
      [[ $existing == "$candidate_parent/"* ]] || next_parents+=("$existing")
    done
    next_parents+=("$candidate_parent")
    private_parents=("${next_parents[@]}")
  done
  sandbox_args=(--die-with-parent --new-session --unshare-net --unshare-pid
    --ro-bind / / --dev /dev --proc /proc)
  for candidate_parent in "${private_parents[@]}"; do
    sandbox_args+=(--tmpfs "$candidate_parent")
  done
  target_private=false
  for candidate_parent in "${private_parents[@]}"; do
    [[ $source == "$candidate_parent/"* ]] && target_private=true
  done
  [[ $target_private == false ]] || sandbox_args+=(--dir "$source")
  sandbox_args+=(--ro-bind "$public_source_fd_path" "$source")
  if [[ $script_directory != "$source" && $script_directory != "$source/"* ]]; then
    target_private=false
    for candidate_parent in "${private_parents[@]}"; do
      [[ $script_directory == "$candidate_parent/"* ]] && target_private=true
    done
    [[ $target_private == false ]] || sandbox_args+=(--dir "$script_directory")
    sandbox_args+=(--ro-bind "$public_helper_fd_path" "$script_directory")
  fi
  sandbox_args+=(--tmpfs "$script_directory")
  for existing in "${public_receipt_module_names[@]}"; do
    module_snapshot_fd=''
    exec {module_snapshot_fd}<"/proc/self/fd/${public_receipt_module_fds[$existing]}" || {
      for fd in "${module_snapshot_fds[@]}"; do exec {fd}<&-; done
      return 125
    }
    module_snapshot_fds+=("$module_snapshot_fd")
    sandbox_args+=(--perms 0444 --ro-bind-data "$module_snapshot_fd" "$script_directory/$existing")
    module_guard_args+=("${public_receipt_module_blobs[$existing]}" "$script_directory/$existing")
  done
  target_private=false
  for candidate_parent in "${private_parents[@]}"; do
    [[ $work_real == "$candidate_parent/"* ]] && target_private=true
  done
  [[ $target_private == false ]] || sandbox_args+=(--dir "$work_real")
  sandbox_args+=(--bind "$public_work_fd_path" "$work_real")
  if /usr/bin/bwrap "${sandbox_args[@]}" -- /usr/bin/env -i \
    PATH="$sandbox_path" HOME=/nonexistent LC_ALL=C LANG=C \
    /usr/bin/bash -ceu '
      count=$1
      shift
      for ((index = 0; index < count; index += 1)); do
        expected=$1
        module=$2
        shift 2
        actual=$(/usr/bin/git hash-object --no-filters -- "$module")
        [[ $actual == "$expected" ]]
      done
      [[ ${1:-} == -- ]]
      shift
      [[ $# -gt 0 ]]
      exec "$@"
    ' _ "${#public_receipt_module_names[@]}" "${module_guard_args[@]}" -- "$@"; then
    status=0
  else
    status=$?
  fi
  for fd in "${module_snapshot_fds[@]}"; do exec {fd}<&-; done
  r2_public_work_identity_ok && r2_public_auxiliary_identities_ok || return 125
  return "$status"
}

r2_display_public_work_path() {
  local path=$1 suffix
  r2_public_work_identity_ok || return 1
  if [[ $path == "$public_work_fd_path"* ]]; then
    suffix=${path#"$public_work_fd_path"}
    [[ -z $suffix || $suffix == /* ]] || return 1
    printf '%s%s\n' "$work_real" "$suffix"
  else
    printf '%s\n' "$path"
  fi
}

r2_public_close_work_directory() {
  if [[ -n ${public_work_fd:-} ]]; then exec {public_work_fd}<&-; fi
  if [[ -n ${public_source_fd:-} ]]; then exec {public_source_fd}<&-; fi
  if [[ -n ${public_helper_fd:-} ]]; then exec {public_helper_fd}<&-; fi
  local name fd
  for name in "${public_receipt_module_names[@]:-}"; do
    fd=${public_receipt_module_fds[$name]:-}
    [[ -z $fd ]] || exec {fd}<&-
  done
  public_work_fd=''
  public_work_fd_path=''
  public_source_fd=''
  public_source_fd_path=''
  public_helper_fd=''
  public_helper_fd_path=''
  public_receipt_module_names=()
}

# Provenance helpers live with the work-directory boundary so the wrapper's
# executable half stays below the shop's decomposition threshold. They remain
# source-only and use the caller-provided fail/run_as_user functions when
# invoked by the supervisor.
record_wrapper_tools() {
  local register=$1
  local temporary=$register.tmp
  local -a tools=(
    bash sh git realpath find stat date mkdir readlink id node jq sudo unshare findmnt losetup
    mount umount blockdev mkfs.xfs xfs_info xfs_quota filefrag fuser fallocate sync setpriv
    bwrap tar ionice du blkid chown cp rm env sha256sum awk grep tr chmod mv dirname pwd cut
  )
  printf 'name\tpath\tversion_argv\tversion_status\tsha256\tversion\n' >"$temporary"
  local tool path version status digest version_text
  local -a version_argv
  for tool in "${tools[@]}"; do
    path=$(type -P -- "$tool") || fail "tool disappeared while recording: $tool"
    path=$(/usr/bin/realpath -e -- "$path") || fail "tool path is not canonical: $tool"
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
    digest=$(/usr/bin/sha256sum "$path" | /usr/bin/awk '{print $1}')
    [[ $digest =~ ^[0-9a-f]{64}$ ]] || fail "tool digest is malformed: $tool"
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$tool" "$path" "${version_argv[*]}" "$status" "$digest" "$version_text" >>"$temporary"
  done
  /usr/bin/chmod 0644 "$temporary"
  /usr/bin/mv -f -- "$temporary" "$register"
}

record_wrapper_user_tool() {
  local register=$1 tool=$2
  local temporary=$register.tmp
  local path version status digest version_text
  [[ -f $register && ! -L $register && ! -e $temporary && ! -L $temporary ]] ||
    fail 'wrapper tool register is not in a clean append state'
  /usr/bin/cp -- "$register" "$temporary"
  # Resolve and execute caller-selected development tools only after the
  # identity/capability drop. The privileged process merely hashes the
  # resulting executable path for provenance.
  # shellcheck disable=SC2016 # The caller-side shell expands its positional parameter.
  path=$(run_as_user /bin/sh -c 'command -v -- "$1"' _ "$tool") ||
    fail "caller tool disappeared while recording: $tool"
  path=$(/usr/bin/realpath -e -- "$path") || fail "caller tool path is not canonical: $tool"
  [[ -f $path && -x $path && ! -L $path ]] || fail "caller tool is not executable: $tool"
  set +e
  version=$(run_as_user "$tool" --version 2>&1)
  status=$?
  set -e
  [[ -n $version ]] || fail "caller tool reported no version: $tool"
  version_text=${version//$'\n'/\\n}
  version_text=${version_text//$'\r'/\\r}
  version_text=${version_text//$'\t'/\\t}
  digest=$(/usr/bin/sha256sum "$path" | /usr/bin/awk '{print $1}')
  [[ $digest =~ ^[0-9a-f]{64}$ ]] || fail "caller tool digest is malformed: $tool"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$tool" "$path" '--version' "$status" "$digest" "$version_text" >>"$temporary"
  /usr/bin/mv -f -- "$temporary" "$register"
}

record_wrapper_tool_json() {
  local register=$1 strict_json=$2
  # The receipt's closed tool register binds exactly the executables used by
  # its operation records. Keep the wider TSV as auditable host evidence,
  # while emitting this small JSON projection for the strict pass validator.
  /usr/bin/jq -R -s '
    split("\n") | .[1:-1] | map(split("\t"))
    | map({key: .[0], value: {path: .[1], version_argv: .[2], version_status: (.[3] | tonumber), sha256: .[4], version: .[5]}})
    | from_entries
  ' "$register" >"$strict_json.tmp"
  /usr/bin/chmod 0644 "$strict_json.tmp"
  /usr/bin/mv -f -- "$strict_json.tmp" "$strict_json"
  /usr/bin/jq -e '
    (keys | sort) == ["awk", "bash", "blkid", "blockdev", "bwrap", "cargo", "chmod", "chown", "cp", "cut", "date", "dirname", "du", "env", "fallocate", "filefrag", "find", "findmnt", "fuser", "git", "grep", "id", "ionice", "jq", "losetup", "mkdir", "mkfs.xfs", "mount", "mv", "node", "pwd", "readlink", "realpath", "rm", "rustc", "rustup", "setpriv", "sha256sum", "sh", "stat", "sudo", "sync", "tar", "tr", "umount", "unshare", "xfs_info", "xfs_quota"]
    and all(.[]; (.path | startswith("/")) and (.version_argv | type == "string" and length > 0)
      and (.version_status | type == "number" and floor == .)
      and (.sha256 | test("^[0-9a-f]{64}$")) and (.version | type == "string" and length > 0))
  ' "$strict_json" >/dev/null || fail 'strict wrapper tool register is incomplete'
}
