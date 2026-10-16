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
- **Puzzle**: compared with Max's clip at 15 s and 40 s; the laser, the foreground sand and the
  base rise now match.
- **Survival**: the moon drop (it falls only for bricks 2.5× or bigger, and no spell here
  makes them that big yet).
- Done since the last list: Endless Puzzle obstacle blocks (checked, they match), Auto Rotate
  wind-up and its sounds, Petrify's stone look in hand and its grass and moss, the cup's stray
  empty medal and medal sounds, two towers touching the Puzzle laser at once (tested).
- About 40 values marked `(?)` in `src/` and the guesses in `assets/config/game.ron`
  (intro timings, cup panel timings, `moments` and `casts` values).

## 3. Look and effects
- Character spacing (`letters::LetterSpacing`) is set on the title names, match text, menu
  hints, intro and Survival count; other texts the prefabs space (-1, -2 on SURVIVAL/SPECIAL,
  -4 on scores) still draw at 0.

## 4. Menus
- Text weights left from the screenshot comparison. Single Player and the title match Max's
  clip.
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
