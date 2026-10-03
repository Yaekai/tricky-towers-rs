# Loading the original's art and sound

The game can draw the original's sprites and play its sounds, taken from your own installed copy.
None of those files are in this repo, and none may be committed: they live in the gitignored
`extracted/` folder next to `Cargo.toml` (or wherever `TT_EXTRACTED` points). Without that folder
the game draws plain coloured shapes and stays silent.

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

## Names the game looks up
| Name | What |
|---|---|
| `brick_I`, `brick_O`, `brick_T`, `brick_S`, `brick_Z`, `brick_J`, `brick_L` | Whole-brick sprite in spawn orientation (T, J and L with the bar on top) |
| `mystery_block`, `mystery_blob`, `mystery_table` | The three Mystery shapes |
| `platform_race`, `platform_survival`, `platform_puzzle` | The base each tower stands on |
| `background_0`, `background_1`, ... | Background layers, back to front |
| `life_orb` | Extra Life's heart orb |
| `music_menu`, `music_race`, `music_survival`, `music_puzzle` | Looping music |
| `sfx_land`, `sfx_move`, `sfx_nudge`, `sfx_rotate`, `sfx_drop_lost`, `sfx_line_collect` | Your brick and tower |
| `sfx_cast_light`, `sfx_cast_dark`, `sfx_countdown`, `sfx_win`, `sfx_lose` | Spells and match events |

Still to be wired up (stage them if found; the game will use them later): `stone_overlay`,
`ice_overlay`, `vine_overlay`, `grass_overlay`, `finish_line`, `spell_line`, `laser`,
`drop_guide`, `heart`, `bubble`, `balloon`, `mist`, `water`, and `spell_icon_<Spell>` for each
spell (Ivy, Petrify, Undo, ExtraLife, Minibase, AutoBuild, Large, Fast, NoRotate, AutoRotate,
Ice, Mist, Slow, Mystery, Bubble, Grassy).

## extracted/notes.md
One line per entry saying what it is and its pixel size, plus anything useful that has no name
above (for example the wizard sprites). Say which cells the Mystery blob sprite covers; the game
currently guesses that shape.

## Checking it
`cargo run` logs `loaded N images and M sounds from extracted` at startup, and warns about any
manifest entry whose file is missing.
