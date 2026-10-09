# Roofs at the end of a round (read from the original's code, written in our words)

The original's roof is a brick-like piece, `ROOF_01` for Race and `ROOF_02` for Survival and
Puzzle (the factories set `roofResourceName`). It is placed by a local roof controller:

1. It spawns at the brick spawn height, in the middle of the tower (not over a lane), and the
   drum roll starts (`SFX_DRUM_ROLL`, looping) with a long rumble on the player's controller.
2. The player steers it like a brick (left, right and nudge), but it never turns.
3. It comes down at the drop speed until it touches the tower (`_HandleBrickCollision`). Then
   it becomes physical, joins the tower, the rumble stops and the roof-placed trigger runs
   (`ROOF_PLACED`, sound `sfx_roof_placed`).
4. A 0.7 s timer starts. When it ends the round finishes with the tower's height as its value.
5. While the roof is up the tower's bricks stop counting toward the height, and the roof's
   upside-down angle (165 to 195 degrees) is recorded as a stat.

Differences in our build (src/gameplay.rs `steer_roofs`, `RoofStage`):
- We steer a roof only for the winner of a Race or Survival match. The original also uses
  the roof for Puzzle (`ROOF_02`), so Puzzle's end should get one too (check on the PC).
- We have no drum roll and no rumble while the roof comes down.
- We rest for `roofs.rest_seconds` (0.7 s, the same as the original) after landing before the
  match moves on.
- The roof's collision, the "roof placed" sound and its art are not checked against the
  original's roof (ROOF_02 art size and where it lands are still to read from the install).

Art names: `ROOF_01` = `Game/Roofs/Roof01`, `ROOF_02` = `Game/Roofs/Roof02` (the install's
resource table, InitResourcesCommand.cs).
