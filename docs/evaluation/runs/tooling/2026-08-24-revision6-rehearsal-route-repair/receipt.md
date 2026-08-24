# Revision-6 rehearsal route repair

- attempted tooling commit: `1a1b52302f020fd210fee5b0974f4c0f4ad620e2`
- classification: non-formal rehearsal infrastructure failure
- formal attempt reserved or launched: no
- operator intervention: none
- operator retries: zero

Fresh Gemini-author/DeepSeek-checker and DeepSeek-debugger/Gemini-checker pairs
completed their tasks. Both exact checker results satisfied revision 6 and
reported pass. Finalization then stopped because the implementation still
allowed only the supplemental Claude/Claude rehearsal pair, despite the
owner-authorized roster already declaring the cross-family Gemini and DeepSeek
routes for evaluation.

The non-final task identities were:

| Record | SHA-256 |
| --- | --- |
| author subject task receipt | `5b6ce14f3d95841619415f6438242f5e2440095da2b4a2e383019ccd6f9a4509` |
| author checker task receipt | `fd206846259894031314c0238cb888b7fd1492dcbef17b63d47bdc7a17eb45ac` |
| author checker result | `e959092a50a3d3a850c9ebb69b62978650ce7e870ff0fc0781b1f25c1592a3d0` |
| debug subject task receipt | `18759e2d7163020a7747be7884be73b0d5975a9dddb0bb0fe7f791c26642c9c9` |
| debug checker task receipt | `34f3e3cca2feb114aea442135306576ba9f5c62d2068b2ac3a457f2985cfeb17` |
| debug checker result | `ba168d87a5c184ce70b17d0e9fe379a931b9af8c9d9244fe620061d8c7825588` |

These records are not admitted as passing rehearsal evidence because their
candidate commit precedes the finalizer repair. The repair uses an exact pair
allowlist: Claude/Claude remains supplemental; author rehearsals may use
Gemini/DeepSeek; debug rehearsals may use DeepSeek/Gemini. Other pairings fail
closed. Fresh packets and sessions at the repaired tooling head are required.
