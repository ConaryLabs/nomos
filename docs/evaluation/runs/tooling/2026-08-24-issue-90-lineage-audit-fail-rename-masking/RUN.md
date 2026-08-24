# Issue 90 lineage audit: fail on rename masking

This fresh DeepSeek-family non-author audit reviewed exact detached commit
`156c24c0f146cf263252d9275a91f462d4806cc1`. All ten required commands passed,
but the reviewer found a blocking false-pass in the new candidate-lineage
proof. Git's default rename detection could pair a deleted protected fixture
with byte-identical evidence under `docs/evaluation/`, report only the allowed
destination path to `--name-only`, and hide the protected deletion.

The audit is correctly retained as `fail`. The repair uses `--no-renames` for
the security decision, explicitly protects `.cargo/`, and adds both deletion
and rename-pair negative fixtures. The finding changes no kernel semantic,
round-one record, protocol rubric, or formal-attempt count.

## Identity and accounting

- target commit: `156c24c0f146cf263252d9275a91f462d4806cc1`
- reviewer: DeepSeek `deepseek-v4-flash-vision-exp`
- Pi: `0.84.2`
- session: `01a031ac-482f-7e5a-baed-dae2da26914c`
- session start: `2026-08-24T02:49:29.391Z`
- environment: host `remi`, detached clean review worktree
- assistant responses: 66
- tool executions: 88
- aggregate provider-reported input tokens: 78,610
- aggregate provider-reported output tokens: 58,177
- aggregate provider-reported cache-read tokens: 4,841,472
- aggregate provider-reported total tokens: 4,978,259
- aggregate provider-reported cost: 0.0408510816 USD
- event-stream SHA-256:
  `d8d0016b9d3938204bf7071a8a695aec7526e465e5d61f1ef74c78edb1a1a17c`
- stderr SHA-256 (empty):
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- exact final-text SHA-256:
  `baa96f97a4a29f785947bcece654e3bdc05c1d35ad7cb90d2cc5857389d0d0af`

No operator message, coaching, retry, or repair occurred during the session.
`audit.json` beside this receipt preserves the complete finding and command
verification in a normalized record with condensed notes. The exact provider
text is identified separately by its final-text and event-stream hashes above.
