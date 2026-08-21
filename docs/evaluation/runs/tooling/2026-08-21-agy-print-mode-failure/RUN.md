# `agy` Gemini print-mode failure

**Verdict:** blocked; zero evidentiary value

**Date:** 2026-08-21

**Issue:** #17

## Environment and route

- Client: `agy` 1.1.17.
- Requested model: `gemini-3.7-flash-high`.
- Host: Linux x86_64, Fedora 44.
- Operator: Mira.
- Repository: `/home/peter/signed-world`.

## Failure

Three print-mode invocations ignored distinct prompts and returned only a
canned Gemini model greeting with exit 0. The attempts included a full
exact-head review in plan mode, the same review without plan mode, and a minimal
`pwd` preflight. No repository inspection or command execution occurred.

The attempts are inconclusive and receive zero review credit. Gemini 3.7 Flash
High may not perform the predeclared formal cold-author role until issue #17
proves a working invocation and adds a cheap command-execution preflight.

## Acceptance for repair

- identify whether the wrapper, agent configuration, or service discards the
  positional prompt;
- document the working invocation and client version;
- make a `pwd` preflight mandatory before a formal attempt; and
- rerun that preflight successfully with Gemini 3.7 Flash High.
