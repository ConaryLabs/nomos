# Revision-6 exact `/dev/null` mount repair

- attempted tooling commit: `e5ecfdcfda97848bedbd621e5b3dda1b73a1aea1`
- repair commit: `e2e0cfb`
- environment: host `remi`, Linux x86_64, Bubblewrap 0.11.2
- classification: non-formal pre-provider rehearsal infrastructure failure
- formal attempt reserved or launched: no

The first live revision-6 author rehearsal stopped before provider execution.
The packet boundary emitted no passing record because Bubblewrap reported:

```text
bwrap: Can't create file at /dev/null: Read-only file system
```

The original packet mount tried to bind `/dev/null` over the host root's
read-only `/dev`. The repair creates a fresh empty tmpfs at `/dev`, binds the
real `/dev/null` as its only child mount, and remounts the directory read-only.
The child device remains readable and writable while new directory entries are
denied.

The repair was reproduced directly with Bubblewrap by proving all of these in
one isolated process:

- `/dev` contains exactly `/dev/null`;
- `/dev/null` is a character device;
- reads and writes succeed;
- `/dev/zero` is absent;
- creating `/dev/other` fails.

The complete offline Pi boundary and evaluation-tooling harnesses passed after
the repair. Fresh live packets and sessions are required; the blocked packet is
not a task record and has no evaluation value.
