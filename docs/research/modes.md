# Survival and Puzzle in the original

Plain-language notes from Max's installed copy. Units are brick cells.

## Shared
- Drop speed = base + highest tower / 10 + 2 per eliminated player.
- Easy modes have no dark magic.
- Race base: a 6-wide "house" platform with a small ledge on its right. Bricks that reach the
  water line at −8.5 are removed.

## Survival (Easy / Normal / Pro "Magic Waves")
- 3 lives on every difficulty. Bricks to place: 33 / 66 / 66. Base drop speed: 2 / 2 / 4. Pro
  gives everyone the same bricks.
- A lost brick (one that falls out of view) costs 1 life, then 1.5 s of invincibility.
- Used bricks = placed + discarded. At the limit a 3 s countdown runs, then a roof goes on and
  that player is done. First to finish wins, otherwise the last wizard standing.
- Light spells: when hurt, Extra life 4, Ivy 3, Minibase 1, Petrify 1, Undo 2. At full health,
  the same without Extra life.
- Normal dark sets: after Ivy or Undo, Large 10, Mystery 10, No-rotate 5, Mist 3. After Petrify or
  Minibase, No-rotate 7, Mist 10, Fast 10, Ghost 10, plus 3 rare combos. After Extra life, harsher
  combos.
- Pro has no dark spells. Curse waves hit everyone every 12 to 18 s; each lasts its own random time
  (for example Large 6 to 8 s, Mist 14 s) and curses every new brick meanwhile. Every 2 waves the
  next of 4 levels applies, with longer waves and double or triple curses.

## Puzzle (Easy / Normal / Pro "Nudge Required")
- Laser height: random 5 to 7 / 3 to 7 / 3 to 7, each +0.2. Everyone gets the same bricks.
- No spell lines; only a start spell: Undo / Ivy / none.
- Floor: Easy is 5 wide and flat-ish (pieces up to 2 lower). Normal is 6 wide with 40% gaps, pieces
  up to 4 lower. Pro is a 3-wide pillar with random 1-cell side ledges (70% per row).
- The lane is 1.2× wider. The steering limit is the larger of the usual limit × 1.2 and the
  tower's half width plus 1.5, measured from the widest settled brick either side of the centre.
- A brick above the laser is zapped and that player's round ends.
- A dropped brick costs no life; the laser effectively moves 0.5 lower.
- When everyone has finished, the most bricks in the tower wins.

## Computer opponents
- Battles in Race, Survival and Puzzle are human-only: 2 to 4 local players, one controller or
  keyboard each, or online. The only computer player that drops bricks is the menu's attract demo,
  which follows a fixed loop of moves at speed 50 without looking at the tower.
- Single player is Trials, where an "enemy wizard" who doesn't build casts dark spells from a
  per-level script (every N seconds, at set times, or as your remaining bricks pass fractions of
  the target), plus Endless modes.
- Endless Race: a pacer line chases your tower. It slides in from height −15 to −5 over 1 s, then
  rises at 0.2 cells/s. Spell lines sit at fixed heights: 10, then every 10 up to 150, then each
  gap one longer (161, 173, ...). Each line you pick up is a checkpoint: the pacer stops, eases
  2 cells down over 1 s, waits 2 s, then rises faster by a step that starts at 0.05 and grows
  0.01 per checkpoint up to 0.1 (top speed 1.5). You lose when it reaches your tower top. Score
  is 10 × height, rounded up. Light spells only, drawn from Ivy, Undo, Ivy, Undo, Petrify. No
  lives: lost bricks are just gone. It has its own versions of the race islands and a tint
  under the pacer line.
- Our AI opponents are an extra beyond the original, not its main mode.

## Survival Pro curse waves, exact (read from the PC install, 2026-10-04)

Timing
- Times are whole seconds, picked with an integer random that excludes the top value. So "12 to 18" really means 12 to 17 s, and a wave listed "6 to 8" lasts 6 or 7 s.
- The first wave comes 12 to 17 s after the start. Each next wave comes 12 to 17 s after the previous one ends. The gap is counted from the end of a wave, not from its start.
- All players share one random seed, so everyone gets the same waves at the same times.

