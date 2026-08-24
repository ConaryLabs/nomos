# Issue 88 boundary-generation audit: rejected

This fresh DeepSeek-family audit reviewed detached commit
`73a3820450b899c1b01387883d925ff63b2777f8`. It is preserved as a failed
predecessor and has no green-proof value.

All eight required checkout and proof commands passed. The reviewer then
constructed a revision-6 checker record whose otherwise self-consistent
packet-run boundary was downgraded from generation 4 to retired generation 3.
The finalizer admitted the stale boundary without the exact-device,
null-readable, or null-writable checks and mechanically produced `pass`. The
reviewer returned `fail` with one major finding.

No operator message, coaching, retry, or repair occurred during the session.
The finding was accepted for prospective repair after the audit ended. A fresh
replacement audit is required; this record is not relabelled.

## Identity and accounting

- target commit: `73a3820450b899c1b01387883d925ff63b2777f8`
- reviewer: DeepSeek `deepseek-v4-flash-vision-exp`
- Pi: `0.84.2`
- session: `01a03117-1610-78ab-846a-19199f3bcf85`
- environment: `Linux 7.1.8-arch1-3 x86_64 GNU/Linux`
- assistant responses: 84
- aggregate provider-reported input tokens: 181696
- aggregate provider-reported output tokens: 52888
- aggregate provider-reported cache-read tokens: 11960192
- aggregate provider-reported total tokens: 12194776
- aggregate provider-reported cost: 0.07373461760000001 USD
- event-stream SHA-256:
  `b2fe84c6cbfd727c2a6cb8df3a691f850766b781452b4032d022eba5c8729cb3`
- stderr SHA-256 (empty):
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `audit.json` SHA-256:
  `232f291e0ac338f38836fb67aec9a792b66003a5f4bfb91a4c99e593fb9caa4e`

The launch used a fresh ephemeral Pi configuration containing only the
existing DeepSeek credential and repository-pinned model catalog. Session
persistence, extensions, skills, prompt templates, themes, and context-file
injection were disabled; only the built-in `read` and `bash` tools were
enabled. The exact user prompt is
`docs/evaluation/issue-88-non-author-audit-prompt.txt` at the target commit.
