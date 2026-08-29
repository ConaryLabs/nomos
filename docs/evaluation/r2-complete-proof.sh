#!/usr/bin/env bash
# Inner, self-isolating R2 disposition proof. The public entry point is the
# fixed-capacity XFS wrapper; this file accepts only its private invocation.
set -euo pipefail
export LC_ALL=C
fail() {
  printf 'R2 complete proof: FAIL: %s\n' "$*" >&2
  exit 1
}
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=docs/evaluation/r2-complete-proof-lib.sh
source "$script_directory/r2-complete-proof-lib.sh"
if [[ ${BASH_SOURCE[0]} != "$0" ]]; then
  return 0
fi
[[ $# -eq 2 && $1 == --output ]] ||
  fail 'usage: r2-complete-proof.sh --output <empty-directory>'
output_argument=$2
[[ $output_argument != *$'\n'* && $output_argument != *$'\t'* ]] ||
  fail 'output path cannot contain a tab or newline'
host_tools=(
  git realpath readlink find grep awk sed sort cmp cut sha256sum stat date du jq
  /usr/bin/time /usr/bin/fallocate /usr/bin/sync /usr/bin/unlink
  ar basename bash bwrap cargo cc chmod cp diff dirname env getconf
  head id install ionice ip ld ln mkdir mktemp mv node paste ps rm rustc rustup seq setpriv
  setsid sh sleep strings sudo tar taskset timeout touch tr uname unshare wc
)
for command in "${host_tools[@]}"; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done
repo_root=$(cd -- "$script_directory/../.." && pwd -P)
[[ $(pwd -P) == "$repo_root" ]] || fail 'run the proof from the repository root'
[[ $(git rev-parse --show-toplevel) == "$repo_root" ]] ||
  fail 'script is not running from its repository checkout'
head=$(git rev-parse --verify 'HEAD^{commit}')
tree=$(git rev-parse --verify 'HEAD^{tree}')
[[ $head =~ ^[0-9a-f]{40}$ && $tree =~ ^[0-9a-f]{40}$ ]] ||
  fail 'HEAD or its tree is not a full Git object id'
validate_checkout() {
  [[ -d $repo_root/.git && ! -L $repo_root/.git ]] ||
    fail 'checkout must have a real local .git directory, not a linked worktree'
  local git_directory common_directory
  git_directory=$(cd -- "$(git rev-parse --absolute-git-dir)" && pwd -P)
  common_directory=$(cd -- "$(git rev-parse --git-common-dir)" && pwd -P)
  [[ $git_directory == "$repo_root/.git" && $common_directory == "$repo_root/.git" ]] ||
    fail 'checkout must be standalone and must not share a Git common directory'
  [[ $(git rev-parse --is-shallow-repository) == false ]] ||
    fail 'checkout must be a full, non-shallow clone'
  ! git symbolic-ref -q HEAD >/dev/null 2>&1 ||
    fail 'checkout must be detached at the candidate commit'
  [[ -z ${GIT_ALTERNATE_OBJECT_DIRECTORIES:-} ]] ||
    fail 'Git object alternates are forbidden'
  [[ ! -s $repo_root/.git/objects/info/alternates ]] ||
    fail 'Git object alternates are forbidden'
  [[ -z $(find "$repo_root/.git/objects" -type f -links +1 -print -quit) ]] ||
    fail 'Git object hardlinks are forbidden'
  ! git config --get-regexp '^(extensions\.partialclone|remote\..*\.promisor)$' \
    >/dev/null 2>&1 || fail 'partial or promisor clones are forbidden'
  git fsck --connectivity-only --no-dangling >/dev/null 2>&1 ||
    fail 'checkout object graph is incomplete'
  [[ -z $(git status --porcelain=v1 --untracked-files=all) ]] ||
    fail 'checkout is not clean'
}
validate_output() {
  [[ $output_argument != / && $output_argument != . && -n $output_argument ]] ||
    fail 'output cannot be a filesystem or checkout root'
  [[ -d $output_argument && ! -L $output_argument ]] ||
    fail 'output must already exist as a real directory'
  local lexical physical relative component cursor
  lexical=$(realpath -sm -- "$output_argument")
  physical=$(realpath -e -- "$output_argument")
  r2_output_spelling_matches_physical "$output_argument" "$lexical" "$physical" ||
    fail 'output path traverses a symlink'
  [[ $physical == "$repo_root/"* ]] || fail 'output must be physically inside the checkout'
  [[ $physical != "$repo_root" && $physical != "$repo_root/target" ]] ||
    fail 'output cannot be the checkout root or target/ root'
  [[ $physical != "$repo_root/.git" && $physical != "$repo_root/.git/"* ]] ||
    fail 'output cannot be inside Git metadata'
  [[ $(stat -c %d "$physical") == "$(stat -c %d "$repo_root")" ]] ||
    fail 'output and checkout must share one filesystem'
  [[ ! -e $repo_root/target || ( -d $repo_root/target && ! -L $repo_root/target ) ]] ||
    fail 'checkout target must be absent or one real directory'
  [[ -z $(find "$physical" -mindepth 1 -print -quit) ]] ||
    fail 'output directory must be empty'
  relative=${physical#"$repo_root/"}
  git check-ignore -q --no-index -- "$relative" ||
    fail 'output directory must be Git-ignored'
  cursor=$repo_root
  IFS=/ read -r -a components <<<"$relative"
  for component in "${components[@]}"; do
    cursor=$cursor/$component
    [[ ! -L $cursor ]] || fail 'output path contains a symlink component'
  done
  for path in \
    target/debug \
    target/release \
    target/executable-gaol \
    target/wasm32-unknown-unknown \
    target/r2-complete-release; do
    [[ ! -e $repo_root/$path && ! -L $repo_root/$path ]] ||
      fail "pre-existing proof target is forbidden: $path"
  done
  output_real=$physical
  output_relative=$relative
}
validate_checkout
validate_output
issue=199
issue_body_sha256=0a701b4238fd6b7f23ba0ae40022bc7c23ca450ad1a8f0febc05ab440f6b3c88
[[ ${NOMOS_R2_XFS_WRAPPER:-} == 1 ]] ||
  fail 'invoke the complete proof through r2-complete-proof-xfs.sh'
[[ ${NOMOS_R2_XFS_UUID:-} =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] ||
  fail 'the XFS wrapper UUID is missing or malformed'
[[ ${NOMOS_R2_XFS_FRAGMENT_SIZE:-} =~ ^[1-9][0-9]*$ ]] ||
  fail 'the XFS wrapper fragment size is missing or malformed'
[[ ${NOMOS_R2_XFS_DEVICE:-} =~ ^/dev/loop[0-9]+$ ]] ||
  fail 'the XFS wrapper loop device is missing or malformed'
[[ ${NOMOS_R2_XFS_MAJOR_MINOR:-} =~ ^[0-9]+:[0-9]+$ ]] ||
  fail 'the XFS wrapper major:minor device is missing or malformed'
# shellcheck source=docs/evaluation/r2-complete-proof-outer.sh
source "$script_directory/r2-complete-proof-outer.sh"

if [[ ${NOMOS_R2_PROOF_INNER:-} != 1 ]]; then
  r2_run_outer_proof
  printf 'R2 complete proof: PASS\n'
  exit 0
fi

[[ $(id -u) == "${NOMOS_R2_CALLER_UID:-missing}" &&
   $(id -g) == "${NOMOS_R2_CALLER_GID:-missing}" ]] ||
  fail 'the isolated proof did not drop back to the invoking uid and gid'
[[ $head == "${NOMOS_R2_EXPECTED_HEAD:-}" && $tree == "${NOMOS_R2_EXPECTED_TREE:-}" ]] ||
  fail 'candidate identity changed across namespace entry'
[[ $output_real == "${NOMOS_R2_OUTPUT_REAL:-}" &&
   $output_relative == "${NOMOS_R2_OUTPUT_RELATIVE:-}" ]] ||
  fail 'output identity changed across namespace entry'
[[ ${NOMOS_R2_PROOF_TOKEN:-} =~ ^[0-9a-f]{64}$ ]] ||
  fail 'proof process token is missing or malformed'
inner_netns=$(readlink /proc/self/ns/net)
[[ $inner_netns == net:\[*\] && $inner_netns != "${NOMOS_R2_HOST_NETNS:-}" ]] ||
  fail 'forged isolation marker or unchanged network namespace'
inner_pidns=$(readlink /proc/self/ns/pid)
[[ $inner_pidns == pid:\[*\] && $inner_pidns != "${NOMOS_R2_HOST_PIDNS:-}" ]] ||
  fail 'forged isolation marker or unchanged PID namespace'
r2_read_allowed_cpu_list /proc/self/status || fail 'could not read proof CPU affinity'
initial_cpu_affinity=$R2_ALLOWED_CPU_LIST
r2_partition_cpu_topology "$initial_cpu_affinity" /sys/devices/system/cpu ||
  fail 'the proof requires two readable, physically disjoint CPU-core groups'
sampler_controller_affinity=$R2_SAMPLER_CPUS
workload_cpu_affinity=$R2_WORKLOAD_CPUS
cpu_topology_groups=$R2_CPU_TOPOLOGY_GROUPS
sampler_physical_groups=$R2_SAMPLER_PHYSICAL_GROUPS
workload_physical_groups=$R2_WORKLOAD_PHYSICAL_GROUPS
evidence_dir=$output_real
r2_prepare_inner_evidence \
  "$repo_root" "$evidence_dir" "$NOMOS_R2_PROOF_TOKEN" "$head" "$tree" ||
  fail 'inner evidence directories or outer XFS shell-validation evidence differ'
cp /proc/self/mountinfo "$evidence_dir/metadata/mountinfo.txt"
readonly_stdout=$evidence_dir/metadata/read-only-negative-control.stdout
readonly_stderr=$evidence_dir/metadata/read-only-negative-control.stderr
set +e
( : >>"$repo_root/README.md" ) >"$readonly_stdout" 2>"$readonly_stderr"
readonly_exit=$?
set -e
[[ $readonly_exit -ne 0 && ! -s $readonly_stdout && -s $readonly_stderr ]] ||
  fail 'bubblewrap did not enforce the read-only repository mount'
jq -n \
  --arg output "$output_relative" \
  --argjson exit_code "$readonly_exit" \
  --rawfile stdout "$readonly_stdout" \
  --rawfile stderr "$readonly_stderr" \
  '{outcome:"pass",mechanism:"bubblewrap",repository_mount:"read-only",
    writable_roots:[$output,"target"],negative_control:{path:"README.md",
    operation:"append",exit_code:$exit_code,stdout:$stdout,stderr:$stderr}}' \
  >"$evidence_dir/metadata/filesystem-isolation.json"

ip -j address show >"$evidence_dir/metadata/ip-address.json"
ip -j -4 route show table all >"$evidence_dir/metadata/ip-route-v4.json"
ip -j -6 route show table all >"$evidence_dir/metadata/ip-route-v6.json"
jq -e '
  length == 1 and .[0].ifname == "lo" and
  (. [0].flags | index("UP")) != null and
  ([.[0].addr_info[].family] | sort) == ["inet", "inet6"]
' "$evidence_dir/metadata/ip-address.json" >/dev/null ||
  fail 'network namespace does not contain one enabled loopback interface'
jq -e 'all(.[]; .dev == "lo" and (.dst | startswith("127.")))' \
  "$evidence_dir/metadata/ip-route-v4.json" >/dev/null ||
  fail 'IPv4 route leakage exists in the proof namespace'
jq -e 'all(.[]; .dev == "lo" and .dst == "::1")' \
  "$evidence_dir/metadata/ip-route-v6.json" >/dev/null ||
  fail 'IPv6 route leakage exists in the proof namespace'

[[ ${NOMOS_R2_EXTERNAL_POSITIVE:-} == connected ]] ||
  fail 'external-connect positive control marker is missing'
network_destination=1.1.1.1:53
outer_positive_stdout=$evidence_dir/metadata/network-outer-positive.stdout
outer_positive_stderr=$evidence_dir/metadata/network-outer-positive.stderr
inner_negative_stdout=$evidence_dir/metadata/network-inner-negative.stdout
inner_negative_stderr=$evidence_dir/metadata/network-inner-negative.stderr
cp "$repo_root/target/.nomos-r2-network-$NOMOS_R2_PROOF_TOKEN.stdout" \
  "$outer_positive_stdout"
cp "$repo_root/target/.nomos-r2-network-$NOMOS_R2_PROOF_TOKEN.stderr" \
  "$outer_positive_stderr"
set +e
r2_network_probe 1.1.1.1 53 >"$inner_negative_stdout" 2>"$inner_negative_stderr"
inner_negative_exit=$?
set -e
[[ $(<"$outer_positive_stdout") == connected && ! -s $outer_positive_stderr &&
   $inner_negative_exit -ne 0 && ! -s $inner_negative_stdout &&
   -s $inner_negative_stderr ]] ||
  fail 'external-connect negative control reached its destination'
external_negative_control=blocked
jq -n \
  --arg destination "$network_destination" \
  --argjson inner_exit "$inner_negative_exit" \
  --rawfile outer_stdout "$outer_positive_stdout" \
  --rawfile outer_stderr "$outer_positive_stderr" \
  --rawfile inner_stdout "$inner_negative_stdout" \
  --rawfile inner_stderr "$inner_negative_stderr" \
  '{outcome:"pass",destination:$destination,
    outer_positive:{outcome:"connected",exit_code:0,stdout:$outer_stdout,stderr:$outer_stderr},
    inner_negative:{outcome:"blocked",exit_code:$inner_exit,
    stdout:$inner_stdout,stderr:$inner_stderr}}' \
  >"$evidence_dir/metadata/network-control.json"

jq -n \
  --arg external "$external_negative_control" \
  '{outcome:"pass",namespace:"fresh",pid_namespace:"fresh",
    external_negative_control:$external,loopback_only:true}' \
  >"$evidence_dir/metadata/isolation.json"

porcelain_start=$(git status --porcelain=v1 --untracked-files=all)
[[ -z $porcelain_start ]] || fail 'checkout became dirty before step 1'
jq -n --arg commit "$head" --arg tree "$tree" --arg porcelain "$porcelain_start" \
  '{outcome:"pass",commit:$commit,tree:$tree,porcelain:$porcelain}' \
  >"$evidence_dir/metadata/clean-start.json"

sha() {
  sha256sum "$1" | awk '{print $1}'
}

r2_contract_sha=$(sha R2.md)
r2_revision_3_authority_sha=$(sha docs/decisions/0025-r2-filesystem-accounting.md)
runtime_contract_sha=$(sha RUNTIME.md)
catalog_sha=$(sha apps/nomos-observed-viewer/src/catalog.mjs)
packet_sha=$(sha docs/evaluation/r2-second-scene-packet/MANIFEST.sha256)
committed_sheet=docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png
contact_sheet_sha=$(sha "$committed_sheet")
plan_one_sha=$(sha fixtures/r2/plans/scene_one.json)
plan_two_sha=$(sha fixtures/r2/plans/scene_two.json)
signature_one_sha=ef11771f3f8c210fdd8c9366e780ab720349a49dad88ae8dca969fcbe16c30d2
signature_two_sha=9afb46dc4d7ddb5b79cdcabd63b67d162b3c230aecfde9383e038128572f0f3d

[[ $r2_contract_sha == 625f4bb1ea7c7400a6717c14b51cc6da51b32421e49bba98cf3d7ed9ff4a1254 ]] ||
  fail 'R2.md digest moved'
[[ $r2_revision_3_authority_sha == a6a50bca56c4a990b44968ffefc31103a88e48b52904728693a166ba0d66d3ae ]] ||
  fail 'R2 revision-3 authority digest moved'
[[ $runtime_contract_sha == dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593 ]] ||
  fail 'RUNTIME.md digest moved'
[[ $catalog_sha == 6259520fbf318ae0393ea4ae69649864acb154db4034d081435416be2ffa9323 ]] ||
  fail 'R2 renderer catalog digest moved'
[[ $packet_sha == d5708087cf7967a420667c56a7b02ed052b7058ed8545af06e6771170003c948 ]] ||
  fail 'second-scene packet digest moved'
[[ $contact_sheet_sha == b76edbd9dd03fce5a99c074200ee7311bf87d5d2e5829c800170c129d00bf576 ]] ||
  fail 'committed contact-sheet digest moved'
[[ $plan_one_sha == 717b91f3f35d815bfa9f9cc777b38f8a091f7a6339d786c57360e94ffe4c7699 ]] ||
  fail 'scene-one plan digest moved'
[[ $plan_two_sha == 1fd08cfb33d07f93a568e4bb337ebfbe8909a22a973d0f137139c92f0481e905 ]] ||
  fail 'scene-two plan digest moved'

jq -n \
  --arg outcome pass \
  --arg commit "$head" \
  --arg tree "$tree" \
  --argjson issue "$issue" \
  --arg issue_body_sha256 "$issue_body_sha256" \
  --arg r2_contract_sha256 "$r2_contract_sha" \
  --arg r2_revision_3_authority_sha256 "$r2_revision_3_authority_sha" \
  --arg runtime_contract_sha256 "$runtime_contract_sha" \
  --arg catalog_sha256 "$catalog_sha" \
  --arg packet_manifest_sha256 "$packet_sha" \
  --arg committed_contact_sheet_sha256 "$contact_sheet_sha" \
  --arg plan_one "$plan_one_sha" \
  --arg plan_two "$plan_two_sha" \
  --arg signature_one "$signature_one_sha" \
  --arg signature_two "$signature_two_sha" \
  '{outcome:$outcome,commit:$commit,tree:$tree,issue:$issue,
    issue_body_sha256:$issue_body_sha256,r2_contract_sha256:$r2_contract_sha256,
    r2_revision_3_authority_sha256:$r2_revision_3_authority_sha256,
    runtime_contract_sha256:$runtime_contract_sha256,catalog_sha256:$catalog_sha256,
    packet_manifest_sha256:$packet_manifest_sha256,
    committed_contact_sheet_sha256:$committed_contact_sheet_sha256,
    plan_sha256:{scene_one:$plan_one,scene_two:$plan_two},
    scene_signature_sha256:{scene_one:$signature_one,scene_two:$signature_two}}' \
  >"$evidence_dir/metadata/source-tree.json"
{
  printf 'commit=%s\n' "$head"
  printf 'tree=%s\n' "$tree"
  printf 'uname='; uname -a
  printf 'architecture=%s\n' "$(uname -m)"
  printf 'cpu_count=%s\n' "$(getconf _NPROCESSORS_ONLN)"
  printf 'initial_cpu_affinity=%s\nsampler_controller_affinity=%s\n' \
    "$initial_cpu_affinity" "$sampler_controller_affinity"
  printf 'workload_cpu_affinity=%s\n' "$workload_cpu_affinity"
  printf 'physical_core_groups=%s\nsampler_physical_core_groups=%s\n' \
    "$cpu_topology_groups" "$sampler_physical_groups"
  printf 'workload_physical_core_groups=%s\n' "$workload_physical_groups"
  printf 'locale=%s\n' "$LC_ALL"
  printf 'timezone=%s\n' "${TZ:-system}"
  printf 'network_namespace=%s\n' "$inner_netns"
  printf 'pid_namespace=%s\n' "$inner_pidns"
  printf 'cargo_net_offline=%s\n' "$CARGO_NET_OFFLINE"
  printf 'cargo_target_tmpdir=%s\n' "${CARGO_TARGET_TMPDIR#"$repo_root/"}"
  printf 'tmpdir=%s\n' "${TMPDIR#"$repo_root/"}"
} >"$evidence_dir/metadata/environment.txt"

