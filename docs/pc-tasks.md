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
- Puzzle floor pieces are wired up (`dress_floor` in src/visuals.rs; 2-wide `puzzle_piece_2` isn't used yet, as our columns are 1 wide). Screenshot Puzzle on each difficulty offscreen and compare the floors and the missing water with the original. Read how the original picks column widths (1 or 2) so `puzzle_piece_2` can be used.
- Wind, the Puzzle steering limit, per-shape vine growth (`vine_overlay_<P>`,
  `vine_overlay_mystery_<name>`, as animations) and the race hold countdown are in. For the
  countdown, say if the original draws the numbers from a sprite sheet rather than the font,
  and stage them if so. Next readout wanted: the Winner! and Wizard out! banners, the trophy
  pop-in and the spotlights (names, where they show, their timings), so the results can be coded.
- Match moments are in (src/results.rs): the countdown, WINNER! with `winner_sunrays`, WIZARD
  OUT!, the spotlights and veils, and a `trophy` that pops in. Stage the cup the original shows
  after a normal local match as `trophy` (say which size it uses when), and screenshot a race
  offscreen until someone wins (a small config with `finish_height: 3.0` and AI opponents
  gets there in about 15 s) to compare with the original.
- Spell effects are wired up (src/effects.rs, bubbles in src/visuals.rs): stage the bubbles as
  `bubble_<2|3|4>_<1|2|3>`, the two Ghost skulls as `ghost_skull_1`, `ghost_skull_2`, and check
  `balloon`'s pivot (it floats 2.5 cells above the brick centre on a string). Next readout
  wanted, for the cast beams: the art names and sizes of the dark cast beam (up from the wizard,
  down the victim's lane, then the lane wave), the light cast's lightning strip, and the spell
  cloud over the panel with its "used" animation; stage them as `cast_beam`, `cast_lane_beam`,
  `cast_lightning`, `spell_cloud`, `spell_cloud_used`. Also the bubble wobble's speed and how far
  above the brick the balloon floats.
- For the title menu: the staged `menu_*` tile names (selected and deselected), where each
  tile sits on screen (as a share of the screen or in units with the view height), their
  order, what moving between them animates (scale, slide, timing), and the logo's place. Our
  menu offers Race, Survival, Puzzle and Endless plus players, AI and difficulty; say how the
  original's local-play screens pick mode, players and difficulty so the flow can match.
- Survival Pro curse waves: the exact levels (which curses each level can pick, how many at
  once, and each one's length), to replace the `(?)` guesses in `WaveRules` in src/config.rs.
- The title tiles are in (src/menu.rs, `TILES`). Stage the original's tile art under the names in
  docs/art-layout.md (`menu_tile_<key>_deselected|_selected|_front`, `menu_tile_header`,
  `menu_corner_large|small`, `menu_sunrays_1..5`). The tile positions you read overlap if taken as
  centres, so they are laid out from the sizes for now: read each tile's anchors, pivot and
  parent so the centres can be exact, and say what moving between tiles animates (scale, timing)
  and where the logo sits. Then take an offscreen picture of the title screen (no `--play`,
  `--shots 2`) and compare.
- Lanes now follow the original's split (`GameConfig::lanes`) and the veil is the two mirrored
  strips. Screenshot a 4-player race until someone wins and say whether the strip's dark side
  points the right way (we flip the left one) and how far the corners of a picked title tile
  pulse out (we guess 8 px).
- The pause menu and the "Game Completed" popup are in as plain text on dark panels (src/menu.rs,
  src/popup.rs). Read their art (panel, button frames, the selected look) and stage it with
  names to suggest, and read how long after a match ends the popup appears (we guess 4 s,
  `moments.popup_after`) and whether WINNER! stays under it. Also read how XP per match is worked
  out and how many XP each wizard level needs, so levels can be added.
- Select Players and Select Mode are in (src/select.rs) with colour fallbacks. Stage their art
  under the names in docs/art-layout.md (`menu_header`, `menu_player_select_*`,
  `menu_mode_title_*`), plus each mode tile's own background if it has one (say its name and
  size). Read where the 318×500 wizard panel's centre sits (we guess y +60), the mode tiles'
  size and spacing, and whether the third difficulty reads "Pro" or "Special" on screen.
  Offscreen pictures: `--menu players --offscreen <dir> --shots 2`, and the same with `modes`
  and `title` (without `--menu` the game now opens on the new "Press Enter to start" screen).
- "Press Enter to start" shows the staged `logo` at 818×420 above a blinking prompt. Read the
  original's layout of that screen (logo place and size, its debris and plants if they move,
  the prompt's exact words, place and blink) so it can match.
- Lost bricks now splash where they hit the water (src/effects.rs; `BrickLost` message). Without
  art it throws up droplets with guessed numbers (`GameConfig::splash`). Find the original's
  splash (particles or sprite sheet, its size, how long it lasts, any sound) and stage it as
  `splash` (an animation if it has frames), and note the numbers in docs/research/.
- Select Players and Select Mode are rebuilt from your readout (two columns, 9-sliced panels,
  per-pose wizard spots). Take offscreen pictures (`--menu players`, `--menu modes`, `--menu
  single`) and list what still differs. Endless has no mode tile name yet: say which picture the
  original's Single Player mode select shows for Endless Race and stage it as `menu_mode_endless`.
- Single Player is now its own screen (src/single.rs): the seat over Trials/Endless, then the
  Endless screen with Race/Survival/Puzzle and a leaderboard. Take offscreen pictures (`--menu
  single`, `--menu endless`) and read what differs: the Endless buttons' centres, the wizard
  faces' spot and size, the leaderboard's place, art and rows, the selected/deselected look of
  a list button, and where the name plate sits under the single seat. Also say what Trials holds.
- The popup and pause now follow your readout (src/popup.rs, src/menu.rs). Take a picture of a
  race win (`--shots 14,20` with a quick finish) and of `--menu pause`, and say what differs:
  the pause panel's size and the pause row pitch are guesses, and so is the popup header (70
  px, left empty). The top-left debug text is gone unless F3 is pressed.
- Base shapes (Max found them while playing): the bases aren't flat. Some have 1-cell notches that line
  up with the art, and the castle one has 2-cell holes. Ours is a flat `platform_width` box. Read every
  base's collider from the original (each mode and difficulty, and which base each uses): its cells or
  polygon points in cell units, measured from the base centre and top. Write them in
  docs/research/modes.md under "Base shapes", and say which picture goes with each shape.
- Single Player and Endless now follow your readout (src/single.rs, docs/research/menus.md):
  pack panel instead of the name plate, sliced list buttons with the cup band and small corners,
  Endless buttons on the right, faces, leaderboard on the left. Stage `menu_leaderboard_panel`
  and `menu_leaderboard_row` (added to tools/extract/stage.ps1). Take offscreen pictures
  (`--menu single`, `--menu endless`) and say what still differs. Read the leaderboard's row
  size, spacing, text (rank, name, score?) and title, whether `menu_button_background`'s borders
  18/16/71/15 are left/right/top/bottom, and the Endless buttons' selected look.
- Survival Pro curse waves: Ghost now strikes with the brick in hand and again with every new
  brick while the wave lasts. Just confirm a `--play survival --difficulty pro` offscreen run
  still loads with 0 errors.
- The bases now use your collider readout (`GameConfig::bases`): the Race house is 6 wide with
  the chimney cell and the under-water ledges, the Survival castle has its merlons and holes.
  Take offscreen pictures of Race and Survival (`--shots 5,20`) and check that bricks sit on the
  art where the boxes are (chimney, holes). Curse waves now lift and hold the brick in hand for
  1 s, shaking: read the original's shake (how far, how fast, or its animation clip) for
  `WaveRules::shake`.
- Losing towers now fall apart before the popup (src/collapse.rs, effects in src/effects.rs,
  numbers in `GameConfig::moments`): after 1.5 s every brick turns into a sensor (no longer
  solid) and drops into the water, every 3rd brick (every brick if the wizard was already out)
  blows up in placement order with the blast shake, then the 2.5 s rumble and the thud. The
  popup opens once every loser is done (`popup_after` is now only a minimum, 0). Please read
  and report: (1) whether "stops being solid" means the bricks pass through everything and
  sink, or only stop colliding with each other; (2) the order bricks blow up in (placement,
  top-down, random?); (3) the explosion's art (sprite sheet or particles, size, frames and
  times) and stage it as `explosion`, plus the sounds as `sfx_explosion`, `sfx_rumble` (the
  loop) and `sfx_brick_heavy`; (4) whether a solo win (no losers) opens the popup at once or
  after a delay. Then screenshot a 2-tower race until someone wins with a quick finish
  (`--shots 14,16,20`) and say how the collapse differs.
