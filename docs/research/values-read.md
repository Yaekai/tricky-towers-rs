# Guessed values read from the install (2026-10-05, PC)

Answers for values marked `(?)` in the code. Units: the original's world units (1 brick
cell = 1 unit), its UI pixels on a 1920×1080 screen, seconds and degrees.

## Results: sunrays behind WINNER! (`results.sunray_spin`)
Two `gph_castEffectSunrays` layers behind the WINNER! text, turned by a script at a steady
60°/s and -90°/s (WinMessage prefab).

## Cup winner: sunrays behind the trophy
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
20× the white pixel tall, at ±0.88 from the centre.
The beam is picked by the caster's wizard, not by the curse: wizard N casts Beam0N / BeamDown0N
(wizards 1-4), the skin wizards their own Animal / Indie beams (stars and shapes, not read yet).
Each has its own column picture (`cast_beam`, `cast_beam_2..4`) and colours:
- Wizard 1: white edge strokes, foot rays and rings pale cyan (0.78, 1, 0.98), glow at 0.153.
- Wizard 2: black edge strokes, magenta rays (0.914, 0, 0.431), glow pale cyan at 0.153. This is
  the dark magenta column in the video.
- Wizard 3: white strokes at 0.516, white rays, glow pale cyan at 0.153.
- Wizard 4: white strokes, pink rays, rings and glow (1, 0.78, 0.84) at 0.498.
The strokes, rays, rings and blobs aren't drawn yet; the column and the foot glow are.

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

## Puzzle sand strip (`weather.sand_strip`)
Two strips, both stretched to the screen width and fixed to the screen (the original's
background camera, 41.54 units tall):
- Back: `bg_puzzle_pink_sand` (our `puzzle_sand`, 46 px tall at 26 px/unit) drawn 2× tall, so
  3.54 units (0.085 of the view height), its bottom edge 1.12 units (0.027) above the screen
  bottom, at 0.803 alpha, behind the ruins.
- Front: `bg_puzzle_foreground_sand` (48 px) drawn 2× tall, 3.69 units (0.089), on the screen
  bottom, in front of the ruins (the front landscape layer), fully opaque.
- Endless Puzzle adds a third (`BackgroundEndlessSand`) behind them at parallax 0.1.

## Menu slides
- Settings pages and Trials tier pages: 0.3 s per page, 1920 px apart, DOTween's default ease
  (OutQuad).
- Trial buttons fill down the left column first: 5 per column, 125 px apart, columns 440 px
  apart; so 1-5 on the left, 6-10 on the right.
- Tier unlock: the locked panel leaves 1000 px upward over 0.5 s after 0.2 s; the buttons
  rise from 1000 px below over 0.5 s, each 0.05 s after the one before; chain sounds 0.2 s
  apart; the lock's centre grows to 1.3× (OutBack) before it goes.

## Brick pack looks
- Diamond: a gloss sweep at random 6-15 s gaps per brick (the first right away): the gloss
  rises to 0.7 in 0.15 s, holds, and falls to 0 in 0.15 s (0.6 s in all), while the gloss
  angle turns a full circle (either way, random) over the same 0.6 s, InOutQuad.
- Hologram: a glitch per brick at random 6-15 s gaps (the first right away): the distortion
  goes 0 → 1 → 0, each half taking the same random 0.1-0.5 s.
- Galaxy: one shared starfield texture behind every Galaxy brick, scrolling (0.05, -0.07) of
  the texture a second; every 2-6 s one random star twirls: grows to 2× in 0.3 s, turns 180°
  over 1 s (InBack), then after 2 s shrinks back to 1× in 0.3 s. Texture scale not read.
- Halloween: 40% of bricks get decorations (at most 2, appearing 0-0.3 s apart). Every 5-15 s
  each decoration either squashes (one axis to 0.8× in 0.1 s then back over 1 s OutElastic,
  the other axis 0.1 s later) or wobbles (25° one way over 0.926 s InBack, back over 0.45 s
  OutElastic). The bricks also go black and white over 1 s after 1 s (InOutQuad).
- Christmas: 25% of bricks get snow. Spring (Easter) has its own modifier, not read.

## Endless Survival curse waves (the pc-tasks.md Endless questions)
- The gap counts from a wave's **end**: when a wave finishes, the next starts after a whole
  number of seconds from 15 to 28 (random, seeded per run).
- Waves per level: **2**. Each wave is drawn from its level without repeats; after 2 waves
  (or when the level runs out) the next level starts, and level 4 repeats for good,
  refilled each time.
- Every wave, counted or timed, starts the same way: brick control pauses, the brick in hand
  shakes and moves to the top, 1 s wait, the shake stops, control resumes. The first brick is
  cursed at once; then each new brick until the wave's count is reached.
- Only Mist hits just the wave's first brick; Large, No Rotate, Fast,
  Ice, Mystery and Ghost hit every brick of the wave. A Mist cloud emits for 10 s, its puffs
  live 5 s, so it clears about 15 s after it appears (fading over the last 1 s); a new Mist
  replaces any that is still there.

## Endless Race and Endless Survival spells and speed
- Spell lines hand out spells by a random pick from the mode's list, never the same as the
  last one picked: Endless Race Ivy, Undo, Ivy, Undo, Petrify (so Ivy and Undo twice as
  likely as Petrify); Endless Survival Ivy, Minibase, Petrify.
- Drop speed starts at 4 in both; Endless Race caps it at 15.
- Endless Race uses `FLOOR_RACE_ENDLESS` at 12.5, Endless Survival `FLOOR_LWS` at 12.5; both
  pick random bricks. Other rules come from the game's general settings, not from a
  difficulty.

## Still open
Cup screen layout and wording, Galaxy texture scale, the Spring pack look.
