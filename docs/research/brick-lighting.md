# Brick lighting (read from the original's shaders, written in our words, 2026-10-10)

Read from the original's Direct3D 9 programs (unpacked and disassembled locally; nothing of it
is in the repo). The lit brick shaders (Default, Christmas, Halloween, Galaxy, Hologram) share
the same lighting:
- The bevel map's red and green are the slope (x2 - 1), turned into world space by the brick's
  rotation, so the light stays put when a brick turns.
- The light comes from a fixed direction, up and to the left: (-0.447, 0.894). The materials'
  `_Angle` isn't used by any of them (only Diamond's sweep uses an angle).
- Colour: the pattern, then the highlight by the slope towards the light times the highlight's
  alpha, then the shadow by the slope away times the shadow's alpha.
- Outline and cracks: a fixed dark purple (0.0745, 0.051, 0.157) by 1 - (1 - blue) x alpha of
  the bevel map (Galaxy and Diamond use their material's border colour instead).
- Alpha: the larger of the pattern's alpha and the bevel map's blue.
- A flash value lifts the colour towards white.

Pack extras, from the same reading:
- Halloween: in hand it's the default pattern with the default colours; placed, `_BlackWhiteEffect`
  fades it to the Halloween texture with its own highlight and shadow colours (the staged
  `brick_halloween_<shape>` entries carry them).
- Christmas: its shader has a lights texture and a time offset, so the blinking lights are part
  of the brick, drawn only on it (not yet read in detail).

In our build: art.rs `light_bricks` now lights from the shader's direction with its outline
colour and alpha (it used the material's 280 degrees and a colour measured from a clip). The
light is still baked per shape, so it turns with the brick; the Galaxy and Diamond materials
already keep it fixed in the world.
