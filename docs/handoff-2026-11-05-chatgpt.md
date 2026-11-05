# ChatGPT handover · 2026-11-05

## Starting point

Resume the clean-room Rust/Bevy rewrite of Tricky Towers on `main` at `4c2003a`
(the merge of PR #175) or later. Check `git status --short --branch` and
`git log -1 --oneline` first. Read `AGENTS.md`, then `PLAN.md`, `README.md` and the
[active backlog](WHATS_NEXT.md). The goal is a 1:1 match with the original
(rules, values, layout, art staging), not a loose clone.

The Windows checkout is `C:\Users\maxen\Desktop\Claude\tricky-towers-rs`. It has
no tracked changes. `.agents/` is local and untracked. The stash “Preserved
unfinished concurrent-audio work before resuming 2026-11-03” holds an older
audio experiment: leave it alone unless a task needs it.

## What landed today

| PR | Behaviour |
| --- | --- |
| [#167](https://github.com/Yaekai/tricky-towers-rs/pull/167) | Puzzle battles split the screen: back camera for the scene, one camera per tower, main camera last for front sand and UI (`src/lanes.rs`) |
| [#169](https://github.com/Yaekai/tricky-towers-rs/pull/169) | WINNER! shows once in Puzzle battles |
| [#170](https://github.com/Yaekai/tricky-towers-rs/pull/170) | Debug scripts: `Hold` stays on over the keyboard and pads (it used to last one step); first PC pass notes |
| [#171](https://github.com/Yaekai/tricky-towers-rs/pull/171) | Puzzle lanes fit GamePositionHelper's 16:9 box (21:9 pillarboxed, 5:4 and 16:10 letterboxed); towers 400 cells apart so a zoomed-out lane never shows a neighbour; a laser per lane |
| [#172](https://github.com/Yaekai/tricky-towers-rs/pull/172) | Split Puzzle HUD cards hang from the box's top (the original anchors each HUD to its tower viewport) |
| [#173](https://github.com/Yaekai/tricky-towers-rs/pull/173) | Split Puzzle HUD sizes from the box's height (HUD canvas: Scale With Screen Size, 1920x1080, Expand) |
| [#174](https://github.com/Yaekai/tricky-towers-rs/pull/174) | Forcefield (BrickClipper) at each split Puzzle lane camera's edges |
| [#175](https://github.com/Yaekai/tricky-towers-rs/pull/175) | Shakes move only the affected tower's lane camera in split Puzzle battles |

`cargo test --locked -j 1` passed (209 tests) before each merge. Silent offscreen
checks with the staged art covered 2-4 seats at 1280x720, 1920x1080, 3440x1440 and
1280x1024. Each tower stays centred in its lane as the view zooms out, with no
neighbour bleed. Lasers, drop guide, front sand, heat haze, cards and banners line
up per lane. Lost-brick dust stays in its lane. Trial 9, Endless Puzzle, Race and
Survival are unchanged. The details are in [What's next](WHATS_NEXT.md) item 1.

## Next, in order

1. **Finish backlog item 1.** What's left needs Max at a window: 125% Windows scaling
   and window resizing on a split Puzzle battle (offscreen runs always render at
   1x). A real wide stack only zooms past about 30 cells (2 seats), 19 (3) or 13 (4)
   at 16:9. To picture a zoom, temporarily set `puzzle_laser: (30.0, 0.0)` and
   `large_scale: 4.0` in `assets/config/game.ron`, then revert. Never commit that.
2. **Race and Survival in the 16:9 box** (read only so far). The original gives
   them the same 16:9-fitted per-tower viewports, cameras and HUD anchoring as
   Puzzle. Each tower also has a background and a foreground camera, centred on
   its lane but as wide as the screen allows. Ours draws both modes with one
   full-screen camera, so 21:9 and 5:4 differ. Decide with Max before changing it.
3. Backlog items 2-5: the Trial 9 stack-height mismatch (keep
   `puzzle.trial_laser_slack` at 0.3 until a demonstrated fix), spell and results
   timing, the display/text/controller pass, then the release checklist.

## How to work here

- Build and test with `cargo test --locked -j 1` (parallel builds ran out of
  memory). `CARGO_PROFILE_DEV_OPT_LEVEL=0` makes local compiles faster.
- Max is often using this PC, so keep everything headless and silent: use
  `--script <file> --offscreen <scratch dir>` (and `--size WxH`), never a window
  or sound. Run one game process at a time, because four at once ran the GPU out
  of memory. The script commands are in [debug tools](debug-tools.md). Gameplay
  starts about 3.5 s after a Puzzle `Start`, and `Hold` now stays on until let go.
- `extracted/` (staged art) and `research/` (the original's ripped scripts and
  scenes) stay on the PC only. Never commit them, original code, or pictures of
  staged art. Write the original's behaviour as plain notes in `docs/`.
- Check the original first (`research/ripped/ExportedProject/Assets/...`) before
  changing a rule. Keep tunables in `GameConfig` / `assets/config/game.ron`.
- Max lets the agent merge its own PRs, but only once `cargo test` has passed in
  the same step. `gh` isn't logged in on this PC: get `GH_TOKEN` from
  `git credential fill` and run `C:\Program Files\GitHub CLI\gh.exe`. Add a dated
  line to the log at the end of `AGENTS.md`. Don't build a release `.exe` unless
  Max asks.

## Paste into ChatGPT

> Read `docs/handoff-2026-11-05-chatgpt.md`, `AGENTS.md` and the active backlog at
> the top of `docs/WHATS_NEXT.md`. Verify `main` and resume the clean-room Tricky
> Towers Rust rewrite, matching the original 1:1. Keep runs silent and offscreen,
> check the original under `research/` before changing a rule, run
> `cargo test --locked -j 1` before every merge, and add a dated line to the log
> in `AGENTS.md`.
