# Revision-6 author rehearsal: rejected evidence shape

- candidate/tooling commit: `66c96f4fef7552b047a260c28168bb52f7e06566`
- classification: non-formal rehearsal; no formal attempt reserved or launched
- subject: Gemini 3.7 Flash, high, fresh Pi session
- checker: DeepSeek V4 Flash Vision Exp, max, fresh Pi session
- operator intervention: none
- operator retries: zero
- disposition: rejected before adjudication because the checker result did not
  satisfy the exact revision-6 schema

The Gemini subject completed the author brief. Its task receipt is
`cc3885a1869bdedb6c56088d7cb6c47bb6a172626e93349bf77d2bab2eea86d6`.
It added `watch_lamp`, validated and compiled the source, preserved complete
accounting, and used only `/workspace` plus the exact permitted `/dev/null`
device.

The DeepSeek checker independently reproduced the package byte-for-byte and
reported semantic pass. Its task receipt is
`03149b9a060fd1a976e5fc8d67c67a7990208d5973a1993d0eb3e95c46a699c9`.
However, checker result
`12545efe46526a2cc9de689919d79cedc31df0df8d5d38c8e03c8b1ba2b4a58f`
added a `checkerReproduction` top-level field. The strict checker-result
validator correctly rejected it because revision 6 allows only `schema`,
`protocolRevision`, `verdict`, `commands`, `reasons`, and optional `evidence`.

The prompt said the checker could include additional hash evidence but did not
state that every additional field had to live inside `evidence`. That ambiguity
is an evaluation-tooling defect, not a Nomos semantic failure. The pair remains
rejected and is not retried. The prompt must be repaired prospectively and a
new pair must use fresh sessions and a new exact tooling commit.
