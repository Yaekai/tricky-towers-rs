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
