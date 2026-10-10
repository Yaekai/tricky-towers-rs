# Galaxy brick pack (read from the original's code and export, written in our words)

Sources: the install's game assembly (decompiled to a scratch folder, not in the repo), and the
export's prefabs and materials (the Galaxy brick pack prefabs and the BrickGalaxy materials).

## The scene
- The pack's scene (prefab BRICK_PACK_GALAXY) is created once, at world x -1000, out of view. It
  has a Background sprite, a Camera (orthographic size 5, so 10 units across) that draws the scene into GalaxyRenderTexture (512x512),
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

## Field size and window (read from the prefab and the clip)
- The camera's orthographic size is 5, so the sky is 10 units across, and the Background sprite is 512 px at 51.2 px to the unit, exactly 10 units. The field is 10 units, not 9.2 (the stars' spread). A brick cell is one unit (the brick sprites are 65 px to the unit: the T is 201x136 px). Our field is now 10 cells.
- The clip (Race trial, Galaxy tower) shows the stars staying put while a brick falls through them, so the bricks are windows onto a world-space sky. Ours already works that way.
- The colour spread didn't change with the field size (blue median still about 107 against the clip's 142): the blue gap is elsewhere.

## Measured in Max's Race trial clip (1080p, a falling L piece, one cell = 28 px)
- Outline: about 2.5 px dark, (46, 31, 87) at its darkest: 0.09 of a cell. Ours is now 0.09 (`GALAXY_LINE_WIDTH`).
- Top bevel: a light band about 4 px, (201, 115, 223). Bottom bevel: a softer band, about (145, 78, 214).
- Body: flat violet, about (147, 31, 185). That blue is above anything in the background picture (its bluest is 141), so the original's shader brightens the sky's blue; how is not readable (the shader is compiled, and AssetRipper's free edition can't decompile it). A RenderDoc capture of the original would show it.

## Why the Galaxy shader can't be read from the install (2026-10-10)
- The game is Unity 5.5.1 and ships its shaders for Direct3D 9 only. Started with `-force-d3d11`, the player logs that Direct3D 11 "was not built from editor, shaders will not be available" and quits.
- AssetRipper Premium's shader decompiler gave empty program bodies for all 58 shaders (only properties, blend modes and variant names), and RenderDoc can't capture Direct3D 9.
- What the export does give: the Galaxy brick shader draws in the Transparent queue with normal alpha blending (SrcAlpha, OneMinusSrcAlpha), no depth writes, no culling, with ICE and WATER variants. Ours draws the same way.
- So the Galaxy colour stays matched to Max's clip by measurement (see above), not read from the shader.

## Body colour matched to the clip (2026-10-10)
- With the sky gain `GALAXY_BODY` (linear (1.38, 1.29, 4.25), from the clip's body (147, 31, 185) against the picture's (127, 27, 95)), our spread over the same tower is R 57 / 131 / 201, G 20 / 35 / 97, B 77 / 146 / 254 (10th / median / 90th), against the clip's R 59 / 121 / 175, G 20 / 36 / 99, B 84 / 142 / 210. The medians match; our brightest parts run a little hot.

## Drift direction (2026-10-10)
- The original moves the sky's texture offset by (0.05, -0.07) a second in Unity's UVs, where v runs up: the sky slides left and up. Max's Race trial clip agrees (inside a settled brick, the sky moves left and a little up between frames half a second apart; the match is rough because the bevel doesn't move).
- Ours slid left and down: our texel rows run down, so the -0.07 had the wrong sign. Fixed in `galaxy_window` and `place_galaxy_stars`. Checked by reasoning and against the clip; an offscreen comparison of two pictures wasn't usable (bricks were placed between them).

## The real shader, disassembled (2026-10-10)
Read from the original's Direct3D 9 programs (exported as Yaml, unpacked and disassembled
locally with Windows' d3dcompiler_47; nothing of it is in the repo). The normal variant
(no ice, no water):
- Sky: sampled in world space at 0.1 (one repeat every 10 cells), plus a sway: 0.1 cos(0.3 t)
  across and 0.2 sin(0.1 t) up and down, in texture widths (t in seconds). On top of the
  scene's own scroll.
- Hue: the sky colour is turned by 2pi x frac(vertex red + 0.5) - pi with a YIQ-style matrix
  (the usual 0.299 / 0.587 / 0.114 constants). The vertex red is each prefab's sprite colour:
  I 0.103, J 0.125, L 0.059, O 0.029, S 0, T 0.051, Z 0.147, so each shape has its own tint.
- Light: the bevel map's red and green are the slope, turned into world space, lit from a fixed
  direction (-0.447, 0.894). Highlight by the slope towards it times the highlight's alpha,
  shadow likewise away from it, then the border colour by 1 - (1 - blue) x alpha. Alpha is the
  larger of the map's alpha and blue. A flash value lifts the colour towards white.
- The ice variant uses fixed bluish colours (0.47, 0.43, 0.75) and (0.33, 0.34, 0.75) for the
  shadow and outline and adds faint diagonal stripes in screen space; the water variant adds
  the water colour and line under the water's cutoff.
- In our build now: per-shape hue skies (`GALAXY_HUES`, baked once), the sway in
  `galaxy_offset`, the fixed light in art.rs `light_galaxy`, and the outline from the bevel
  map (our own thin line is gone). The light is baked per shape, so it turns with a rotated
  brick, where the original's stays fixed (to do).
- Measured over the same tower: R 45 / 100 / 192, G 7 / 30 / 124, B 58 / 114 / 202, against the
  clip's R 59 / 121 / 175, G 20 / 36 / 99, B 84 / 142 / 210. The shapes in the two towers
  differ, so the spreads aren't like for like; ours reads a little darker. Not chased further
  until a like-for-like frame (same shapes) is compared.
