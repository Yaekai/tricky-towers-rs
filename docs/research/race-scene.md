# The Race scene (menus and Race matches)

Read from the install's code, prefabs and shader programs (PC, 2026-10-10). Our own words.

## What the menus show
The original's menus have no backdrop picture. A background controller runs the Race
background and foreground live behind every menu screen; the Endless screen and Select Mode
show the picked mode's own background instead (Select Mode in the picked difficulty's look, so
Special shows the Special scene).

## Layers, back to front
- Sky gradient, then two large clouds (3 rounds), drifting right 0.15-0.25 units/s, parallax 0.2.
- Islands (parallax 0.3), all rooted at the water's top (6 units up), drawn at 0.9 scale round
  their roots. Each has a stretched waterline strip (offsets and sizes in `GameConfig::race_scene`).
  All but the back one bob: to a random 0.2-0.4 above or below rest over 2.5-4 s, in-out quad,
  then back the other way.
- A reflected light band at the water's top (0.22 of its picture's height), tint
  (0.86, 0.92, 1), alpha 0.5 +/- 0.2 on a 5 s sine.
- Water: a 6-unit gradient, light (137, 146, 217) at its top to (61, 82, 201) at the bottom.
- Wave crests: one every 1.4 s per row, at a random place across the view and height in the
  row; the higher the smaller. Over 6 s each fades in and out, grows from 0.2 to 0.7 of its
  width and back, and drifts right 4 times its scale (all out-quad). Back row: 3 units up plus
  up to 2, scale 0.5 down to 0.2, parallax 0.3. Front row (over the towers): 2.5 plus up to
  0.5, scale 0.7 down to 0.5, parallax 1.
- Small clouds: 6 pictures and 4 pictures, 3 rounds each, 0.5 to 2 views up, 0.4-0.5 units/s,
  parallax 0.6 and 0.8. A cloud that leaves the right edge comes back just before the left.
- Foreground water along the bottom (80 units wide).

## The island waterline (island shader)
Across the picture (x from 0 to 1) the line sits at `offset + bob + amplitude/2 x
sin(5x - t/2) sin(8x + t) sin(3x + t)` of the picture's height (amplitude 0.03, offset 0.05;
the back island 0 and 0). The bob is the island's lift over its height, taken off, so the line
stays still while the island moves. Just above the line (0.008 of the height) the picture
whitens, by 0.8 at the line; just under it is a white foam fading from 0.8 to 0.1 of the
picture's alpha over 0.015; under that the island isn't drawn.

## The foreground water (flag wave shader)
The picture is read from `y + 0.07 + 0.07 x sin(9x + w) sin(30x + w/10) sin(5x + w/2)` (w = 2t),
with x across its 80 units, so its light top edge ripples.
