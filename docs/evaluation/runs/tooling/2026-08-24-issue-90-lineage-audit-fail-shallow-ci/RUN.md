# Issue 90 lineage audit: fail on shallow CI coupling

This fresh DeepSeek-family non-author audit reviewed exact detached commit
`1cecdcb06ff9618e320915f55391f945e81b1f62`. The exact final response was valid
JSON. It passed every required local command but found a blocking CI regression:
the new full-history candidate-lineage test had been inserted into the general
evaluation harness, while `verify.yml` intentionally uses GitHub's default
depth-1, tagless checkout. The harness therefore could not resolve
`gate-k-rc1` in the environment whose success issue #90 requires.

The audit is retained as `fail`. The repair removes the accidental coupling.
Candidate lineage remains an explicit, separately run full-history proof; the
existing general harness remains portable to depth-1 checkouts. Changing either
workflow to fetch history would violate this issue's byte-identical workflow
surface and consume extra CI, so it is not the honest repair.

## Identity and accounting

- target commit: `1cecdcb06ff9618e320915f55391f945e81b1f62`
- reviewer: DeepSeek `deepseek-v4-flash-vision-exp`
- Pi: `0.84.2`
- session: `01a031cf-7afe-79d4-b817-aeed0c830769`
- session start: `2026-08-24T03:27:56.158Z`
- environment: host `remi`, detached clean review worktree
- assistant responses: 75
- tool executions: 117
- aggregate provider-reported input tokens: 67,182
- aggregate provider-reported output tokens: 217,888
- aggregate provider-reported cache-read tokens: 9,135,104
- aggregate provider-reported total tokens: 9,420,174
- aggregate provider-reported cost: 0.0959924112 USD
- event-stream SHA-256:
  `3a99f6aa9a2ec1bb373f03a932532fe38e690027ca852404b9edb8a9782e9b37`
- stderr SHA-256 (empty):
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- exact final-text SHA-256:
  `36cae7468fc0544900db1e00d7a19d2c42fa01de727f5182bf4681db120bd550`

No operator message, coaching, retry, or repair occurred during the session.
`audit.json` beside this receipt preserves the blocking finding and complete
command disposition in a normalized record; the exact provider text is
identified by the hashes above.
