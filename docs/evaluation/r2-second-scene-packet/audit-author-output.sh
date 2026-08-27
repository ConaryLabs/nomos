#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 second-scene author output audit: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 1 ]] || fail 'usage: audit-author-output.sh <packet-directory>'
packet=$(cd "$1" && pwd -P)
manifest=$packet/MANIFEST.sha256
[[ -f $manifest && ! -L $manifest ]] || fail 'manifest is absent or unsafe'
(cd "$packet" && sha256sum -c MANIFEST.sha256 >/dev/null) || fail 'a manifested packet input changed'

temporary=$(mktemp -d)
trap 'rm -r -- "$temporary"' EXIT
awk '{print $2}' "$manifest" | sort >"$temporary/manifested"
find "$packet" -type f -printf '%P\n' | sort >"$temporary/all"
comm -13 "$temporary/manifested" "$temporary/all" | grep -v '^MANIFEST.sha256$' >"$temporary/added" || true
cat >"$temporary/allowed" <<'EOF'
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/BROWSER_RECEIPT.json
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SCENE_SIGNATURES.json
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SECOND_AUTHOR_RECEIPT.md
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_1.png
docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/scene_2.png
fixtures/r2/plans/scene_two.json
fixtures/r2/scenes/scene_two.json
EOF
cmp -s "$temporary/allowed" "$temporary/added" || fail 'added-file set differs from the exact author allowlist'
if find "$packet" -mindepth 1 ! -type d ! -type f -print -quit | grep -q .; then
  fail 'packet contains a symlink or non-regular entry'
fi

printf 'R2_SECOND_SCENE_AUTHOR_OUTPUT_AUDIT PASS\n'
printf 'added_files 8\n'
