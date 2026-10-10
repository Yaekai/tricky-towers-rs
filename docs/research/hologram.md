# Hologram pack (measured, 2026-10-10)

- At rest the bricks are the base skin, a little see-through. Max's frame (his skin screenshots) and our build now agree: median brightness 137 against 137, median saturation 112 against 107 (brick pixels, sky left out).
- Before, a cyan tint at rest (0.8, 1, 1) made ours darker and greyer (126 and 85). The cyan now shows only during a glitch.
- The shader is Direct3D 9 only and can't be read (see premium-export.md), so the glitch's look stays as measured from clips.

## The real shader, disassembled (2026-10-10)
Read from the original's Direct3D 9 programs (unpacked and disassembled locally; nothing of it
is in the repo), the normal variant:
- At rest the colour is the base pattern, solid (alpha is the bevel map's, times the sprite's,
  which is 1). There is no landing dip: the only alpha change is the glitch.
- Scan lines: the colour blends towards (0.8, 1, 1) by k x frac(3 y + t), with y the world
  height in cells (three bands a cell, moving down a third of a cell a second) and
  k = 0.2 + 0.025 (sin 50 t + 1), a fast flicker between 0.2 and 0.25.
- Glitch (HologramBrickModifier): distortion D from 0 to 1 over a random 0.1 to 0.5 s and back
  over the same time, both with DOTween's default OutQuad; the first right at spawn, then every
  6 to 15 s. While D is up: alpha 1 - D x h, with h a hash that changes ten times a second;
  rows (10 a cell height of the texture) shift sideways by up to a tenth of the texture; some
  bands (30 a height) brighten by D/6; the outline turns from black to D x (0.8, 1, 1).
- In our build now: rest solid, scan lines per cell (`HoloScan`, a tiled ramp), the flicker
  alpha and the OutQuad glitch (`glitch_amount`). The landing see-through curve and the cyan
  body tint are gone (both were guesses). Not done: the bright bands and the cyan outline in a
  glitch; the tear is per cell, not per texture row.
- Our blending is in linear light and the original's in gamma space, so the scan lines' alpha
  is scaled by 0.35 (`HOLO_SCAN_LINEAR`), set by eye and by measuring against Max's frame
  (brightness about 140 against 137; saturation about 85 against 107, noisy between runs).
