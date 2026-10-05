# Tricky Towers, rebuilt in Rust + Bevy

A clean-room, 1:1 rebuild of WeirdBeard's physics brick-stacking game *Tricky Towers*, made easy
to mod. No code or assets from the original are in this repo: the art, sound and letters are
loaded at runtime from your own install (without them the game draws plain shapes).

| | |
|---|---|
| ![Title screen](docs/shot-title.png) | ![Select Players](docs/shot-players.png) |
| ![A race against three AI wizards](docs/shot-race4.png) | ![Survival Pro at night](docs/shot-survival.png) |

![Puzzle Pro under the eclipse](docs/shot-puzzle.png)

## Run
```
cargo run                                   # play
cargo run -- --play survival --difficulty pro   # skip the menu
cargo test                                  # config and headless gameplay tests
```
To use the original's art and sound, export your install with AssetRipper and run the scripts
in `tools/extract/` (steps in its README). They fill the gitignored `extracted/` folder.

**Keys** (the original's defaults): arrows move, Up / Space turn, Down drops faster, U / I nudge,
J light spell, K dark spell, Esc pause. Players 2 to 4 use gamepads.

## How close to 1:1 it is
✅ matches the original (values read from the install) · 🟡 in, with gaps listed · ❌ not done

| Area | | Left to reach 1:1 |
|---|---|---|
| Controls, physics, drop speeds | ✅ | |
| Race, Survival, Puzzle at Easy / Normal / Pro | ✅ | |
| Spells (17) and how they're awarded | ✅ | Auto Rotate wind-up, stone and ice transitions |
| Local multiplayer (1-4 players) | ✅ | |
| Wizards (14), animations and voices | ✅ | |
| Brick packs (8) | 🟡 | Diamond's gloss angle sweep (needs the original shader); Spring's look |
| Trials (50 in 5 tiers) and tier unlock | 🟡 | draw the enemy wizard; the mode stinger at the start |
| Endless Race / Survival / Puzzle | 🟡 | Endless Puzzle obstacle blocks |
| Backgrounds and weather | 🟡 | Race Pro: softer lightning flash and the wave rows |
| In-match HUD and effects | 🟡 | remove the extra spell bubbles; cast beam foot decorations; explosion art |
| Match end (WINNER!, collapse, Game Completed) | 🟡 | second sunray layer |
| Menus: start, title, players, modes, single player, endless, trials, settings, pause | 🟡 | key caps cut "Enter"; Stats shows an extra BACK; the cup screen's layout |
| Tutorial (explanation) and Credits popups | ❌ | layouts read (`docs/research/menus.md`), not built |
| Text from the original's language files (19 languages) | ❌ | format read (`docs/research/text-keys.md`); menus use built-in English |
| Values still guessed | 🟡 | 43 marked `(?)` in `src/` |
| Online battle, Shop, Help | ❌ | not planned |

Not in the original, kept as extras: AI wizards in battles, local cups, and the debug tools
(F1 panel, F3 readout, scripts), all listed in `docs/release-checklist.md`.

## More
- `PLAN.md`: approach, architecture, milestones. `docs/research/`: what was read from the original.
- `docs/debug-tools.md`: offscreen pictures and scripted runs. `docs/modding.md`: the `mods/`
  folder (config changes and art packs).

## Credits
*Tricky Towers* is by WeirdBeard Games. This is an unaffiliated fan rebuild; to play the real
thing, buy it. Built with [Bevy](https://bevyengine.org) and [avian](https://github.com/Jondolf/avian);
art is exported from your own install with [AssetRipper](https://github.com/AssetRipper/AssetRipper).
The approach follows the [AI Game Modding Guides](https://github.com/trevaintdead/ai-game-modding-guides)
and [universal-modder](https://github.com/rehan-remade/universal-modder).
