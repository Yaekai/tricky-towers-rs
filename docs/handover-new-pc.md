# Handover: moving to a new PC (2026-10-05)

Max is switching PCs. Everything is pushed; this note is what the next PC session needs to get
back to where the old one stopped. Read AGENTS.md and docs/pc-tasks.md too.

## State at handover
- `main` holds everything up to PR #39. **PR #41** (branch `claude/project-thread-odpoew`) is
  open and mergeable, waiting on Max's merge: finish line staging fix, Puzzle Pro / Race Pro
  backgrounds, Puzzle drop speed 2, Endless Race 4, Auto Rotate elastic ease, No Rotate jumps,
  minibase 2 wide, and this note. The PC session could not merge PRs itself (the permission
  check blocked it even with Max's OK), so Max merges.
- A release build was made on the old PC outside the repo
  (`C:\Users\hugoc\Documents\Claude\TrickyTowers-build`); it is local only (it contains staged art
  from the install) and is rebuilt on the new PC as below.

## Setting up the new PC
1. **Game and tools to install** (with Max's go-ahead, as before): Steam + Tricky Towers, Git,
   Rust (rustup, stable), Node.js (the repo's `.claude/hooks` run with `node`), Python 3 (Graphify
   needs it; on the old PC it was missing from PATH so Graphify's command never ran), AssetRipper.
   The GitHub CLI wasn't installed on the old PC; PRs were opened through the GitHub API with the
   git credential manager's token. Installing `gh` makes that simpler.
2. **Claude Code plugins** (docs/pc-tasks.md, first section): Ponytail, Graphify, claude-code-setup,
   rust-analyzer, context7 (it timed out on the old PC; check it connects), and OmniRoute as the
   fallback only. The repo's `.claude/` (game-files guard, rustfmt hook, conventions and
   offscreen-shot skills, 1:1 reviewer agent) comes with the clone.
3. **Clone** `Yaekai/tricky-towers-rs`, then rebuild the local-only folders (both gitignored):
   - Export the install with AssetRipper ("Export Unity project") into `research\ripped`.
   - `powershell -File tools\extract\guid_index.ps1 -Assets research\ripped\ExportedProject\Assets -Out research\guid_index.tsv`
   - Then, in order: `stage.ps1`, `wizards.ps1`, `inventory.ps1`, `effects.ps1`,
     `rival_towers.ps1`, `race_scene.ps1` (all `powershell -File tools\extract\<name> -Repo .`). The old PC staged
     519 entries with 1 missing (`page_dot_on`, a known sprite-name mismatch) and 2,198 manifest
     entries in all.
   - The old PC's `research\notes.md` and `research\spec_*.md` (raw readout notes) don't come
     across; everything that matters from them is already in `docs/research/`.
4. **Check**: `cargo test` (all green at handover), then offscreen pictures of each mode
   (`cargo run -- --offscreen <dir> --shots 5 --play <mode>`; or `--script` files, see
   docs/debug-tools.md, which can now start a trial with `Start(mode: ..., trial: Some(n))`).
   Never run plain `cargo run` while Max is using the PC: offscreen only, no window, no sound.

## Tips learned on the old PC
- If Max has the game open, `target\debug\tricky_towers.exe` is locked; build into another folder
  (`$env:CARGO_TARGET_DIR='target-pc'`). That folder isn't covered by `.gitignore`'s whitelist
  check for every case, so never `git add -A`: add the files you changed by name.
- A fresh build takes about 6 minutes in release, longer in dev.
- The exe needs `assets\` and `extracted\` next to it (or in the working folder):
  `cargo build --release`, then copy `tricky_towers.exe`, `assets` and `extracted` into one
  folder **outside the repo**. Don't share that folder: `extracted` is art from Max's install.
- PowerShell gotchas: `R`, `Rp` are built-in aliases (don't name helper functions that); a
  backtick in a double-quoted string eats the next letter (`` `t `` is a tab); files are CRLF.
- `shipped_config_matches_defaults` needs `game.ron` to equal the defaults: after changing a
  default, run `TT_WRITE_CONFIG=1 cargo test --lib write_shipped_config`.

## Next work (in order)
1. Picture the screens the cloud built without a GPU: trials tier unlock, Game Info, Stats,
   Keybinding, trial banners and clock; fix what looks wrong.
2. Read the wizard voice packs and stage per-character lines (pc-tasks.md, Queued, first item).
3. Read the text format, tutorial and Credits popups (pc-tasks.md, Queued).
4. Remaining guessed values marked `(?)` in the code: cast beams, lightning, spell clouds,
   hearts and bubbles under the wizard, sunray spin, Puzzle sand strip, wizard level names,
   menu slide times and layouts, pack looks (Diamond gloss, Hologram flicker, Galaxy drift).
5. Race Pro's dark island pictures (`_DARK`) weren't found by sprite name; look in the Race Pro
   background script or prefabs for how they're made.
6. The enemy wizard in trials isn't drawn yet (it floats 7 cells left of its lane centre, 2 in
   Puzzle trials; which wizard it is wasn't read).
