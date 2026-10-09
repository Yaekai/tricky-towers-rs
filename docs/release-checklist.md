# Release checklist

Things in the code that are not in the original game. They stay for now as development and
test tools; remove them (or switch them off) before the final build. Every one is marked in
the code with a `DEBUG-TOOL` comment, so `grep -rn DEBUG-TOOL src` finds them all.

| Tool | Where | Why it exists |
|---|---|---|
| R restarts a match | `src/input.rs` (`RESTART`) | Quick retries while testing. The original restarts through Pause → Retry or the popup's Rematch. |
| Pad Start restarts a finished match | `src/input.rs` (`read_gamepads`) | Same, for gamepads. The original's popup handles this. |
| F3 debug text | `src/visuals.rs` (`update_hud`) | A text readout of height, spells and results for development. |
| WASD and numpad arrows in menus | `src/menu.rs`, `src/select.rs`, `src/popup.rs` | Numpad arrows are how scripted key presses arrive on Windows; WASD is a habit. Pending the PC check of the original's menu keys. |
| Debug commands and `--script` files | `src/debug.rs`, `--script` in `src/main.rs`, picture hook in `src/offscreen.rs`; docs/debug-tools.md | Set up rare moments exactly in tests and offscreen pictures. |
| F1 debug panel | `src/debug_panel.rs`, the `debug-panel` feature and `bevy_egui` in `Cargo.toml` | Try things by hand while playing. Build with `--no-default-features` to leave it out. |
| Command-line options | `src/main.rs` (`--play`, `--difficulty`, `--offscreen`, `--shots`, `--menu`, config path) | Scripted checks and offscreen pictures on the PC. |
| Frame-time probe | `tests/perf.rs` (ignored test) | Times one frame per mode with four AI towers, to check speed work. Delete the file. |

Not tagged, decide before release:
- The `mods/` folder (`src/mods.rs`, `docs/modding.md`): not in the original, but "easy to mod"
  is one of this project's goals.
- The next-brick preview (`src/visuals.rs`): not yet confirmed in the original (queued for the PC).
| Galaxy frame switch | `src/packs.rs` (`GalaxyDebug`), `src/debug.rs` (`GalaxyFrame`) | Turns the Galaxy cell lines off and pauses the starfield's scroll, for a picture of one piece on the PC's black-bar test. |
| Start with a brick pack | `src/debug.rs` (`Start`, `pack`) | Every tower uses one brick pack in a scripted match, so Galaxy and the others can be pictured without Select Players. |
