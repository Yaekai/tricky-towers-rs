# Loading the original's art and sound

The game can draw the original's sprites and play its sounds, taken from your own installed copy.
None of those files are in this repo, and none may be committed: they live in the gitignored
`extracted/` folder next to `Cargo.toml` (or wherever `TT_EXTRACTED` points). Without that folder
the game draws plain coloured shapes and stays silent.

The game finds the folder whatever it is started from: `TT_EXTRACTED` first, then `extracted/` in
the working folder, then `extracted/` next to the executable or any folder above it (so
`target\debug\tricky_towers.exe` finds the one beside `Cargo.toml`). The first one holding a
`manifest.ron` wins. An image that fails to load is dropped with a warning and drawn as plain shapes.

## extracted/manifest.ron
A RON map from a logical name to a file inside `extracted/`:

```ron
{
    "brick_I": (file: "bricks/I.png", pixels_per_unit: 100.0, pivot: (0.5, 0.5)),
    "platform_race": (file: "platforms/race.png", pixels_per_unit: 100.0, pivot: (0.5, 1.0)),
    "sfx_land": (file: "audio/land.ogg"),
}
```

- `file` is relative to `extracted/`. PNG for images; OGG or WAV for sounds.
- `pixels_per_unit` is the sprite's pixels per brick cell (Unity's value; 100 if left out).
  Bricks ignore it and are stretched to their cells.
- `pivot` is the sprite's pivot from its bottom left, 0 to 1 (the centre if left out). A platform's
  pivot should sit at the middle of its top surface.
- Any name can be left out; that piece falls back to plain shapes.

Background layers take four more optional fields:

```ron
"background_3": (file: "bg/island.png", pixels_per_unit: 100.0, pivot: (0.5, 1.0),
    position: (0.0, -1.0), z: 3, parallax: 0.6, scale: 1.0),
```

- `position`: where the pivot sits, in cells from the middle of the platform's top surface
  (x right, y up). A layer with a position is drawn once in every tower's lane, at
  `pixels_per_unit`. A layer without one is stretched to cover the whole window.
- `z`: draw order, back to front (the layer number if left out). Every background is behind the
  bricks.
- `parallax`: how the layer follows the camera climbing: 1 (default) moves with the tower, 0 stays
  at the same height on screen, 0.5 moves half as fast. Lanes never shift sideways.
- `scale`: extra size factor (1 if left out).

## Names the game looks up
| Name | What |
|---|---|
| `brick_I`, `brick_O`, `brick_T`, `brick_S`, `brick_Z`, `brick_J`, `brick_L` | Whole-brick sprite in spawn orientation (T, J and L with the bar on top), drawn untinted |
| `brick_<P>_stone`, `brick_<P>_ice` | The same brick turned to stone (Petrify, Auto Build, ivy onto stone) and iced (Ice curse); a grey or blue tint of the plain sprite if left out |
| `mystery_<name>_stone`, `mystery_<name>_ice` | Same, for the Mystery shapes |
| `mystery_block`, `mystery_blob`, `mystery_table` | The three Mystery shapes |
| `platform_race`, `platform_survival`, `platform_puzzle` | The base each tower stands on |
| `background_0`, `background_1`, ... | Background layers, back to front (fields above) |
| `background_race_0`, `background_survival_0`, `background_puzzle_0`, ... | Layers for one mode only; when any exist for a mode they replace the plain `background_N` set |
| `finish_line`, `laser` | Race finish and Puzzle laser, repeated sideways across the lane; pivot on the line |
| `spell_line` | The spell line, repeated sideways; pivot on the line |
| `spell_orb_light`, `spell_orb_dark` | The light and dark pickups at the ends of the spell line |
| `water` | Water under all lanes, repeated sideways; pivot at the middle of its top edge (sits at the depth where dropped bricks are lost) |
| `drop_guide` | The beam under your brick, stretched to the brick's width and drop distance |
| `wizard_1` ... `wizard_4` | Portrait in each lane's player panel, left to right |
| `heart` | One per life in a player panel |
| `spell_slot_light`, `spell_slot_dark` | Frames of the two spell slots in a player panel |
| `spell_icon_<Spell>` | Spell icon in a slot, for Ivy, Petrify, Undo, ExtraLife, Minibase, AutoBuild, Large, Fast, NoRotate, AutoRotate, Ice, Mist, Slow, Mystery, Bubble, Grassy, Ghost |
| `life_orb` | Extra Life's heart orb |
| `music_menu`, `music_race`, `music_survival`, `music_puzzle` | Looping music |
| `sfx_land`, `sfx_move`, `sfx_nudge`, `sfx_rotate`, `sfx_drop_lost`, `sfx_line_collect` | Your brick and tower |
| `sfx_cast_light`, `sfx_cast_dark`, `sfx_countdown`, `sfx_win`, `sfx_lose` | Spells and match events |

Still to be wired up (stage them if found; the game will use them later): `vine_overlay`,
`grass_overlay`, `bubble`, `balloon`, `mist` and `minibase`.

## extracted/notes.md
One line per entry saying what it is and its pixel size, plus anything useful that has no name
above (for example the wizard sprites). Say which cells the Mystery blob sprite covers; the game
currently guesses that shape.

## Checking it
`cargo run` logs `loaded N images and M sounds from extracted` at startup, and warns about any
manifest entry whose file is missing.

## Matching the original's look
Bricks are drawn as staged, with no colour on top. To place the background layers like the
original, give each one the `position`, `z` and `parallax` the original uses for a lane (the
values are plain numbers about where a picture sits, not code). The camera shows each tower from
just below the water line up to a few cells above the spawn point.