Levels
- There are 4 levels. Within a level, waves are drawn at random without repeats.
- After 2 waves (or sooner, if the level's list runs out) the next level starts. Level 4 then repeats, with a fresh copy of its list each time.

When a wave starts
- The current brick is paused and shakes.
- It is moved to the top, the game waits 1 s, then control comes back.
- The wave's curses hit that brick, and then every new brick while the wave lasts. Mist is the exception: it is applied once, at the start.

The 7 curses
- **Large**: the brick grows, with a camera shake and a rumble sound.
- **No rotate.**
- **Fast.**
- **Ice.**
- **Mist**: spawned over the top of the tower, once per wave.
- **Ghost**: the last brick placed on the tower turns ghost for 1.5. It skips bricks that are already ghost, stone or ivy.
- **Mystery**: the current brick becomes a Mystery brick.

Waves per level (curses, length in s; the top value is excluded)

| Level | Waves |
|---|---|
| 1 | Large 6-8; Mist 14-14 (14 s); Mystery 6-8; Fast 6-8; No rotate 6-8 |
| 2 | Large 8-11; Mystery 8-11; Ice 6-8; Fast 8-11; No rotate 8-11; Fast+Large 4-6; Mystery+No rotate 4-6; Mist+Large 6-8 |
| 3 | Large 12-16; Ice 8-11; No rotate 11-15; Ghost 6-8; Mist+No rotate 8-11; Mystery+Ice 6-8; No rotate+Fast 8-11; No rotate+Large 8-11 |
| 4 | Fast 12-16; No rotate 12-16; Ice 11-15; Mystery 12-16; Ghost+Fast 6-8; Ice+Large 6-8; Mystery+Ice 8-11; Large+Large 6-8; Mist+Mystery 8-11; Large+Ghost 7-10; Mystery+Large+No rotate 6-8; Large+Fast+No rotate 6-8 |

`WaveRules` now holds these four tables (src/config.rs). At a wave's start the brick in hand is lifted to the spawn height and held, shaking, for `WaveRules::hold` (1 s); the shake's reach and speed (`WaveRules::shake`) are guesses. Ghost strikes with the brick in hand and again with every new brick during the wave.

## Base shapes (read from the PC install, 2026-10-04)

Cells. x is measured from the base centre and y from the base top (the y = 0 line the bricks rest on). Every collider is a plain box.

**Race, all difficulties** (FLOOR_RACE, picture `platform_race` = gph_house02). Endless Race uses the same shapes (FLOOR_RACE_ENDLESS) with the Endless reflection art.
- Body: x -3..3, y -9..0. The top is 6 cells wide.
- Chimney bump: x -1..0, y 0..1. One cell sticks up, just left of the centre.
- Side ledges lower down, under the water: x -4..-3, y -9..-5.57 and x 3..4, y -9..-5.

**Survival, all difficulties, Endless Survival too** (FLOOR_LWS, picture `platform_survival` = gph_house01_world02, the castle).
- Merlons on top, each 1 cell high (y -1..0): x -3..-2, x -1..1, x 2..3.
- That leaves two holes, each 1 cell wide and 1 deep: x -2..-1 and x 1..2. The tops of the merlons are the y = 0 line.
- Body under the merlons: x -3..3, y -5..-1.
- Wider band: x -3.5..3.5, y -7..-5.
- Foot: x -2.5..3, y -11..-7.
- A challenge variant (FLOOR_LWS_PETRIFIED_BRICK) adds a stone I brick lying at x 2..6, y 0..1.

**Puzzle.** The floors are built from pieces that are single columns. All pieces are 13 tall (y -13..0) unless noted.
- Easy and Normal use FLOOR_PUZZLE: piece 1 (gph_house01_world03) is 1 wide, piece 2 (gph_house02_world03) is 2 wide, piece 3 (gph_house03_world03) is 1 wide.
- Pro uses FLOOR_PUZZLE_PRO: a centre column (gph_house03_world03) 1 wide, x -0.5..0.5. Pro pieces 1-12 and the top piece (gph_house08/11_world03 …) are 1×1 cells, x -0.5..0.5, y -1..0.

In the rewrite these boxes are `GameConfig::bases` (src/config.rs); the AI, Auto Build and the on-tower check use the top's half width (`base_half_width`, 3 cells for both houses). The petrified-brick challenge variant isn't built (Trials aren't in yet).

## Endless Survival and Endless Puzzle (public descriptions only; superseded by the install readout below)

From reviews and store pages, not from Max's copy, so nothing here is confirmed:
- Endless Survival: three lives, stack as many bricks as you can in one tower; it has a
  leaderboard. (Giant Bomb wiki, TrueAchievements review.)
- Endless Puzzle: the puzzle-trial rules (bricks under a laser line; a brick above the line
  or a dropped brick fails) that keep going until you fail. (Same sources.)

In the rewrite these are `ModeKind::EndlessSurvival` / `EndlessPuzzle` with the guesses in
`GameConfig::endless_play` (src/config.rs, every field marked `(?)`). The PC readout queued in
docs/pc-tasks.md replaces them with the original's values.

## Curse wave start: lift and shake (read from the PC install, 2026-10-04)
When a wave starts, the brick in hand goes through: player control paused, shake on, move up,
wait 1 s, shake off, control back.
- **Move up:** to 4 cells below the camera's top edge, over exactly 1 s, eased in (quadratic:
  slow start, fast end), with collisions off during the move and back on at its end.
- **Shake:** every frame, only the brick's picture (not its body) jumps to a random offset,
  x and y each uniform in ±0.1 cell divided by the brick's scale (so an enlarged brick shakes
  less). It runs only while the brick isn't frozen. When the shake ends, the picture snaps
  back to the centre.
