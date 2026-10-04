# Notes for AI agents working on this repo

Read `PLAN.md` first, then `README.md` for what works today.

## Ground rules
- Clean-room: no decompiled code, no original assets, no Unity/IL dumps in this repo.
  Research about the original (what the rules are, what a value is) goes in
  `docs/research/*.md` as plain-language notes, written by you.
- Anything extracted from a local install goes in `extracted/` or `research/`,
  both ignored by the whitelist `.gitignore`. Don't loosen that file.
- Keep gameplay headless-safe: `src/gameplay.rs`, `src/spells.rs` and `src/ai.rs` must not
  depend on rendering or keyboard. Tests in `tests/headless.rs` run them with `MinimalPlugins`.
- Every tower, human or AI, goes through the same systems; only the writer of its
  `PlayerInput` differs. Keep it that way so bots, replays and netplay stay easy.
- Tunable numbers belong in `GameConfig` (`src/config.rs`) and
  `assets/config/game.ron`, not as literals in systems.

## Build
- `cargo run` plays; `cargo run -- path/to/mod.ron` loads another config.
- `cargo test` runs config and headless gameplay tests.
- Linux needs `libasound2-dev libudev-dev libxkbcommon-dev libwayland-dev`.

## Log
Append a short dated line here after each working session: what changed, what broke.
- 2026-10-03: scaffold. Controlled brick, physics hand-off, race and survival rules, placeholder visuals.
- 2026-10-03: multiple towers, AI wizards, spells (Vines, Stone, Shrink, Enlarge, Slippery), puzzle mode, menu, drop guide, next preview. Vines need a global joint basis or bricks snap to one angle and explode.
- 2026-10-03: fixed Vines crashes in avian islands: only join bricks that have a BodyIslandNode, cut vines before a brick turns to Stone, and despawn joints before their bodies. `cargo test -- --ignored` runs a 2-minute AI stress test.
- 2026-10-03: spells, controls and physics rebuilt from Max's install (docs/research/). Spell pairs from spell lines, 15 spells, no wall kicks, nudge, drop acceleration, gravity 18, mass 1, 50 Hz. AI needs the real drop speed (3) to steer in time; at 8 it drops lots.
- 2026-10-03: art/sound loader (src/art.rs, docs/art-layout.md) reading gitignored extracted/. Survival grace period and finish countdown; puzzle drops lower the laser. Full debug info filled the cloud disk (28 GB target); dev profile now keeps line tables only.
- 2026-10-03: real-art fixes from the PC run: the extracted folder is found from the exe's folder too, failed images fall back to shapes, bricks draw untinted with `_stone`/`_ice` versions, positioned and parallax background layers per lane, finish/laser/spell line/water/drop guide art, camera down to the water line. Player panels along the bottom: portrait, hearts, spell slots with icons.
- 2026-10-03: screen-pinned background layers and sky gradient, blue drop guide, rainbow spell-line pickups, vine/grass/bubble/mist art, controls moved to the menu, next preview lowered.
- 2026-10-03: `--play <mode>` skips the menu for scripted screenshots; the menu also takes numpad arrows (scripted key presses on Windows can arrive as those).
- 2026-10-03: water band gradient, player panels beside each base, bigger next preview using brick art, font_main support.
- 2026-10-03: sounds and fonts recognised by file type, mask recolouring for two-colour sprites, per-mode sky and water band.
- 2026-10-03: manifest animations (frames with times) play on any looked-up name; panel wizards animate.
- 2026-10-03: text drawn with the original's letters when a letter sheet is staged as font_main (src/letters.rs mirrors every UI Text).
- 2026-10-03: wizards float beside each tower from staged parts and clips, react to bricks, lines, spells, lost lives and going out (src/wizards.rs, wizards in game.ron). Masks read as list or pair, blue outlines transparent by default.
- 2026-10-04: difficulty levels (menu Tab, --difficulty), Puzzle floors per difficulty, Survival Pro curse waves, cursed and Mystery bricks come back up to 3 times, per-player brick order except on Pro and in Puzzle. README and PLAN brought up to date.
- 2026-10-04: Endless Race (`ModeKind::Endless`, `EndlessRules`): a pacer line per tower that speeds up, pauses and drops back each 10-cell stage; caught means out; best height is the score. Uses race art via `ModeKind::look()`.
- 2026-10-04: PC extract scripts committed under tools/extract (AssetRipper export -> guid index -> stage -> inventory); 1,636 manifest entries staged locally, all modes load with 0 errors.
- 2026-10-04: `--offscreen <dir> --shots 5,20`: no window, no sound, saves pictures and quits (src/offscreen.rs; views use the `ViewSize` resource instead of the window). Race Pro wind (`Match::wind`, guessed push) and wider Puzzle lanes (`GameConfig::lane_reach`).
- 2026-10-04: Endless Race rebuilt from the PC's readout: intro slide, checkpoints on picked-up spell lines (fixed heights), ease and wait, growing speed step, score 10 × height, light spells only, `background_endless_N`. Panels move beside the base when there's no room under it.
- 2026-10-04: font sheet, wizards 1-4, Endless pacer and Puzzle floor pieces staged by tools/extract; offscreen screenshots of every mode (no window, no sound) all load with 0 errors.
- 2026-10-04: Puzzle floors drawn with the staged floor pieces (`FloorBlock { part }`, `dress_floor`); the water strip hides in modes whose water band is off.
- 2026-10-04: exact Race Pro wind (`blow_wind` rays per row + `push_wind` forces), Puzzle steering limit from the tower's settled half width, per-shape vine growth animations, race hold countdown over the finish line (`show_hold_counts`).
- 2026-10-04: src/results.rs: match text (shadow + gradient-tinted face), race hold countdown, WINNER! with sunrays and trophy pop, WIZARD OUT!, winner spotlight and loser veils, timings in `GameConfig::moments`.
- 2026-10-04: src/effects.rs: Slow balloon on a string, Ghost skulls flying in to the brick's outline; bubbles pick the size/look variant and wobble ±2.5%.
- 2026-10-04: from the PC's win screenshot: no cup after a single match, WINNER! kept on screen through its bounce (and hidden until laid out), loser veils drawn dark.
- 2026-10-04: title screen of tiles like the original (`MenuScreen::Main`, `TILES` in src/menu.rs); Local Battle opens the mode menu, Esc goes back. Letter text takes `FontSize::Vh`. Tile centres are laid out from the sizes until the anchors are read.
- 2026-10-04: pause menu (`MenuScreen::Pause`: Resume, Retry, Quit; Esc or Start) and the "Game Completed" popup (src/popup.rs: place from `gameplay::placings`, Next match, Rematch, Main menu; no XP yet). Systems that change `MenuScreen` on a key press run after `MenuSystems`, or Enter gets read twice.
- 2026-10-04: title tiles at the original's exact centres with grey/colour front art, overlay, turning rays and pulsing corners (`GameConfig::menu`); lanes from the original's screen split (`GameConfig::lanes`, replaces `tower_spacing`); the loser veil is two mirrored strips beside the winner (docs/research/menus.md, match-moments.md).
- 2026-10-04: Select Players and Select Mode (src/select.rs, `Lobby`) replace the text mode menu; Local Battle → seats → modes, Single Player → Endless. Menu input systems run in reverse screen order so one press never acts twice. `--menu players|modes|single` for pictures.
- 2026-10-04: "Press Enter to start" screen with the logo before the title tiles (`MenuScreen::Start`; tests tap Enter past it). Veil strips flipped the right way, corner pulse 5 px linear. Blink letter text by hiding a parent box, not by changing TextColor (that rebuilds the letters every frame).
- 2026-10-04: mods folder (src/mods.rs, docs/modding.md): config changes merge field by field at the RON text level (ron::Value loses enum names, so don't round-trip through it), art packs register a `mods://` source and replace manifest entries.
- 2026-10-04: lost bricks splash in the water: gameplay writes a `BrickLost` message from both loss points, effects.rs draws `splash` art or droplets (`GameConfig::splash`, guessed).
- 2026-10-04: Select Mode rebuilt as the original's two columns (mode, difficulty; `Lobby::column`), Select Players with 9-sliced panels, seat rays and per-pose wizard spots; Pro shows as "Special". One `Part` component drives every select-screen node.
- 2026-10-04: lost bricks float like the original: `sink` takes them out of play (no `Brick`, a `Sunk` marker), nearly stops them and lifts them with `GameConfig::water`; `drain_water` removes them later. The splash is the original's particles (`splash_1..3` fronts, white disc drops, `sfx_splash_1..4`).
- 2026-10-04: Single Player screen (seat over Trials/Endless) and Endless screen (Race/Survival/Puzzle beside a this-run leaderboard) in src/single.rs; only Endless Race plays. Letter text hides with its parent's Visibility, not its own (the select join text showed on every seat).
- 2026-10-04: Game Completed popup slides into the bottom-right corner (Rematch, Select Mode, Main menu) beside WINNER!; pause is a left column over a dark overlay; the top-left debug text only shows after F3. `--menu pause` for pictures.
- 2026-10-04: Survival Pro curse waves from the original's 4 fixed tables (`WaveRules::levels`, `WaveDeck`): whole seconds, gap counted from a wave's end, the brick in hand cursed too, Mist and Ghost once per wave. The 1 s pause at a wave's start isn't done.
- 2026-10-04: Single Player and Endless screens laid out from the PC readout (pack panel, sliced list buttons with cup band and corners, Endless column on the right, leaderboard on the left with `menu_leaderboard_*` art). Curse-wave Ghost strikes with every new brick (`TowerState::wave_ghost`, `ghosts_due`, `Placed` order stamp). Base shapes still wait on the PC readout.
- 2026-10-04: bases built from the PC's collider readout (`GameConfig::bases`, compound box colliders; Race house 6 wide with chimney and under-water ledges, Survival castle with merlons and holes); AI/Auto Build use `base_half_width`. Curse waves lift and hold the brick in hand 1 s, shaking (`WaveRules::hold`, `shake` guessed). The narrower base shifted AI timing, so `bubble_lifts_the_last_brick` now casts right after a placement.
- 2026-10-04: losing towers fall apart before the popup (src/collapse.rs: wait, bricks turn to sensors, every 3rd blows up, rumble, thud; effects.rs draws shards, shakes the camera, plays `sfx_explosion`/`sfx_rumble`/`sfx_brick_heavy`). Losses after the match ends no longer change lives or the score. Endless leaderboard saved to `single::save_dir()` (not in offscreen runs or tests).
- 2026-10-04: src/casts.rs from the PC's stage.ps1 notes: dark cast beams (up, down onto each victim, lane wave), light cast lightning, spell clouds beside each wizard (Show/Float/Used/Hide), Large grows the brick in hand over 0.3 s with a shake (draws the new size, scales the cell sprites up from the old one). Timings in `GameConfig::casts`; `(?)` marks sizes still to read.
- 2026-10-04: XP and wizard levels as data (`XpRules` in config.rs: XP per place by difficulty, level thresholds, names), tested; not shown anywhere until cups exist.
- 2026-10-04: Endless Survival and Endless Puzzle playable from public descriptions (`ModeKind::EndlessSurvival`/`EndlessPuzzle`, `ModeKind::rules()` maps them to Survival/Puzzle rules, `look()` to their art). No brick limit; every specific value is a guess in `GameConfig::endless_play`, listed for the PC in pc-tasks.md. `--play endless-survival|endless-puzzle`. Leaderboard keeps one board per Endless mode.
- 2026-10-04: controller rumble for losing players (input.rs `rumble_losers`, `CollapseKind::Begin`). Trials engine: `GameConfig::trials` (`TrialDef`, 3 sample trials marked as guesses), `config.trial` picks one and `TrialDef::apply` runs in `start_match` after the difficulty; src/trials.rs runs the race clock and the enemy wizard (`EnemyCasts`, logged as `spells::ENEMY_WIZARD`) and saves beaten trials; src/trials_menu.rs is the list screen (`MenuScreen::Trials`, `--menu trials`).
