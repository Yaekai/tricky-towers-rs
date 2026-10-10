# Galaxy brick pack (read from the original's code and export, written in our words)

Sources: the install's game assembly (decompiled to a scratch folder, not in the repo), and the
export's prefabs and materials (the Galaxy brick pack prefabs and the BrickGalaxy materials).

## The scene
- The pack's scene (prefab BRICK_PACK_GALAXY) is created once, at world x -1000, out of view. It
  has a Background sprite, a Camera that draws the scene into GalaxyRenderTexture (512x512),
  and a Stars parent with 49 star sprites at fixed positions.
- Every Galaxy brick's colour pattern is that render texture (the material's pattern slot). The
  bricks show it through their shape, so every brick is a window onto the same sky.

## Motion
- The sky scrolls by moving the Background's texture offset: 0.05 across and -0.07 down per
  second, forever (the texture repeats). Nothing scrolls per brick.
- Each star has a random start rotation, and an idle pulse: scale 1 + 0.3 x sin(2 t + x y),
  so the stars don't pulse in step.
- Every 2 to 6 s (random) one random star twirls: scale to 2 over 0.3 s; spin 180 degrees over
  1 s (InBack ease); after a 2 s delay, scale back to 1 over 0.3 s. While it spins, its spin
  speed starts at 600 degrees a second and loses 200 a second squared.

## The brick
- Lit like the default bricks: its material is the brick shader with the Galaxy colours
  (highlight (1.0, 0.809, 0.910, 0.653), shadow (0.774, 0.526, 0.993, 0.616), angle 280,
  cutoff 0.5), and a border colour (0.125, 0.029, 0.243). The light-line colour (0.722, 0.729,
  0.914) is in the material too.
- The bricks are see-through where the picture is.

## Not yet read
- How the seams between cells are drawn (the overlap or the line on a shared edge). The shader
  is in the export as a stub, so the seams can't be read from the install. The bars stayed with
  our lines off on the PC, so they are probably seams in our build, not the original's lines.
- How many stars show at once in the original's frame (the 49 are all in the scene).

## Stars: size and glow (read from the prefab)
- Each star's sprite is 18 px at 26 px to the unit: 0.69 units, at scale 1 in the scene. Our star size is now 0.69 cells (`GALAXY_STAR_SIZE`).
- The star renderers use the default sprite material, white, no tint, sorting order 1. The prefab has no separate glow layer or bloom, so any glow in the frames comes from the star art itself. Our star art is the same sprite.

## Colour (checked against Max's frames, not yet matched)
- Median brick colour on the same crop: the original about (131, 35, 146), ours about (120, 32, 98) with the bevel. We are too red and not blue enough.
- A tint on the starfield made ours darker, not bluer, so it was reverted. The gap is probably in the field's own colours or the bevel's highlight (the highlight colour is pinkish). Not fixed.
- The field: the Background sprite is gph_galaxy_background, the same picture we stage. Its median colour is (127, 27, 95), so the raw field is already our colour; the missing blue comes from the shading on top. The original's shadow colour is violet (0.77, 0.53, 0.99 at 0.62), and that's where the extra blue would come from. So the next check is the bevel's light direction and how much of each brick is in shadow, not the field.

## Shading strength test (not kept)
- Doubling the bevel's shading strength moved every channel: the median went to about (138, 33, 112) against the clip's (121, 36, 142), so red overshot and blue barely moved. Reverted; the blue gap isn't a shading-strength problem.
