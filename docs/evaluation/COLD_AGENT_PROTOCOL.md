---
title: Cold-agent evaluation protocol
status: Contractual for formal cold-agent claims
revision: 6
supersedes_protocol_revision: 5
date: 2026-08-23
decision_record: docs/decisions/0015-gate-k-round-two.md
applies_to: Gate K, Gate 3, Gate 4, cold design review
---

# Cold-agent evaluation protocol

A claim that an LLM-native system works must include an author or debugger that
did not help invent the system and was not in the conversation when it was
built. “A stranger succeeded” is not evidence unless the stranger's information,
tools, limits, and interventions are recorded.

Cold review is fuzzing, not authority. A human owner adjudicates findings and
accepts or rejects changes.

## 1. Roles

- **Owner:** chooses the brief, accepts the evaluation plan, and decides the
  result's significance.
- **Operator:** creates the blind packet, launches the model session, provides
  only permitted tools, and records evidence. The operator may not coach.
- **Subject model:** the cold author, cold debugger, or cold reviewer.
- **Adjudicator:** reviews the evidence against the predeclared rubric. The owner
  may be the adjudicator; the implementation author may not be the sole
  adjudicator of their own proof.

## 2. Model eligibility

A formal cold subject must:

- come from a different provider/model family than the principal author of the
  evaluated slice;
- run in a fresh session with no access to the founding conversation;
- have no prior participation in the repository's design or implementation;
- have exact provider-reported model identifier, client/product, mode, and date
  recorded;
- disclose memory, personal-context, file-library, connector, and web access;
- avoid any persisted project memory where the client permits it.

A product nickname is insufficient when an exact identifier is available. If
the provider does not expose an exact identifier, record the most precise label
available and mark the limitation.

Different temperature, prompt, or account settings on the same underlying model
family do not make a subject cold.

## 3. Blind packet construction

The packet is content-addressed and recorded before the run. It contains only
what the applicable task permits.

### Gate K cold-author packet

Allowed:

- `README.md`;
- the authoring-relevant portions of `KERNEL.md`;
- source schema/reference documentation;
- approved primitive catalog documentation;
- CLI help output;
- one valid example fixture;
- the room-change brief;
- an isolated writable copy of the fixture package.

Excluded:

- Rust or other kernel source;
- founding-review transcript or synthesis;
- issue and pull-request discussion;
- implementation notes not included in public author docs;
- previous cold-run transcripts;
- hidden hints from the operator.

The Gate K brief is fixed:

> Add a second approved `iron_barred_door` to the fixture at the supplied legal
> anchor. Give it a distinct stable symbolic ID, preserve the original three
> primitive kinds and catalog-value rules, and reach a clean validation and
> compile without changing kernel source or unrelated packages.

### Gate K cold-debug packet

Allowed:

- `README.md`;
- user-facing diagnostic and CLI documentation;
- the compiled immutable world package;
- the seeded failing replay/command log;
- run artifacts, diagnostics, state hashes, and causal receipts;
- the defect brief;
- an isolated output directory.

Excluded:

- kernel source;
- the mutation that seeded the defect;
- the expected answer;
- issue/PR history and founding review;
- previous debugger transcripts;
- operator hints.

The Gate K debugger must name the actual semantic cause and cite the package,
receipt, projection, or diagnostic evidence that distinguishes it from plausible
alternatives. If the defect is repairable through content or package input, it
must also produce and verify the repair. If repair requires kernel source, the
formal requirement is correct localization and repair class, not an impossible
source edit.

### Later Gate 3 and Gate 4 packets

Later gates replace the small fixture with the approved room brief, content
schema, catalog, examples, CLI, render contact sheets, replays, and forensic
artifacts. The same exclusions and evidence rules apply.

## 4. Tool policy

Default formal-run tools:

- read permitted packet files;
- write only inside the isolated task workspace;
- execute the published `nomos` CLI;
- inspect structured JSON and generated artifacts;
- use ordinary file operations such as `ls`, `cat`, and `diff`.

The packet root and all sandbox paths outside the single declared task output
subtree are read-only, with exactly one exception: the device path `/dev/null`.
In particular, packet runs expose no writable `/tmp` or home directory.

Reading `/dev/null`, writing it, or redirecting a command stream to it is
allowed. It supplies no project information, remains present in ordered-command
accounting, and cannot replace required transcript, diagnostic, or artifact
evidence. No alias, symlink, file-descriptor path, or other device receives this
exception.

A model-requested boundary probe or access to any other outside-workspace path
fails operational compliance even when denied. A subject-requested successful
undeclared access also fails operational compliance; if undeclared information
enters the task, independence integrity fails. An unrequested harness exposure
is a harness failure and cannot become passing subject evidence. Finalization
fails closed for each case.

