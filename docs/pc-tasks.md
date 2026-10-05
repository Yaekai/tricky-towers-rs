# Standing tasks for the extract and test session on Max's PC

Max has asked this session to follow this file after every pull of `main` (work has gone there since 2026-10-05; `features/spells-puzzle-menu` is retired) until the 1:1 art
work is done, without waiting to be asked. The cloud session edits this file to queue work.
Read-only on the Steam install, nothing from the game goes into the repo (see AGENTS.md),
and nothing gets installed without Max's own go-ahead.

## First: install the token-saving tools (Max's go-ahead given 2026-10-05)
Max wants these in use on this project. Install them once on the PC, then
check they're active at the start of every session. Read each repo's own install notes first;
the steps below are from their READMEs on 2026-10-05.

1. **Ponytail** (https://github.com/dietrichgebert/ponytail): a "lazy senior dev" prompt that
   makes the agent write the smallest working change (reuse before writing, stdlib before
   new code, shortest diff). In Claude Code send these as two separate prompts:
   `/plugin marketplace add DietrichGebert/ponytail`, then `/plugin install ponytail@ponytail`.
   Needs `node` on PATH for its two hooks. It stays on every session; `/ponytail` changes the
   level. It doesn't override the 1:1 rule: matching the original exactly still comes first,
   even when a shorter version would do.
2. **Graphify** (https://github.com/Graphify-Labs/graphify): maps the repo into a knowledge
   graph so the agent queries `graphify query` / `graphify explain` instead of reading whole
   files. Needs Python 3.10+ and uv: `uv tool install graphifyy` (two y's; other `graphify`
   packages on PyPI aren't theirs), then `graphify install` in the repo, then `/graphify .`
   in Claude Code. Code is parsed locally (tree-sitter, no LLM). It respects our whitelist
   `.gitignore`, so `extracted/`, `research/` and the game files stay out of the graph; keep
   it that way (never `--no-gitignore`). `graphify-out/` stays uncommitted (the whitelist
   already ignores it). `graphify install --project --strict` makes the agent use the graph
   before reading files; use it if the plain install gets ignored. Rebuild the graph after
   big changes (`graphify hook install` adds a post-commit refresh).
3. **OmniRoute** (https://github.com/diegosouzapw/OmniRoute): a fallback, not always on. Max
   uses it only after hitting the daily Claude limit, to finish a task that was cut off. It's
   a local proxy server, not a Claude plugin: `npm install -g omniroute` (runs on
   localhost:20128), then `omniroute run claude` (or `omniroute configure`) points Claude
   Code at it. Set up per Max (2026-10-05): **all providers enabled**, with its automatic
   fallback between them. Compression: use the `coding-safe` profile unless Max says
   otherwise, since lossy modes can drop code or numbers from the research notes. Normal
   sessions run straight on Claude, without OmniRoute. Rules while running through it:
   - Never send anything from `extracted/`, `research/` or the game install through it (other
     providers would get the original's files); keep to the repo's own source and notes.
   - The model doing the work may not be Claude. Add the AGENTS.md log line for that session
     with "via OmniRoute", so the next Claude session reviews that work before building on it.
   Write the setup actually used (combo name, compression profile) here once done.
4. **claude-code-setup** (Anthropic's official plugin, from
   https://github.com/anthropics/claude-plugins-official, `plugins/claude-code-setup`): looks
   at the codebase and recommends Claude Code automations (hooks, skills, MCP servers,
   subagents). Install: `/plugin marketplace add anthropics/claude-plugins-official`, then
   `/plugin install claude-code-setup@claude-plugins-official` (or enable it from claude.ai
   if Max already added it there). Run it once on this repo after Graphify is set up, write
   its recommendations here, and apply only the ones Max agrees to. Any hook it adds must keep
   the rules in AGENTS.md (nothing from `extracted/` or `research/` committed or sent out).

### Setup status (PC, 2026-10-05)
- Installed (user scope): `ponytail@ponytail` 4.12.0, `claude-code-setup@claude-plugins-official`
  1.0.0, and uv (winget `astral-sh.uv`, Graphify's prerequisite). Python itself isn't installed;
  uv fetches its own.
- OmniRoute installed with Max's OK (`npm install -g omniroute`, needs
  `--allow-scripts=omniroute,keytar,onnxruntime-node,@parcel/watcher,@swc/core,protobufjs,esbuild`).
  Not configured yet: providers need Max's accounts, and it's only used after the Claude limit.
- Graphify: Max ran `uv tool install --python 3.12 graphifyy` (the sandboxed session can't;
  uv's Python folder link is refused there). `graphify.exe` is in `~/.local/bin`, not on the
  session PATH. `graphify install --platform claude` done (user skill + `~/.claude/CLAUDE.md`).
  Graph built with `graphify update .` (code only, no LLM, nothing sent out): 2,000 nodes,
  no `extracted/` or `research/` files in it. The whitelist didn't ignore `graphify-out/` after
  all (`GRAPH_REPORT.md` matched `!*.md`), so `/graphify-out/` is now in `.gitignore`.
- claude-code-setup was run before the graph existed; it only reads the repo.

### claude-code-setup recommendations (waiting for Max; none applied)
Note first: the whitelist `.gitignore` ignores `.claude/`, so a project hook, skill or agent
there stays on this PC unless `.gitignore` gets an exception like `!.claude/settings.json`.
Pick per item.
1. **Plugin `rust-analyzer-lsp`** (same official marketplace): gives the agent go-to-definition
   and errors in Rust without a full `cargo check`. Needs rust-analyzer on PATH (rustup
   component). Cheapest token win after Graphify.
2. **MCP `context7`**: live docs lookup for Bevy 0.19, avian2d 0.7 and bevy_egui 0.42, whose
   APIs change between versions and are past most models' training. Sends only library names
   and questions out, never repo files.
3. **Hook, PreToolUse guard**: refuse any Bash/Edit/Write/Read call whose command or path
   touches `extracted/`, `research/`, `game/` or the Steam install when it also involves
   `git add -f`, `curl`, `gh`, upload or a non-Claude endpoint. Enforces the clean-room rule
   in code instead of by memory.
4. **Hook, PostToolUse `rustfmt`** on edited `.rs` files: keeps diffs small and consistent.
   Only worth it if the code is already rustfmt-clean (run `cargo fmt --check` first).
5. **Skill `offscreen-shot`** (user-only): runs `cargo run -- --offscreen <dir> --shots ...`
   or a `--script file.ron`, saves to the scratch folder, never opens a window or plays sound.
   Matches the "PC work stays headless" rule and replaces retyping the flags.
6. **Skill `project-conventions`** (Claude-only): the AGENTS.md rules as a checklist the agent
   applies on every change (values in `GameConfig`, `DEBUG-TOOL` tags plus
   release-checklist entry, headless-safe gameplay files, AGENTS.md log line).
7. **Subagent `one-to-one-reviewer`**: reviews a diff against `docs/research/*.md` and flags
   guessed values not marked as guesses, literals outside `GameConfig`, and untagged debug
   tools. Read-only.
Skipped as not useful here: GitHub MCP (the cloud threads already handle PRs), database,
frontend and test-writer suggestions.

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
- **Puzzle ruins colours (staging only, no code needed)** (gap-scan.md 1): stage the Puzzle
  ruins with `mask` from the read colours: back #F4D9B0 → #F7E0BE (parallax 0.3), middle
  #E0AC92 → #D69883 (0.5), front #C47464 (0.5). For Puzzle Pro, stage the same layers again as
  `background_puzzle_pro_N` with the Pro colours: back #C82446 → #D72C53, middle #7E163C →
  #670E2B, front #2E0405; they replace the normal ones on Pro (`visuals::pro_look`). Picture
  Puzzle Normal and Pro and compare.
- **Survival curse-wave background is in** (2026-10-05, cloud): during a wave the background
  layers grey to (166,166,166) and drifting layers go 138× faster, shifting with the sky over
  1 s (`WeatherLook::survival_wave_grey/drift`); Endless Survival's counted waves now shift the
  sky too. Read: which layers count as "clouds and islands" (we grey every background layer),
  the normal drift speeds of Survival's clouds (stage them with `drift`), what the "overlay"
  colour (144,52,110 / 91,60,175) is drawn on, the mist's 5× speed-up, and the stars'
  twinkle (positions, art, one every 2-5 s) and the swinging moon.
- **Trial banners, race-trial clock and finish flags are in** (2026-10-05, cloud): trials end
  on CHALLENGE COMPLETE / CHALLENGE FAILED (drawn at 0.6× the WINNER! size, ?), a race trial's
  finish line shows the seconds left beside `hourglass` (fading in over 1 s; place and size
  guessed, the 0.5 s slide not done), and `finish_line_end` flags sit at both ends of the
  finish line (left one mirrored, ?). Stage `hourglass` and `finish_line_end`, read the
  banners' exact wording and size, the clock's place and what it slides from, and the flags'
  size and place. Still not done from gap-scan.md 3-4: gravel (when does it spawn?), impact
  dust on later hard hits (what counts as hard enough?), the Endless Race score line's look,
  the "continue" prompt after a trial or Endless run, and DRAW.
- **Race Pro background, ready for staging** (2026-10-05, cloud): screen layers now take
  `drift` and `lit` (docs/art-layout.md), and Race Pro's water band falls back to the read
  gradient (13,119,154 → 49,21,138, parallax 0.3, height 6) when `water_band_race_pro` isn't
  staged. Stage `background_race_pro_N`: the clouds (`race_pro_cloud`, parallax 0.2,
  `drift: (0.15, 0.25)`, in the units you read them in; say which), the race islands' `_DARK`
  pictures at parallax 0.3 with the day picture as `lit`, and `water_pro` (stretched 80×).
  Then picture Race Pro with a strike and compare. The waves' rows (y 3 / 2.5, scales
  0.2-0.5 / 0.5-0.7) aren't drawn yet: say how the original lays out a wave row.
- **Game Info and Stats popups are in** (2026-10-05, cloud; src/game_info.rs, src/stats.rs,
  stats saved as `stats.ron`). Read from the original and picture with a script:
  1. Game Info (Tab / Y): where the title, mode lines and player rows sit inside the 750×550
     panel; the row art (`gph_popupTextBackground02`, stage it as `popup_text_background`);
     what the name and level lines say for local players and AI; whether it pauses the
     match or closes by itself; the avatar's size and which picture each of the 14 wizards
     uses (we use `wizard_N`, staged for 1-4 only).
  2. Stats: the rows of Game stats (we show games and wins per mode) and Other stats (we show
     cups won, time played and the 3 Endless bests), their wording (language keys), text
     sizes, the gap between the panels and their heights on screen, the time format, and
     what counts (we count player 1's local battles, not trials or Endless runs).
- **Readout for the original's text and tutorials (next cloud work, plain notes only):**
  1. The format of `Resources\language\<lang>.txt`: one example line pattern with the value
     replaced by `...`, and the full list of key names (names only, no translations) in
     docs/research/text-keys.md. With that the game can show the original's wording from the
     staged `text_<lang>` files at runtime.
  2. The explanation (tutorial) popups' layout: panel art and size, where the title, text,
     mode picture, difficulty, Pro subtitle and the two icons sit, sizes, and the keys that
     close it; which key each popup's title and text use.
  3. The Credits popup's layout and scroll speed. The names themselves stay in the game's
     files: say where they live (a language key, a text asset) so they can be read at runtime.
- **Keybinding popup is in** (2026-10-05, cloud; src/keybinding.rs, saved as `keys.ron`):
  Settings → Keybinding lists the 10 keyboard actions with their keys, Enter waits for a new
  key, a Reset row, Esc closes. Read from the original: the action list, order and wording
  (language keys), the reset row's wording, how the waited-on key looks, what happens when a
  key is already used (we swap), which keys can be bound, whether pads can be rebound, and
  the row width/height and the list's place on screen.
- **Trials unlock animation and menu button hints are in** (2026-10-05, cloud;
  src/trials_menu.rs, src/navbar.rs), both from the readouts, neither pictured yet:
  1. Stage the unlock's sounds as `sfx_chain_1` .. `sfx_chain_4` (SFX_CHAIN_1..4) and
     `sfx_brick_spawn` (SFX_BRICK_SPAWN); `sfx_roof_placed` is already used.
  2. Picture it: beat 8 trials (or set `done` in the save's `trials.ron`, keeping
     `tiers_shown: 1`), open Trials offscreen with a script, and take shots through the 3 s.
     Check the chain's turn direction (Unity angles are drawn negated for the UI's down-y,
     ?), and that the panel slide starts 0.2 s after the padlock's fade ends (?).
  3. The CONTINUE / BACK hints: their exact yellow and red, the font, and the button icon
     art for keys and pad buttons (now drawn as text on `key_background`). Which screens
     show which items (we show CONTINUE only on the title tiles, none on the start screen).
- **Batch for the trials/obstacles/values thread (2026-10-05, read in one go, notes in
  docs/research/, plain language, no code or assets in the repo; no screenshots needed).**
  Write each part's notes and push them as soon as that part is done, so the cloud can code it
  while you read the next.
  1. **Rival towers** (trials.md, ENEMY_TOWER_LEVEL_01..13): for each level, what it is made
     of (one picture, or bricks with shapes, cells and rotations: list them in pop-in order),
     where it stands (x offset from the player's base in cells, or its own lane), which floor
     it stands on, its scale, and whether it ever changes during the trial. The pop-in: per
     brick time, poof art and size, the bubble pop sound name and its pitch steps. Stage any
     art as `rival_tower_NN` / `rival_floor` and say the names.
  2. **Trial floors**: what "FLOOR_LWS at 12.5" means (a y offset, an x position, a scale?),
     and FLOOR_LWS_PETRIFIED_BRICK's stone I (is it a real physics brick the player can knock
     off, or part of the floor; its art). Also how puzzle trials' floors differ from battle
     Puzzle's, if they do.
  3. **Tier unlock animation** (menus.md "Locked tier"): when it plays (on entering the Trials
     screen after the medal count passes the tier, or right after the trial), whether the page
     turns to the new tier first, the 4 chain sound names and the brick spawn / roof placed
     sound names; the start stinger name per mode.
  4. **Endless Puzzle obstacle blocks** (modes.md step 3): what a block is (shape, size, a
     brick that is petrified/ice, or a fixed piece; its art), how many per level-up, and the
     rule that picks each position from the tower height, the new laser height and the camera,
     in words or as a short formula (no code). Also the 1.2x zoom's effect on the lane, and the
     3 move-laser and 3 spawn sound names.
  5. **Survival roof and Puzzle roof** (ROOF_02 and the Survival one): art names and sizes,
     whether the roof is a physics body or a picture, where it lands (centred on what), how it
     comes in (drop from where, time, ease, bounce, poof, sound), what happens to the tower
     after (frozen, sensors, still loses bricks?), and the 3 s countdown's look. Also the race
     winner's roof if it differs.
  6. **Guessed values** still marked `(?)` in the code. For each, the original's value or
     rule, or "not in the original":
     - config.rs: race `hold_seconds`; drop speeds marked `(?)` (Race/Puzzle base drop
       speed); Endless Survival lives (the max health setting's value) and drop speed; trial
       `medals_per_trial`; `minibase_width`; Auto Rotate `auto_rotate_interval` and
       `auto_rotate_back`; No Rotate `no_rotate_shake` and `no_rotate_reach`; results
       `sunray_spin`; Puzzle `sand_strip`; wizard `names` and `master`; casts `beam_width`,
       `beam_reach`, `lightning`, `lightning_wobble`, `cloud_size`, `cloud_icon`; hud
       `heart_size`, `heart_below`, `bubble_size`, `bubble_at`.
     - Endless: whether the wave gap counts from a wave's start or end; waves before the next
       wave level; lift/hold/shake at a counted wave's start; how long a counted Mist fogs;
       whether Endless runs use Normal's other values; how Endless Race hands out spells.
     - Menus: Trials and Settings page slide time; trial button order (down the left column
       first?); where the trial rows and the medal panel sit; the medal count's total; the
       chain's place on the locked panel; the cup screen's layout and wording; the popup's
       first item during a cup; cup ranking ties; whose XP (profile or seat); the Settings
       icon box and arrow art direction.
     - Looks: the WAVE N message's length; particle speed units; the spell cloud's pop-away;
       Diamond gloss (how often, sweep or flash); Hologram flicker period; the pack that pulses
       to 0.8x (period); Galaxy texture scale and drift speed.
- Bevy now builds without its 3D parts (Cargo.toml). After the next pull, take the usual
  offscreen pictures of each mode and a menu, and say if anything looks different (colours,
  missing sprites or text) or if the log shows new errors.
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
- **Endless Survival and Endless Puzzle now follow your readout** (modes.md, 2026-10-05;
  `EndlessPlay` in src/config.rs, `EndlessPlay::apply` in `start_match`). Done: Survival drop
  speed 4, light spells Ivy/Minibase/Petrify only, the 4 wave tables counted in bricks, waves
  15-29 s apart; Puzzle one life, drop speed 2, laser from 6.2, levels of 6, 8, 10 ... 26
  bricks, level-up wait 0.3 s / rise 4 cells over 0.5 s / wait 0.3 s, one Undo each level;
  score = bricks in the tower for both. Still guessed (?): Endless Survival lives (the max
  health setting, we use 5); whether the wave gap counts from a wave's start (we do) or its
  end; how many waves before the next wave level (we use Survival Pro's 2); the lift, hold and
  shake at a wave's start (Survival Pro's); how long a counted Mist wave fogs (Mist's 15 s).
  Not done: Endless Puzzle obstacle blocks at level-ups (their placement needs the code that
  picks them), its 1.2× camera zoom, removing bricks 4 cells above the view, the cream drop
  guide, the level-up sounds, the Large wave's camera shake, the end countdown after a zap,
  roof pieces (Puzzle roof / ROOF_02).
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
- **Cups now follow your readout** (match-moments.md "Cups"; src/cups.rs, still a DEBUG-TOOL):
  first to 6 / 12 / 18 points (Tiny / Big / Super), 3-2-1-0 per match, overtime while the
  leaders tie at or above the target, no XP for local cups. Still guessed (?): the order of the
  players below the winner (we break ties by the last match). Not done: the third Select Mode
  column (we still cycle with C), random modes per match from the picks, the medal rows in the
  popup, the winner popup with place counts, fireworks and the 12 s close.
- **Settings screen now follows your layout readout** (menus.md; src/settings.rs): two sliding
  pages (Sound effects, Music, Explanations, Stats, Credits, Keybinding / Full screen,
  Resolution, Quality, V-sync, Language, Apply), 800×78 rows 100 px apart, icons, slider bars,
  checkboxes with tick/cross, choice arrows, small corners on the picked row, page arrows at
  ±500. Page 2 waits for Apply. Guessed (?): the in-between resolutions, that the arrow art
  points right, the slide time (0.3 s), that the row stays the same when the page turns, that
  Language waits for Apply, how icons fit their 100 px box, that the slider fill is stretched
  rather than cropped. Not done: Stats/Credits/Keybinding popups, Quality's effect, the 18
  languages, Explanations' tips. Picture it on the PC: `--menu settings --offscreen <dir>`
  (the cloud has no GPU).
- From your notes, now in the game: Auto Rotate's motion (winds back 15° in 0.2 s, swings 105°
  in 0.45 s, waits; `auto_rotate_angle`), No Rotate's 0.3 s shake on a refused turn, Race Pro's
  spell rules (far behind at 20, Minibase [3] in the normal pool, line wait growing with players
  only on Normal and Pro), and the wizard's Action → Cheer when the Survival roof goes on.
  Please check and read:
  1. Auto Rotate: is each part an even speed or eased (ease curve?), which way it turns
     (we wind back clockwise and swing counter-clockwise), and what happens when the swing hits
     something (we place the brick there, like a blocked manual turn).
  2. No Rotate shake: how far and how fast it shakes (ours: 0.08 cells, 30 a second, a guess).
  3. Spell lines, two rules we could NOT build without more detail:
     a. "A line the tower has passed by more than 6 cells without touching is replaced": what
        counts as touching (a brick crossing the line inside the lane? any brick?) — ours
        collects a line as soon as the tower top reaches it, so it can't be passed.
     b. "Shown once it is 5 cells above the tower": when exactly a line is hidden and shown
        (does a new line stay hidden until it is 5 cells above the tower top?).
  4. Screenshot a Race Pro match with AI opponents for 60 s (`--play race --difficulty pro`)
     and a Survival finish, and say if anything looks off.
- New source notes in docs/research/wiki-notes.md (fan wiki + official FAQ quotes). Please
  confirm against the install, and say where they disagree with your readouts: the default
  keyboard keys (U/I nudge, J/K magic, Space rotates left), the Settings options and defaults,
  the 5 trial tiers with 0/8/17/26/36 medals to unlock and how many medals one trial gives,
  whether cups exist outside online play, how the Endless Puzzle laser rises per group of
  bricks (and the group size), and whether Large changes a brick's mass ("large and heavy").
- **Reference pictures and full pages from the web** (the cloud session can't reach these sites;
  your PC can). Everything downloaded stays on the PC in the gitignored `research/reference/`
  folder: never commit pictures, pages or video from the original or its sites. Write only
  plain-language findings into docs/research/ (wiki-notes.md has what search quotes gave so far).
  1. Download the press kit screenshots, logos and trailer stills from
     https://www.weirdbeard.nl/presskit/sheet.php?p=TrickyTowers (and the Steam store page
     screenshots, https://store.steampowered.com/app/437920/) into `research/reference/`.
  2. For each picture, say what it shows (mode, players, menu screen) and take the matching
     offscreen picture of ours (`--play <mode>` / `--menu <screen>`, `--offscreen
     research/reference/ours --shots ...`). Compare them side by side, read the pictures
     yourself (don't open them on screen), and list every difference: layout, sizes, colours,
     HUD/panel placement, text and wording, effects. Fix the ones that are only staging (art
     names, manifest fields) and queue the code ones here for the cloud.
  3. Read these wiki pages in full and add what search snippets missed to wiki-notes.md:
     Trials (every trial's goal if listed, medals per trial and how they're earned), Controls
     (gamepad layout too), Settings, Spells (exact on-screen names), Single player, Local
     battle, Online battle, Achievements (all 29), Characters, DLCs.
     https://tricky-towers.fandom.com/wiki/Tricky_Towers is the start page.
  4. Read the official FAQ (https://www.trickytowers.com/faq.php) and the tournament kit rules
     (https://www.trickytowers.com/tournamentkit.php) and note anything about local play, cups
     and match rules.
  5. Check the spell names our game shows against the original's on-screen names (the wiki
     says Tower Piece, Fast Brick, Ice Brick, Grass, Slow Down, Ghosts, Super, Super Duper; and
     check the no-rotate curse's real name), so `SpellKind::name` can match.
- **Max put press kit pictures on the PC** in the `images presskit` folder inside the gitignored
  release folder: start the reference comparison above with those (keep them out of git).
- **Wiki facts are now in the game; please double-check each on the install** (source:
  docs/research/wiki-notes.md):
  1. Keyboard (src/input.rs): Left/Right move, Down drop, Up turns right, Space turns left, U/I
     nudge, J light spell, K dark spell. Our old keys (A/D/W/S, Shift-nudge, Z, Q/E) are commented
     out in input.rs. Read the install's default keybinding list (and the gamepad layout) and say
     where it differs. Menus still also take WASD and numpad arrows: say if the original does.
  2. Settings (src/settings.rs): Sound effects and Music are separate volumes, both 100% by
     default; Full screen is on by default. Confirm the defaults and the 10% step, and list the
     options we don't have (Explanations, Statistics, Credits, Keybinding, Resolution, Quality,
     V-sync, Language) with what each does, so they can be built.
  3. Trial tiers (`TrialTiers`): 5 tiers of 10 (Rookie, Apprentice, Pro, Expert, Master) opening
     at 0/8/17/26/36 medals; we give 1 medal per trial beaten (guess). Read how medals are earned
     (per trial: 1? up to 3 by score/time?) and what a locked tier looks like.
  4. Endless Puzzle: bricks come in groups; the laser rises once a whole group is placed. We
     guess groups of 10 and a 1-cell rise (`EndlessPlay::laser_every`, `laser_rise`): read the
     real group size, the rise, and how the score ("conserve as many bricks as possible") counts.
- The start screen animates from the MoveIn readout (src/start.rs, `GameConfig::start`). Stage
  `logo_plain` (gph_logo, added to stage.ps1) with the `logo_brick`, `logo_debris_*` and
  `logo_plants_*` pieces, take offscreen pictures at 0.4, 1.7 and 3 s (`--shots`, no `--play`),
  and compare with your video frames. The brick's fall before 1.63 s is a guess: read it.
- **Debug tools are in** (docs/debug-tools.md): write small `--script` files to set up exact
  moments (a curse, a collapse, a Survival roof, a spell cloud) and picture them offscreen with
  `Picture`, instead of waiting for them in play. Use them for the comparisons above. The F1
  panel is for Max when playing in a window.
- **Skins are in** (cloud, from skins.md): each seated player picks a wizard (14) and a brick
  pack (8) on Select Players with Up/Down (Character / Brick pack) and Left/Right; towers draw
  with them (`TowerSkin`, `BrickPack`, `brick_<pack>_<shape>`, `next_brick_<pack>_<shape>`,
  `wizard<n>`). Guessed: the picker layout (two lines under the name plate) and the Spirit
  Animal names ("Spirit Animal 1..4"). When there's a PC session again: read the pickers' layout,
  art and wording, and the four Spirit Animal names.
- **Brick pack looks are in** (src/packs.rs): Christmas lights blink and snow caps (25% per top
  edge) grow in, Halloween decorations (up to 2, 40% each) pulse, Galaxy cells show one shared
  scrolling starfield in a purple border with twirling stars, Hologram glitches, Diamond
  flashes. Guessed: the lights' blink rate, the pulse length, the Galaxy scroll speed and
  scale, the Hologram look (cyan, see-through, sideways jitter) and Diamond's gloss (a flash
  instead of a sweep). Spring's grass and eggs aren't staged: stage them (names, sizes) and
  say where they sit. Picture each pack with a script (`Start` then wait) and compare.
- **Skies and weather are in** (src/weather.rs, `GameConfig::weather`, `visuals::pro_look`):
  Race Pro night sky (3-colour gradient from your readout), rain (front streaks and the back
  layer, slanting with the wind) and lightning (flash + bolt with branches); every Puzzle's sand
  strip and two-layer scrolling sandstorm; Puzzle Pro's sky, eclipsed sun and corona; Survival's
  sky shifting to the curse-wave colours over 1 s. A `sky_race_pro` / `sky_puzzle_pro` and
  `background_race_pro_N` / `background_puzzle_pro_N` in the manifest win over the built-in
  colours. Not done: the Race Pro clouds, `water_pro` and the `_DARK` islands lit by the flash,
  the Puzzle Pro ruin colours, the reflected gradients, the Survival clouds/stars/moon changes.
  Guessed: the sun's place and size on our screen, the sand strip height, the corona spin, the
  bolt shape, the rain drop width. Check: Race Pro's rain (about 1,100 drops a second) runs
  smoothly on the PC; picture Race Pro, Puzzle, Puzzle Pro and a Survival Pro wave with scripts.
- **Particles and HUD bits are in** (src/particles.rs, src/hud.rs): spawn poof, landing dust,
  lost-brick mist (Race/Survival) or desert dust (Puzzle) with the spinning lost cross, the
  WAVE N cloud message, hearts with lost slots, J/K key caps under the spell bubbles. Guessed:
  particle speeds (taken as tenths of the readout's numbers), the WAVE message's place, size and
  2 s length, the key cap's place. Stage `gph_cross` as `heart_lost` and the pad B buttons
  (`pad_b_up`/`pad_b_down`) so lost hearts and pad hints can use them. Not done: gravel, impact
  dust on later hard hits (we only puff on landing), confetti (cups are a debug tool), the
  hourglass timer on race trials, the Endless score line, `finish_line_end` flags.
- **The 50 original trials are in** (src/trial_list.rs, made from docs/research/trials.md):
  goals, drop speeds, fixed brick orders, each trial's own spell lines (spaced `every` cells
  plus 25% per extra player, given in order, "(random)" ones cast at once, none after the
  last; trials without lines show none), and enemy curses on heights (`CastWhen::Height`) and
  bricks (`CastWhen::Brick`, fires while that brick is in hand). A dropped brick fails a
  puzzle trial. Guessed: `MAX_TRIAL_LIVES = 5` for SS_20's "max lives"; that `h` heights are
  measured like our tower height (from the base top) and that `h0` fires with the first
  brick; that `bN` counts the brick in hand. Not done: trial floors other than the mode's
  (FLOOR_LWS at 12.5, FLOOR_LWS_PETRIFIED_BRICK, puzzle floors), the rival tower pictures
  (ENEMY_TOWER_LEVEL_NN), the trial names shown on the list (we show the ids). Check: play
  SR_10, SS_10 and SS_11 against the original and compare when the curses land.
- **Trials screen now follows the layout readout** (menus.md; src/trials_menu.rs): one sliding
  page per tier with the tier name as title, 10 trial buttons in 2 columns of 5 (mode picture,
  number, wizard face, medal once beaten, small corners on the pick), medal panel top right,
  locked panel with chain and padlock over unearned tiers, page arrows and bumpers. Guessed (?):
  where the rows sit (top row at y 230), that trials run down the left column first, the medal
  panel's place (read "120 right / 100 above the corner" as inward), the count's total (all
  medals), the face size, the chain's place and the slide time. Not done: the unlock animation
  and the start stinger. Picture it: `--menu trials --offscreen <dir>`.
- **Refresh `docs/screenshot.png`** (the README's picture, from 2026-10-03): take a new offscreen
  shot of a 4-player race with the real art and replace it. It shows on the GitHub page.
