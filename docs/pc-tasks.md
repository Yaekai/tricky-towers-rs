# Standing tasks for the extract and test session on Max's PC

Max has asked this session to follow this file after every pull of PR #1 until the 1:1 art
work is done, without waiting to be asked. The cloud session edits this file to queue work.
Read-only on the Steam install, nothing from the game goes into the repo (see AGENTS.md),
and nothing gets installed without Max's own go-ahead.

## After every pull
1. Build and run `cargo test`.
2. Re-stage `extracted\` per `docs/art-layout.md`, filling in any newly documented names or
   fields from the original. Keep `extracted\inventory.md` (see the Inventory section)
   current.
3. Screenshots, offscreen only. Max uses the PC while this runs (2026-10-04 12:05Z: "you can
   run the test on my pc but i need to see and hear nothing"). So never start the game with a
   window, and never play sound. Use only
   `target\debug\tricky_towers.exe --play race --offscreen screenshots --shots 5,20`
   (and `survival`, `puzzle`, `endless`, plus `--difficulty pro` where a check needs it). It
   opens no window, plays no sound, saves `screenshots\<mode>_<t>s.png` and quits by itself.
   Don't steal focus or open the pictures on screen; read them yourself.
4. Report what changed, what is still missing against the original, and anything only Max
   can decide.

## Queued
- Write `extracted\inventory.md` and stage everything it lists.
- Convert the Survival and Puzzle backgrounds to screen layers, with their own `sky` and
  `water_band` (as `background_survival_*`, `background_puzzle_*`; a per-mode sky can be added
  if needed, say so).
- Stage `water_band` with the measured colours and height, and `font_main` if the original's
  font exists as a .ttf or .otf (or can be found as one).
- Stage `menu_background` and `logo` from the original's title screen.
- Stage `sky_survival` and `sky_puzzle` (and `water_band_<mode>`), and `mask` colours on the
  Puzzle ruins from your notes. Check the mask rule above against the original's shader and
  say in your report if it differs.
- In your next reports, send the cloud session the plain facts it needs to code each group in
  `inventory.md`, one group per report, starting with wizards: the logical names, what each is
  for, animation frames in order with times, and where and when the original shows them. Then
  the Puzzle floor rule (the 3 piece types), spell effects, match moments, menus and results.
- Per-mode skies, water bands and mask recolouring are in the game since 0febbb8: stage
  `sky_survival`, `sky_puzzle`, `water_band_survival: (colors: [])` (no water there) and `mask`
  on the ruins, then check the screenshots.
- Max picked the original letters for the font (2026-10-03 23:34): write `extracted\fonts\main.ron` with the
  letter picture's file, and for each character its rectangle in the picture, offset and
  advance, plus line height and base line (plain numbers), and stage the picture. The format
  is in `docs/art-layout.md` under "The original's letters"; point `font_main` at that file.
- Wizards are in the game (see "Wizards" in `docs/art-layout.md`): stage `wizard1` to
  `wizard4` as Wizard01–04 with their parts, part positions and pivots, each clip's part
  animations from the clips you listed, and `wizard<n>_say_<clip>` voice lines. Screenshot a
  match with 2 or more towers so the wizards show.
- Masks now read in both forms, and blue outlines take `outline` (transparent if left out).
- Difficulty levels are in (menu Tab, or `--difficulty easy|normal|pro`). Screenshot Puzzle on
  each difficulty to compare the floors with the original, and read the original's exact curse
  wave levels (which curses, how many at once, how long) and Race Pro wind, for the notes.
- Send the remaining inventory groups: Puzzle floor art (the 3 piece types and how they are
  laid out), spell effects, match moments, menus and results.
- Endless Race now follows your readout (checkpoints on spell lines, light spells only, the
  Endless islands as `background_endless_N`). Screenshot it offscreen. `pacer_layer` isn't drawn
  yet: say how the original places and sizes it.
- Race Pro wind is in as a guess (see `docs/research/spell-awards.md`). Read from the original
  what the wind pushes (the steered brick, placed bricks, or both), how its strength turns into
  a push, and whether it changes direction during a match. Also how the Puzzle steering limit
  grows with the tower.
- Player panels: when a base's art reaches below the bottom of the screen, the panel now sits
  just right of the base along the bottom edge instead of on top of it. Read from the original
  where it puts each player's panel (portrait, name, hearts, spell slots) in 1 to 4 player
  matches, as screen positions or relative to the lane, so it can match exactly.
