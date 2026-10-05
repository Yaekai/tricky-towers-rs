# Trial lanes, rival towers, trial floors, roofs, Endless Puzzle obstacles

Read from Max's install on 2026-10-05 (PC). Plain-language notes; numbers are the original's.
Units are cells (1 cell = 1 brick square).

## 1. Rival towers (race trials)
- A race trial has two lanes, the usual two-player screen split: the player on the left, the
  enemy wizard's lane on the right. The enemy lane holds a race house (the same base as Race,
  `FLOOR_RACE`) with a pre-built rival tower standing on it at the lane's centre (x 0, y 0 =
  the top of the house). Survival and Puzzle trials have the enemy lane too, but empty (no floor,
  no tower), just the enemy wizard.
- The enemy wizard floats 7 cells left of its lane's centre (2 in Puzzle trials). Its camera
  follows the player's camera up and down, so the rival tower scrolls with the player's view.
- Each rival tower (ENEMY_TOWER_LEVEL_01..13) is a list of stone bricks: shape, centre,
  rotation and a scale (1, or 2 for an enlarged brick). Some have a minibase ledge (left one,
  `gph_towerPiece01L_var01`) and some bricks carry full-grown ivy. The bricks are pictures only,
  they have no colliders. Brick art: `gph_brick<shape>_stone` (a stone look of its own, not the
  Petrify one). The layouts are staged from the install as `extracted/rival_towers.ron` by
  `tools/extract/rival_towers.ps1`, not kept in the repo.
- Level 1 has 12 bricks reaching about 12 cells; level 13 has about 70 reaching about 73.
- The pop-in, at the trial's intro: all bricks start hidden and appear in list order over
  1.5 s (the number shown at time t is round(count × t / 1.5)). Each brick shows a small spawn
  poof at its spot (`BRICK_SPAWN_POOF_SMALL`). A bubble pop (`SFX_BUBBLE_POP`) plays at most
  every 0.075 s, pitch 1 + 1.5 × progress. The tower never changes after that.
- When the player wins, the enemy wizard just cheers; it has no roof.

## 2. Trial floors
- "FLOOR_X at 12.5": the number is the floor's height, used only by the camera: how far below
  the base's top the view may go (the camera's lowest bottom is −height) and the brick spawn
  height (max(20 + bottom, tower height) + 17). It is not a position or a scale.
