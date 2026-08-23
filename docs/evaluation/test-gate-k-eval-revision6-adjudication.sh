#!/usr/bin/env bash

# Sourced by test-gate-k-eval-tooling.sh after the canonical author pair exists.

revision6_base="$tmp_dir/author-adjudication.json"

jq '.checkerReproduction = []' \
  "$tmp_dir/author-checker-record/artifacts/checker.json" \
  >"$tmp_dir/revision6-extra-checker-field.json"
assert_blocked 'top-level fields differ from a declared protocol shape' \
  revision6-extra-checker-field python3 \
  "$repo_root/docs/evaluation/gate-k-eval-validate-documents.py" checker-result \
  "$tmp_dir/revision6-extra-checker-field.json"

jq 'del(.records.subject.dimensions.semantic_merit)' "$revision6_base" \
  >"$tmp_dir/revision6-missing-dimension.json"
assert_blocked 'dimension names differ from the protocol' revision6-missing-dimension \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/revision6-missing-dimension.json"

jq '.records.subject.dimensions.semantic_merit.evidence = {}' "$revision6_base" \
  >"$tmp_dir/revision6-evidence-type.json"
assert_blocked 'evidence must be a nonempty array' revision6-evidence-type \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/revision6-evidence-type.json"

jq '.records.subject.dimensions.semantic_merit.verdict = "assisted"' "$revision6_base" \
  >"$tmp_dir/revision6-dimension-value.json"
assert_blocked 'semantic_merit verdict is invalid' revision6-dimension-value \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/revision6-dimension-value.json"

jq '.records.subject.dimensions.semantic_merit.evidence[0].sha256 =
  "0000000000000000000000000000000000000000000000000000000000000000"' \
  "$revision6_base" >"$tmp_dir/revision6-evidence-digest.json"
assert_blocked 'evidence 0 digest differs' revision6-evidence-digest \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/revision6-evidence-digest.json"

jq '.records.subject.verdict = "fail"' "$revision6_base" \
  >"$tmp_dir/revision6-record-mismatch.json"
assert_blocked 'subject verdict must derive as pass' revision6-record-mismatch \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/revision6-record-mismatch.json"

jq '.verdict = "inconclusive"' "$revision6_base" \
  >"$tmp_dir/revision6-overall-mismatch.json"
assert_blocked 'overall verdict must derive as pass' revision6-overall-mismatch \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/revision6-overall-mismatch.json"

jq '
  .records.subject.dimensions.semantic_merit.verdict = "inconclusive" |
  .records.subject.dimensions.semantic_merit.reason = "fixture semantic evidence is incomplete" |
  .records.subject.verdict = "inconclusive" |
  .records.subject.reason = "fixture record derives inconclusive" |
  .verdict = "inconclusive" |
  .reason = "fixture overall verdict derives inconclusive"
  ' "$revision6_base" >"$tmp_dir/revision6-inconclusive.json"
"$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  "$tmp_dir/revision6-inconclusive.json" "$tmp_dir/revision6-inconclusive-run" >/dev/null
jq -e '.verdict == "inconclusive" and
  .records.subject.dimensions.semantic_merit.verdict == "inconclusive"' \
  "$tmp_dir/revision6-inconclusive-run/result.json" >/dev/null

jq '
  .records.subject.dimensions.semantic_merit.verdict = "inconclusive" |
  .records.subject.dimensions.semantic_merit.reason = "fixture semantic evidence is incomplete" |
  .records.subject.verdict = "inconclusive" |
  .records.checker.dimensions.operational_compliance.verdict = "fail" |
  .records.checker.dimensions.operational_compliance.reason = "fixture operational failure" |
  .records.checker.verdict = "fail" |
  .verdict = "fail" |
  .reason = "fixture failure takes precedence over inconclusive"
  ' "$revision6_base" >"$tmp_dir/revision6-fail-precedence.json"
python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/revision6-fail-precedence.json" >/dev/null

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/revision6-retry-checker-record"
jq -S -c '.operatorRetries = 1' \
  "$tmp_dir/revision6-retry-checker-record/task-receipt.json" \
  >"$tmp_dir/revision6-retry-checker-record/task-receipt.update"
mv -- "$tmp_dir/revision6-retry-checker-record/task-receipt.update" \
  "$tmp_dir/revision6-retry-checker-record/task-receipt.json"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/revision6-retry-checker-record" "$tmp_dir/revision6-retry-pass.json"
assert_blocked 'retry breach does not fail independence' revision6-retry-mapping \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/revision6-retry-checker-record" "$tmp_dir/revision6-retry-pass.json"
jq '
  .records.checker.dimensions.independence_integrity.verdict = "fail" |
  .records.checker.dimensions.independence_integrity.reason = "operator retry was recorded" |
  .records.checker.verdict = "fail" |
  .records.checker.reason = "independence failure derives fail" |
  .verdict = "fail" |
  .reason = "operator retry forces a failed result"
  ' "$tmp_dir/revision6-retry-pass.json" >"$tmp_dir/revision6-retry-fail.json"
"$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/revision6-retry-checker-record" "$tmp_dir/revision6-retry-fail.json" \
  "$tmp_dir/revision6-retry-run" >/dev/null
grep -F -- '- Operator retries: `0` subject, `1` checker' \
  "$tmp_dir/revision6-retry-run/RUN.md" >/dev/null

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/revision6-assisted-checker-record"
jq -S -c '.operatorIntervention = "substantive-help"' \
  "$tmp_dir/revision6-assisted-checker-record/task-receipt.json" \
  >"$tmp_dir/revision6-assisted-checker-record/task-receipt.update"
