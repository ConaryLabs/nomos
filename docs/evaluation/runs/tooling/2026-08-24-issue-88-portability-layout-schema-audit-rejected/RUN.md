# Issue 88 portability/layout/schema audit: rejected

This fresh DeepSeek-family audit reviewed detached commit
`202a308ca47e6069c206d1f3e8f127867b44b225`. It is preserved as a failed
predecessor and has no green-proof value.

All eight required commands passed in the full-history checkout. The reviewer
then returned `fail` with three findings:

1. the frozen rc1 test still required the candidate Git object in a shallow
   checkout;
2. committed revision-6 rehearsal runs contained all task-record bytes but no
   supported reconstruction layout, so their adjudications were not directly
   tool-verifiable without the retained host records; and
3. generated checker receipts and run results lacked complete standalone exact
   schema validation.

No operator message, coaching, retry, or repair occurred during the session.
All findings were accepted for prospective repair after the audit ended. A
fresh replacement audit is required; this record is not relabelled.

## Identity and accounting

- target commit: `202a308ca47e6069c206d1f3e8f127867b44b225`
- reviewer: DeepSeek `deepseek-v4-flash-vision-exp`
- Pi: `0.84.2`
- session: `01a03137-bf7d-7159-a3fd-973f7ba5aa7b`
- environment: `Linux 7.1.8-arch1-3 x86_64 GNU/Linux`
- assistant responses: 90
- aggregate provider-reported input tokens: 150190
- aggregate provider-reported output tokens: 69150
- aggregate provider-reported cache-read tokens: 10487680
- aggregate provider-reported total tokens: 10707020
- aggregate provider-reported cost: 0.069754104 USD
- event-stream SHA-256:
  `35cf32e42fb33a6f2fa3e3f825bfac20f37c95cebce9645f2083e3f434eaafbe`
- stderr SHA-256 (empty):
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `audit.json` SHA-256:
  `5ba881dc85a0809105ec1daf739ffc182d8e2ec11d2879accfababd624e0755a`

The launch used a fresh ephemeral Pi configuration containing only the
existing DeepSeek credential and repository-pinned model catalog. Session
persistence, extensions, skills, prompt templates, themes, and context-file
injection were disabled; only the built-in `read` and `bash` tools were
enabled. The exact user prompt is
`docs/evaluation/issue-88-non-author-audit-prompt.txt` at the target commit.