- Endless scores are now saved between runs (`%APPDATA%\tricky-towers-rs\leaderboard.ron`;
  offscreen runs never save). Read whether the original saves the Endless leaderboard per
  profile and what each row shows (rank, name, score, wizard?), and the rules of Endless
  Survival and Endless Puzzle (what ends a run, how the score counts, any speed-up), so they
  can be played.
- Cast effects are in (src/casts.rs, numbers in `GameConfig::casts`, taken from your stage.ps1
  notes): the dark cast's beam up from the caster, then a beam down onto each victim's brick,
  then the lane wave; the light cast's lightning from the wizard to the brick; spell clouds
  beside each wizard (Show, Floating, Used after a cast, Hide when thrown away); Large grows the
  brick in hand over 0.3 s with a camera shake. Please check, in this order:
  1. Stage everything listed under "Cast beams" in stage.ps1 and run
     `--play race --offscreen screenshots --shots 30,45,60` with 1 AI opponent (a small config
     with `magic: (line_spacing: 2.0)` makes casts come quickly). Say how the beams, wave,
     lightning and clouds differ from the original.
  2. Read the sizes we guessed (marked `(?)` in `CastLook`): the beam's width and how far up it
     reaches, the lightning's width and wobble, the spell cloud's drawn size in cells (its
     picture's size and pixels per unit × 1.31), the icon's size in the cloud, the Floating
     clip's bob, and what the Used clip does (we swell to 1.3× then shrink while rising).
  3. Large: the camera shake's size and length, and the sound it plays; stage that sound as
     `sfx_large`.
  4. Where wizard 4's clouds sit (we reuse wizard 1's), and whether the clouds flip sides
     when a lone wizard floats on the left of the tower.
  5. The rays, rings, blobs and sparks at the beam's foot: we only draw `cast_circle_gradient`
     there. Say which ones matter most to look right and their sizes and spin speeds.
