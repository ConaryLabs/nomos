# Capture receipt

Captured on 2026-08-25 with:

```sh
experiments/executable-gaol/gaol capture
experiments/executable-gaol/gaol verify
```

- AreaCollection SHA-256: `8098dafb059ca436eec9f1fb2e02501205d9f8ceeadfdf344fc717bc9bd9ffd9`
- North Gaol RenderingPlan SHA-256: `82d39f0a1b41ac60fe7e7a1ed142f8409a17c58a05d8063dc0302c69fe1bbd16`
- Cistern Walk RenderingPlan SHA-256: `2f7b5a0e7efd53f933d7175eee529c73c7e3f43c756c43afd746be5daba1ba82`
- Ember Vault RenderingPlan SHA-256: `9f24a2a114d40fae7a20cd160e8748106a6990067f89f4ca31ab9631ffe53f12`
- Ossuary Reach RenderingPlan SHA-256: `36f9e2ed454884371704b777a691cccd7bbe402bdced39588d7596641fbc5dbb`
- cross-area contact-sheet.svg SHA-256: `d30ec3fb140c5ddad8a6aacaf3b07a348f8195a8540b900fc63b5cb350a76795`
- cross-area contact-sheet.png SHA-256: `af9c834a39e3045bca48d9209d27190978f80c7130de839ea70031de9a8b3eec`
- verification: `EXECUTABLE_GAOL_VERIFY PASS`

Two consecutive clean captures produced all seven exact hashes. Ossuary Reach
was added without changing either renderer; its insertion changed the
collection, Ember route metadata, and cross-area sheet while leaving the shared
look digest and the other plan bytes exact. The committed PNG and example plans
are review conveniences; the command regenerates them from the content,
subsystem projections, and real runtime states.
