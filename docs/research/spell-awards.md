# How the original awards spells

Plain-language notes on Max's installed copy of Tricky Towers, written from reading it on their PC.
No original code or data is in this repo. Units are brick cells; each player's lane is 16 cells wide.

## Spell lines
- Each player has one horizontal spell line across their lane. When the tower touches it, the
  player collects a pair: one light spell and one dark spell.
- Only one of the pair can be used. Casting light (on yourself) throws the dark one away, and
  casting dark (on every other player) throws the light one away.
- The next line appears after the pickup. A line the tower has passed by more than 6 cells
  without touching is replaced.
- Line spacing: 9 cells, plus 25% for each player beyond two (9, 11.25, 13.5).
- In Race a new line appears at the higher of its scheduled height and the tower top plus one
  spacing. It waits 18 s (plus 25% per player beyond two, on Normal and Pro), then slides down
  toward the tower at 0.5 cells/s plus half of however far it is beyond 9 cells. A line more than
  19 cells above the tower starts moving straight away. The wait restarts after each pickup.
- A line is hidden within 2 cells of the finish line and shown once it is 5 cells above the tower.

## Which spells (Race)
The first matching rule picks the light spell (weights in brackets):
1. Within 15 cells of the finish: Ivy [1], Undo [1].
2. 15 or more behind the leader (20 on Pro): Auto-build [2], Petrify [1], Minibase [1],
   with the "far behind" dark set.
3. 8 or more behind the leader: Petrify [1], Minibase [1], with the "behind" dark set.
4. Otherwise: Ivy [10], Undo [10], Petrify [3] (Pro adds Minibase [3]).

The same light spell never comes twice in a row. Easy has no dark spells.

Dark sets:
- After Ivy or Undo: Large 10, Grassy 10, Bubble 10, Mystery 10, Auto-rotate 10, Slow 6.
- After Petrify or Minibase: Slow 10, Ice 10.
- Behind: Slow 10, Ice 10, Large + Mystery 4, Large + Auto-rotate 4, Large + Ice (one brick) 4.
- Far behind: Slow 4, Ice 4, Large + Mystery 10, Large + Auto-rotate 10, Large + Ice (one brick)
  10, Ice + Auto-rotate + Mystery on 4 bricks 3.

## Race per difficulty (Easy / Normal / Pro)
- Finish height: 26 / 45 / 38 cells.
- The finish starts coming down after 90 / 120 / 120 s, at 0.2 cells/s.
- Base drop speed: 2 / 3 / 4 cells/s.
- Pro only: every player gets the same bricks, and wind of strength 3 to 4 starting in a random
  direction.
  - In our build the wind pushes every loose (placed, moving) brick sideways at 0.1 cells/s² per
    point of strength, always the same way for the whole match. What the original's wind acts
    on, how hard, and whether it turns are guesses until read.

Drop speed during a match is the base, plus a tenth of the highest tower's height, plus 2 for
each player who is out.

## Respawning bricks
A brick carrying a curse (Large, Fast, Ice, No-rotate, Auto-rotate, Slow) or a Mystery brick
that is thrown away comes back where it was lost, up to 3 times. Normal bricks never come back.
