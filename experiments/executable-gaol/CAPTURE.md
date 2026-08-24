# Capture receipt

Captured on 2026-08-24 with:

```sh
experiments/executable-gaol/gaol capture
experiments/executable-gaol/gaol verify
```

- AreaCollection SHA-256: `09fed4ea297406719fb17c3bce8129bc0258eb763c2e93811a78a5efe53a2d34`
- North Gaol RenderingPlan SHA-256: `82d39f0a1b41ac60fe7e7a1ed142f8409a17c58a05d8063dc0302c69fe1bbd16`
- Cistern Walk RenderingPlan SHA-256: `2f7b5a0e7efd53f933d7175eee529c73c7e3f43c756c43afd746be5daba1ba82`
- Ember Vault RenderingPlan SHA-256: `e87288b9d183b4216974c51e6a0f5d965be7bf39a7e9229eb9b1c040d0a6f192`
- cross-area contact-sheet.svg SHA-256: `04b706f067db4dd2464f557246b868cb53161925aacae6be270b81f60b2341e5`
- cross-area contact-sheet.png SHA-256: `26273aba009f5228e63d2c0b8100857b0aae8bc15d493894fdb87de9861a5175`
- verification: `EXECUTABLE_GAOL_VERIFY PASS`

Two consecutive clean captures produced all six exact hashes. The plan hashes
changed when the bounded `exit_via` objective became part of each plan; the
collection and contact-sheet bytes remained exact. The committed PNG and
example plans are review conveniences; the command regenerates them from the
content, subsystem projections, and real runtime states.
