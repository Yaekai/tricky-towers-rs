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
3. If Max isn't using the PC, take screenshots of each mode started with
   `target\debug\tricky_towers.exe --play race` (and `survival`, `puzzle`). If windows get
   closed from outside, stop and wait for the next pull.
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
