# Agent guide

Read [README.md](README.md), then [THESIS.md](THESIS.md), then
[KERNEL.md](KERNEL.md). For changes to the acceptance contract, also read the
latest record under `docs/decisions/`.

This repository is the thesis and, later, the kernel that tests it. Nothing here
is authority for any other project.

## Working rules

- **Acceptance precedes implementation.** `KERNEL.md` is the executable contract.
  Code may discover that the contract is ambiguous, contradictory, impossible,
  or based on a falsified assumption; it may not silently reinterpret it.
- **Contract repair is explicit.** A correction requires an owner-authorized
  decision record containing the prior wording, replacement wording, reason,
  effect on existing evidence, owner disposition, and new contract revision.
  Weakening a criterion merely because an implementation failed it is forbidden.
- **No free-floating architecture.** Until Gate K passes, `THESIS.md` changes only
  to record a resolved disagreement, repair a contradiction, or add an open
  question. New mechanism belongs in executable code with a test, or nowhere.
- **Engineer, do not patch the accepted path.** Workarounds and shims may not
  enter the kernel merely to make a test pass. When a rewrite is the honest
  answer, say so.
- **Quarantined experiments are allowed.** Disposable work may live under
  `experiments/` or on an explicitly experimental branch. It is non-authoritative,
  cannot satisfy acceptance, and must be promoted through a clean implementation
  before entering the accepted kernel.
- **Fix or file immediately.** Anything found mid-work is fixed in that change or
  recorded as an issue with evidence and a clear disposition.
- **Nothing is green until someone other than its author reruns the proof.** The
  rerun receipt records the commit, command, environment, result, and reviewer.
- **Measure budgets; never assume them.** Build time, peak disk, validation
  latency, replay throughput, and package size are numbers in the record, not
  adjectives in a meeting.
- **Cold review is fuzzing, not authority.** A different model family may attack
  design or code under `docs/review/`; a human owner decides what matters. Use
  `docs/evaluation/COLD_AGENT_PROTOCOL.md` for formal cold-author and cold-debug
  gates.
- **Compiled worlds are immutable evidence.** Runtime commands, migrations, and
  tests write new state or package artifacts; they never edit the input package
  in place.
- **Do not import another project's bureaucracy by accident.** Validators,
  document families, routing tools, and conventions from other repositories
  require a recorded decision here before use.

## Change flow

1. Start from an issue with falsifiable acceptance.
2. Create a feature branch; never develop directly on `main`.
3. Keep the branch scoped to one implementation slice.
4. Run the available proof locally or through CI.
5. Open a draft pull request describing evidence, unresolved limits, and issue
   coverage.
6. Obtain a non-author rerun before calling the slice green.
7. Leave merge and contract disposition to the owner.

## Repository layout

```text
README.md        status and reading order
THESIS.md        exploratory design thesis, currently revision 2
KERNEL.md        Gate K acceptance contract, currently revision 2
docs/decisions/  owner-authorized contract and architecture decisions
docs/evaluation/ reproducible evaluation protocols
docs/review/     review syntheses, provenance notes, and cold-review records
experiments/     optional disposable work that cannot satisfy acceptance
```

The Gate K workspace layout and permitted dependency edges are defined directly
in `KERNEL.md`; do not infer them from the historical disagreement table.
