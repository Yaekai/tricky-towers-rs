# Handover for PC Hugo (2026-10-08)

Written at the end of the session that merged PRs #105 to #117. Read AGENTS.md, then
docs/pc-tasks.md (its "Batch" sections list what was checked and what is open).

## Merged this session
- #105 Survival: curse waves recolour the sky; a Large brick knocks a star loose.
- #106 Hologram glitch lasts in-plus-out and rises and falls.
- #107 Cup rules checked against CupMatchFlowController (no change).
- #108 Losing pad rumble at full strength (2 s, big motor).
- #109 To-do notes for batch 6.
- #110 Survival reflected band is the original's overlay sprite (stretched, tinted, curse colour).
- #111 Notes for what batch 7 could not settle.
- #112 The moon falls when a brick of scale 2.5 lands (two Large curses stack to 2.5).
- #113 Notes: Max's three screenshots checked (Race clouds, Hologram skin).
- #114 Ivy group rule (now reverted by #116; the group-size rule was not in the original).
- #115 Bricks draw in placement order, so later bricks sit in front.
- #116 Ivy follows the original: a cluster is static only when a brick is solid; contact margin halved.
- #117 Wiki and original notes on Ivy.

## Open, and what Hugo should check first on the PC
1. **Ivy slide (most important).** A vined block still slides in unsteered tests. Ruled out: the
   mass rule (the cluster already takes the sum of member masses), the static rule (now as the
   original), the contact margin (now as the original), brick friction (0.4, as the original).
   Not yet explained. Hugo: cast Ivy on four bricks in a row and watch whether the block moves.
2. **Ivy crash (open, found this session).** Removing an old Ivy cluster body while it still has
   contacts trips avian's assertion `island.contact_count == 0` (avian2d 0.7, islands/mod.rs:1372)
   in `grow_ivy` / `loosen_ivy`. Reproduce with a headless sweep: steer each brick to x = 0, cast
   Ivy four times with seeds 7777 to 56000. Debug builds panic; the fix attempts (retire the body
   at once, or after two steps) still panicked, so the change was dropped. Leaving the old bodies
   alive avoids the panic, but leaves empty bodies behind.
3. **Hologram alpha.** Ours at rest is (0.8, 1.0, 1.0) alpha 0.85; the original's looks a bit more
   translucent in Max's screenshot. Tune on the PC if the bricks look solid.
4. **Moon drop.** Built and checked offscreen (two Large casts, then a drop). Confirm on the PC.
5. **Reflected band.** Drawn as the original's sprite now; checked by eye only.
6. **Race cloud density.** Matches the original's Race screenshot (150031); nothing to change.
7. **Survival curse wave**: the band and sky colours are as the original's code; confirm it looks right.

## Rules of work (Max, 2026-10-08)
- Check every change against the original's decompiled code first (research/ripped/...). Cite the
  file in the PR. Note what the install can't settle (stub shaders) in docs/pc-tasks.md.
- Merge only when `cargo test` passes; rebuild the exe in Desktop\TrickyTowers-build after a batch.
