# Release checklist

Things in the code that are not in the original game. They stay for now as development and
test tools; remove them (or switch them off) before the final build. Every one is marked in
the code with a `DEBUG-TOOL` comment, so `grep -rn DEBUG-TOOL src` finds them all.

| Tool | Where | Why it exists |
|---|---|---|
| R restarts a match | `src/input.rs` (`RESTART`) | Quick retries while testing. The original restarts through Pause → Retry or the popup's Rematch. |
| Pad Start restarts a finished match | `src/input.rs` (`read_gamepads`) | Same, for gamepads. The original's popup handles this. |
| F3 debug text | `src/visuals.rs` (`update_hud`) | A text readout of height, spells and results for development. |
| Local cups | `src/cups.rs`, `src/cups_menu.rs`, the C / Select picker on Select Mode, "Next match" in `src/popup.rs` | Cups are an online feature in the original (docs/research/wiki-notes.md). Kept to test cup rules, the trophy and XP before online play exists. |
| WASD and numpad arrows in menus | `src/menu.rs`, `src/select.rs`, `src/popup.rs` | Numpad arrows are how scripted key presses arrive on Windows; WASD is a habit. Pending the PC check of the original's menu keys. |
| Command-line options | `src/main.rs` (`--play`, `--difficulty`, `--offscreen`, `--shots`, `--menu`, config path) | Scripted checks and offscreen pictures on the PC. |

Not tagged, decide before release:
- The `mods/` folder (`src/mods.rs`, `docs/modding.md`): not in the original, but "easy to mod"
  is one of this project's goals.
- The next-brick preview (`src/visuals.rs`): not yet confirmed in the original (queued for the PC).
