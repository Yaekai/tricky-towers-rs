# What's next

The open work, most useful first. "Estimated" means the value is ours, not yet read from the
original; those are marked `(?)` in `src/`. The status table in the [README](../README.md) gives
the overview; `docs/pc-tasks.md` and `docs/research/compare-2026-10-06.md` keep the detailed
working notes.

## 1. Check the newest work against the original
Most recent features were checked in offscreen pictures only. Compare them in play with the
original and fix what differs:
- The brick, water, island and grab-pass shaders (lit bricks, Galaxy, Hologram, Diamond, zap
  dissolve, drop guide overlay, Puzzle heat haze, cast lanes, Endless perspective clouds).
- The cup standings panel and the cup winner popup (confetti, fireworks, the flying wizard).
- Select Players, Select Mode, Single Player, Settings and the pause screen.

## 2. Gameplay still missing or estimated
- **Puzzle**: the base rise speed and the half-cell rise per lost brick; the two-player
  layout compared frame by frame with the original.
- **Survival**: the moon drop (it falls only for bricks 2.5× or bigger, and no spell here
  makes them that big yet) and the curse-wave overlay colours.
- **Cups**: Big and Super Cup layouts and the slide and fill timings.
- **Ice**: the ice bricks' own collision sounds (`SFX_BRICK_PLACED_ICE_1..4`, played on every
  hit in the original).
- Done since the last list: Endless Puzzle obstacle blocks (checked, they match), Auto Rotate
  wind-up and its sounds, Petrify's stone look in hand and its grass and moss, the cup's stray
  empty medal and medal sounds, two towers touching the Puzzle laser at once (tested).
- About 40 values marked `(?)` in `src/` and the guesses in `assets/config/game.ron`
  (intro timings, cup panel timings, `moments` and `casts` values).

## 3. Look and effects
- The up beam's foot: its looping rays (the Sunrays clip), blobs, sine waves, droplets and
  vertical lines. The down beam's impact (rays, rings, glow) and the collapse explosion
  particles are in.
- Spring brick pack look; Easter decoration timings.
- Endless Race clouds: the cloud planes' texture direction.

## 4. Menus
- Small layout differences left from the screenshot comparison (spacing on Single Player,
  text weights).
- Title tiles: the wizard over the Local Battle frame and the earth's size need those parts
  staged as their own layers.
- Text from the original's 19 language files (the format is read in
  `docs/research/text-keys.md`; menus use built-in English today).

## 5. Before a release build
- Remove or switch off everything in [`release-checklist.md`](release-checklist.md)
  (`grep -rn DEBUG-TOOL src`), and decide whether AI seats in local battles stay (the original
  has none).

## Not planned
Online battle, the Shop and Help. XP and wizard levels exist as data only, since the original
gives XP only online.
