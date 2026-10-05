# Trials in the original (read from the PC install, 2026-10-04)

Plain-language summary of the 50 single-player trials, in the order the game lists them.
Values are as the install defines them; units are cells and seconds.

## Tiers and medals
- 5 tiers of 10 trials: Rookie, Apprentice, Pro, Expert, Master. A tier opens when your medal
  count reaches 0 / 8 / 17 / 26 / 36.
- **One medal per trial**: the medal count is simply the number of trials completed (state
  "COMPLETED"). There are no stars, grades or times.
- The Single Player screen shows "medals earned / total".

## How to read the table
- **Goal.** Race: reach the height within the time limit. Survival: place that many bricks
  without losing all lives (the brick supply is exactly that many). Puzzle: place exactly that
  many bricks, all staying under the laser at the given height, with 1 life.
- **Drop.** The starting drop speed, when the trial sets one ("-" = the mode's default).
- **Base.** Survival and Puzzle name the floor piece and its height. Race trials use the race
  floor plus a pre-built **enemy tower** picture beside yours (ENEMY_TOWER_LEVEL_01..13). At the
  start it pops in brick by brick over 1.5 s, with a small poof per brick and a bubble pop every
  0.075 s rising in pitch.
- **Bricks.** "random" = normal random bricks. "fixed" = this exact order.
- **Spell lines.** How many spell lines appear in the run, spaced that many cells apart (plus
  25% more spacing per extra player), and the spells they give, in order. "(random)" means
  the spell is used the moment the line is collected instead of kept.
- **Enemy curses.** The enemy wizard's script: `hN X` casts curse X on your brick when your
  tower reaches N cells, `bN X` when you're on your Nth brick. Combined names (LARGE_MYSTERY)
  are one brick with both curses. "(once)" marks the only curse that isn't reused.

| Trial | Id | Mode | Goal | Drop | Base | Bricks | Spell lines | Enemy curses |
|---|---|---|---|---|---|---|---|---|
| Rookie (0 medals) 1 | SR_10 | Race | reach 14 cells in 60 s | - | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_01 | random | - | h11 LARGE |
| Rookie (0 medals) 2 | SS_17 | Survival | place 15 bricks, 3 lives | 3 | FLOOR_LWS at 12.5 | random | - | b5 LARGE, b10 LARGE, b15 LARGE |
| Rookie (0 medals) 3 | SP_20 | Puzzle | place 9 bricks under a laser at 2.2, 1 life | - | FLOOR_PUZZLE_BASIC at 15.5 | fixed: O Z S T T O T T I | - | - |
| Rookie (0 medals) 4 | SP_16 | Puzzle | place 6 bricks under a laser at 5.2, 1 life | - | FLOOR_OVERHANG_TUTORIAL at 12 | fixed: J I S Z L J | - | - |
| Rookie (0 medals) 5 | SR_11 | Race | reach 16 cells in 60 s | - | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_02 | random | - | h4 GRASSY, h8 LARGE, h11 GRASSY |
| Rookie (0 medals) 6 | SS_10 | Survival | place 20 bricks, 3 lives | 3 | FLOOR_LWS_PETRIFIED_BRICK at 12.5 | random | 1 line(s) every 12 cells: PETRIFY | b5 MYSTERY, b10 MYSTERY, b13 MYSTERY, b20 LARGE_MYSTERY |
| Rookie (0 medals) 7 | SP_15 | Puzzle | place 7 bricks under a laser at 1.2, 1 life | - | FLOOR_PUZZLE_LEVEL01 at 15 | fixed: O Z S L S J Z | - | - |
| Rookie (0 medals) 8 | SR_18 | Race | reach 22 cells in 70 s | - | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_03 | random | 1 line(s) every 9 cells: PETRIFY | h5 GRASSY, h9 LARGE, h10 GRASSY, h15 SLOW |
| Rookie (0 medals) 9 | SP_10 | Puzzle | place 6 bricks under a laser at 2.2, 1 life | - | FLOOR_PUZZLE_LONG_I at 14 | fixed: T J L I L O | - | - |
| Rookie (0 medals) 10 | SS_11 | Survival | place 26 bricks, 3 lives | 3 | FLOOR_LWS at 12.5 | random | 1 line(s) every 9 cells: PETRIFY (random) | b1 MIST, b11 MIST, b19 MIST, b23 MIST, b24 MYSTERY, b25 MYSTERY, b26 MYSTERY |
| Apprentice (8) 1 | SP_21 | Puzzle | place 6 bricks under a laser at 5.2, 1 life | - | FLOOR_OVERHANG_CHALLENGE_J at 12 | fixed: I L Z T J I | - | - |
| Apprentice (8) 2 | SR_22 | Race | reach 24 cells in 75 s | 4 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_04 | random | 1 line(s) every 12 cells: IVY (random) | b10 BUBBLE, b20 BUBBLE, b30 BUBBLE, b40 BUBBLE, b50 BUBBLE, b60 BUBBLE, b70 BUBBLE, b80 BUBBLE, b90 BUBBLE, b100 BUBBLE |
| Apprentice (8) 3 | SS_13 | Survival | place 25 bricks, 3 lives | 4 | FLOOR_LWS at 12.5 | random | 1 line(s) every 9 cells: IVY | b1 NO_ROTATE, b6 NO_ROTATE, b11 NO_ROTATE, b14 NO_ROTATE, b18 NO_ROTATE, b21 LARGE_NO_ROTATE, b25 LARGE_NO_ROTATE_MYSTERY |
| Apprentice (8) 4 | SR_14 | Race | reach 27 cells in 60 s | 4 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_05 | random | 1 line(s) every 12.5 cells: IVY (random) | h0 MYSTERY, h5 BUBBLE (once), h9 LARGE_MYSTERY, h20 GRASSY |
| Apprentice (8) 5 | SP_12 | Puzzle | place 8 bricks under a laser at 5.2, 1 life | - | FLOOR_PUZZLE_LARGE_INTRO at 12 | fixed: L J Z O T Z O I | - | b5 LARGE |
| Apprentice (8) 6 | SS_14 | Survival | place 30 bricks, 3 lives | 4 | FLOOR_LWS at 12.5 | random | 1 line(s) every 9 cells: IVY (random) | b1 FAST, b11 FAST, b21 FAST_MIST, b26 FAST |
| Apprentice (8) 7 | SP_29 | Puzzle | place 7 bricks under a laser at 4.2, 1 life | - | FLOOR_PUZZLE_NO_ROTATE_NEED_EASY at 14 | fixed: L T Z L Z S I | - | - |
| Apprentice (8) 8 | SP_22 | Puzzle | place 7 bricks under a laser at 7.2, 1 life | - | FLOOR_OVERHANG_CHALLENGE_J_I at 12 | fixed: Z T S L I O I | - | b7 LARGE |
| Apprentice (8) 9 | SS_16 | Survival | place 33 bricks, 3 lives | 4 | FLOOR_LWS at 12.5 | random | 2 line(s) every 9 cells: IVY (random) | b1 NO_ROTATE_MYSTERY, b7 NO_ROTATE, b14 LARGE_NO_ROTATE, b21 NO_ROTATE_MIST, b29 FAST_NO_ROTATE, b31 NO_ROTATE, b33 NO_ROTATE |
| Apprentice (8) 10 | SP_24 | Puzzle | place 7 bricks under a laser at 5.2, 1 life | - | FLOOR_PUZZLE_SPARE_UNIT at 14 | fixed: T L S I I J J | - | - |
| Pro (17) 1 | SS_18 | Survival | place 18 bricks, 3 lives | 5 | FLOOR_LWS at 12.5 | random | 2 line(s) every 9 cells: IVY, MINIBASE | b2 LARGE, b4 LARGE, b6 LARGE, b8 LARGE, b10 LARGE, b12 LARGE, b14 LARGE, b16 LARGE, b18 LARGE |
| Pro (17) 2 | SP_18 | Puzzle | place 7 bricks under a laser at 0.2, 1 life | - | FLOOR_PUZZLE_NUDGE_TUTORIAL at 15 | fixed: Z O S Z O O O | - | - |
| Pro (17) 3 | SP_11 | Puzzle | place 9 bricks under a laser at 4.2, 1 life | - | FLOOR_PUZZLE_FILLER_01 at 14 | fixed: J S J T Z T I I J | - | - |
| Pro (17) 4 | SS_12 | Survival | place 33 bricks, 3 lives | 5 | FLOOR_LWS at 12.5 | random | 2 line(s) every 9 cells: MINIBASE, IVY | b1 NO_ROTATE, b2 NO_ROTATE, b3 NO_ROTATE, b4 NO_ROTATE, b5 NO_ROTATE, b6 NO_ROTATE, b7 NO_ROTATE, b8 NO_ROTATE, b9 NO_ROTATE, b10 NO_ROTATE, b11 NO_ROTATE, b12 NO_ROTATE, b13 NO_ROTATE, b14 NO_ROTATE, b15 NO_ROTATE, b16 NO_ROTATE, b17 NO_ROTATE, b18 NO_ROTATE, b19 NO_ROTATE, b20 NO_ROTATE, b21 NO_ROTATE, b22 NO_ROTATE, b23 NO_ROTATE, b24 NO_ROTATE, b25 NO_ROTATE, b26 NO_ROTATE, b27 NO_ROTATE, b28 NO_ROTATE, b29 NO_ROTATE, b30 NO_ROTATE, b31 NO_ROTATE, b32 NO_ROTATE, b33 LARGE_NO_ROTATE |
| Pro (17) 5 | SP_30 | Puzzle | place 7 bricks under a laser at 0.35, 1 life | - | FLOOR_PUZZLE_BRIDGE at 14 | fixed: L S T J Z T I | - | - |
| Pro (17) 6 | SR_16 | Race | reach 32 cells in 97 s | 5 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_06 | random | 2 line(s) every 12 cells: IVY, IVY | h8 ICE, h17 MYSTERY, h25 ICE |
| Pro (17) 7 | SP_17 | Puzzle | place 7 bricks under a laser at 0.35, 1 life | - | FLOOR_PUZZLE_NO_ROTATE at 15 | fixed: T I L I J I L | - | b2 NO_ROTATE, b6 NO_ROTATE |
| Pro (17) 8 | SR_17 | Race | reach 40 cells in 128 s | 5 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_07 | random | 3 line(s) every 12 cells: IVY, MINIBASE, IVY | h7 ICE_MYSTERY, h18 LARGE_ICE, h32 ICE |
| Pro (17) 9 | SP_28 | Puzzle | place 8 bricks under a laser at 2.2, 1 life | - | FLOOR_PUZZLE_NO_ROTATE_NEED at 14 | fixed: L J Z J S T I I | - | - |
| Pro (17) 10 | SS_15 | Survival | place 42 bricks, 3 lives | 5 | FLOOR_LWS at 12.5 | random | 3 line(s) every 9 cells: IVY, MINIBASE, IVY | b1 FAST_LARGE, b11 MIST, b13 FAST, b21 NO_ROTATE, b27 FAST_MYSTERY, b31 LARGE_NO_ROTATE, b39 FAST, b42 LARGE_NO_ROTATE |
| Expert (26) 1 | SP_13 | Puzzle | place 8 bricks under a laser at 0.2, 1 life | - | FLOOR_PUZZLE_BAK at 15 | fixed: L T S I T O J L | - | - |
| Expert (26) 2 | SR_19 | Race | reach 50 cells in 155 s | 5 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_08 | random | 3 line(s) every 13 cells: MINIBASE, PETRIFY, IVY | h7 AUTO_ROTATE, h12 MYSTERY, h24 GRASSY, h32 SLOW_AUTO_ROTATE, h34 AUTO_ROTATE, h46 LARGE_AUTO_ROTATE |
| Expert (26) 3 | SP_25 | Puzzle | place 9 bricks under a laser at 0.2, 1 life | - | FLOOR_PUZZLE_DOUBLE at 15 | fixed: S L T J O T O J O | - | b5 LARGE |
| Expert (26) 4 | SP_23 | Puzzle | place 9 bricks under a laser at 5.2, 1 life | - | FLOOR_OVERHANG_CHALLENGE at 12 | fixed: J I L Z T J I J J | - | - |
| Expert (26) 5 | SS_21 | Survival | place 40 bricks, 3 lives | 5 | FLOOR_LWS at 12.5 | random | 2 line(s) every 9 cells: MINIBASE, MINIBASE | b1 LARGE, b2 GHOST, b10 GHOST, b24 GHOST, b35 GHOST, b40 GHOST |
| Expert (26) 6 | SR_20 | Race | reach 20 cells in 44 s | 5 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_09 | random | 1 line(s) every 12 cells: IVY (random) | b1 ICE, b7 ICE, b13 ICE, b19 ICE, b25 ICE, b31 ICE, b37 ICE, b43 ICE, b49 ICE, b55 ICE, b61 ICE, b67 ICE, b73 ICE, b79 ICE |
| Expert (26) 7 | SP_31 | Puzzle | place 9 bricks under a laser at 3.2, 1 life | - | FLOOR_PUZZLE_BRIDGE01 at 14 | fixed: S T J T I T I S I | - | - |
| Expert (26) 8 | SR_21 | Race | reach 56 cells in 119 s | 5 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_10 | random | 3 line(s) every 15 cells: IVY, MINIBASE, IVY | h10 LARGE, h20 MYSTERY, h30 LARGE, h40 MYSTERY, h50 LARGE_MYSTERY |
| Expert (26) 9 | SP_14 | Puzzle | place 8 bricks under a laser at 6.2, 1 life | - | FLOOR_PUZZLE_LOCKS at 10 | fixed: L L O I S L I T | - | b1 NO_ROTATE, b6 NO_ROTATE, b8 NO_ROTATE |
| Expert (26) 10 | SS_06 | Survival | place 50 bricks, 3 lives | 5 | FLOOR_LWS at 12.5 | random | 3 line(s) every 9 cells: IVY, MINIBASE, IVY | b10 FAST, b17 FAST, b24 FAST, b31 FAST, b38 FAST, b46 FAST, b49 LARGE, b50 LARGE_MYSTERY |
| Master (36) 1 | SR_24 | Race | reach 35 cells in 95 s | 5 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_11 | random | 3 line(s) every 11 cells: IVY, IVY, IVY | b5 AUTO_ROTATE, b6 AUTO_ROTATE, b7 AUTO_ROTATE, b8 AUTO_ROTATE, b15 AUTO_ROTATE, b16 AUTO_ROTATE, b17 AUTO_ROTATE, b18 AUTO_ROTATE, b25 LARGE_AUTO_ROTATE, b29 LARGE_AUTO_ROTATE, b35 AUTO_ROTATE, b36 AUTO_ROTATE, b37 AUTO_ROTATE, b38 AUTO_ROTATE, b45 AUTO_ROTATE, b46 AUTO_ROTATE, b47 AUTO_ROTATE, b48 AUTO_ROTATE |
| Master (36) 2 | SS_07 | Survival | place 55 bricks, 3 lives | 5 | FLOOR_LWS at 12.5 | random | 5 line(s) every 9 cells: UNDO, IVY, MINIBASE, IVY, IVY | b1 NO_ROTATE, b2 NO_ROTATE, b3 NO_ROTATE, b4 NO_ROTATE, b5 LARGE_NO_ROTATE, b6 NO_ROTATE, b7 NO_ROTATE, b8 NO_ROTATE, b9 NO_ROTATE, b10 NO_ROTATE, b11 NO_ROTATE, b12 NO_ROTATE, b13 NO_ROTATE, b14 NO_ROTATE, b15 FAST_NO_ROTATE, b16 NO_ROTATE, b17 NO_ROTATE, b18 NO_ROTATE, b19 NO_ROTATE, b20 NO_ROTATE, b21 NO_ROTATE, b22 NO_ROTATE, b23 NO_ROTATE, b24 NO_ROTATE, b25 NO_ROTATE, b26 LARGE_NO_ROTATE, b27 NO_ROTATE, b28 NO_ROTATE, b29 NO_ROTATE, b30 LARGE_NO_ROTATE, b31 NO_ROTATE, b32 NO_ROTATE, b33 NO_ROTATE, b34 NO_ROTATE, b35 NO_ROTATE, b36 NO_ROTATE, b37 NO_ROTATE_MYSTERY, b38 NO_ROTATE, b39 NO_ROTATE_MYSTERY, b40 NO_ROTATE, b41 NO_ROTATE, b42 NO_ROTATE, b43 NO_ROTATE, b44 NO_ROTATE, b45 NO_ROTATE, b46 NO_ROTATE, b47 NO_ROTATE, b48 NO_ROTATE, b49 NO_ROTATE, b50 NO_ROTATE, b51 NO_ROTATE, b52 NO_ROTATE, b53 NO_ROTATE, b54 NO_ROTATE, b55 LARGE_NO_ROTATE_MYSTERY |
| Master (36) 3 | SP_33 | Puzzle | place 10 bricks under a laser at 7.2, 1 life | - | FLOOR_PUZZLE_BLOCK at 12 | fixed: J S O J S I O T L T | - | - |
| Master (36) 4 | SP_27 | Puzzle | place 7 bricks under a laser at 0.35, 1 life | - | FLOOR_PUZZLE_SOLID_BRICKS02 at 15 | fixed: L I S Z I T T | - | - |
| Master (36) 5 | SR_03 | Race | reach 30 cells in 45 s | 5 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_12 | random | - | h10 SLOW |
| Master (36) 6 | SS_19 | Survival | place 56 bricks, 3 lives | 5 | FLOOR_LWS at 12.5 | random | 5 line(s) every 16 cells: UNDO, IVY, PETRIFY, MINIBASE, IVY | b2 LARGE, b4 LARGE, b6 LARGE, b8 LARGE_MYSTERY, b10 LARGE, b12 LARGE, b14 LARGE, b16 LARGE, b18 LARGE, b20 LARGE, b22 LARGE, b24 LARGE, b26 LARGE, b28 LARGE, b30 LARGE, b32 LARGE, b33 NO_ROTATE, b34 LARGE_NO_ROTATE, b36 LARGE, b38 LARGE, b40 LARGE, b42 LARGE, b44 LARGE, b46 LARGE, b48 LARGE, b50 LARGE, b52 LARGE, b53 MYSTERY, b54 LARGE, b55 MYSTERY, b56 LARGE_MYSTERY |
| Master (36) 7 | SP_19 | Puzzle | place 9 bricks under a laser at 7.2, 1 life | - | FLOOR_PUZZLE_HARD at 11 | fixed: S L O O T J T L T | - | b0 LARGE, b9 LARGE |
| Master (36) 8 | SR_23 | Race | reach 75 cells in 145 s | 5 | FLOOR_RACE, rival ENEMY_TOWER_LEVEL_13 | random | 7 line(s) every 9 cells: UNDO, MINIBASE, UNDO, IVY, MINIBASE, UNDO, IVY | h10 LARGE, h20 LARGE, h30 LARGE, h40 LARGE, h50 LARGE, h60 LARGE, h70 LARGE |
| Master (36) 9 | SP_26 | Puzzle | place 8 bricks under a laser at 0.35, 1 life | - | FLOOR_PUZZLE_SOLID_BRICKS at 15 | fixed: O I L T L T Z J | - | - |
| Master (36) 10 | SS_20 | Survival | place 99 bricks, max lives lives | 5 | FLOOR_LWS at 12.5 | random | 7 line(s) every 9 cells: IVY, UNDO, PETRIFY, IVY, UNDO, MINIBASE, IVY | b5 LARGE, b6 LARGE, b7 LARGE_MYSTERY, b13 MYSTERY, b14 MYSTERY, b15 MYSTERY, b28 NO_ROTATE, b29 NO_ROTATE, b30 NO_ROTATE, b31 NO_ROTATE, b40 FAST_NO_ROTATE, b41 NO_ROTATE, b42 NO_ROTATE, b43 NO_ROTATE, b44 NO_ROTATE, b50 FAST_MIST, b54 FAST, b65 LARGE_NO_ROTATE, b67 LARGE_NO_ROTATE, b69 LARGE_NO_ROTATE, b71 LARGE_NO_ROTATE, b90 FAST_MYSTERY, b91 LARGE, b93 MIST, b95 LARGE_NO_ROTATE_MYSTERY, b99 FAST_LARGE_NO_ROTATE_MYSTERY |

## Notes
- Survival trials always use 3 lives, except the last Master Survival (99 bricks), which uses
  the game's maximum lives.
- The petrified-brick challenge is Rookie 6: its floor is FLOOR_LWS_PETRIFIED_BRICK, a Survival
  floor with a petrified brick on it, and its single spell line gives Petrify.

## The enemy wizard (read 2026-10-05, PC)
- Which: by trial kind, not per trial. Survival trials: wizard 2 (Wizard02); Race trials:
  wizard 3 (Wizard03); Puzzle trials: wizard 4 (Wizard04). It uses the normal wizard prefab
  and animations, with the player's brick pack.
- Where: x = 7 cells left of the lane centre (-7); in Puzzle trials 2 cells right (+2).
  y = the highest tower + 4 cells, followed with the same smoothing as the players' wizards.
  It has no tower of its own and no HUD seat; its curses come from the trial's curse spawner.
- It laughs hard in the trial intro, and once the player fails it laughs hard again every
  3-4 s (random each time) until the screen changes.
- The single-player explanation popups show before the first three trials only: SR_10
  (race), SS_17 (survival), SP_20 (puzzle).
