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
- Anything not in the original (dev and test tools) is marked `DEBUG-TOOL` in the code and
  listed in `docs/release-checklist.md`, to remove before the final build. Mark new ones the same way.
- Tunable numbers belong in `GameConfig` (`src/config.rs`) and
  `assets/config/game.ron`, not as literals in systems.

## Token-saving tools
Max wants Ponytail and Graphify in use on this project at all times, and OmniRoute as a fallback;
install and setup notes are the first section of `docs/pc-tasks.md`. In short:
- Ponytail: smallest working change. The 1:1 rule above still wins over a shorter version.
- Graphify: query the graph (`graphify query`, `graphify explain`, `graphify path`) before
  reading whole files; rebuild it after big changes. Never graph `extracted/` or `research/`.
- OmniRoute: only after the daily Claude limit is hit, to finish a cut-off task; all providers
  on. Never send `extracted/`, `research/` or game files through it. Log that session's line
  with "via OmniRoute" so the next Claude session reviews the work before building on it.
If Ponytail or Graphify isn't active in a session (a cloud container, a fresh machine), say so at the
start instead of working around it silently.

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
- 2026-10-04: local cups (src/cups.rs: `Cup`, points per place, standings, XP into `XpBook` saved as xp.ron; src/cups_menu.rs: C/Select picks a cup on Select Mode, standings line, trophy pop). The popup's first item reads Next match while a cup runs. All cup rules are guesses listed in pc-tasks.md.
- 2026-10-04: Settings screen (src/settings.rs: Volume, Music, Full screen; `GlobalVolume`, the music's `AudioSink` and the window mode follow it; saved as settings.ron). Placeholder list until the original's is read.
- 2026-10-04: 1:1 fixes from the notes: Auto Rotate winds back and swings (`gameplay::auto_rotate_angle`, `MagicRules::auto_rotate_back/swing`), No Rotate shakes a refused turn (`ActiveBrick::refused`, drawn in casts.rs), Race Pro award rules (`AwardRule::difficulties`), line wait per player only on Normal/Pro (`line_wait_extra_on`), wizard Action → Cheer on the Survival roof. Two spell-line rules wait on the PC (pc-tasks.md).
- 2026-10-04: wiki facts applied (to be checked on the PC): original default keys (U/I nudge, J/K magic, Space turns left; old keys commented out in input.rs), separate Sound effects / Music volumes at 100% and full screen on by default (`set_effect_volume`), trial tiers with medal unlocks (`TrialTiers`), Endless Puzzle laser rising per group of 10 bricks. Sources and notes in docs/research/wiki-notes.md.
- 2026-10-04: audit of extras: kept as debug tools, tagged `DEBUG-TOOL` and listed in docs/release-checklist.md (R restart, pad Start restart, F3 text, local cups, WASD/numpad in menus, command-line options).
- 2026-10-04: start screen intro in src/start.rs from the MoveIn clip (`GameConfig::start`, keyed values): logo drop and bounce, brick landing with dip, tilt and rubble hop, prompt after 2.62 s blinking 0.5/0.5. Falls back to the combined `logo`. Menu tests run without `Assets<Image>`, so it's an Option.
- 2026-10-04: debug tools (DEBUG-TOOL): `DebugCommand`s through `DebugQueue` (src/debug.rs), timed `--script file.ron` runs with `Picture`/`Quit` for offscreen pictures, and the F1 egui panel (src/debug_panel.rs, `debug-panel` feature, bevy_egui 0.42). docs/debug-tools.md explains them.
- 2026-10-04: in-match look from the PC's video and press kit notes (src/hud.rs, `GameConfig::hud`): no bottom panels; next-brick card from the top of each lane with the Survival count, 1ST/2ND or hearts under the wizard, spell bubbles beside it; full-height pale drop guide per tower; spell lines are one-colour diamond rows; close camera (`hud.camera_cells`) following the tower tops, pulling back at the match end; roof dropped on the winner, losers turn pale and their wizards shrink away; leader's hold countdown gold, others pink. Curse waves lift the brick to 4 cells under the view top over 1 s (collisions off), hold 1 s, random shake 0.1 cell (`WaveRules::lift`, `config.camera_top`).
- 2026-10-05: skins from skins.md: `BrickPack` (8) and 14 wizards picked per seat on Select Players (Up/Down row, Left/Right value; `Lobby::skins`, `config.skins` by tower index), `TowerSkin` on each tower; bricks, the next card and wizards draw with them, falling back to default art.
- 2026-10-04: the original 50 trials (src/trial_list.rs from docs/research/trials.md): fixed brick orders, per-trial spell lines (auto-cast "(random)" ones via `TowerState::auto_cast_light`), enemy curses on heights and bricks (`CastWhen::Height`/`Brick`), a dropped brick fails a puzzle trial. Trial floors and rival towers wait on the PC.
- 2026-10-04: Endless Survival/Puzzle from the PC readout (`EndlessPlay::apply`): brick-counted curse waves (`Wave::bricks`, `TowerState::wave_bricks`), Puzzle levels with a rising laser and Undo (`level_up`, `level_end`), one life, score = bricks in the tower (`tower_losses`). `start_match` now restores the loaded mode rules before each match, so trial and Endless changes don't leak into later matches.
- 2026-10-04: cups from the PC readout: `CupSize::target` (first to 6/12/18), overtime on a tie at the top (`Cup::overtime`), no XP for local cups (`cup_xp` removed).
- 2026-10-04: Settings screen rebuilt from the layout readout: two sliding pages (`settings::PAGES`, `Row`), sliders/toggles/choices, page 2 held in `SettingsMenu::pending` until Apply; resolution and V-sync applied to the window. The cloud container has no GPU, so offscreen pictures only work on the PC.
- 2026-10-04: pad defaults from the PC check (A/d-pad up/RT turn right, LT turn left, bumpers nudge, X light, B dark); Settings Quality Low/Medium/High and the 19 languages by name.
- 2026-10-05: Trials screen from the layout readout: tier pages that slide (`TrialPart`), 2×5 trial buttons, medal panel, locked panel; arrows move in columns and turn pages at the edges (`trials_menu::step`). Not yet pictured (no GPU in the cloud).
- 2026-10-05: Ponytail, Graphify and OmniRoute queued for the PC (pc-tasks.md, first section) and set as always-on tools in this file.
- 2026-10-05: OmniRoute changed to a fallback for when the daily limit is hit, with all providers on (Max).
- 2026-10-05: Anthropic's claude-code-setup plugin queued for the PC (pc-tasks.md, first section, item 4): run once, apply only what Max agrees to.
- 2026-10-05: optimisation pass. Measured with 4 AI towers in the dev build: about 4.5 ms CPU per frame in every mode (core loop 1.6 ms), so play speed is fine; a fresh build took 14 min. Bevy now builds without 3D (`default-features = false`, 2d/ui/audio/wav): 12 min fresh build, game file 12% smaller, frames about 18% faster. `tests/perf.rs` (DEBUG-TOOL) prints the frame times. Ponytail/Graphify not active in the cloud.
- 2026-10-05: resumed after the PC ran out of tokens: merged its last 3 commits (trial lanes/rival towers/floors, Select Players pictures, Ivy rigid body; PR #31). Trials tier unlock animation from the readout (`TrialsMenu::unlock`, `TrialProgress::tiers_shown`, exact chain layout) and the CONTINUE/BACK hints on menu screens (src/navbar.rs). Ponytail/Graphify not active in the cloud. The cloud disk filled during the build (two Bevy feature sets in target/); deleting target/debug/incremental freed it.
- 2026-10-05: Settings → Keybinding (src/keybinding.rs): `KeyBindings` resource (saved by key name in keys.ron, swap on clash) now drives player 1's keys in input.rs and the move/rotate sounds; the list opens from the Keybinding row, Esc closes it before leaving Settings.
- 2026-10-05: Game Info popup in a match (src/game_info.rs, Tab / pad Y) and Settings → Stats (src/stats.rs: `Stats` counts player 1's local battles per mode and difficulty, cups won and time played, saved as stats.ron). Bevy's plugin tuples stop at 15, so main.rs adds GameInfoPlugin on its own. Credits and the tutorial popups wait on the text-format readout (pc-tasks.md).
- 2026-10-05: Race Pro background pieces from the readout: water band fallback (`WeatherLook::race_pro_water*`), manifest `drift` (sliding clouds) and `lit` (bright copy shown with the flash, `weather::Flash`) for screen layers. Staging waits on the PC.
- 2026-10-05: trials end on CHALLENGE COMPLETE / FAILED (results.rs), the race-trial clock with an hourglass on the finish line (`TrialClock`, world `Text2d`), `finish_line_end` flags.
- 2026-10-05: Survival curse waves grey the background layers and speed up drifting ones (`weather::WaveMix`, `grey_wave_layers`); fixed: Endless Survival's brick-counted waves didn't shift the sky (it only looked at `wave_left`).
