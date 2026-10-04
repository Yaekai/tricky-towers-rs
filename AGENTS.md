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
