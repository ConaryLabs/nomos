# Issue 94 Claude non-author audit: blocked before execution

A fresh supplemental Claude Opus 5 evidence-audit session was attempted against
detached commit `eb3844a9cc237331c9f91ee19eed2ffae54179b8`. Anthropic rejected the
first request before any model output or tool execution because the account was
out of extra usage. The attempt is not an audit result and did not inspect or
modify the checkout.

- provider: `anthropic`
- model: `claude-opus-5`
- thinking: `high`
- Pi: `0.84.2`
- session: `01a033fb-4087-77b7-8d80-5d3b498b19f5`
- model input tokens: `0`
- model output tokens: `0`
- tool calls: `0`
- event-stream SHA-256:
  `2abc6ca21de0e2825547b72b6027d2264feca77b90899ea86a51858962f0efcd`
- stderr SHA-256 (empty):
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- provider disposition: `invalid_request_error` — out of extra usage

The replacement evidence audit uses a fresh DeepSeek V4 Pro session. It is a
supplemental non-author repository/evidence audit only, uses a different model
from the formal DeepSeek V4 Flash Vision Exp subject, and does not consume or
substitute for the Gemini-family semantic checker reserved for issue #96.
