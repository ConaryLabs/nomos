# Gate 0 gaol visual target

Status: **owner-disposed: visual thesis compelling**

Issue: [#83](https://github.com/ConaryLabs/nomos/issues/83)

Authority: [decision 0014](../../docs/decisions/0014-quarantined-gaol-visual-target-experiment.md)

This is a static, quarantined visual study. It is not a renderer plan, an asset
pipeline, a rendering projection, a semantic surface, or Gate K evidence. Gate
K remains failed under decision 0013.

## Question

> Is there a visual game in the Gate K gaol that Peter actually wants to look
> at?

The normal gameplay frame is the primary acceptance surface. The hero frame is
supporting material only.

## Declared camera and frame

- fixed oblique three-quarter view, approximately 35 degrees downward;
- restrained perspective rather than a strongly converging cinematic lens;
- 16:9 landscape frame at 1672 by 941 pixels for this study;
- stable arena scale with actors occupying a small gameplay-readable fraction
  of the frame;
- `north_gate`, `flooded_section`, and `brazier_02` legible together.

The images propose this camera visually. They do not define executable camera
parameters or authorize a renderer.

## Visual invariants

### Scene

- `north_gate` is the upper-center iron-barred door inside a broad beveled
  stone arch. A dim pale-cyan geometric ward and lock communicate its blocked
  state without labels.
- `flooded_section` crosses the middle of the room as shallow dark water with a
  visibly bounded dry route. Reflections use a few broad bands rather than
  high-frequency noise.
- `brazier_02` sits on the dry platform to the gate's right. Its effective
  state creates one bounded amber pool; its extinguished state removes that
  pool without making navigation unreadable.
- Architecture uses broad extruded masonry, chunky bevel response, stable
  world scale, sparse trim, and low internal-detail density.

### Actors and interaction

- The player has a narrow, agile silhouette and dark desaturated teal family.
- The gaoler has a broad, shield-led silhouette and muted rust/ochre family.
- Shape, value, and hue all contribute to separation. A bright outline is not
  required.
- Contact uses a tiny amber spark. The restrained spell uses the ward's pale
  cyan family, occupies a small part of the frame, and never erases an actor or
  landmark.

### Rendering language

- polished stylized 3D game image rather than painterly concept art;
- deliberately low internal-detail frequency with crisp enlargement;
- low-poly massing, broad bevel highlights, tiny-atlas or vertex-color feel;
- quantized palette ramps and subtle controlled shadow dithering;
- charcoal blue-gray stone, oxidized iron, cold blue-gray water, bounded amber
  flame, teal player, rust enemy, and pale-cyan semantic accents;
- environment contrast stays below actors and immediate interaction state.

### Interface

- UI occupies edges or immediate semantic anchors, not the combat lane;
- bars and icons remain readable at low resolution;
- gate, water-cost, enemy-state, and ability cues use the established bounded
  palette and geometric language;
- no minimap, floating damage values, quest paragraph, opaque center panel, or
  ornamental frame is part of this target.

## Allowed variation

- exact masonry block arrangement and sparse wear marks;
- actor pose within the declared silhouette and color families;
- water ripple placement within the broad-reflection rule;
- small camera translation for the supporting hero frame only;
- timing and curvature inside the restrained spell envelope;
- final icon geometry and UI spacing after legibility testing;
- precise palette values, provided the bounded family and value hierarchy are
  preserved.

These are visual targets, not shipping meshes, textures, animation frames, UI
widgets, or executable specifications.

## Frame intent

| File | Intent |
| --- | --- |
| `gameplay-camera.png` | Primary ordinary-play frame: all landmarks, both actors, spell, and baseline HUD at once. |
| `hero-environment.png` | Strongest environmental statement of the same playable room, with HUD and spell removed. |
| `actor-silhouettes.png` | Player alone, enemy alone, and an overlap check against stone and water at the gameplay camera. |
| `combat-overlap.png` | Compact melee contact at the flooded boundary with the gate state still readable. |
| `spell-effect.png` | Screen-area, luminance, reflection, and silhouette test for one pale-cyan crescent spell. |
| `low-light.png` | Side-by-side lit and extinguished `brazier_02` states with no spell-light confound. |
| `ui-overlay.png` | Representative gate, enemy, water-cost, vitals, and ability pressure over normal play. |
| `materials-palette.png` | Bounded palette ramps plus stone, iron, water, flame, actor, ward, spell, and UI studies. |
| `motion-timing.png` | Static key-pose strip for idle, locomotion, water entry, anticipation, contact, recovery, and spell fade. |

## Provenance and prompt record

All accepted bitmaps were produced on 2026-08-23 with OpenAI's built-in
`image_gen` tool. No external visual reference, shipping asset, repository
source image, paintover, or human pixel edit was used. `gameplay-camera.png` was
generated from the written brief. Every other bitmap used that accepted frame
as its visual reference.

The normalized final prompt set was:

1. **Gameplay camera:** fixed oblique ordinary game view of the compact gaol;
   upper-center barred and warded gate, middle flooded route, right-side
   brazier, small teal player, broad rust gaoler, restrained cyan spell,
   minimal edge HUD; stylized low-poly 3D, bounded palette, sparse texture,
   quantized ramps, controlled dithering; no cinematic framing, bloom, clutter,
   text, logo, or watermark.
2. **Hero environment:** preserve the gameplay frame's room identity, topology,
   scale, palette, and materials; make the gate and water reflection the
   environmental focus; remove HUD and spell; retain actors only as scale cues.
3. **Actor silhouettes:** preserve actor identity and actual camera grammar;
   show teal player alone, rust gaoler alone, and one-third silhouette overlap
   against the same gate, water, and brazier language; no spell or HUD.
4. **Combat overlap:** preserve room and camera; place both actors at the flooded
   edge near the gate; compact horizontal attack into shield; one tiny amber
   contact spark; no spell, HUD, gore, or oversized effect.
5. **Spell effect:** preserve room, actors, camera, and scale; add one narrow
   pale-cyan crescent, three sparse geometric motes, and a broken water
   reflection; effect below flame-core luminance and under eight percent of the
   frame; no broad glow or obstruction.
6. **Low light:** repeat the same view side by side with only brazier effective
   state changing; lit amber pool versus extinguished basket and cool ambient
   readability. A targeted second edit removed the generator-carried spell
   from both halves and changed nothing else.
7. **UI overlay:** preserve the accepted gameplay image; retain compact vitals
   and abilities; add small gate-progress, enemy-state, water-cost, and lock
   cues; icons and bars only; under ten percent screen area; no minimap,
   paragraphs, center prompt, or opaque panels.
8. **Materials and palette:** derive only from the accepted frame; grouped
   bounded ramps and seven unlabelled studies for arch stone, iron gate, water,
   brazier, player, enemy, and cyan ward/spell/UI, plus one gaol inset; no new
   families or external imagery.
9. **Motion timing:** preserve actor/effect identity; one room anchor strip and
   discrete unlabelled rows for idle/locomotion/water entry, long anticipation
   through compact contact and recovery, and restrained spell gather/travel/
   contact/fade; timing ticks only, no motion blur or text.

Exact accepted output hashes and generation/edit relationships are recorded in
`manifest.json`.

## Known risks

- Image generation introduces small shape and costume differences between
  sheets. The invariant is the silhouette, palette family, massing, camera
  grammar, and scene identity—not pixel-identical production assets.
- The images demonstrate desired readability but do not prove it under motion,
  input latency, arbitrary encounters, accessibility settings, or different
  displays.
- The apparent water cost, gate lock, ward, and effective-light state are
  visual hypotheses. No new kernel semantics are asserted.
- The palette sheet is directional. Exact numerical colors require later human
  art direction if work is ever authorized beyond this experiment.
- Generated key poses are a timing target, not an animation source or proof of
  interpolation quality.

## Owner disposition

Peter Permenter reviewed the complete pack on 2026-08-23, including the normal
gameplay-camera frame as the primary surface, and recorded:

> **visual thesis compelling**

The pack is preserved as the desired visual target. Decision 0014 still
requires a fresh, prospectively governed Gate K attempt before renderer
architecture or project adoption. This verdict does not amend the failed Gate K
record and does not authorize implementation.