Default forbidden tools:

- web search;
- repository history, issues, pull requests, and hidden branches;
- source-code search outside the packet;
- personal memory or project memory;
- communication with another model or human expert;
- arbitrary package installation;
- editing kernel source.

A gate may explicitly permit another tool, but the exception must be declared in
the run plan before launch and included in the final record.

## 5. Run constraints and resource accounting

The formal independence constraints are:

```text
fresh model sessions                 1
operator substantive hints           0
operator retries after model failure 0
```

Tokens, assistant turns, tool calls, and exact ordered commands are recorded
when the client exposes them. They have no protocol ceiling and do not terminate
an otherwise valid run. Resource use is evidence for owner review, not a proxy
for task merit. An owner may change a later run's predeclared model or effort
level if observed use is unreasonable; a run already in progress is not
retrofitted with a new resource limit.

A transport or client crash may be restarted once only if:

- the full prior transcript is supplied unchanged to the replacement session;
- the restart is recorded;
- no new hint is added;
- the result is marked `restarted`.

If the provider does not expose token usage, record it as unavailable.

## 6. Operator conduct

The operator may:

- relay exact tool output;
- correct a broken path or unavailable command caused by the harness;
- state that an attempted action violates a predeclared tool boundary;
- stop a run whose transport or isolation boundary fails.

The operator may not:

- explain a compiler error;
- suggest a file, primitive, capability, machine, or repair;
- paraphrase diagnostics to make them easier;
- reveal an expected output;
- ask leading questions;
- modify the subject's patch;
- grant a new tool after seeing the subject struggle.

Any substantive help changes the verdict to `assisted`. An assisted run can be
useful research; it cannot satisfy a cold gate.

## 7. Pass criteria

### Cold author

A pass requires all of the following:

- exact model eligibility is satisfied;
- packet and tool boundaries are respected;
- no substantive human intervention occurs;
- the second door uses an approved primitive and legal anchor;
- symbolic IDs remain distinct and typed references resolve;
- no kernel source or unrelated package changes;
- validation and compile both exit successfully;
- generated package and diagnostics contain the new door;
- token, turn, tool-call, and ordered-command accounting is preserved;
- the subject supplies a short explanation of what it changed and why the
  compiler accepted it;
- an independent checker reproduces the clean compile from the subject's output.

### Cold debugger

A pass requires all of the following:

- model eligibility, packet, tools, and intervention constraints are satisfied;
- the subject identifies the seeded semantic cause, not merely the symptom;
- cited artifacts support that diagnosis;
- at least two plausible alternatives are excluded by evidence or the causal
  chain is specific enough that no ambiguity remains;
- the proposed repair targets the owning fact, rule, package input, or subsystem
  boundary rather than masking the symptom;
- a content/package-level repair is verified when such repair is possible;
- an independent checker confirms the diagnosis against the hidden mutation.

### Cold design review

A design review has no pass/fail authority. It records findings against a blind
rubric covering:

- hidden coupling;
- unowned facts;
- ambiguous composition;
- impossible migration;
- state-machine cycles;
- authoring escape hatches;
- performance assumptions;
- unverifiable claims;
- places where taste is disguised as logic.

The owner dispositions each finding separately.

## 8. Verdicts

Every formal subject and checker receives three separately evidenced dimension
results:

- `semantic_merit` — whether every task-specific authoring, debugging, or
  checking result and explanation criterion is correct;
- `independence_integrity` — whether model eligibility and fresh-session
  requirements hold, only declared information entered the task, and no
  substantive outside help occurred;
- `operational_compliance` — whether every declared tool, path, execution,
  permitted-change, evidence-accounting, and record-completeness requirement
  was obeyed.

Each dimension is exactly `pass`, `fail`, or `inconclusive`. One passing
dimension never compensates for another failed or inconclusive dimension.

Each run also receives exactly one overall verdict:

- `pass` — all three dimension criteria met;
- `fail` — a declared task criterion was violated;
- `assisted` — substantive human or external help occurred;
- `inconclusive` — environment or harness failure prevented a fair result.

The overall verdict is derived in this order: substantive human or external
help produces `assisted`; otherwise any failed dimension produces `fail`;
otherwise any inconclusive dimension produces `inconclusive`; only three
passing dimensions produce `pass`. Those dimensions collectively cover every
declared run criterion.

Only overall `pass` satisfies a cold gate. It does not replace the applicable
task criteria or independent-checker requirement. Repeated failures may inform
design, but the best run cannot be cherry-picked without reporting all formal
attempts against the same brief.