record_tool() {
  local label=$1
  local path=$2
  path=$(realpath -e -- "$path")
  [[ -f $path && -x $path && ! -L $path ]] || fail "host tool is not one executable file: $label"
  printf '%s\t%s\t%s\n' "$label" "$path" "$(sha "$path")"
}
{
  printf 'tool\tpath\tsha256\n'
  for tool in "${host_tools[@]}"; do
    tool_label=${tool##*/}
    [[ $tool != /usr/bin/time ]] || tool_label=gnu-time
    record_tool "$tool_label" "$(command -v "$tool")"
  done
  record_tool cargo-toolchain "$(rustup which cargo)"
  rustc_toolchain=$(rustup which rustc)
  record_tool rustc-toolchain "$rustc_toolchain"
  host_triple=$(rustc -vV | sed -n 's/^host: //p')
  record_tool rust-lld \
    "$(dirname "$rustc_toolchain")/../lib/rustlib/$host_triple/bin/rust-lld"
  record_tool chrome "$CHROME_BIN"
} >"$evidence_dir/metadata/tools.txt"
tools_record=$evidence_dir/metadata/tools.txt
{
  r2_emit_recorded_tool_version "$tools_record" git git --version
  # shellcheck disable=SC2016 # The recorded child shell expands BASH_VERSION.
  r2_emit_recorded_tool_version "$tools_record" bash bash \
    -c 'printf '\''%s\n'\'' "$BASH_VERSION"'
  r2_emit_recorded_tool_version "$tools_record" rustc rustc-toolchain --version
  r2_emit_recorded_tool_version "$tools_record" cargo cargo-toolchain --version
  r2_emit_recorded_tool_version "$tools_record" rustup rustup --version
  r2_emit_recorded_tool_version "$tools_record" node node --version
  r2_emit_recorded_tool_version "$tools_record" jq jq --version
  r2_emit_recorded_tool_version "$tools_record" bubblewrap bwrap --version
  r2_emit_recorded_tool_version "$tools_record" cc cc --version
  r2_emit_recorded_tool_version "$tools_record" ld ld --version
  r2_emit_recorded_tool_version "$tools_record" chrome chrome --version
} >"$evidence_dir/metadata/tool-versions.txt"

# Establish and retain finalization headroom before either du crosscheck or
# the live sampler. The backing XFS already enforces the full 8 GiB ceiling.
filesystem_evidence_dir=$evidence_dir/measurements/filesystem
filesystem_evidence_helper=$script_directory/r2-filesystem-evidence.mjs
[[ -f $filesystem_evidence_helper && ! -L $filesystem_evidence_helper ]] ||
  fail 'R2 filesystem evidence helper is missing or symlinked'
reservation=$evidence_dir/host/finalization.reserve
reservation_stdout=$filesystem_evidence_dir/reservation-fallocate.stdout
reservation_stderr=$filesystem_evidence_dir/reservation-fallocate.stderr
[[ ! -e $reservation && ! -L $reservation ]] ||
  fail 'finalization reservation path is not fresh'
reservation_started_ns=$(date +%s%N)
set +e
/usr/bin/fallocate --posix --length 16777216 "$reservation" \
  >"$reservation_stdout" 2>"$reservation_stderr"
reservation_status=$?
set -e
reservation_ended_ns=$(date +%s%N)
reservation_length_bytes=0
reservation_allocated_bytes=0
if [[ -f $reservation && ! -L $reservation ]]; then
  reservation_length_bytes=$(stat -c %s "$reservation")
  reservation_blocks=$(stat -c %b "$reservation")
  [[ $reservation_length_bytes =~ ^(0|[1-9][0-9]*)$ &&
    $reservation_blocks =~ ^(0|[1-9][0-9]*)$ ]] ||
    fail 'finalization reservation stat is malformed'
  reservation_allocated_bytes=$((10#$reservation_blocks * 512))
fi
jq -n \
  --arg path "$reservation" \
  --arg cwd "$repo_root" \
  --arg started_ns "$reservation_started_ns" \
  --arg ended_ns "$reservation_ended_ns" \
  --arg length_bytes "$reservation_length_bytes" \
  --arg allocated_bytes "$reservation_allocated_bytes" \
  --arg stdout "${reservation_stdout#"$evidence_dir/"}" \
  --arg stderr "${reservation_stderr#"$evidence_dir/"}" \
  --argjson status "$reservation_status" \
  '{schema:"nomos-r2-finalization-reservation/1",outcome:(if $status == 0 then "pass" else "red" end),
    invocation:{argv:["/usr/bin/fallocate","--posix","--length","16777216",$path],
      cwd:$cwd,
      status:$status,started_ns:$started_ns,ended_ns:$ended_ns,
      stdout_path:$stdout,stderr_path:$stderr},
    file:{path:$path,length_bytes:$length_bytes,allocated_bytes:$allocated_bytes}}' \
  >"$filesystem_evidence_dir/reservation.json"
[[ $reservation_status -eq 0 && -f $reservation && ! -L $reservation &&
  $reservation_length_bytes -eq 16777216 &&
  $reservation_allocated_bytes -ge 16777216 ]] ||
  fail 'finalization reservation is absent, sparse, underallocated, or the wrong size'

filesystem_cli_args=(
  --checkout "$repo_root"
  --target "$repo_root/target"
  --output "$evidence_dir"
  --device "$NOMOS_R2_XFS_DEVICE"
  --major-minor "$NOMOS_R2_XFS_MAJOR_MINOR"
  --fragment-size "$NOMOS_R2_XFS_FRAGMENT_SIZE"
  --uuid "$NOMOS_R2_XFS_UUID"
)
setup_du_json=$(node "$filesystem_evidence_helper" du-check \
  "${filesystem_cli_args[@]}" --phase setup) ||
  fail 'setup du crosscheck or immediate statfs snapshot failed'
printf '%s\n' "$setup_du_json" >"$filesystem_evidence_dir/du-setup.json"

commands_ledger=$evidence_dir/commands.tsv; command_argv_ledger=$evidence_dir/commands.argv.ndjson
printf 'ordinal\tcommand_id\tstarted_ns\tended_ns\texit_code\tstdout_path\tstderr_path\tcommand\n' \
  >"$commands_ledger"
r2_init_command_argv_ledger "$command_argv_ledger" || fail 'command argv ledger is not fresh'
next_ordinal=1

run_step() {
  local command_id=$1
  local display=$2
  shift 2
  [[ $command_id =~ ^[a-z0-9-]+$ ]] || fail "unsafe command id: $command_id"
  [[ $display != *$'\t'* && $display != *$'\n'* && -n $display ]] ||
    fail "unsafe command display for $command_id"
  local number stdout_relative stderr_relative stdout_file stderr_file started ended status
  printf -v number '%02d' "$next_ordinal"
  stdout_relative=logs/$number-$command_id.stdout
  stderr_relative=logs/$number-$command_id.stderr
  stdout_file=$evidence_dir/$stdout_relative
  stderr_file=$evidence_dir/$stderr_relative
  started=$(date +%s%N)
  set +e
  r2_execute_step "$stdout_file" "$stderr_file" "$@"
  status=$?
  set -e
  ended=$(date +%s%N)
  r2_record_command_argv "$command_argv_ledger" "$next_ordinal" "$command_id" "$@" ||
    fail "command $number argv record is invalid"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$next_ordinal" "$command_id" "$started" "$ended" "$status" \
    "$stdout_relative" "$stderr_relative" "$display" >>"$commands_ledger"
  [[ $status -eq 0 ]] || fail "command $number $command_id exited $status"
  if [[ ${sampler_running:-0} -eq 1 ]] &&
    ! r2_sampler_identity_stable "$disk_sampler_pid" "$disk_sampler_start_ticks" \
      "$sampler_controller_affinity"; then
    fail "disk sampler exited during command $number $command_id"
  fi
  next_ordinal=$((next_ordinal + 1))
}

disk_samples=$filesystem_evidence_dir/public.tsv
disk_raw_samples=$filesystem_evidence_dir/raw.tsv
disk_identity=$filesystem_evidence_dir/identity.json
disk_sampler_stop=$filesystem_evidence_dir/stop
disk_sample_state=$evidence_dir/host/tmp/disk-sample-state
disk_sampler_ready=$disk_sample_state/ready
mkdir "$disk_sample_state"
disk_sampler_started=
disk_sample_period_ns=50000000
disk_sampler_pid=
disk_sampler_start_ticks=
sampler_running=0
stop_sampler() {
  local incoming=${1:-0} result
  if [[ $sampler_running -eq 1 ]]; then
    if r2_prepare_and_stop_sampler \
      "$disk_sampler_pid" "$disk_sampler_start_ticks" \
      "$sampler_controller_affinity" "$disk_sampler_stop" \
      "$disk_sample_state" "$incoming"; then
      result=0
    else
      result=$?
    fi
    if [[ $result -eq 0 ]]; then sampler_running=0; fi
    return "$result"
  fi
  return "$incoming"
}
cleanup_sampler() {
  local incoming=$? result attempt; trap - EXIT INT TERM
  result=$incoming; for ((attempt = 0; attempt < 2 && sampler_running == 1; attempt += 1)); do
    if stop_sampler "$result"; then result=0; else result=$?; fi
  done
  exit "$result"
}
trap cleanup_sampler EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
set +m
sampler_launch_signal=0
# Defer INT/TERM across launch/identity capture so EXIT cleanup has the child identity.
trap 'sampler_launch_signal=130' INT
trap 'sampler_launch_signal=143' TERM
# Vacate the sampler's physical core before launch. The child explicitly enters
# that mask; every remaining proof command inherits only the workload mask.
taskset -pc "$workload_cpu_affinity" "$BASHPID" >/dev/null || fail 'CPU isolation failed'
r2_read_allowed_cpu_list /proc/self/status || fail 'could not verify workload CPU affinity'
[[ $R2_EXPANDED_CPU_LIST == "$workload_cpu_affinity" ]] || fail 'workload CPU affinity differs'
setsid taskset -c "$sampler_controller_affinity" \
  node "$script_directory/r2-filesystem-sampler.mjs" sample \
  --root "$repo_root" \
  --target "$repo_root/target" \
  --output "$evidence_dir" \
  --raw "$disk_raw_samples" \
  --public "$disk_samples" \
  --identity "$disk_identity" \
  --state-dir "$disk_sample_state" \
  --stop "$disk_sampler_stop" \
  --device "$NOMOS_R2_XFS_DEVICE" \
  --major-minor "$NOMOS_R2_XFS_MAJOR_MINOR" \
  --fragment-size "$NOMOS_R2_XFS_FRAGMENT_SIZE" \
  --uuid "$NOMOS_R2_XFS_UUID" \
  --period-ns "$disk_sample_period_ns" \
  --max-gap-ns 100000000 \
  --max-rows 100000 &
disk_sampler_pid=$!
sampler_running=1
sampler_session_bound=0
for ((attempt = 0; attempt < 100; attempt += 1)); do
  if r2_read_process_stat "/proc/$disk_sampler_pid/stat"; then
    disk_sampler_start_ticks=$R2_PROC_START
  fi
  if [[ -n $disk_sampler_start_ticks &&
      $R2_PROC_GROUP == "$disk_sampler_pid" &&
      $R2_PROC_SESSION == "$disk_sampler_pid" && $R2_PROC_STATE != Z ]] &&
    r2_read_allowed_cpu_list "/proc/$disk_sampler_pid/status" &&
    [[ $R2_EXPANDED_CPU_LIST == "$sampler_controller_affinity" ]]; then
    sampler_session_bound=1
    break
  fi
  kill -0 "$disk_sampler_pid" 2>/dev/null || break
  sleep 0.001
done
trap 'exit 130' INT
trap 'exit 143' TERM
[[ $sampler_launch_signal -eq 0 ]] || exit "$sampler_launch_signal"
[[ $sampler_session_bound -eq 1 ]] || fail 'disk sampler does not own its session'
if ! r2_wait_for_sampler_ready \
  "$disk_sampler_pid" "$disk_sampler_start_ticks" \
  "$sampler_controller_affinity" "$disk_sampler_ready"; then
  fail "disk sampler readiness ${R2_SAMPLER_READY_REASON:-unknown} before its initial row"
fi
r2_read_decimal_control_marker "$disk_sampler_ready" ||
  fail 'disk sampler ready marker is malformed'
disk_sampler_started=$R2_CONTROL_MARKER

# Step 1: accepted workspace proof.
run_step workspace-fmt \
  'cargo fmt --all -- --check' \
  cargo fmt --all -- --check
run_step workspace-clippy \
  'cargo clippy --workspace --all-targets --locked --offline -- -D warnings' \
  cargo clippy --workspace --all-targets --locked --offline -- -D warnings
run_step workspace-test \
  'cargo test --workspace --locked --offline' \
  cargo test --workspace --locked --offline
run_step workspace-boundary \
  'cargo xtask boundary' \
  cargo xtask boundary

# Step 2: unchanged accepted R1 components, with the app and its adjacent dist
# in an exact git-archive mirror beneath the proof output.
run_step r1-gaol-verify \
  'experiments/executable-gaol/gaol verify' \
  experiments/executable-gaol/gaol verify
r1_wasm_build() {
  local binary=target/wasm32-unknown-unknown/wasm/nomos_play.wasm
  local first_digest second_digest
  crates/nomos-play/build-wasm.sh --offline
  first_digest=$(sha "$binary")
  cp "$binary" "$evidence_dir/r1/wasm/first-build.wasm"
  find target/wasm32-unknown-unknown -depth -delete
  crates/nomos-play/build-wasm.sh --offline
  second_digest=$(sha "$binary")
  [[ $first_digest == "$second_digest" ]] || fail 'the two R1 wasm builds differ'
  printf 'build_1_sha256 %s\nbuild_2_sha256 %s\n' "$first_digest" "$second_digest"
}
run_step r1-wasm-build \
  'build R1 wasm, remove its exact target subtree, rebuild, and compare digests' \
  r1_wasm_build
run_step r1-native-build \
  'cargo build --locked --offline -p nomos-play' \
  cargo build --locked --offline -p nomos-play

r1_mirror() {
  local mirror=$evidence_dir/r1/viewer-mirror
  mkdir -p "$mirror" "$evidence_dir/r1/wasm"
  git archive --format=tar HEAD apps/nomos-viewer | tar -xf - -C "$mirror"
  (
    git ls-tree -r --name-only HEAD apps/nomos-viewer | while IFS= read -r path; do
      printf '%s  %s\n' "$(sha "$path")" "$path"
    done
  ) >"$evidence_dir/r1/viewer-source.sha256"
  (
    cd "$mirror"
    find apps/nomos-viewer -type f -print | sort | while IFS= read -r path; do
      printf '%s  %s\n' "$(sha256sum "$path" | awk '{print $1}')" "$path"
    done
  ) >"$evidence_dir/r1/viewer-mirror.sha256"
  cmp "$evidence_dir/r1/viewer-source.sha256" "$evidence_dir/r1/viewer-mirror.sha256"
  [[ $(wc -l <"$evidence_dir/r1/viewer-source.sha256") -eq 33 ]] ||
    fail 'R1 viewer mirror does not contain exactly 33 tracked source files'
  mkdir -p "$mirror/target/debug" "$mirror/target/wasm32-unknown-unknown/wasm"
  cp target/debug/nomos-play "$mirror/target/debug/nomos-play"
  cp -a target/executable-gaol "$mirror/target/executable-gaol"
  cp target/wasm32-unknown-unknown/wasm/nomos_play.wasm \
    "$mirror/target/wasm32-unknown-unknown/wasm/nomos_play.wasm"
  cp target/wasm32-unknown-unknown/wasm/nomos_play.wasm \
    "$evidence_dir/r1/wasm/nomos_play.wasm"
}
run_step r1-viewer-mirror \
  'git archive HEAD apps/nomos-viewer and byte-verify output-local mirror' \
  r1_mirror

r1_viewer_build() {
  local mirror=$evidence_dir/r1/viewer-mirror
  (
    cd "$mirror"
    node apps/nomos-viewer/build.mjs \
      --from target/executable-gaol \
      --wasm target/wasm32-unknown-unknown/wasm/nomos_play.wasm \
      --out apps/nomos-viewer/dist \
      --receipt "$evidence_dir/r1/viewer-build.json"
  )
  mkdir -p "$evidence_dir/r1/viewer-dist"
  cp -a "$mirror/apps/nomos-viewer/dist/." "$evidence_dir/r1/viewer-dist/"
}
run_step r1-viewer-build \
  'node apps/nomos-viewer/build.mjs in byte-identical output-local mirror' \
  r1_viewer_build

r1_viewer_tests() {
  (
    cd "$evidence_dir/r1/viewer-mirror"
    node --test apps/nomos-viewer/test/*.test.mjs
  )
}
run_step r1-viewer-tests \
  'node --test apps/nomos-viewer/test/*.test.mjs (byte-identical mirror)' \
  r1_viewer_tests
grep -Eq '^# tests 104$' "$evidence_dir/logs/10-r1-viewer-tests.stdout" ||
  fail 'R1 viewer test count is not 104'
grep -Eq '^# pass 104$' "$evidence_dir/logs/10-r1-viewer-tests.stdout" ||
  fail 'R1 viewer tests did not all pass'
grep -Eq '^# fail 0$' "$evidence_dir/logs/10-r1-viewer-tests.stdout" ||
  fail 'R1 viewer tests report a failure'
grep -Eq '^# skipped 0$' "$evidence_dir/logs/10-r1-viewer-tests.stdout" ||
  fail 'R1 viewer tests skipped a required test'

run_step r1-browser-smoke \
  'NOMOS_PLAY_BIN=target/debug/nomos-play NOMOS_PLAY_AREAS=target/executable-gaol/areas node apps/nomos-viewer/smoke/smoke.mjs --dist <output>/r1/viewer-dist --out <output>/r1/viewer-smoke --require-chrome' \
  env NOMOS_PLAY_BIN="$repo_root/target/debug/nomos-play" \
    NOMOS_PLAY_AREAS="$repo_root/target/executable-gaol/areas" \
    node apps/nomos-viewer/smoke/smoke.mjs \
      --dist "$evidence_dir/r1/viewer-dist" \
      --out "$evidence_dir/r1/viewer-smoke" \
      --require-chrome

run_step r1-native-replay \
  'target/debug/nomos-play replay target/executable-gaol/areas --session <output>/r1/viewer-smoke/session.json' \
  target/debug/nomos-play replay target/executable-gaol/areas \
    --session "$evidence_dir/r1/viewer-smoke/session.json"
cp "$evidence_dir/logs/12-r1-native-replay.stdout" "$evidence_dir/r1/native-replay.stdout"

r1_facts() {
  node --input-type=module - \
    "$evidence_dir/r1/viewer-smoke/receipt.json" \
    "$evidence_dir/r1/viewer-smoke/session.json" \
    "$evidence_dir/r1/wasm/nomos_play.wasm" \
    "$evidence_dir/r1/facts.json" <<'NODE'
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [smokePath, sessionPath, wasmPath, outputPath] = process.argv.slice(2);
const smoke = JSON.parse(readFileSync(smokePath));
const session = JSON.parse(readFileSync(sessionPath));
const wasm = readFileSync(wasmPath);
const facts = {
  outcome: "pass",
  areas: smoke.result.areas_cleared,
  commands: session.log.length,
  moves: smoke.result.moves,
  cost: smoke.result.cost,
  chain_head: session.receipt_chain_head,
  unexpected_viewer_test_skips: 0,
  external_requests: smoke.external_requests?.length ?? 0,
  wasm: { bytes: wasm.length, sha256: createHash("sha256").update(wasm).digest("hex") },
};
writeFileSync(outputPath, `${JSON.stringify(facts, null, 2)}\n`, { flag: "wx" });
NODE
  jq -e '
    .outcome == "pass" and .areas == 6 and .commands == 77 and .moves == 65 and
    .cost == 95 and .external_requests == 0 and .unexpected_viewer_test_skips == 0 and
    (.chain_head | test("^[0-9a-f]{64}$")) and
    .wasm.sha256 == "e8e03c125667ad937939f4a628b67df9ff813a88823fecd859784ed241673c97"
  ' "$evidence_dir/r1/facts.json" >/dev/null
}
run_step r1-facts 'derive and assert accepted R1 facts' r1_facts

# Step 3: R2 registers, plants, maximum fixture, and compiler tests.
run_step r2-schema-ownership \
  'docs/evaluation/r2-schema-ownership.sh' \
  docs/evaluation/r2-schema-ownership.sh

run_step r2-schema-plants \
  'three isolated git-archive schema-ownership plants must fail' \
  env R2_SCHEMA_PLANTS_PARENT="$evidence_dir/host/tmp" \
  docs/evaluation/r2-schema-ownership-plants.sh
run_step r2-source-provenance \
  'docs/evaluation/r2-source-provenance.sh' \
  docs/evaluation/r2-source-provenance.sh
run_step r2-source-provenance-plants \
  'docs/evaluation/r2-source-provenance.test.sh' \
  docs/evaluation/r2-source-provenance.test.sh
run_step r2-adopter-neutrality \
  'docs/evaluation/r2-adopter-neutrality.sh' \
  docs/evaluation/r2-adopter-neutrality.sh
run_step r2-adopter-neutrality-plants \
  'docs/evaluation/r2-adopter-neutrality.test.sh' \
  docs/evaluation/r2-adopter-neutrality.test.sh
run_step r2-maximum-fixture \
  'node docs/evaluation/r2-maximum.test.mjs' \
  node docs/evaluation/r2-maximum.test.mjs
r2_compiler_tests() {
  cargo test --locked --offline -p nomos-observed-scene
  cargo build --release --locked --offline -p nomos-observed-scene
  R2_PACKET_COMPILER="$repo_root/target/release/nomos-observed-scene" \
    docs/evaluation/r2-second-scene-packet.test.sh
}
run_step r2-compiler-tests \
  'compiler tests, release compiler, and exact frozen second-scene packet plants' \
  r2_compiler_tests

# Step 4: ten independent publications of each scene, then signatures.
compile_scene_ten() {
  local scene=$1
  local destination=$2
  local expected=$3
  local ordinal output
  mkdir -p "$destination"
  for ordinal in $(seq 0 9); do
    printf -v output '%s/plan-%02d.json' "$destination" "$ordinal"
    [[ ! -e $output ]] || fail "scene reproduction output already exists: $output"
    target/release/nomos-observed-scene compile --input "$scene" --out "$output"
    cmp "$expected" "$output"
  done
}
run_step r2-scene-one-repro \
  'compile scene_one ten times to unique outputs and compare committed plan' \
  compile_scene_ten fixtures/r2/scenes/scene_one.json "$evidence_dir/r2/scene-a" \
    fixtures/r2/plans/scene_one.json
run_step r2-scene-two-repro \
  'compile scene_two ten times to unique outputs and compare committed plan' \
  compile_scene_ten fixtures/r2/scenes/scene_two.json "$evidence_dir/r2/scene-b" \
    fixtures/r2/plans/scene_two.json

r2_signatures() {
  node docs/evaluation/r2-scene-signature.mjs \
    fixtures/r2/scenes/scene_one.json fixtures/r2/scenes/scene_two.json \
    >"$evidence_dir/r2/signatures.json"
  jq -e \
    --arg one "$signature_one_sha" --arg two "$signature_two_sha" \
    '.scenes[0].sha256 == $one and .scenes[1].sha256 == $two and
     .scenes[0].sha256 != .scenes[1].sha256 and
     .scenes[0].axis_sha256.crop != .scenes[1].axis_sha256.crop and
     .scenes[0].axis_sha256.terrain != .scenes[1].axis_sha256.terrain and
     .scenes[0].axis_sha256.actors != .scenes[1].axis_sha256.actors and
     .scenes[0].axis_sha256.actions != .scenes[1].axis_sha256.actions' \
    "$evidence_dir/r2/signatures.json" >/dev/null
}
run_step r2-scene-signatures \
  'node docs/evaluation/r2-scene-signature.mjs scene_one scene_two' \
  r2_signatures

# Step 5: strict consumer tests, first clean distribution, and 20-sample smoke.
r2_viewer_tests() {
  node --test \
    apps/nomos-observed-viewer/test/*.test.mjs \
    docs/evaluation/r2-scene-signature.test.mjs \
    docs/evaluation/r2-complete-proof-process.test.mjs \
    docs/evaluation/r2-complete-proof-receipt.test.mjs \
    docs/evaluation/r2-complete-proof-xfs-evidence.test.mjs \
    docs/evaluation/r2-complete-proof-xfs-receipt.test.mjs \
    docs/evaluation/r2-filesystem-accounting.test.mjs \
    docs/evaluation/r2-filesystem-evidence.test.mjs
  docs/evaluation/r2-complete-proof.test.sh
}
run_step r2-viewer-tests \
  'node --test apps/nomos-observed-viewer/test/*.test.mjs docs/evaluation/r2-scene-signature.test.mjs docs/evaluation/r2-complete-proof-process.test.mjs docs/evaluation/r2-complete-proof-receipt.test.mjs docs/evaluation/r2-complete-proof-xfs-evidence.test.mjs docs/evaluation/r2-complete-proof-xfs-receipt.test.mjs docs/evaluation/r2-filesystem-accounting.test.mjs docs/evaluation/r2-filesystem-evidence.test.mjs; docs/evaluation/r2-complete-proof.test.sh' \
  r2_viewer_tests

run_step r2-viewer-build \
  'node apps/nomos-observed-viewer/build.mjs --plan scene_one --plan scene_two --out <output>/r2/viewer-proof/dist --receipt <output>/r2/viewer-proof/receipt.json' \
  node apps/nomos-observed-viewer/build.mjs \
    --plan fixtures/r2/plans/scene_one.json \
    --plan fixtures/r2/plans/scene_two.json \
    --out "$evidence_dir/r2/viewer-proof/dist" \
    --receipt "$evidence_dir/r2/viewer-proof/receipt.json"
run_step r2-browser-smoke \
  'node apps/nomos-observed-viewer/smoke/smoke.mjs --dist <output>/r2/viewer-proof/dist --out <output>/r2/browser-smoke --samples 10' \
  node apps/nomos-observed-viewer/smoke/smoke.mjs \
    --dist "$evidence_dir/r2/viewer-proof/dist" \
    --out "$evidence_dir/r2/browser-smoke" \
    --samples 10

# Step 6: guaranteed-new release target and clean public builds.
fresh_release_target=$repo_root/target/r2-complete-release
[[ ! -e $fresh_release_target ]] || fail 'fresh release target already exists'
run_step clean-release-build \
  'LC_ALL=C /usr/bin/time -v cargo build --workspace --release --locked --offline (fresh target)' \
  /usr/bin/time -v -o "$evidence_dir/measurements/clean-release-time.txt" \
    env CARGO_TARGET_DIR="$fresh_release_target" \
      cargo build --workspace --release --locked --offline

inventory_tree() {
  local root=$1
  local output=$2
  (
    cd "$root"
    find . -mindepth 1 -type l -print -quit | grep -q . && exit 1
    find . -type f -printf '%P\n' | sort | while IFS= read -r path; do
      printf '%s  %s\n' "$(sha256sum "$path" | awk '{print $1}')" "$path"
    done
  ) >"$output"
}

clean_r1_viewer_build() {
  node apps/nomos-viewer/build.mjs \
    --from target/executable-gaol \
    --wasm target/wasm32-unknown-unknown/wasm/nomos_play.wasm \
    --out "$evidence_dir/r1/clean-viewer/dist" \
    --receipt "$evidence_dir/r1/clean-viewer/receipt.json"
  inventory_tree "$evidence_dir/r1/viewer-dist" "$evidence_dir/r1/viewer-dist.sha256"
  inventory_tree "$evidence_dir/r1/clean-viewer/dist" "$evidence_dir/r1/clean-viewer.sha256"
  cmp "$evidence_dir/r1/viewer-dist.sha256" "$evidence_dir/r1/clean-viewer.sha256"
}
run_step clean-r1-viewer-build \
  'clean R1 viewer build and byte comparison with proof distribution' \
  clean_r1_viewer_build

build_r2_viewer() {
  local root=$1
  node apps/nomos-observed-viewer/build.mjs \
    --plan fixtures/r2/plans/scene_one.json \
    --plan fixtures/r2/plans/scene_two.json \
    --out "$root/dist" --receipt "$root/receipt.json"
}
run_step clean-r2-viewer-build-a \
  'clean R2 viewer build A' \
  build_r2_viewer "$evidence_dir/r2/viewer-a"
run_step clean-r2-viewer-build-b \
  'clean R2 viewer build B' \
  build_r2_viewer "$evidence_dir/r2/viewer-b"

compare_r2_viewers() {
  inventory_tree "$evidence_dir/r2/viewer-a/dist" "$evidence_dir/r2/viewer-a.sha256"
  inventory_tree "$evidence_dir/r2/viewer-b/dist" "$evidence_dir/r2/viewer-b.sha256"
  inventory_tree "$evidence_dir/r2/viewer-proof/dist" "$evidence_dir/r2/viewer-proof.sha256"
  cmp "$evidence_dir/r2/viewer-a.sha256" "$evidence_dir/r2/viewer-b.sha256"
  cmp "$evidence_dir/r2/viewer-a.sha256" "$evidence_dir/r2/viewer-proof.sha256"
}
run_step clean-r2-viewer-compare \
  'compare full regular-file inventories for all clean R2 distributions' \
  compare_r2_viewers

# Step 7: maximum-scene process benchmark. Receipt assembly follows closure so
# its own command cannot recursively appear in the ledger it digest-binds.
mkdir -p "$evidence_dir/r2/compile-benchmark"
run_step maximum-compile-benchmark \
  'node docs/evaluation/measure-r2-compile.mjs --binary <fresh-release>/nomos-observed-scene --fixture maximum --output <output>/r2/compile-benchmark' \
  node docs/evaluation/measure-r2-compile.mjs \
    --binary "$fresh_release_target/release/nomos-observed-scene" \
    --fixture fixtures/r2/maximum-observed-scene.json \
    --output "$evidence_dir/r2/compile-benchmark"

[[ $next_ordinal -eq 34 ]] || fail 'ordered command ledger does not contain exactly 33 commands'

# Step 8: prove every other process closed while the sampler still covers the
# checkout, stop it for its final row, prove final closure and immutability,
# then assemble and verify the non-recursive receipt.
namespace_children_before_file=$evidence_dir/metadata/namespace-children-before-sampler-stop.txt
r2_measure_process_closure \
  "$inner_netns" "$NOMOS_R2_PROOF_TOKEN" "$namespace_children_before_file" \
  "$disk_sampler_pid" "$disk_sampler_pid" "$disk_sampler_start_ticks" ||
  fail 'a proof process remains while the sampler is active'
namespace_children_before=$(jq -Rsc \
  'split("\n") | map(select(length > 0) | tonumber)' \
  "$namespace_children_before_file")

stop_sampler
trap - EXIT INT TERM

[[ ${R2_SAMPLER_STOP_REQUESTED_NS:-} =~ ^(0|[1-9][0-9]*)$ ]] ||
  fail 'filesystem sampler did not retain its canonical stop timestamp'
shutdown_du_json=$(node "$filesystem_evidence_helper" du-check \
  "${filesystem_cli_args[@]}" --phase shutdown) ||
  fail 'shutdown du crosscheck or immediate statfs snapshot failed'
a_before_bytes=$(printf '%s\n' "$shutdown_du_json" | jq -er \
  '.snapshot.used_bytes | select(test("^(0|[1-9][0-9]*)$"))') ||
  fail 'shutdown statfs A_before is missing or malformed'
/usr/bin/unlink "$reservation"; [[ ! -e $reservation && ! -L $reservation ]] || fail 'finalization reservation was recreated'
/usr/bin/sync -f "$evidence_dir"
release_json=$(node "$filesystem_evidence_helper" release-check \
  "${filesystem_cli_args[@]}" \
  --reservation-path "$reservation" \
  --reservation-length-bytes "$reservation_length_bytes" \
  --reservation-allocated-bytes "$reservation_allocated_bytes" \
  --a-before-bytes "$a_before_bytes") ||
  fail 'finalization reservation release did not free its allocated bytes'

# Only after A_after is captured may finalization evidence be written.
printf '%s\n' "$shutdown_du_json" >"$filesystem_evidence_dir/du-shutdown.json"
printf '%s\n' "$release_json" >"$filesystem_evidence_dir/release.json"
summary_json=$(node "$filesystem_evidence_helper" summarize \
  "${filesystem_cli_args[@]}" \
  --identity-json "$disk_identity" --raw "$disk_raw_samples" \
  --public "$disk_samples" \
  --setup-du-json "$filesystem_evidence_dir/du-setup.json" \
  --shutdown-du-json "$filesystem_evidence_dir/du-shutdown.json" \
  --finalization-json "$filesystem_evidence_dir/release.json" \
  --stop "$disk_sampler_stop" \
  --nominal-interval-ns "$disk_sample_period_ns") ||
  fail 'filesystem summary, schedule, crosschecks, or ceiling differs'
closed_maximum_bytes=$(printf '%s\n' "$summary_json" | jq -er \
  '.maximum_allocated_bytes | select(test("^(0|[1-9][0-9]*)$"))') ||
  fail 'filesystem summary maximum is missing or malformed'
summary_origin_ns=$(printf '%s\n' "$summary_json" | jq -r '.sampler_origin_ns')
summary_stop_ns=$(printf '%s\n' "$summary_json" | jq -r '.stop_requested_ns')
[[ $summary_origin_ns == "$disk_sampler_started" &&
  $summary_stop_ns == "$R2_SAMPLER_STOP_REQUESTED_NS" ]] ||
  fail 'filesystem summary differs from the parent sampler handoff'
printf '%s\n' "$summary_json" >"$filesystem_evidence_dir/summary.json"

porcelain_end=$(git status --porcelain=v1 --untracked-files=all)
end_head=$(git rev-parse --verify 'HEAD^{commit}')
end_tree=$(git rev-parse --verify 'HEAD^{tree}')
[[ -z $porcelain_end && $end_head == "$head" && $end_tree == "$tree" ]] ||
  fail 'candidate HEAD, tree, or clean state changed during proof'
jq -n --arg commit "$end_head" --arg tree "$end_tree" --arg porcelain "$porcelain_end" \
  '{outcome:"pass",commit:$commit,tree:$tree,porcelain:$porcelain}' \
  >"$evidence_dir/metadata/clean-end.json"

namespace_children_file=$evidence_dir/metadata/namespace-children.txt
r2_measure_process_closure \
  "$inner_netns" "$NOMOS_R2_PROOF_TOKEN" "$namespace_children_file" ||
  fail 'a proof process remains after sampler closure'
namespace_children=$(jq -Rsc \
  'split("\n") | map(select(length > 0) | tonumber)' \
  "$namespace_children_file")
jq -n --argjson namespace_children "$namespace_children" \
  --argjson namespace_children_before "$namespace_children_before" \
  '{outcome:"pass",checked_while_sampler:true,checked_after_sampler:true,
    leaked_processes:$namespace_children,
    namespace_children_before_sampler_stop:$namespace_children_before,
    namespace_children:$namespace_children}' \
  >"$evidence_dir/metadata/process-closure.json"

jq -n \
  --arg output "$output_relative" \
  '{outcome:"pass",output_relative:$output,allowed_roots:[$output,"target"],
    outside_writes:[],inputs_unchanged:true}' \
  >"$evidence_dir/metadata/write-boundary.json"

receipt_helper=docs/evaluation/r2-complete-proof-receipt.mjs
[[ -f $receipt_helper && ! -L $receipt_helper ]] ||
  fail 'R2 receipt assembler/verifier is missing or symlinked'
node "$receipt_helper" assemble \
  --repo "$repo_root" \
  --output "$evidence_dir" \
  --commit "$head" \
  --tree "$tree" \
  --issue "$issue" \
  --issue-body-sha256 "$issue_body_sha256"
node "$receipt_helper" verify \
  --repo "$repo_root" \
  --output "$evidence_dir"
node "$filesystem_evidence_helper" final-check \
  "${filesystem_cli_args[@]}" \
  --closed-maximum-bytes "$closed_maximum_bytes" >/dev/null ||
  fail 'post-receipt statfs allocation exceeds the closed maximum'

printf 'R2 complete proof inner: PASS commit=%s tree=%s\n' "$head" "$tree"
