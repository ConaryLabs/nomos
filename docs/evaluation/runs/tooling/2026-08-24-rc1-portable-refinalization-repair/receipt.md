# Frozen rc1 portable re-finalization repair

Draft PR #89 first ran CI at head
`6afcf5c304ae2de15bfba0904c7a3317ca4bcf53`. The complete Gate K evidence
matrix passed, but verify run `32675315638` failed in
`docs/evaluation/test-gate-k-rc1-refinalization.sh`.

The finalizer re-authenticated the frozen Gemini qualification by trying to
read its historical absolute provider-extension path on the current machine:

```text
Gemini provider extension cannot be authenticated: [Errno 2] No such file or
directory: '/work/signed-dev/.local/lib/node_modules/pi-antigravity/src/index.ts'
```

That path exists on the original evidence host but correctly does not exist on
the GitHub runner. The historical qualification, task receipt, provider package
identity, path suffix, and expected provider digest are already bound by one of
the four explicitly allowlisted frozen task-receipt hashes. Requiring the
original host path to remain present made byte-preservation verification
machine-local without adding authentication.

The prospective repair skips only that current-host file read when the supplied
task receipt hashes to one of the four frozen legacy records. It retains the
exact provider package, install recipe, absolute path suffix, receipt digest,
qualification digest, and pinned provider SHA checks. Current or constructed
records still authenticate the provider entry point from the live filesystem.
The existing strictness regression proves a constructed receipt cannot claim
the frozen compatibility path, and the rc1 re-finalization suite exercises all
four exact frozen receipts.

This changes no frozen byte, verdict, semantic input, kernel source, or current
revision-6 record rule.
