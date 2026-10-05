---
name: project-conventions
description: Rules every change to this repo must follow (1:1 with the original, clean-room, GameConfig values, DEBUG-TOOL tags, headless gameplay, AGENTS.md log). Use before finishing any code change here.
user-invocable: false
---

# Checklist before calling a change done

- 1:1 first: the original's rule or value wins over a shorter or nicer version. A value not yet
  read from the original is marked as a guess and listed in docs/pc-tasks.md.
- Clean-room: no decompiled code, original assets or Unity dumps in the repo. Findings go in
  `docs/research/*.md` as plain notes. Nothing from `extracted/`, `research/` or the install is
  committed or sent out; don't loosen the whitelist `.gitignore`.
- Tunable numbers live in `GameConfig` (src/config.rs) and `assets/config/game.ron`.
- `src/gameplay.rs`, `src/spells.rs`, `src/ai.rs` stay free of rendering and keyboard code;
  every tower goes through the same systems, only its `PlayerInput` writer differs.
- Anything not in the original is tagged `DEBUG-TOOL` and listed in docs/release-checklist.md.
- `cargo test` passes; add a headless test for new rules.
- Append a dated line to the AGENTS.md log (add "via OmniRoute" if the session ran through it).