- So the brick shakes for about 2 s in all: 1 s rising and 1 s held at the top.

## Endless Survival and Endless Puzzle (read from the PC install, 2026-10-05)
**Endless Puzzle**
- One life. Drop speed starts at 2 (with the usual height bonus). Random bricks. No wind.
  Puzzle floor `FLOOR_PUZZLE`. Camera zoomed 1.2× closer than battle Puzzle; bricks more than
  4 cells above the camera's top are removed. The drop guide is cream (1, 0.92, 0.74, 80%).
- The laser starts at height 6.2. The run comes in **levels**: level 1 allows 6 bricks. When
  you've used all the bricks of the level and none was zapped, a level-up runs:
  1. Brick spawning stops, the guide hides, wait 0.3 s.
  2. The laser and the height limit both rise by 4 cells over 0.5 s, with one of 3 "move
     laser" sounds.
  3. New **obstacle blocks** are placed on the tower (positions picked from the tower height,
     the new laser height and the camera), one every 0.1 s, each with one of 3 spawn sounds.
     Then wait 0.3 s.
  4. The laser zaps (laser start sound), spawning resumes, and you're given one Undo spell.
  5. The next level allows min(26, 6 + 2 × level) more bricks (8, 10, 12, … up to 26).
- **Failing:** a brick zapped by the laser (settled above it) starts the end countdown; a lost
  brick costs the only life. Either ends the run.
- **Score:** the number of bricks in the tower (`BRICK_COUNT`) at the end.
- There's no difficulty choice. The roof at the end is the Puzzle roof.

**Endless Survival**
- Lives: the game's maximum health setting (the value lives in the game's settings data, not
  in the scripts; Survival battle uses 3). Drop speed starts at 4. Random bricks. Floor
  `FLOOR_LWS` at 12.5 (the Survival base).
- Spell lines give a random spell from Ivy, Minibase and Petrify (light spells only).
- **Curse waves** ("shared agony"): the first wave and each following one come 15 to 29 s apart.
  A wave lasts a number of bricks, not seconds. There are 4 difficulty levels of waves (the
  game moves up through them as the run goes on). Each wave curses that many bricks with its
  curse(s):
  - Level 1: Large ×2, Mist ×3, Mystery ×2, Fast ×4, No Rotate ×3.
  - Level 2: Large ×3, Mystery ×3, Ice ×3, Fast ×5, No Rotate ×4, Fast+Large ×2,
    Mystery+No Rotate ×2, Mist+Large ×2.
  - Level 3: Large ×4, Ice ×4, No Rotate ×5, Ghost ×2, Mist+No Rotate ×4, Mystery+Ice ×3,
    No Rotate+Fast ×4, No Rotate+Large ×3.
  - Level 4: Fast ×5, Large ×7, No Rotate ×6, Ice ×5, Mystery ×6, Ghost+Fast ×3,
    Ice+Large ×4, Mystery+Ice ×5, Mist+Mystery ×5, Large+Ghost ×5,
    Mystery+Large+No Rotate ×4, Large+Fast+No Rotate ×5.
  The Large curse also shakes the camera and plays the rumble. Mist targets the top of the
  tower; Ghost targets the last brick placed (1.5 s).
- **Score:** the number of bricks in the tower at the end. The run ends when the last life is
  lost. The roof piece is `ROOF_02`.
- Both modes keep one online leaderboard each (see menus.md, "Endless leaderboard rows").