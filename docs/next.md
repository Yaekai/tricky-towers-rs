# What's next (written 2026-10-09)

This is the working list for the next thread. "Done" means pushed and tested offscreen or
headless; nothing here is confirmed in the game unless Max says so. Values marked (guess) are
ours and need checking against the original (see docs/research/*.md and the videos).

## Done today (branch claude/project-thread-g9oll8)
- Tiny Cup standings panel, medal slots one per point, trophy on a decided winner's row, CONTINUE.
- Battle intro: mode title ribbon (per difficulty, OVERTIME in a tie), Race camera sweep,
  Survival count-up with the next-brick card, 3-2-1, then play.
- Mode ambience and music: Race water and breeze (Race Special storm), Survival ghosts,
  Puzzle wind; battle music starts after the 3-2-1.
- Each wizard casts its own beam (Beam01-04); wizard 2's is the magenta column.
- Curse orb falls down the victim's column.
- Puzzle: floor art (dirt with grass, as in the original) and floor collision (solid like a base);
  one shared full-width laser; a fallen brick raises the base (eased, kinematic, bricks carried);
  a brick touching the laser stops that tower, a 3-2-1 runs, then bricks are counted.
  No brick limit in battles; no WIZARD OUT! banner in Puzzle; no bricks-to-come column in battles.
- Select Mode third column (single match, Tiny, Big, Super Cup); a Random cup re-picks a
  mode and difficulty each match.
- Mouse on every menu: hover selects, click picks (title tiles, Settings rows, Select Mode
  columns, Select Players seats, pause options, Game Completed popup).
- Menu details: start screen sounds (brick fall, land nudge, crumble) and display-letter prompt;
  title tile labels bigger; "?" in the display letters; header bar on small tiles; menu cursor
  sound (four select sounds, random); Select Players Down confirms the wizard, then the pack;
  Settings rows centred; Settings row words heavier and near-black.
- Title screen: six islands float (bob), wave crests scroll along the water, rays stay inside
  the picked tile.
- Exe rebuilt at Documents\Claude\TrickyTowers-build\TrickyTowers.exe (15:44; rebuild after
  today's last pushes, see below).

## Open: Puzzle (highest priority, Max tests it most)
1. Verify in the game: the base rise is seamless (no blocks through the ground), the bricks
   carried up by the base, and the eased rise speed (base_rise_speed, 3 cells/s, guess).
2. Scoring after the count (Max's list, not built): when the round is counted, each tower's
   brick count shows big above its lane (winner gold, the others pink), then a winner popup
   with the score. The 3-2-1 should show over the lane that touched the laser. Only the
   winner's score is highlighted; ties go to overtime rules (check against the original).
3. "We both won": with no brick limit this should no longer happen; check two players
   touching the laser together.
4. "Solo puzzle feel" in local two-player play: Max says it's not the original's multiplayer
   look. Compare a two-player round frame by frame with his videos (lane split, camera
   framing, laser width, where the bricks come from, what the HUD shows).
5. Puzzle difficulties (Easy, Normal, Pro floors) against the videos: floor heights, ledges.
6. Laser drop rule: Max confirmed the base rises instead of the line lowering; confirm the
   half-cell amount in the game (puzzle.drop_penalty, 0.5, guess).
7. Laser zap flash (bright laser flashes when bricks above it are zapped) and the zapped
   bricks' flicker: check against the video.

## Open: menus
1. Title water and waves: the water band is still a flat gradient in the composed backdrop
   (compose_menu.ps1). Max says "the water visuals in the menu isn't done": check the other
   menus (select, start, pause) for the same water and waves as the title screen.
2. Done: islands bob like the original's BobbingIsland (back one still, others 0.2-0.4 units
   over 2.5-4 s a leg, in-out quad).
3. Wave crest scroll speed (GameConfig::title_wave_speed, 30 px/s, guess).
4. Ray fit (RAY_FRACTION 0.75, guess): compare the ray size and brightness to the original.
5. Title tile label sizes (4.0 and 3.0 Vh, guesses) and the "?" size (10 Vh, guess).
6. Settings value words (1920 X 1080, Magical, the flags) weight, like the row words.
7. Mouse: check the pause and popup hit positions in the game (estimated from the layout code).
8. Done: slider clicks set the nearest tenth, arrows step (SettingsSliderItem; no drag in the
   original).

## Open: cups
1. Overtime: seen in Max's clip_1,791,548,470,628.mp4 (a Puzzle Tiny Cup). The ribbon shows
   the mode title with OVERTIME under it (ours matches); a draw in overtime gives both players
   first place (3 gold medals each) and the cup plays on. Medals past the target show as "+N"
   at the row's end (done). Puzzle draws (both "1 DRAW") belong to Puzzle scoring.
2. Medal rule confirmed from TournamentResultsItem: 4 - place medals of that place's kind per
   match; only the latest match's medals animate (done). Not done: a 4th place sometimes (1 in
   7) shows one empty medal for that match only; the medal sounds (gold LEVEL_UP, silver,
   bronze, empty 1-6).
3. Big and Super Cup layout: unmeasured (same as Tiny; slot squeeze guessed).
4. Cup timing: slide 0.4 s, fill 0.5 s then 0.55 s a slot (guesses).
5. Trophy and badges: sizes and places are guesses (cup_panel in game.ron).

## Open: intro and modes
1. Done: Survival intro card flips through random bricks during the count (one per two
   counted, AnimateBrickCountEffect).
2. Puzzle intro: check the laser sweep and the count against the video (1:06 and the first
   match of the recording).
3. Music start after the 3-2-1 in a real run.

## Guesses to check (all in config or constants)
- GameConfig: puzzle.count_seconds (3), puzzle.base_rise_speed (3 cells/s), puzzle.drop_penalty
  (0.5), intro timings (intro.*), cup_panel timings (slide, fill_start, fill_step), title islands,
  title_wave_speed, moments and casts values marked (?).
- Constants: RAY_FRACTION (menu.rs), TITLE_PIXELS (26, from the backdrop), menu label sizes.

## PC checks (see docs/pc-tasks.md for the full list)
- Curse orb in game; match-start ribbon art; Select Mode third column art; Survival and Puzzle
  end details; camera pull-back start; place labels; Puzzle score.

## Process
- Build the exe after the last push: `cargo build --release`, then copy `target\release\
  tricky_towers.exe` to Documents\Claude\TrickyTowers-build\TrickyTowers.exe with `assets` and
  `extracted` beside it.
- Run `cargo test` before every push; regenerate the shipped config after changing defaults:
  `TT_WRITE_CONFIG=1 cargo test --lib write_shipped_config`.
- Keep DEBUG-TOOL tags on non-original code and docs/release-checklist.md current.
- Max prefers 1:1 with the original without asking; ask him only for things the install and
  videos can't answer.
