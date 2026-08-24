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

The next CI attempt at head `75a08b513d2bab94eefa7b93df1d905b5907dcd2`
passed the provider-extension check and exposed the same machine-local
assumption for the historical `/usr/bin/bwrap` path. Verify run `32677239613`
failed because the GitHub image does not install Bubblewrap. The frozen
qualification already binds the exact `/usr/bin/bwrap` path, Bubblewrap version,
SHA-256, and task-receipt digest. The compatibility rule now skips that live
file read for the same four exact receipts while continuing to require all
recorded identities. Current and constructed records still authenticate the
live Bubblewrap binary.

After the complete packet snapshots and revision-6 reconstruction test were
added, CI at head `74a8f0cccbc9349e96201df86928832b2fee8b32` exposed the
corresponding assumption in the archived revision-6 Gemini qualification.
Verify run `32681551678` passed rc1 re-finalization, then failed because the
original `pi-antigravity` entry point was absent from the GitHub runner.

The compatibility set is now named explicitly as the four frozen legacy task
receipts plus the four accepted revision-6 rehearsal task receipts. For only
those eight exact hashes, qualification validation trusts the recorded absolute
path suffixes and already-bound executable digests without reopening the
original host paths. Live or constructed receipts still authenticate the
current Pi executable, provider extension, boundary extension, and Bubblewrap
bytes. A regression rewrites every original-host path in an archived
qualification to a known-absent prefix and proves that exact receipt passes,
then proves the same qualification with a non-archived receipt fails.

This third portability repair changes no archived byte, task result, verdict,
semantic input, formal-attempt state, or live-record rule.
