# Issue 88 schema-mix audit: rejected

This fresh DeepSeek-family audit reviewed detached commit
`a1cc26cd41157352a714860bb39c50e3324ddeb1`. It is preserved as a failed
predecessor and has no green-proof value.

All eight required checkout and proof commands passed. The reviewer then
constructed two additional adversarial cases and returned `fail` with two
minor findings:

1. a revision-6 composite admitted a stale `checker_result@1` and mechanically
   finalized `pass`; and
2. a correctly failed synthetic retry record produced a `RUN.md` that falsely
   hardcoded zero subject and checker retries.

No operator message, coaching, retry, or repair occurred during the session.
Both findings were accepted for prospective repair after the audit ended. A
fresh replacement audit is required; this record is not relabelled.

The model's raw final response prefixed two Markdown backticks with JSON
backslashes, so `audit-raw.txt` is not valid JSON. `audit.json` is a disclosed
mechanical normalization that removes only those two unnecessary backslashes
and sorts object keys; it does not change any claim, verdict, finding, or note.

## Identity and accounting

- target commit: `a1cc26cd41157352a714860bb39c50e3324ddeb1`
- reviewer: DeepSeek `deepseek-v4-flash-vision-exp`
- Pi: `0.84.2`
- session: `01a030ea-0735-73aa-9934-f64dd8dba320`
- environment: `Linux 7.1.8-arch1-3 x86_64 GNU/Linux`
- assistant responses: 88
- aggregate provider-reported input tokens: 180670
- aggregate provider-reported output tokens: 56946
- aggregate provider-reported cache-read tokens: 11019520
- aggregate provider-reported total tokens: 11257136
- aggregate provider-reported cost: 0.07209333599999998 USD
- event-stream SHA-256:
  `63f4eddf8995c226cf010dfb124d0e7ec8494a4f630ab536c64cc785bb0fb289`
- stderr SHA-256 (empty):
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- raw final-response SHA-256:
  `4cd504bff65bb26c16cdf4ade164a3e09f7baca8ee1f42ca35b9d5e745136f31`
- normalized `audit.json` SHA-256:
  `e8c0ca7640525a529d3a10f5c10e176de512ddc28ec1dd1852fccef7a8921c04`

The launch used a fresh ephemeral Pi configuration containing only the
existing DeepSeek credential and the repository-pinned model catalog. Session
persistence, extensions, skills, prompt templates, themes, and context-file
injection were disabled; only the built-in `read` and `bash` tools were
enabled. The exact user prompt is
`docs/evaluation/issue-88-non-author-audit-prompt.txt` at the target commit.
