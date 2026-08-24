# Issue 90 lineage audit: rejected malformed return

This fresh DeepSeek-family non-author audit reviewed exact detached commit
`912e3d3f5b0856ff339d9ef24ca1c15d3537fbb1`. Its substantive review reported
zero findings, independently passed all ten required commands, added three
useful protected-surface probes, and left the checkout clean. It is rejected as
issue-90 acceptance evidence because the final response omitted commas between
adjacent strings in `notes`, so the demanded JSON object does not parse.

No engineering finding or operator intervention occurred. The rejected return
is not repaired, normalized, or promoted to a pass. A prompt amendment now
requires the reviewer to parse its exact final bytes before sending, and a
fresh non-formal audit is required at the resulting exact head.

## Identity and accounting

- target commit: `912e3d3f5b0856ff339d9ef24ca1c15d3537fbb1`
- reviewer: DeepSeek `deepseek-v4-flash-vision-exp`
- Pi: `0.84.2`
- session: `01a031a4-2762-75d8-b32f-4fd6126d7b17`
- session start: `2026-08-24T02:40:36.706Z`
- environment: host `remi`, detached clean review worktree
- assistant responses: 33
- tool executions: 50
- aggregate provider-reported input tokens: 60,590
- aggregate provider-reported output tokens: 24,250
- aggregate provider-reported cache-read tokens: 1,443,200
- aggregate provider-reported total tokens: 1,528,040
- aggregate provider-reported cost: 0.01931356 USD
- event-stream SHA-256:
  `8b9a6eeba4d1439a79701ccf225dc8cf1d3d79038cf54b6d47852e2a3ba49fd5`
- stderr SHA-256 (empty):
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- exact malformed final-text SHA-256:
  `0d45302e73b087ddfee9945b49e2e519b663d4d6a96da8a169f6d9442972d24d`

The raw stream remains non-canonical scratch evidence. This receipt preserves
its identity and rejection reason without misrepresenting malformed text as a
validated `nomos.issue90.non_author_audit@1` document.
