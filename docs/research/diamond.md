# Diamond pack (read from the original's shader, written in our words, 2026-10-10)

Read from the original's Direct3D 9 programs (unpacked and disassembled locally; nothing of it
is in the repo), the normal variant (no ice, no water). Material values from BrickDiamondT.mat
(the other shapes have their own materials).

- Colour: the brick's own pattern texture.
- Bands, all diagonal in the brick's own texture space (along x + y, in units of the bevel map's
  pixels at 65 to the unit), all moving:
  - wide bands of the first gloss: a 1.5-unit period, 40% of it lit, moving 0.2 units a second;
  - two sets of thin bands of the second gloss, both with a 2.4-unit period: a twelfth lit,
    moving 0.3 a second, and an eighth lit, moving 0.4 a second.
  Where lit, the colour is overlay-blended (the Photoshop "overlay" mode: darks multiplied by
  twice the colour, lights screened) with that gloss colour, then mixed in by its alpha.
  T: GlossColor1 (0.956, 0.893, 0.759) at 0.766, GlossColor2 (0.585, 1, 0.522) at 0.603. All
  seven shapes' colours are in `DIAMOND_GLOSS` (src/packs.rs).
- Gloss sweep: white, on the bevel faces that face the direction `_Angle`, by the slope times
  `_Gloss`. The modifier animates `_Gloss` (0 to 0.7 and back) and turns `_Angle` a full circle
  over 0.6 s (docs/research/values-read.md).
- Border: the border colour by 1 - (1 - blue) x alpha of the bevel map, as in the other packs.
  T: (0.003, 0.103, 0.078).
- A flash value lifts the colour towards white.

In our build (2026-10-10): Diamond bricks are one quad per piece drawn by our own material
(src/brick_material.rs) from these notes, with the sweep timing from DiamondBrickModifier
(`diamond_sweep`). Checked offscreen: the diagonal gloss bands show across each brick as in
Max's frame.
