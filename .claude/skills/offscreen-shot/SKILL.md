---
name: offscreen-shot
description: Take in-game pictures with no window and no sound (offscreen mode or a timed script), for checking a mode, menu or effect on the PC.
disable-model-invocation: true
---

# Offscreen pictures

PC work stays headless and silent: never run plain `cargo run` for pictures.

- One mode: `cargo run -- --offscreen <dir> --shots 5,20 --play <mode>`
  (modes: race, survival, puzzle, endless, endless-survival, endless-puzzle).
- A menu: add `--menu players|modes|single|trials|pause` instead of `--play`.
- A sequence: `cargo run -- --offscreen <dir> --script path/to/script.ron`
  (`Picture` and `Quit` commands; see docs/debug-tools.md).

Save into the session scratchpad, not the repo. Pictures of the original game, or with its art
staged from `extracted/`, are never committed or sent out; only `docs/*.png` drawn by our own
build may be committed, and only when asked. Check the log for art load errors (expect 0).
