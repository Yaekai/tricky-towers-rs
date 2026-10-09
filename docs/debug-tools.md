# Debug tools

DEBUG-TOOL: none of this is in the original game. It is here to test and picture the game,
and goes before the final build (docs/release-checklist.md).

## Debug commands (src/debug.rs)
One set of commands drives everything below. Each runs at the start of the next fixed step:
- `Start(mode: Race, difficulty: Some(Pro), players: Some(1), opponents: Some(2))`: start or
  restart a match (unset fields keep the current config).
- `Press(tower: 0, key: CastDark)`: press a button for a tower once. Keys: `Left`, `Right`,
  `NudgeLeft`, `NudgeRight`, `RotateRight`, `RotateLeft`, `Drop`, `CastLight`, `CastDark`.
- `Hold(tower: 0, key: Left, on: true)`: hold or let go of `Left`, `Right` or `Drop`.
- `GiveLight(tower: 0, spell: Ivy)`, `GiveDark(tower: 0, spells: [Large, Ice])`.
- `Curse(spells: [Mystery])`: curse every tower at once, like a trial's enemy wizard.
- `SetLives(tower: 0, lives: 1)`, `Win(tower: 0)`, `Lose(tower: 1)`.
- `Tap(key: "Enter")`: tap a menu key (`Enter`, `Escape`, `Tab`, `Space`, `Up`, `Down`, `Left`, `Right`, `C`).
- `Picture(name: "after_curse")`: save `<name>.png` (offscreen runs only).
- `Quit`.

In tests, push commands into the `DebugQueue` resource
(see `debug_commands_and_scripts_drive_a_match` in tests/headless.rs).

## Script files
A RON list of `(at: seconds, run: Command)`, run in time order from the moment the game starts.
Battles open with their intro (mode title, Race's camera sweep, 3-2-1; src/intro.rs), so play
starts about 6.5 s after a Race `Start` (4.5 s in Survival, 3.5 s in Puzzle):

```ron
// Start a race with one AI opponent, curse both towers with Large + Ice,
// take two pictures, quit.
[
    (at: 0.5, run: Start(mode: Race, difficulty: Some(Normal), opponents: Some(1))),
    (at: 2.0, run: Curse(spells: [Large, Ice])),
    (at: 2.4, run: Picture(name: "curse_large_ice_0")),
    (at: 4.0, run: Picture(name: "curse_large_ice_1")),
    (at: 6.0, run: Quit),
]
```

Run it silently on the PC (no window, no sound), saving the pictures in `screenshots\`:

    tricky_towers.exe --script my_script.ron --offscreen screenshots

`--size 3440x1440` sets the picture size (1280x720 if left out).

With `--script`, an offscreen run doesn't quit after its `--shots`; the script's `Quit` ends
it. Keep script files outside the repo, or paste them in docs like this one: the repo's
whitelist `.gitignore` doesn't take loose `.ron` files outside `assets/config/`.

## The F1 panel (src/debug_panel.rs)
In a window, F1 opens a panel: start any mode at any difficulty with N AI opponents, give,
cast or curse any spell on any tower, set lives, make a tower win or lose, and a live line per
tower (status, height, bricks, lives, spells). It is the `debug-panel` Cargo feature, on by
default; `cargo build --release --no-default-features` leaves it (and egui) out.