- `FLOOR_LWS` is the Survival castle (House02), the same base as battle Survival.
  `FLOOR_LWS_PETRIFIED_BRICK` is the castle with a stone I lying flat on the right merlon,
  overhanging to the right: a 4 × 1 box from x 2 to 6, y 0 to 1. It is part of the floor (the
  floor's own collider, kinematic, cannot be knocked off), drawn with `gph_brickI_petrified01`
  (our `brick_I_stone`).
- Puzzle trial floors are their own pieces, not battle Puzzle's random columns. Each is a set
  of static boxes plus pictures: column pieces (`gph_house01/02/03_world03`, flipped for the left
  side), petrified desert bricks (`gph_brick<shape>_petrified_desert`) set into the floor,
  `gph_house04_world03` and `gph_house05_world03` blocks, and two nudge signs in the nudge
  tutorial. All boxes and pictures are in `GameConfig::trial_floors` (box x and y ranges from the
  base centre and top; pictures with their spot, flip and turn).
- Puzzle trials put the player's lane at 2/3 of the screen width; bricks may be steered 15.5
  cells either side.

## 3. Tier unlock animation (Trials screen)
- Plays when the Trials screen opens and a tier is unlocked that hasn't been shown unlocked
  yet (after earning the medals, on the way back from a trial). 0.8 s wait first if a trial was
  just completed (after the level-complete animation), else none.
- The page first slides to that tier (0.3 s, the usual page move), then the chain breaks: each
  link (all but the padlock) shrinks to nothing in 0.1 s, starting after (1 − d / dmax) × 0.6 s,
  where d is the link's distance from the padlock: the far links go first. A chain sound
  (`SFX_CHAIN_1..4`, random) plays as a link starts, at most every 0.2 s.
- When the nearest link has gone: 0.3 s later the padlock turns back to 0° and grows to 1.3×
  over 0.45 s (ease out back), fading out over 0.25 s from 0.5 s; `SFX_BRICK_SPAWN` and
  `SFX_ROOF_PLACED` play as it starts.
- Then, after 0.2 s, the locked panel slides 1000 px up in 0.5 s. The tier's trial buttons
  start 1000 px low and slide up into place in 0.5 s each, 0.05 s apart.
- Chain layout on the locked panel (px from its centre, rotation): padlock at (−323, 28.3),
  8.5°. A long run of 13 links from (−278, 35.1) to (137.4, 96.9), 34.6 px apart along x and
  5.15 px up each, turned 188.5°, ending in a short link at (161.3, 101.8). Left of the lock, a
  link end at (−372.9, 20.9), −351.6°. Hanging down: links at (−318.4, −16.3) and (−313.2, −50.9)
  and an end at (−307.9, −85.5), all −261.4°. Up: a link at (−332.2, 74.6) and a short link at
  (−338.0, 102.2), both 278.7°. Links 48 × 35 px, short links 24 × 17, padlock 68 × 63.

## 4. Endless Puzzle obstacle blocks
- A block is a fixed 1 × 1 floor piece (`FLOOR_PUZZLE_PRO_PIECE_1..11`, random; drawn with
  the Pro ledge looks `gph_house08/09/10_world03`), static, not counted in the tower height.
  Half the time a grass tuft (`GRASS_0..6`, random, tinted #9DA247, randomly flipped) sits on
  top: it waits 0.5 s, grows to 0.7× in 0.1 s, then to full size in 0.4 s (ease out elastic).
- 4 blocks at each level-up. Positions: rows from ceil(tower height) + 1 up to
  floor(new laser height − camera offset); the block fills the cell under that row line. Columns
  at x = ±0.5, ±1.5 ... ±6.5 from the lane centre with weights 0.01, 0.05, 0.1, 0.2, 0.3, 0.5,
  2 (the outermost far likelier). Each of the 4 is drawn by weight from all (column, row) spots
  left, without repeats. The camera offset is 0.5 per dropped brick (0 while none is dropped).
- Level-up order: the drop guide hides and bricks stop; 0.3 s wait; the laser rises 4 cells
  over 0.5 s (sound `MOVE_LASER_1..3`); then per block 0.1 s, a spawn sound
  (`SPAWN_PUZZLE_BLOCK_1..3`) and the block; 0.3 s; then the laser zaps (`SFX_LASER_START`),
  bricks resume and the level spell is handed out. The next level allows min(26, 6 + 2 × level)
  bricks. The run continues only while no brick has been lost.
- No 1.2× zoom was found in the Endless Puzzle scripts.

## 5. Roofs
- The winner of a Race or Survival match (battle and trials; Endless Race and Endless Survival
  have one too) gets a roof to place: it spawns at the brick spawn height over the lane centre
  and is steered like a brick (left/right, nudges, faster fall held) but cannot turn, falling at
  the default drop speed 2. The winner's tower stops moving (physics stopped) for the roof.
  A drum roll (`SFX_DRUM_ROLL`) loops from the spawn until the roof lands.
- On the first contact the roof becomes part of the tower and the drum roll stops (the roof
  placed effect plays `018_Roof_placed`). 0.7 s later the roof is done and the match moves on
  (WINNER!, then the popup). A roof steered off the side counts as placed once its top falls
  below the camera's bottom.
- `ROOF_01` (Race, Endless Race): a stepped spire, art `gph_roof01` drawn at its origin. Boxes
  (from the origin): 4.5 × 2 at (0, −0.415); 3 × 1 at (0, −1.915); two 1 × 1 feet at (±1, −2.911);
  3 × 1 at (0, 1.085); 2 × 1 at (0, 2.085); 1 × 1 at (0, 3.085).
- `ROOF_02` (Survival, Endless Survival; set for Puzzle but Puzzle never drops one): art
  `gph_roof02` drawn at (0, −7.13). Boxes: 1 × 0.5 at (0, −5.13); 4 × 3 at (0, −6.88); two
  1 × 1 feet at (±1.5, −8.88).
- Puzzle has no roof: the winner's wizard cheers and the match finishes.
- Upside down: a roof turned 165-195° earns a stat (not shown in play).

## 6. Values from the game's settings
- Default drop speed 2 cells/s; brick spawn offset 17 above the tower; up to 20 cells of tower
  on screen; maximum health 3 (Endless Survival's lives).

## 7. Finish line
- Every finish line (battle and trials) is a row of `gph_finishLineEnd` tiles, 1.2 cells apart
  across the lane (the original's names are the wrong way round: the "End" picture is the
  repeating tile), with a `gph_finishLinetile` cap at each end: as is on the left, turned 180�
  on the right, about 1.4 cells past the last tile. Staged as `finish_line` (tile) and
  `finish_line_end` (cap).

## 8. More values read
- Race win: the 3-2-1 countdown, one number a second (`hold_seconds` 3).
- Drop speeds: Race 2 / 3 / 4 (Easy / Normal / Pro), Survival 2 / 2 / 4, Puzzle 2 on all three,
  Endless Race 4, Endless Survival 4, Endless Puzzle 2.
- Medals: one per trial beaten, out of as many as there are trials.
- Auto Rotate: winds back 15� in 0.2 s, swings +105� in 0.45 s, both ease in-out elastic, then
  waits 1 s.
- No Rotate: a refused turn shakes the brick's picture for 0.3 s, a random jump of up to 0.1 cell
  each way every frame (divided by the brick's scale); the padlock on it jumps up to 0.06.
- Minibase: the ledge piece's flat top is 2 cells wide, with a raised lip on its outer side.