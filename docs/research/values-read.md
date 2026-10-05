# Guessed values read from the install (2026-10-05, PC)

Answers for values marked `(?)` in the code. Units: the original's world units (1 brick
cell = 1 unit), its UI pixels on a 1920×1080 screen, seconds and degrees.

## Results: sunrays behind the trophy (`results.sunray_spin`)
Two ray layers on a 2 s loop, both turning at a steady speed (linear keys): the back layer
+90°/s (180° per loop), the front layer -45°/s (the other way). The front layer is also
scaled 1.2, holds 1 s, then eases to 1.1 by the end of the loop (and snaps back).
Clip `RotateSunrays` (TrophySunraysAnimatorController).

## Hearts under the wizard (`hud.heart_size`, `hud.heart_below`)
- The heart holder sits 1.53 units below the wizard's origin; the first row 0.3 below that
  (1.83 under the wizard).
- Rows of 3, 1.3 units apart, centred on the wizard; each next row 1.3 lower. The whole set
  is shifted 1.4 units left (the heart picture's offset inside its prefab).
- Heart picture `heart` is 29×29 px at 26 px/unit (1.12 units). A lost heart shows
  `gph_cross` in its place. On the first show the hearts pop in one by one, 0.3 s apart.
  Blink constants: 0.7 s, every 1.5 s (when it blinks wasn't read, ?).

## Spell clouds (`casts.cloud_size`, `cloud_icon`)
The icon's backing (`spell_slot_light` / `spell_slot_dark`, 139×140 px at 52 px/unit) is drawn
at 1.31×, so 3.50 units across. Wizard 1's light cloud holder sits at (-1.95, 2.45) from the
wizard and its cloud at (+2.53, +2.35) inside it; the dark one at (0.34, 1.18) + (2.53, 2.35).
The button hint under each cloud sits at (∓1.1, -1.25) from the cloud.

## Dark cast beams (`casts.beam_width`, `beam_reach`)
`cast_beam` (87×14 px at 26 px/unit) is drawn 1× wide and 60× tall: 3.35 units wide,
32.3 units tall, centred on the beam's origin. The two white edge strokes are 0.1 wide and
20× the white pixel tall, at ±0.88 from the centre. Beam02/03 (bigger curses) use their own
backgrounds (`fx_cast_effect02_background` 48 px wide, so 1.85 units).

## Light cast lightning (`casts.lightning`, `lightning_wobble`)
- Lasts 0.4 s (clip `Lightning`, its end event closes it).
- A mesh strip 3 units thick, one segment per unit of length, texture repeated every 50
  units (0.02 per unit).
- Wobble: each point moves sideways by sin(time + 5 × position) × amplitude × sin(π ×
  position), position 0..1 along the bolt, so the ends stay put. Amplitude goes 0 → 2 at
  0.167 s → 0 at 0.4 s.
- The sunray burst at the wizard and at the brick turns 120° over the 0.4 s.

## Race Pro lightning (background)
Every 5-14 s (random), never during the intro: a white screen flash at 0.65 alpha for 0.1 s,
a bolt drawn for 0.4 s then fading over 0.2 s,
ending at a random spot along the bottom (±5/48 of the screen width); after its third
segment it may branch (80% chance, at most 4 branches, the branch ending up to 20 units to the
side). The islands switch to their lit pictures and fade back over 0.2 s.

## Wizard levels (`xp.names`, `xp.master`)
- XP to reach each level: 0, 500, 2,500, 10,000, 25,000, 45,000, 75,000, 112,500, 162,500,
  225,000, 300,000; after that every 75,000 more is one more level.
- Levels 0-9 are named by `WIZARD_LEVEL_0` .. `WIZARD_LEVEL_9`; level 10 and up use
  `WIZARD_LEVEL_MAX` with `{subLevel}` = level - 9 (so level 10 is sub-level 1).

## Still open
Puzzle sand strip, menu slide times and layouts, cup screen, pack looks (Diamond gloss,
Hologram flicker, Galaxy drift), the Endless questions in pc-tasks.md.
