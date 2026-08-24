#!/usr/bin/env bash

# Shared formal-ledger checks for the task finalizer. The caller owns `set -e`
# policy and supplies the exact validator and ledger paths.

gate_k_validate_closed_formal_ledger() {
  local validator=$1
  local ledger=$2
  local frozen_inventory

  python3 "$validator" validate "$ledger" || return 1
  frozen_inventory=$(mktemp) || return 1
  head -n 4 "$ledger" >"$frozen_inventory"
  if ! python3 "$validator" validate-frozen-inventory "$frozen_inventory"; then
    rm -f -- "$frozen_inventory"
    return 1
  fi
  rm -f -- "$frozen_inventory"
}

gate_k_count_formal_receipt_events() {
  local ledger=$1
  local event=$2
  local attempt=$3
  local record=$4
  local receipt_sha=$5

  jq -s --arg event "$event" --arg attempt "$attempt" \
    --arg commit "$(jq -r .candidateCommit "$record/task-receipt.json")" \
    --arg shape "$(jq -r .shape "$record/task-receipt.json")" \
    --arg provider "$(jq -r .identity.provider "$record/task-receipt.json")" \
    --arg model "$(jq -r .identity.model "$record/task-receipt.json")" \
    --arg thinking "$(jq -r .identity.thinking "$record/task-receipt.json")" \
    --arg manifest "$(jq -r .digests.packetManifestSha256 "$record/task-receipt.json")" \
    --arg receipt "$receipt_sha" --arg outcome "$(jq -r .outcome "$record/task-receipt.json")" '
    [.[] | select(.event == $event and
      ($attempt == "" or .attemptId == $attempt) and
      .candidateCommit == $commit and .shape == $shape and
      .provider == $provider and .model == $model and .thinking == $thinking and
      .packetManifestSha256 == $manifest and .taskReceiptSha256 == $receipt and
      .outcome == $outcome)] | length
    ' "$ledger"
}

gate_k_validate_formal_receipt_event() {
  local ledger=$1
  local record=$2
  local receipt_sha=$3
  local receipt_schema receipt_shape event attempt expected matches

  receipt_schema=$(jq -r .schema "$record/task-receipt.json")
  receipt_shape=$(jq -r .shape "$record/task-receipt.json")
  event=close
  attempt=$(jq -r '.attemptReservation.attemptId // empty' "$record/task-receipt.json")
  if [[ $receipt_schema == nomos.gate_k.task_receipt@1 ]]; then
    event=import-close
    attempt=
    case $receipt_shape in
      author) expected=732af45918ebc27c02675f6c75c32e7718407545c9fa3a39de327d3591d382a8 ;;
      author-checker) expected=2e8c97d5a939ddd6fa9b33769f6e24b80fc242b1420c2660eef7f9742d542db3 ;;
      debug) expected=2820d2f46b2d895abc22b6677f4f3ba908199cdb9d057aee181b477eaeb82390 ;;
      debug-checker) expected=0053d3df610e7e31322a2cfd9dfc641e160d3e5c64582df387d34cd4ddd37d37 ;;
      *) return 1 ;;
    esac
    [[ $receipt_sha == "$expected" ]] || return 1
  elif [[ $receipt_schema == nomos.gate_k.task_receipt@2 ]]; then
    [[ -n $attempt ]] || return 1
  else
    return 1
  fi
  matches=$(gate_k_count_formal_receipt_events \
    "$ledger" "$event" "$attempt" "$record" "$receipt_sha")
  [[ $matches -eq 1 ]]
}
