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
- Halloween: it starts as the default pattern with the default colours; `_BlackWhiteEffect`
  then fades it to the Halloween texture with its own highlight and shadow colours (the staged
  `brick_halloween_<shape>` entries carry them). The fade starts 1 s after the brick spawns and
  takes 1 s (InOutQuad, HalloweenBrickModifier), so it usually happens while the brick falls,
  not on landing. Ours now does the same (`fade_halloween`); the grey tint on landing is gone.
- Christmas: its shader draws the lights from a two-frame picture (one frame above the other),
  one frame at a time over the brick, mixed in by the lights' alpha before the light. The frame
  is floor((offset + t) / 2) mod 2, so it switches every 2 s; each brick's offset is a random
  0 to 3 s (ChristmasBrickModifier). Frame 0 is the lower half in Unity's UVs. Our lights layer
  covered the box with the whole picture (both frames squashed), which put lights over thin
  air; it now shows one half at a time (`switch_lights`).
- Snow and Easter's grass and flowers (EdgeDecorationModifier): each brick prefab has anchor
  points along its edges; when the brick spawns, only anchors facing up (within 45 degrees,
  in the world) are kept, each gets a decoration by chance (snow 25%), with a random flip; it
  grows to 0.7 over 0.1 s (linear) after a 0 to 0.3 s delay, then to full size over 0.4 s
  (OutElastic), and turns with the brick after that.

In our build: art.rs `light_bricks` now lights from the shader's direction with its outline
colour and alpha (it used the material's 280 degrees and a colour measured from a clip). The
light is still baked per shape, so it turns with the brick; the Galaxy and Diamond materials
already keep it fixed in the world.

## Decoration anchors (read from the brick prefabs' decoration containers, 2026-10-10)
- Edge anchors sit on every outside edge of the shape, turned to face out (0 for a top edge, 90
  and -90 for the sides, 180 for a bottom edge). Only the ones facing up within 45 degrees are
  kept, which for a brick as it spawns are the cells' top edges.
- Front anchors sit at every cell's centre, four of them turned 0, 90, -90 and 180; only the
  upright one survives the same 45-degree rule, so it's one anchor per cell.
- Easter: grass on edge anchors (25% each, 3 kinds), an egg or flower on front anchors (40%
  each, 8 eggs and 8 flowers). Halloween: front anchors, 40% each, at most 2, turned up to 15
  degrees either way. Christmas: snow on edge anchors, 25% each.
- In our build: Easter now decorates every cell's face (40%) and every top edge (25%), as above
  (visuals.rs); before, a brick had at most one flower or egg.

