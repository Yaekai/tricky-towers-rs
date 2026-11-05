# ChatGPT handover · 2026-11-05 (evening)

## Starting point

This handover resumes the clean-room Rust/Bevy rewrite of Tricky Towers on `main`. The baseline is
`4c2003a`, the merge of PR #175. Start by running `git status --short --branch` and
`git log -1 --oneline`. Then read `AGENTS.md`, `PLAN.md`, `README.md` and the
[active backlog](WHATS_NEXT.md), whose ordered list is the current plan.

The goal is a 1:1 rewrite: the same rules, values, menus, look and sound as the original. Max's
own Steam install is the reference. Original code and assets never enter the repo. Art and
sound load at runtime from a local, git-ignored `extracted/` folder. Readouts of the original
stay in git-ignored `research/` on the PC, and only plain-language notes go into
`docs/research/`.

## What landed today (backlog item 1: multiplayer Puzzle viewports)

| PR | Behavior |
| --- | --- |
| [#167](https://github.com/Yaekai/tricky-towers-rs/pull/167) | `src/lanes.rs`. Puzzle battles with 2 or more towers split the screen. A back camera draws the scene, each tower's lane camera draws its GamePositionHelper rectangle, and the main camera draws the front sand and the UI last. HUD, scores and banners add `LaneViews::shift`. Grab-pass shaders sample by whole-picture UV. |
| [#168](https://github.com/Yaekai/tricky-towers-rs/pull/168) | PC check queue for the split screen (`docs/pc-tasks.md`) |
| [#169](https://github.com/Yaekai/tricky-towers-rs/pull/169) | A Puzzle battle's WINNER! follows the brick count only, so a debug `Win` no longer adds a second one |
| [#170](https://github.com/Yaekai/tricky-towers-rs/pull/170) | Script `Hold` commands stay held over keyboard and pad input (debug tool). Records the first real-art PC pass. |
| [#171](https://github.com/Yaekai/tricky-towers-rs/pull/171) | Lanes sit in the original's 16:9 box (`lanes::screen_box`): 21:9 is pillarboxed, 5:4 and 16:10 are letterboxed. Puzzle battle towers are `gameplay::PUZZLE_BATTLE_SPACING` (400 cells) apart, so a zoomed-out lane never shows a neighbour; this stands in for the original's per-tower layers. Every tower draws its own copy of the battle laser. |
| [#172](https://github.com/Yaekai/tricky-towers-rs/pull/172) | The next-brick card hangs from the top of the 16:9 box |
| [#173](https://github.com/Yaekai/tricky-towers-rs/pull/173) | Split HUD items take their size from the box's height (the 1920x1080 reference with Expand matching) |
| [#174](https://github.com/Yaekai/tricky-towers-rs/pull/174) | The Forcefield (BrickClipper) is drawn at each lane camera's edges |
| [#175](https://github.com/Yaekai/tricky-towers-rs/pull/175) | A shake moves only the affected tower's lane camera |

Verification:
- `cargo test --locked` passed on Linux and Windows, and CI was green for each PR.
- Silent offscreen passes with the staged art on PC Max covered 2 to 4 seats at 1280x720, 1920x1080, 3440x1440 and 1280x1024. Each tower stays centred in its lane as the view zooms. Cuts, lasers, HUD, lost-brick dust, Forcefield and shakes are all per lane.
- A Puzzle trial, solo Endless Puzzle, Race and Survival look the same as before.

The details and the original's references are in item 1 of [What's next](WHATS_NEXT.md).

## Still open

- **Item 1 leftovers:**
  - 125% Windows scaling and window resizing with the split screen. This needs a windowed check by Max, because offscreen runs always render at 1x.
  - Race and Survival battles in the 16:9 box. The original gives them per-tower cameras in the same box, plus wider background and foreground cameras per tower. Ours draws them with one full-screen camera. Item 1 has notes on this, and it's best done as its own task.
- **Items 2 to 5 of the backlog**, unchanged:
  - Trial 9 stack-height mismatch. Keep `puzzle.trial_laser_slack = 0.3` until a demonstrated fix.
  - Spell and results timing in play, and the remaining `(?)` values.
  - The display, text and controller pass.
  - The offline release checklist.
- Online play and the Shop are separate or out of scope.

## Working rules

- **The PCs:**
  - PC Max is `C:\Users\maxen\Desktop\Claude\tricky-towers-rs`, used today.
  - PC Hugo is `C:\Users\hugoc\Documents\Claude\tricky-towers-rs`.
  - Both have staged `extracted/` art and `research/` readouts, and both stay local.
  - Leave the untracked `.agents/` folder and the stash "Preserved unfinished concurrent-audio work before resuming 2026-11-03" alone.
- **Headless and silent.** Max uses the PC while work runs on it. Use `--offscreen <dir>` with `--play`, `--script` or `--menu`, and never open a window or play sound. Run one debug build at a time, because four at once ran the GPU out of memory. Scripts are described in [debug tools](debug-tools.md).
- **Tests.** Run `cargo test --locked -j 1` on the PC before merging (parallel builds ran out of memory there). Batch about 6 fixes per round. Merge tested work into `main` via PRs, and record each dated change in `AGENTS.md`.
- **No .exe** unless Max asks for one.
- **Formatting.** Don't run `cargo fmt` on the whole crate. The repo isn't rustfmt-clean, and it rewrites thousands of lines. Format single new files only.
- **Values.** Tunables belong in `GameConfig` / `assets/config/game.ron`. Mark guessed values `(?)`, and mark debug-only code `DEBUG-TOOL`.
- **Evidence before guesses.** When something isn't solved quickly, read the original's scripts and prefabs in `research\` on the PC. Don't guess values from screenshots.

## Paste into ChatGPT

> Read `docs/handoff-2026-11-05-chatgpt.md`, `AGENTS.md` and the active backlog at the top of
> `docs/WHATS_NEXT.md` in Yaekai/tricky-towers-rs. Verify `main` (baseline `4c2003a`). Then
> continue the clean-room 1:1 Tricky Towers Rust/Bevy rewrite with backlog item 2, the Trial 9
> stack-height mismatch, keeping the 0.3 laser slack until a demonstrated fix. If Max asks for it,
> fit Race and Survival battles into the 16:9 box first. Keep PC work headless and silent, run
> `cargo test --locked -j 1` before each merge, and log the dated change in `AGENTS.md`.