mv -- "$tmp_dir/revision6-assisted-checker-record/task-receipt.update" \
  "$tmp_dir/revision6-assisted-checker-record/task-receipt.json"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/revision6-assisted-checker-record" "$tmp_dir/revision6-assisted.json"
jq '
  .records.checker.dimensions.independence_integrity.verdict = "fail" |
  .records.checker.dimensions.independence_integrity.reason = "substantive help was recorded" |
  .records.checker.verdict = "fail" |
  .records.checker.reason = "independence failure derives fail" |
  .verdict = "assisted" |
  .reason = "substantive help takes precedence"
  ' "$tmp_dir/revision6-assisted.json" >"$tmp_dir/revision6-assisted.update"
mv -- "$tmp_dir/revision6-assisted.update" "$tmp_dir/revision6-assisted.json"
"$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/revision6-assisted-checker-record" "$tmp_dir/revision6-assisted.json" \
  "$tmp_dir/revision6-assisted-run" >/dev/null
jq -e '.verdict == "assisted" and
  .records.checker.dimensions.independence_integrity.verdict == "fail"' \
  "$tmp_dir/revision6-assisted-run/result.json" >/dev/null

jq '
  .schema = "nomos.gate_k.command_adjudication@1" |
  del(.protocolRevision, .records)
  ' "$revision6_base" >"$tmp_dir/revision6-stale-adjudication.json"
assert_blocked 'legacy adjudication requires legacy task receipts' revision6-stale-generation \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  "$tmp_dir/revision6-stale-adjudication.json" "$tmp_dir/revision6-stale-run"

checker_record="$tmp_dir/author-checker-record"
checker_packet="$tmp_dir/author-checker-1"
checker_record_tree_before=$(tree_sha "$checker_record")
checker_packet_tree_before=$(tree_sha "$checker_packet")
cp -R "$checker_record" "$tmp_dir/revision6-checker-record-backup"
cp -R "$checker_packet" "$tmp_dir/revision6-checker-packet-backup"
jq -S -c 'del(.protocolRevision) | .schema = "nomos.gate_k.checker_result@1"' \
  "$tmp_dir/revision6-checker-record-backup/artifacts/checker.json" \
  >"$checker_record/artifacts/checker.json"
install -m 644 "$checker_record/artifacts/checker.json" \
  "$checker_packet/output/checker.json"
refresh_record_receipt_digests "$checker_record"
write_pass_adjudication "$tmp_dir/author-subject-record" "$checker_record" \
  "$tmp_dir/revision6-stale-checker-result.json"
assert_blocked 'checker result generation or content is invalid' \
  revision6-stale-checker-result "$finalizer" \
  "$tmp_dir/author-subject-record" "$checker_record" \
  "$tmp_dir/revision6-stale-checker-result.json" \
  "$tmp_dir/revision6-stale-checker-result-run"
cp -R "$tmp_dir/revision6-checker-record-backup/." "$checker_record/"
cp -R "$tmp_dir/revision6-checker-packet-backup/." "$checker_packet/"

jq -S -c 'del(.protocolRevision) | .schema = "nomos.gate_k.packet_manifest@1"' \
  "$tmp_dir/revision6-checker-record-backup/packet-manifest.json" \
  >"$checker_record/packet-manifest.json"
install -m 644 "$checker_record/packet-manifest.json" \
  "$checker_packet/packet-manifest.json"
stale_manifest_sha=$(sha256sum "$checker_record/packet-manifest.json" | cut -d' ' -f1)
jq -S -c --arg manifest "$stale_manifest_sha" \
  '.packetManifestSha256 = $manifest' "$checker_record/boundary.json" \
  >"$checker_record/boundary.update"
mv -- "$checker_record/boundary.update" "$checker_record/boundary.json"
refresh_record_runtime_evidence "$checker_record"
write_pass_adjudication "$tmp_dir/author-subject-record" "$checker_record" \
  "$tmp_dir/revision6-stale-packet-manifest.json"
assert_blocked 'packet manifest generation or structure is invalid' \
  revision6-stale-packet-manifest "$finalizer" \
  "$tmp_dir/author-subject-record" "$checker_record" \
  "$tmp_dir/revision6-stale-packet-manifest.json" \
  "$tmp_dir/revision6-stale-packet-manifest-run"
cp -R "$tmp_dir/revision6-checker-record-backup/." "$checker_record/"
cp -R "$tmp_dir/revision6-checker-packet-backup/." "$checker_packet/"
[[ $(tree_sha "$checker_record") == "$checker_record_tree_before" ]]
[[ $(tree_sha "$checker_packet") == "$checker_packet_tree_before" ]]

jq '
  .findings[0].kind = "undeclared_information_ingress" |
  .findings[0].pathToken = "/workspace/undeclared-source"
  ' "$tmp_dir/forbidden-device-adjudication.json" \
  >"$tmp_dir/revision6-ingress-unmapped.json"
assert_blocked 'does not force independence integrity to fail' revision6-ingress-mapping \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/forbidden-device-checker-record" "$tmp_dir/revision6-ingress-unmapped.json"

for token in /dev/fd/1 /proc/self/fd/1 /workspace/output/null-alias; do
  label=${token//\//-}
  jq --arg token "$token" '.findings[0].pathToken = $token' \
    "$tmp_dir/forbidden-device-adjudication.json" \
    >"$tmp_dir/revision6-exact-exception$label.json"
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
    "$tmp_dir/forbidden-device-checker-record" \
    "$tmp_dir/revision6-exact-exception$label.json" >/dev/null
done

printf 'gate-k revision-6 adjudication regressions: PASS\n'
