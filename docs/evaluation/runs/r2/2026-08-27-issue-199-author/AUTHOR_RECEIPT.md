# R2 final-proof implementation-author receipt

Status: final-proof source authored for issue #199; the exact candidate and
author/non-author executions are bound by the generated external proof receipts,
not recursively by this source receipt.

## Authority and baseline

- Issue: #199, `R2 final evidence and owner disposition`
- Baseline commit: `6cbce64cb867aef24faf227e62bdfc585bbcbd5d`
- Baseline tree: `6dada35f44e178f0d6cafc5ac2b5c94ab3fd0522`
- Contract: owner-authorized `R2.md` revision 2
- Contract SHA-256:
  `770740bad1c85cf7ea9dcd16f8c25e01766064d3b59d7f0bb9d438c289a6e638`
- Revision-2 authority: decision 0024, SHA-256
  `0356b3918a5c2643c36e16555e8ef78155bf893a8c3c21e4f75263f8289feea0`
- Unchanged R1 contract: `RUNTIME.md` revision 4, SHA-256
  `dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593`
- Issue-body SHA-256:
  `8ffd30e7a213e991732ea6031743542eb68d9b80fe6d4989ed58052617352dcc`
  over `gh api repos/ConaryLabs/nomos/issues/199 --jq .body`, including the
  command's final LF
- Author: Codex primary agent and its bounded GPT-5 family implementation
  subagents

## Consulted inputs

The implementation used the repository authority named by `AGENTS.md`, the
exact issue body, the accepted R1 and authorized R2 contracts and decisions,
the existing R1/R2 proof scripts, the committed R2 fixtures and evidence, and
the owner authorization conversation. The implementation did not consult or
copy an adopter repository, payload, frame, palette, asset, prose, schema,
coordinate set, mechanic, or governance document.

## Scope and method

The final slice adds a decomposed orchestration harness, an independent receipt
assembler/verifier, their plant tests, source-provenance routing, operational
CI wiring, and an evidence-only extension to the R2 browser smoke receipt. The
extension retains the already-checked per-launch browser facts so the final
verifier can audit them independently; it changes no rendering or visual
behaviour. The slice does not edit an R1 crate/viewer/contract, an R2
compiler/decoder/catalog/renderer/UI, either scene or expected plan, the
second-author packet, or the committed browser evidence.

The R1 viewer's unchanged tests require `dist/` beside their source. To satisfy
that accepted path assumption without writing the committed application tree,
the harness copies and digest-verifies the exact tracked viewer files plus the
required generated runtime inputs beneath its supplied output, stages the
distribution in that output-local mirror, and requires all 104 unchanged tests
to pass with zero skips. This is proof topology only; it changes no accepted R1
byte or meaning.

The harness validates and digest-records host tools before proving an outer
network connection, creating a fresh network namespace, and proving that the
same connection is blocked inside it. It enables loopback only, drops
privileges back to the invoking user, and runs the proof beneath a read-only
root/filesystem view whose only persistent writable roots are the supplied
output and checkout-local `target/`. Its independent verifier recomputes the
recorded counts, hashes, timing arithmetic, ceilings, distribution trees,
mount controls, and process/network closure before it emits a PASS receipt.
Generated evidence is kept external to the candidate and binds the exact
candidate commit/tree; committing a generated run would move the candidate and
require a new run.

Commands used during implementation include repository reads, `apply_patch`,
shell and Node syntax/tests, the four accepted workspace checks, output-local
R1/R2 rehearsals, and the final standalone network-isolated proof. Development
failures are not evidence. The PR and external author/non-author receipts bind
the exact final green commands, environment, outputs, and candidate identity.
