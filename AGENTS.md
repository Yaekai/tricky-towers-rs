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