- XP rules are in as data (`GameConfig::xp`, `XpRules::for_place`, `level`, `name`); nothing
  shows them yet, since only online matches and cups give XP. Please confirm: (1) past Expert
  Wizard, is level 11 "Master Wizard Level 1" (our guess) or does the count start elsewhere;
  (2) what "second to last" and "third to last" get in a 2- or 3-player match when those places
  are also the winner's (we give the winner the full amount); (3) where the original shows XP
  and the level (profile screen, after a cup), with the art names, so it can be drawn when cups
  are added.
- Readouts that unblock the next cloud work (no screenshots needed, just notes in
  docs/research/):
  1. **Cups:** how a local cup is set up (how many matches, which modes, picked how), how
     points per place add up, what decides the cup winner and ties, which trophy size
     (`trophy`, `trophy_big`, `trophy_super`) goes with which cup, what the screen between
     matches shows, and when XP is given.
  2. **Survival roof:** what goes on top of a tower that has used all its bricks (art name,
     where it sits, how it animates in), and whether the tower is frozen after.
  3. **Menu attract demo:** when it starts (idle time on which screen), which mode and base,
     the "fixed loop of moves" (the move list, or how it is built), and what ends it.
  4. **Trials:** how many levels, and for each: mode, target, the enemy wizard's spell script
     (every N s, at set times, or at brick fractions; which spells), stars or medals, and the
     level select layout and art names.
  5. **Endless Survival and Endless Puzzle:** what ends a run, how the score counts, and any
     speed-up or rule change over time (already asked above; listed here so it isn't missed).
- **Endless Survival and Endless Puzzle are now playable, built on GUESSES** (public
  descriptions only; src/config.rs `EndlessPlay` and `ModeKind::EndlessSurvival` /
  `EndlessPuzzle`). Please read the original and correct each of these, writing the real
  values in docs/research/modes.md (replace the "public descriptions only" section):
  1. Endless Survival score: we give 10 per brick placed. Is it bricks, height, or something
     else? How is it rounded and shown?
  2. Endless Survival lives (we use Survival's 3), the grace time after a lost life (we use
     Survival's 1.5 s) and drop speed (we use Survival Normal's 2 + the usual height bonus).
     Does it speed up over time?
  3. Endless Survival spells: we hand out Survival's light spells on spell lines, no dark ones.
     Are there spell lines at all, which spells, and are there curse waves?
  4. Endless Puzzle laser: we start at Puzzle Normal's height (3.2 + random 0..4) and raise it 1
     cell every 3 bricks placed. How does the original move the laser (per brick, per height,
     over time, by stages)?
  5. Endless Puzzle failing: we end the run on any dropped brick or a brick settled above the
     laser. Is a drop a fail, or does it lower the laser like battle Puzzle?
  6. Endless Puzzle score: we give 10 per brick placed. What does the original count?
  7. Endless Puzzle start spell (we give Puzzle Normal's Ivy) and floor (Puzzle Normal's).
  8. Both: do they use a difficulty (we force Normal), which base and background, and is
     the leaderboard per mode (we keep one per mode, 10 rows)?
  Then offscreen screenshots: `--play endless-survival` and `--play endless-puzzle`
  (`--offscreen screenshots --shots 5,20`), and say what differs.
- Controller rumble is in (src/input.rs `rumble_losers`): a losing player's pad rumbles 2 s when
  their end sequence starts and 2.5 s with the rumble after the blasts. The strength (0.6 on
  the strong motor) is a guess: read the original's rumble strength (both motors) for each.
- **Trials are now playable on SAMPLE data** (src/trials.rs, src/trials_menu.rs,
  `GameConfig::trials` = `TrialDef` list, `sample_trials()` in config.rs). The engine follows
  your notes: a goal per mode, and an enemy wizard who casts dark spells every N seconds, at set
  times, or when a share of the bricks is used (`CastWhen::Every/At/BricksUsed`). The three
  sample trials and everything below are GUESSES. Please read and write in docs/research/:
  1. All 50 trials, in order, as rows the cloud can paste into `trials` in game.ron: name,
     mode, goal (Race: finish height and time limit; Survival: bricks and lives; Puzzle:
     bricks and laser height), drop speed, start spell, base or floor, and the enemy script
     lines (when + which spells). Note the petrified-brick challenge variant too.
  2. How a trial is won and lost beyond the goal (does a Race trial need the 3 s hold? does
     the Survival roof countdown apply?), and whether trials give stars, medals or times.
  3. Unlocking: are trials opened one by one, by stars, or all at once? (Ours: all open.)
  4. The Trials screen: layout, art names, what each row shows, and how trials are grouped.
  5. The enemy wizard: which art, where it floats, how its casts look (we only draw the beam
     down onto your brick), and whether light spell lines appear in trials.
  6. What the game saves for trials (we save the names beaten, in `trials.ron` next to the
     leaderboard).
- **Cups are now playable on GUESSES** (src/cups.rs, src/cups_menu.rs, `GameConfig::cups`).
  On Select Mode, C (or the pad's Select) cycles Single match / Tiny Cup / Big Cup / Super
  Cup; a cup is a series of matches of the picked mode, the popup's first item reads "Next
  match" until the last one, the standings line runs along the top, and the winner gets the
  cup's trophy and everyone XP by cup place (saved per player seat in `xp.ron`). Please read
  and correct, in docs/research/:
  1. How a local cup is picked in the original (screen, wording, art) and how long each is.
  2. Points per place (ours: 3, 2, 1, 0), tie-breaks (ours: better place in the last match),
     and whether modes rotate or come from several picked modes ("Next match" appears "when
     more than one mode was picked" in your popup notes, so a cup may BE the mode list).
  3. Which trophy goes with which cup, where it is shown and how it animates (ours: centred,
     the existing 0.6 s pop).
  4. When XP is given in a cup (per match or at the end) and how it is shown and saved
     (ours: at the end, by cup place, with `XpRules::for_place`).
  5. What the screen between cup matches shows (standings? a scoreboard screen?).