Before a formal provider task launches, the operator appends a content-addressed
reservation to `docs/evaluation/gate-k-formal-attempt-ledger.jsonl` and commits
it. After all prelaunch checks, the operator appends and commits a distinct
`launch` event immediately before invoking the provider. Reservations and
launches are globally ordered and hash-chained. The launcher requires that exact
committed marker and binds the open reservation, candidate, packet manifest,
prompt, shape, provider, model, and thinking level. The complete recorded task directory is validated:
the canonical receipt and exact launcher must agree with the retained manifest,
transcript, commands, qualification, stderr, boundary, and artifact tree. The
plan's prompt digest must equal the reserved prompt digest. The launcher status
mechanically determines the receipt outcome, and its ledger
commit must be the repository HEAD that contains the launch marker. Only then
are the derived receipt hash and outcome appended as a closing event, including
an inconclusive transport. Close uses finalization's same semantic task-record
proof; a bare hash, skeletal launcher, or consistently rehashed invalid record
cannot close an attempt. A new reservation
is forbidden while an earlier one is open. This prelaunch record, rather than a
finished receipt's self-reported retry count, makes abandoned or discarded
formal transports visible.

The four historical `gate-k-rc1` imports are accepted only as their exact
canonical events. Gate K final assembly requires the exact frozen four-event
inventory, so a later reservation, close, cancellation, or import cannot be
omitted from the final disposition. Decision 0015 authorizes a future round-two
attempt only after revision-6 tooling and rehearsal pass and a new exact
`gate-k-rc2` candidate is frozen and mechanically proved.

If the operator cancels before the committed launch marker, the reservation is closed only
by an explicit `discarded-before-launch` event with a non-empty reason. It has no
task receipt and cannot be represented as a completed or inconclusive provider
run. Once the marker exists, cancellation is forbidden; a failed transport
remains open until an authenticated inconclusive receipt closes it.

## 9. Required evidence record

Store formal runs under:

```text
docs/evaluation/runs/<gate>/<date>-<model>-<task>/
```

Each record contains:

```text
RUN.md                 human-readable summary and verdict
plan.json              predeclared packet, tools, independence constraints, rubric
packet-manifest.json   file paths, schema versions, and hashes
prompt.txt             exact initial task prompt
transcript.*           complete model/operator exchange when exportable
commands.json          ordered CLI/tool invocations
artifacts/             subject outputs or content-addressed references
checker.json           independent reproduction/check result
```

The three public packet documents use exact allowlisted schemas:
`plan.json`, `packet-manifest.json`, and `task-receipt.json` are canonical
sorted compact JSON with strict scalar types. Every evaluation JSON parser
rejects duplicate keys and non-finite numbers, including finite syntax whose
magnitude overflows the host numeric representation, before a declared result
shape is inspected.

Pi event streams are parsed before provider signatures are removed. Only
`textSignature` on a text content block and `thinkingSignature` on a thinking
content block may be removed, and the raw-stream digest is retained. Boundary
schema `nomos.pi_cold_agent_boundary@3` records the resolved path and SHA-256 of
Pi, Bubblewrap, and any provider extension; the task receipt repeats the exact
runtime identity. The four frozen `gate-k-rc1` task records retain their legacy
`@2` boundaries and are not retroactively upgraded.

`RUN.md` records:

- repository commit and branch;
- package/fixture hashes;
- exact model identifier and family;
- provider, client, mode, and date;
- memory/context/tool disclosures;
- token, turn, tool-call, and ordered-command accounting;
- operator interventions, including none;
- restarts or environment failures;
- result against every rubric item;
- the three dimension results, their evidence, and the derived overall verdict;
- adjudicator and owner disposition.

Secrets, private credentials, and unrelated personal context are never stored.
Where a client cannot export a complete transcript, record that limitation and
preserve the most complete available evidence. A non-exportable transcript may
support research but weakens reproducibility and must be called out.

## 10. Non-author rerun

Cold-agent success does not replace the repository rule that nothing is green
until someone other than the change author reruns the proof.

The non-author checker receives the subject's committed output and exact commands,
rebuilds or recompiles in a clean environment, and records:

- commit;
- toolchain/environment;
- commands;
- resulting hashes;
- pass/fail;
- reviewer identity.

The cold subject and non-author checker may be the same only if that subject did
not author the evaluated implementation or content change before the formal run.

## 11. Protocol amendments

Changes to a protocol after a formal run begins do not apply retroactively. A
protocol revision records prior wording, replacement, reason, effect on existing
runs, owner disposition, and revision number under `docs/decisions/`.
