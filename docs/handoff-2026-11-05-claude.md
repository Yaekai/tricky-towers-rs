# Claude Code handover · 2026-11-05

## Starting point

Resume the clean-room Rust/Bevy rewrite of Tricky Towers on `main`. The verified
implementation baseline is `bee6196`, the merge of PR #165. Start by checking
`git status --short --branch` and `git log -1 --oneline`. Read `AGENTS.md`, then
`PLAN.md`, `README.md` and the [active backlog](WHATS_NEXT.md). Continue item 1
on Claude's existing Puzzle viewport branch.

Claude reported starting that work in the cloud from `bee6196`. The approach
gives each tower a lane camera and keeps the shared background full screen.
Claude is building and testing with silent placeholder-art captures. Continue
that implementation and use the acceptance checks below to review it. The
Hugo PC export and staged art can support a second visual pass when available.

The Windows checkout used for this work is
`C:\Users\maxen\Desktop\Claude\tricky-towers-rs`. It had no tracked working-tree
changes at the baseline. `.agents/` is local and untracked. A stash named
“Preserved unfinished concurrent-audio work before resuming 2026-11-03” contains
an older audio experiment. Leave it in place unless the task calls for it.

## What landed

| PR | Merged behavior |
| --- | --- |
| [#161](https://github.com/Yaekai/tricky-towers-rs/pull/161) | Final-trial Credits, wizard presentation, frozen match clock during pause, and the cup winner appearance poof |
| [#162](https://github.com/Yaekai/tricky-towers-rs/pull/162) | In-match cup standings fade over 0.1 seconds of real time |
| [#163](https://github.com/Yaekai/tricky-towers-rs/pull/163) | Fade to black before screen changes and restore the Start logo rumble sequence |
| [#164](https://github.com/Yaekai/tricky-towers-rs/pull/164) | Fade the full pause UI and reuse private glyph materials during UI fades |
| [#165](https://github.com/Yaekai/tricky-towers-rs/pull/165) | Clip attached Race and Survival decorations at their owning lane's brick bounds |

The baseline passed 204 regular tests on Windows and the standard
[Linux CI job](https://github.com/Yaekai/tricky-towers-rs/actions/runs/42051416948).
The ignored AI spell stress test passed separately after #164. Silent captures
verified the fades and showed no test-colored decoration pixels between lanes
after #165. Physical rumble and GPU frame-time profiling remain unchecked.

## Puzzle viewport acceptance checks

The original allocates a fixed screen rectangle to each local Puzzle tower.
`GamePositionHelper` supplies the margins and gaps. `WidthTowerCamera` uses the
same widest-tower measurement across the match, keeps the bottom on the floor,
and zooms independently within each rectangle. Its width target is the larger
of 10 cells and the widest tower plus 2. Its vertical span is at least 40 cells
and otherwise follows that width divided by the lane's aspect ratio. Rust has
the basic sizing rule in `visuals::frame_towers`, but projects every tower
through one shared camera, so centers and clipping drift as the view widens.

- Begin with `visuals::frame_towers`, `config::Lanes` and `hud::place_items`.
  Trace camera consumers before choosing the render layout, including
  `visuals::update_drop_guide`, `post::follow_grab`, Puzzle heat haze, the laser,
  sand, wizard and result overlays. The original also has foreground and
  background camera passes. Reuse the existing lane ownership and keep the
  change in presentation so tower simulation stays headless-safe.
- Use a two-player Puzzle battle with a narrow and a wide stack as the first
  acceptance check. Each tower should stay centered in the same screen region
  as the widest tower grows, and owned art should clip at the viewport edge.
  Check that the HUD follows the same projection and that the shared laser,
  pause and results remain coherent. Expand verification as listed in the
  [active backlog](WHATS_NEXT.md).
- Keep `puzzle.trial_laser_slack = 0.3` while working on the cameras. Trial 9's
  physics discrepancy is a separate investigation. Read the newest dated
  Trial 9 notes in [What's next](WHATS_NEXT.md) before revisiting its hitboxes.

## Build and local reference

- Run `cargo test --locked -j 1` before merging. Parallel compilation previously
  exhausted memory on this PC. The last Windows iteration used
  `CARGO_PROFILE_DEV_OPT_LEVEL=0` for a faster local compile. CI used the repository
  profile. Linux system packages are listed in `AGENTS.md` and the test workflow.
- Keep automated play silent and windowless. Use `--offscreen` with `--play`
  or `--script`, save captures to a scratch folder, and check the error log.
  See [debug tools](debug-tools.md) and `.agents/skills/offscreen-shot/SKILL.md`
  on this PC. Read [PC tasks](pc-tasks.md) for the standing capture and staging
  workflow. When staging needs a refresh, follow the [extract guide](../tools/extract/README.md)
  and the latest working order in the [November 3 handoff](handoff-2026-11-03.md).
- Use the staged `extracted/` and local original readouts in `research/` only on
  the PC. Keep both folders, screenshots of staged art and original code out of
  Git and shared output. Record original behavior as plain notes in
  `docs/research/`. Keep tunables in `GameConfig` and `assets/config/game.ron`.
- Query Graphify before wide source reads and keep the structural graph local.
  Apply the project's Ponytail and clean-room conventions. Record the dated
  change and verification in `AGENTS.md`, then merge via PR after tests pass.

## Paste into Claude Code

> Read `docs/handoff-2026-11-05-claude.md` and the active backlog at the top of
> `docs/WHATS_NEXT.md`. Verify the current `main` and resume the clean-room
> Tricky Towers Rust rewrite. Continue your multiplayer Puzzle viewport work
> from the branch you already started. Preserve the Trial 9 slack while working on rendering. Use silent
> offscreen captures and the full test suite, then update the project log.
