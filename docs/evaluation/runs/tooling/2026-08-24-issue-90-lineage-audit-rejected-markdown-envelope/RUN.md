# Issue 90 lineage audit: rejected Markdown envelope

This fresh DeepSeek-family non-author audit reviewed exact detached repair
commit `576cbce6cad1c65db8a39f6be0dc05d0a4bef31e`. It reported substantive
`pass`, zero findings, all ten required commands passing, the protected surface
unchanged, the round-one evidence preserved, the formal ledger unchanged, and
a clean checkout before and after. It also independently exercised the repaired
deletion and rename-masking boundary plus additional protected mode/deletion
probes.

The audit is nevertheless rejected as issue-90 acceptance evidence because the
provider wrapped the valid object in a Markdown `json` code fence. Therefore
the exact final response bytes do not parse as the demanded JSON object. The
object is not stripped, normalized, or promoted. A further prompt amendment
requires byte 1 `{`, final non-whitespace byte `}`, a single-line value, and no
fence, language label, preface, or suffix. A fresh non-formal audit is required.

## Identity and accounting

- target commit: `576cbce6cad1c65db8a39f6be0dc05d0a4bef31e`
- reviewer: DeepSeek `deepseek-v4-flash-vision-exp`
- Pi: `0.84.2`
- session: `01a031bc-f1ee-7a27-a527-428f6be15b4d`
- session start: `2026-08-24T03:07:41.422Z`
- environment: host `remi`, detached clean review worktree
- assistant responses: 44
- tool executions: 61
- aggregate provider-reported input tokens: 71,902
- aggregate provider-reported output tokens: 102,197
- aggregate provider-reported cache-read tokens: 3,174,272
- aggregate provider-reported total tokens: 3,348,371
- aggregate provider-reported cost: 0.0475694016 USD
- event-stream SHA-256:
  `600008fe0f431815b80fbe796ca8c525fb6e56f8dd1d10d2ed5ac49ead3130eb`
- stderr SHA-256 (empty):
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- exact fenced final-text SHA-256:
  `c6a02ac98393ae49b53350c6bd564af3c4a8bfc5dead87dff1fe67d832d1e1c0`

No operator message, coaching, retry, or repair occurred during the session.
The raw stream remains non-canonical scratch evidence; this receipt preserves
its identity and exact rejection reason without treating the fenced inner
object as the provider's requested return format.
