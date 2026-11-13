# Release checklist

Development controls and diagnostics to remove or switch off before the final
build. Use `rg -n DEBUG-TOOL Cargo.toml src tests` to find their entry points.

| Tool | Where | Why it exists |
|---|---|---|
| R restarts a match | `src/input.rs` (`RESTART`) | Quick retries while testing. The original restarts through Pause → Retry or the popup's Rematch. |
| Pad Start restarts a finished match | `src/input.rs` (`read_gamepads`) | Same, for gamepads. The original's popup handles this. |
| F3 debug text | `src/visuals.rs` (`update_hud`) | A text readout of height, spells and results for development. |
| WASD and numpad arrows in menus | `src/menu.rs`, `src/select.rs`, `src/popup.rs` | Numpad arrows are how scripted key presses arrive on Windows; WASD is a habit. Pending the PC check of the original's menu keys. |
| Debug commands and `--script` files | `src/debug.rs`, `--script` in `src/main.rs`, picture hook in `src/offscreen.rs`; docs/debug-tools.md | Set up rare moments exactly in tests and offscreen pictures. |
| F1 debug panel | `src/debug_panel.rs`, the `debug-panel` feature and `bevy_egui` in `Cargo.toml` | Try things by hand while playing. Build with `--no-default-features` to leave it out. |
| Command-line options | `src/main.rs` (`--play`, `--difficulty`, `--offscreen`, `--shots`, `--menu`, config path) | Scripted checks and offscreen pictures on the PC. |
| Galaxy frame switch | `src/packs.rs` (`GalaxyDebug`), `src/debug.rs` (`GalaxyFrame`) | Pauses the Galaxy starfield's scroll, for a still picture of one piece. |
| Start with a brick pack | `src/debug.rs` (`Start`, `pack`) | Every tower uses one brick pack in a scripted match, so Galaxy and the others can be pictured without Select Players. |
| Start with chosen wizards | `src/debug.rs` (`Start`, `wizards`) | Each tower's wizard in a scripted match, so the Animal and Indie wizards' beams can be pictured without Select Players. |
| Pack line | `src/debug.rs` (`PackLine`) | Spawns one static brick of each shape in a row, in one brick pack (iced with `ice: true`), for side-by-side pictures against a reference. |
| Language command | `src/debug.rs` (`Language`) | Sets the menu language from a script, for pictures of the translated menus. |
| Asset lookup log | `src/art.rs`, `src/debug.rs`, `src/main.rs` | Lists loaded assets with `TT_LOG_ASSETS` or the executable's `log_assets.txt` switch. |
| Trial brick log | `src/gameplay.rs` (`track_towers`) | Records the height of bricks crossing the trial laser for the Trial 9 investigation. |

The frame-time probe in `tests/perf.rs` and pause automation in
`tests/pause_script.rs` run from the test suite. Keep their checks available
while preparing the release.

## Offline release checks

- Run the full test suite. Smoke-test the release build with placeholder art,
  then with locally staged art and sound.
- Verify scaling and resize behavior on Windows and check the controller glyphs,
  rumble and pause audio mix on hardware.
- Review the [remaining parity work](WHATS_NEXT.md) and record open release
  decisions for local AI seats and mod tooling.
- Check that the distribution excludes locally staged art, research exports,
  and personal settings or save files.
