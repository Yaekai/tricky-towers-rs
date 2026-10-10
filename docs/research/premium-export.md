# AssetRipper Premium export compared with the free one (2026-10-10)

Max bought one month of AssetRipper Premium so we could read the original's shaders. The
install was re-exported with Premium (shader export set to Decompilation) into a local,
gitignored folder and compared file by file with the earlier free export.

## What changed
- Both exports have the same 10,690 files. Premium added none and dropped none.
- 2,543 files differ, all of them only in reference IDs (Unity GUIDs are reassigned on every
  export). Checked by hand on an animation (Cheer) and a material (BrickGalaxyT): the values are
  the same, only the IDs of the sprites, shaders and textures they point to changed.
- So Premium gives no new data for this game apart from the shader setup below.

## Shaders
- None of the 58 shaders came out with real code. Each has its properties, render queue, blend
  mode, culling, depth writes and variant names, but empty program bodies.
- The reason: the game is Unity 5.5.1 and ships its shaders for Direct3D 9 only. Started with
  `-force-d3d11`, the player logs that Direct3D 11 was not built from the editor and shaders
  won't be available, then quits (the error Max saw). Neither AssetRipper's decompiler nor
  RenderDoc reads Direct3D 9.
- The shader setup it does show, for the brick packs: Transparent queue, normal alpha blending
  (source alpha, one minus source alpha), no depth writes, no culling, with ICE and WATER
  variants. Ours draws bricks the same way.

## Premium's other feature
- Static mesh separation: for 3D meshes; this game is 2D sprites, so it doesn't apply.

## What this means
- Shader maths (Galaxy, Hologram, Diamond, the brick lighting, Race Pro's lit islands) stays
  matched by measuring Max's clips and screenshots, as in docs/research/galaxy.md.
