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
