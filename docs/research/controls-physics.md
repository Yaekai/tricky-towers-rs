# Controls and physics in the original

Plain-language notes from Max's installed copy. Units are brick cells; the game steps at 0.02 s.

## Controls
Each player has eight actions: move left/right, nudge left/right, rotate left/right, drop, light
spell and dark spell.
- **Move:** 0.5 cells per step. The first step is immediate, the second after 0.2 s, then one every
  0.05 s. The brick's centre stays within 7 cells of the lane centre (lanes are 16 wide).
- **Nudge:** its own button, moves 1 cell at once. If it hits a brick, both get a 4.5 impulse and
  the brick is placed.
- A move blocked by more than 0.25 cells bumps: 0.6 impulse on the brick, 1.2 on the one it hits,
  and the brick is placed.
- **Drop:** holding speeds up by 60 cells/s² to at most 60. Letting go slows down by 100 cells/s²
  back to the drop speed.
- **Placement:** the steered brick is moved directly, not simulated, with its colliders shrunk 0.15
  on each side. The first contact places it, with no lock delay, and it becomes a normal physics
  body with zero velocity.
- **Spawn:** lane centre, at max(20 + camera bottom, tower) + 17.
- **Rotation:** 90° per press, separate left and right buttons, no auto-repeat, around the centre
  of the brick's bounding box. No wall kicks: the turn is tried at 15° steps; if the full 90° is
  free it turns, otherwise it turns until it touches and the brick is placed there.

## Bricks
- T, J and L spawn with the three-cell bar on top and the extra cell below (centre, right end,
  left end). S and Z boxes are 0.98 tall.
- Every brick has mass 1, whatever its shape or size. Mystery bricks are 1.5 or 1.75.
- A 7-bag randomizer per player; Race Pro gives everyone the same seed.
- **Drop guide:** a translucent column the brick's width over the whole visible lane, colour
  (133, 177, 255) at 75% opacity.

## Physics
- Gravity 18. Friction 0.4 (ice 0.06), no bounce. No linear drag; angular drag 0.05.
- Solver: 16 velocity and 6 position iterations; bodies sleep after 0.5 s.
- A steered brick ignores gravity until it touches something.
- "Stable" means both x and y speed under 0.5 for 1 s.

## Height and winning
- Tower height is the top of any placed brick, settled or not. Bubbled or ghost-held bricks don't
  count.
- Race: at or above the finish a 3-2-1 countdown runs (1 s each); dropping below cancels it.
  Otherwise the last player still in wins.

## Splash (read from the PC install, 2026-10-04)

- A falling brick splashes once, when its centre drops below the water line (y -8.5 cells).
- The splash appears at the brick's centre, moved down by a random 0 to 0.4. It is gone within 2 s.
- One of four splash sounds plays at random.
- On the splash the brick gets a push of -0.9 times its velocity, so it almost stops. Its spin is damped by 0.25. After that, an upward force of 17 acts on it every physics step, so it floats, unless it is a Bubble brick.
- Front layer (`splash_1`..`splash_3`, one picture per particle, picked at random): 5 at once, spread along a 2.86-wide horizontal line. They don't move. Each is 1.3 to 2.5 units across and lives 1 s. Size over life: 0, full at 0.10, 0.92 at 0.42, gone at 0.68. They draw in front of the brick.
- Drops: 12 small white circles, 0.3 to 0.4 units across, all at once from a disc of radius 1.5. They shoot straight up in a narrow cone (about 4.6 degrees) at 10 to 18 units/s. Gravity is twice the world's (2 × 18 = 36 units/s²). They keep full size until 0.45 s and shrink to nothing by 0.93 s. They draw behind the front layer.
