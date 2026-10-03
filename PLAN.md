# Plan: Tricky Towers in Rust + Bevy

**One-sentence goal for v0:** a single-player race against the finish line that feels like the
original: steer a brick down in half-cell steps, it lands, the tower wobbles under real physics.

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
4. **Assets.** The repo ships placeholder art. A later `tools/extract.py` can convert sprites and
   sounds from the player's own install into `extracted/` (gitignored) for the engine to load at
   runtime, the same pattern as gang-beasts-rust. Mods can also ship their own art.

## Architecture
| Layer | Where | Swap it to... |
|---|---|---|
| Config (physics feel, bricks, mode rules) | `src/config.rs`, `assets/config/game.ron` | tune or add bricks without code |
| Core loop (headless) | `src/gameplay.rs` | new modes, spells, AI |
| Input | `src/input.rs` writes `PlayerInput` | gamepad, bots, replays, netplay |
| Visuals | `src/visuals.rs` | real art, themes |

Physics: avian2d (Bevy-native, deterministic fixed step). The controlled brick is kinematic and
moved with shape queries for crisp control; on contact it becomes dynamic.

## Milestones
1. **Core loop** (done): spawn, steer, rotate, land, physics hand-off, camera,
   race finish line with hold timer, survival lives, dropped-brick detection, restart.
2. **Opponents, spells, puzzle, menu** (done): towers side by side with AI wizards, five spells,
   puzzle mode with a laser line, mode menu, drop guide, next-brick preview.
3. **Match the original** (needs Max's copy): side-by-side tuning of fall speed, nudge timing,
   friction and wobble; check which spells exist, what each does exactly and how they are earned
   (currently every 5 bricks), and add the missing ones. Rebuild from observed behaviour.
4. **Trials/campaign** levels described in RON files.
5. **Local multiplayer:** 2 to 4 players on gamepads, each driving a tower's `PlayerInput`.
6. **Presentation:** wizard characters, sound, themes. Optional loader for art and sound from
   your own install into the gitignored `extracted/` folder.
7. **Mod loading:** a `mods/` folder of RON configs and asset packs, merged at startup.
