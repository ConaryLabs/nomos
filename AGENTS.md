# Agent guide

Read [THESIS.md](THESIS.md) first, then [KERNEL.md](KERNEL.md). This repository
is the thesis and, later, the kernel that tests it. Nothing here is authority
for any other project.

## Rules for working here

- **The kernel's acceptance criteria are fixed before code.** KERNEL.md changes
  only by an owner decision recorded in its own history, never to make a test
  pass.
- **No free-floating architecture.** Until the kernel compiles, THESIS.md is
  extended only to record a resolved disagreement (§20) or an open question
  (§21). New mechanism goes in code with a test, or nowhere.
- **Engineer, don't patch.** No workarounds, shims, or temporary fixes. When a
  rewrite is the honest answer, say so.
- **Fix or file, immediately.** Anything found mid-work is fixed in that change
  or recorded as an issue with evidence.
- **Nothing is green until re-run by someone other than its author.**
- **Measure budgets; never assume them.** Build time, disk, latency — numbers
  in the record, not adjectives.
- **Cold review is fuzzing, not authority.** Design and code reviews by a model
  family other than the author's are welcome and recorded under
  `docs/review/`; a human decides what matters.
- This repository has its own conventions. Do not import another project's
  validators, document families, or routing tooling here without a recorded
  decision.

## Layout

```text
THESIS.md        the design thesis (non-authoritative, exploratory)
KERNEL.md        acceptance criteria for the executable semantic kernel
docs/review/     verbatim adversarial-review transcripts that produced the thesis
```

The kernel's workspace layout, when it exists, follows THESIS.md §20
(`core / schema / compiler / sim / cli` at minimum, hard-bounded crates).
