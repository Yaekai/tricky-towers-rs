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
