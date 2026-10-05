# Plan: Tricky Towers in Rust + Bevy

**Goal:** a 1:1 rebuild of the original: the same rules, values, menus, look and sound, with the
art and sound loaded from the player's own install, and easy to mod. Where a value isn't read
from the original yet, the code marks it `(?)` and `docs/pc-tasks.md` lists it for checking.

## Approach: clean-room, rules first
Following the "Rust rewrites and ports" guide's order of preference:

1. **Public knowledge first.** The rules of Tricky Towers are well documented by play, wikis and
   videos. Most of the game (modes, spells, brick behaviour) can be rebuilt from that alone.
2. **Measure, don't copy.** Where feel matters (fall speed, friction, nudge timing, wobble),
   play the original next to this build and tune `game.ron` until they match. Write the numbers
   and how they were measured in `docs/research/`.
3. **Only if needed, look inside your own install.** The original is a Unity game (verify on your
   install: a `*_Data` folder and `UnityPlayer.dll`). If tuning by eye is not enough, Unity's own
   data files and, for Mono builds, the game's managed assembly can be inspected locally with
   standard tools (AssetRipper/UABEA for data, ILSpy/dnSpy for logic). Rules for that work:
   it stays on your machine in the ignored `research/` folder, only plain-language notes come
   into the repo, and nothing is redistributed. Leave DRM and online services alone.
4. **Assets.** The repo ships placeholder art. `tools/extract/` (PowerShell, see its README)
   converts sprites, sounds, the letters and the wizards from the player's own install into
   `extracted/` (gitignored) for the engine to load at runtime. Mods can also ship their own art.

## Architecture
| Layer | Where | Swap it to... |
|---|---|---|
| Config (physics feel, bricks, mode rules) | `src/config.rs`, `assets/config/game.ron` | tune or add bricks without code |
| Core loop (headless) | `src/gameplay.rs`, `src/spells.rs`, `src/ai.rs` | new modes, spells, AI |
| Trials, cups, match end (headless) | `src/trials.rs`, `src/trial_list.rs`, `src/cups.rs`, `src/collapse.rs` | new trials, cup rules |
| Input | `src/input.rs` writes `PlayerInput` | gamepad, bots, replays, netplay |
| Art and sound loading | `src/art.rs`, `src/letters.rs`, `src/mods.rs` | art packs, other installs |
| In-match look | `src/visuals.rs`, `src/hud.rs`, `src/wizards.rs`, `src/packs.rs`, `src/weather.rs` | themes, brick packs |
| Effects | `src/effects.rs`, `src/casts.rs`, `src/particles.rs`, `src/results.rs` | splashes, beams, banners |
| Menus | `src/menu.rs`, `src/start.rs`, `src/select.rs`, `src/single.rs`, `src/trials_menu.rs`, `src/settings.rs`, `src/popup.rs`, `src/cups_menu.rs` | new screens |
| Debug tools (removed before release) | `src/debug.rs`, `src/debug_panel.rs`, `src/offscreen.rs` | scripted pictures, testing |

Physics: avian2d (Bevy-native, deterministic fixed step). The controlled brick is kinematic and
moved with shape queries for crisp control; on contact it becomes dynamic.

## Milestones
1. **Core loop** (done): spawn, steer, rotate, land, physics hand-off, camera, race finish,
   survival lives, dropped bricks, restart.
2. **Opponents, spells, puzzle, menu** (done).
3. **Match the original's rules** (mostly done, notes in `docs/research/`): controls, physics,
   spell awards and all 17 spells, Survival grace and countdown, Puzzle lasers, floors and
   scoring, difficulty levels, Survival Pro curse waves, cursed-brick respawn, Endless Race (all
   values from the original). Race Pro wind and the wider Puzzle lane, from the original.
   Survival Pro curse waves follow the original's four tables. Losing towers fall apart before
   the popup (`src/collapse.rs`). Endless Survival and Endless Puzzle, local cups (a debug
   extra: first to 6/12/18 points with overtime) and the pad defaults follow the PC readouts.
   XP and wizard levels are in as data (`XpRules`); only online play gives XP in the original.
   Still to do: Endless Puzzle obstacle blocks, the Survival roof (both wait on readouts queued
   in `docs/pc-tasks.md`).
4. **Local multiplayer** (done): up to 4 players, keyboard and gamepads.
5. **The original's look and sound** (in progress): loaded at runtime from the player's own
   install via the gitignored `extracted/` folder (`docs/art-layout.md`). Done: bricks, bases,
   backgrounds, skies, water, lines, panels, letters, animated wizards, music and sounds,
   countdown, win/lose moments, the start screen, title tiles, player and mode select, pause,
   the Game Completed popup, the lost-brick splash, vines growing, the Slow balloon, Ghost
   skulls, cast beams, lightning, spell clouds and the Large grow (`src/casts.rs`), tower
   collapse blasts and shakes, wizard and brick-pack skins, Race Pro rain and lightning, Puzzle
   sandstorm and eclipse, Survival's wave sky, spawn puffs and landing dust, the Single Player,
   Endless, Trials and Settings screens. None of the newer screens has been pictured yet (the
   cloud has no GPU). To do: check the cast effects and collapse against the original
   on Max's PC and fix the `(?)` sizes, the Auto Rotate wind-up, stone and ice transitions, the
   decorations at a beam's foot, and the explosion art.
6. **Trials** (mostly done): the original's 50 trials in 5 tiers opened by medals
   (`src/trial_list.rs`, also editable in `game.ron`), with goals, fixed brick orders, their own
   spell lines and the enemy wizard's curses. To do: trial floors and rival towers, the tier
   unlock animation.
7. **Mod loading** (done): a `mods/` folder of RON config changes and art packs, merged at
   startup in name order (`src/mods.rs`, `docs/modding.md`).
8. **Not planned yet:** Online battle, Shop, Help, the Stats/Credits/Keybinding popups, text in
   languages other than English.

## How work is split
- Code is written in the cloud on a `claude/*` branch and merged into
  `features/spells-puzzle-menu` through a pull request once `cargo test` passes; that branch
  is merged into `main` (the GitHub front page) from time to time.
- A Claude session on Max's PC follows `docs/pc-tasks.md` after every pull: build, test,
  re-stage `extracted\` from the install, screenshot each mode with `--play`, and report. Queue
  PC work by adding to that file.
