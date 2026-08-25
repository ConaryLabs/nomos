# Agent guide

Read [README.md](README.md), then [THESIS.md](THESIS.md), then
[KERNEL.md](KERNEL.md), then [RUNTIME.md](RUNTIME.md). For changes to the
acceptance contract, also read the latest record under `docs/decisions/`.

Nomos is the project/runtime; The Signed World is the thesis it tests. Nothing
here is authority for any other project.

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
- **Touching a code file over ~1,000 lines means decomposing it in that change.**
  This is an engineering rule of the shop, not a Gate K acceptance criterion:
  Gate K is judged on dependency boundaries and observed behaviour, but a
  routinely read or edited code file that has grown past about a thousand lines
  is reorganised by whoever touches it next, so the tree is set up right for the
  next author. Guiding documents such as `THESIS.md` are not subject to this
  code-organisation rule.
- **Fix or file immediately.** Anything found mid-work is fixed in that change or
  recorded as an issue with evidence and a clear disposition.
- **Nothing is green until someone other than its author reruns the proof.** The
  rerun receipt records the commit, command, environment, result, and reviewer.
- **Measure budgets; never assume them.** Build time, peak disk, validation
  latency, replay throughput, and package size are numbers in the record, not
  adjectives in a meeting.
- **The kernel crates stay dependency-free by decision, not superstition.** The
  six kernel crates admit no third-party dependency and `cargo xtask boundary`
  still fails closed on them. Outside them, R1's dependency policy is set by
  `docs/decisions/0017-post-gate-k-runtime-epoch.md`: a committed lockfile, each
  dependency vendored or digest-pinned with its license preserved, and each
  addition recorded in `RUNTIME.md`. This was never a permanent claim that later
  epochs should reimplement mature libraries.
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
README.md          status and reading order
THESIS.md          exploratory design thesis, currently revision 2
KERNEL.md          Gate K acceptance contract, currently revision 7
RUNTIME.md         R1 epoch contract, currently draft revision 1
docs/decisions/    owner-authorized contract and architecture decisions
docs/evaluation/   reproducible evaluation protocols
docs/review/       review syntheses, provenance notes, and cold-review records
docs/workspace.md  crate map, boundary tool, and pinned workspace choices
crates/            the six Gate K kernel crates
xtask/             workspace tooling; the dependency-boundary check
.github/workflows/ the verification lane
experiments/       optional disposable work that cannot satisfy acceptance
```

The Gate K workspace layout and permitted dependency edges are defined directly
in `KERNEL.md`; do not infer them from the historical disagreement table.
`docs/workspace.md` records where those crates live and how the boundary check
proves them, and `cargo xtask boundary` fails closed on any new workspace member
that neither `KERNEL.md` nor that document declares.
