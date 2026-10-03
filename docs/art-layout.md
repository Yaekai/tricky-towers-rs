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

A layer can instead be pinned to the screen, like the original's background scene (sky, clouds,
islands on the water band), by adding `screen`:

```ron
"background_5": (file: "bg/island_left.png", pixels_per_unit: 100.0, pivot: (0.0, 0.0),
    screen: (0.0, 0.0), position: (0.4, 0.6), view_height: 10.0),
"background_8": (file: "bg/cloud_2.png", pixels_per_unit: 100.0,
    screen: (0.5, 0.7), position: (-3.0, 0.0), parallax: 0.4),
"sky": (colors: [(66, 133, 230), (120, 180, 245), (200, 230, 255)]),
```

- `screen`: the point of the window the layer counts from, 0 to 1 from its bottom left
  ((0, 0) bottom-left corner, (0.5, 0) bottom middle, (1, 0) bottom-right corner).
- `position`: offset from that point, in screen units (0 if left out).
- `view_height`: how many units tall the window is; `position` and `pixels_per_unit` use these
  units (10 if left out). Use the original background camera's height.
- `parallax`: how far the layer slides down as the camera climbs the tower, as a share of the
  climb (0, fixed, if left out).
- `sky` has no file: `colors` are the gradient behind everything, top to bottom, 0 to 255.
- `water_band` has no file either: a gradient along the bottom of the screen, `colors` top to
  bottom, `position: (0, height)` in screen units (6 if left out), with `view_height`, `z` and
  `parallax` as above.
- `sky_race`, `sky_survival`, `sky_puzzle` and `water_band_<mode>` replace `sky` and
  `water_band` in that mode.
- `mask: [(r, g, b), (r, g, b)]` on any image: it is a two-colour mask the original recolours.
  Red areas take the first colour, yellow areas the second, shades in between blend, and the
  mask's brightness and alpha are kept.
- `font_main`: a .ttf or .otf used for all text.
- Animations are entries with no file: `"wizard_1": (frames: [(0.0, "wizard_1_idle_0"),
  (0.2, "wizard_1_idle_1")], length: 0.6, looping: true)`. Each frame names a staged image and
  shows from its time until the next. Any name the game looks up can be an animation and will
  play (for example `wizard_1` in the player panel). `looping` is true if left out; `length` is
  the last time plus the step before it if left out.
- Sounds and fonts are told apart from images by file type (.ogg, .wav, .mp3, .flac; .ttf,
  .otf), so their names are free.

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
| `spell_line` | One pickup of the spell line; `count` of them (16 if left out) are spread across the lane, tinted in turn with `colors` (0 to 255; a 9-colour rainbow if left out) |
| `spell_orb_light`, `spell_orb_dark` | The light and dark pickups at the ends of the spell line |
| `water` | Water under all lanes, repeated sideways; pivot at the middle of its top edge (sits at the depth where dropped bricks are lost) |
| `drop_guide` | The beam under your brick, stretched to the brick's width and drop distance and tinted blue (133, 177, 255) at 75% |
| `vine_overlay`, `grass_overlay` | Drawn over a brick's sprite when it has vines or grass, stretched to the brick (a green tint if left out) |
| `bubble` | Drawn around a brick lifted by Bubble |
| `mist` | The Mist fog over a tower, faded in and out |
| `wizard_1` ... `wizard_4` | Portrait in each lane's player panel, left to right |
| `heart` | One per life in a player panel |
| `spell_slot_light`, `spell_slot_dark` | Frames of the two spell slots in a player panel |
| `spell_icon_<Spell>` | Spell icon in a slot, for Ivy, Petrify, Undo, ExtraLife, Minibase, AutoBuild, Large, Fast, NoRotate, AutoRotate, Ice, Mist, Slow, Mystery, Bubble, Grassy, Ghost |
| `life_orb` | Extra Life's heart orb |
| `menu_background`, `logo` | Title screen: a picture filling the menu, and the game's logo above the options |
| `music_menu`, `music_race`, `music_survival`, `music_puzzle` | Looping music |
| `sfx_land`, `sfx_move`, `sfx_nudge`, `sfx_rotate`, `sfx_drop_lost`, `sfx_line_collect` | Your brick and tower |
| `sfx_cast_light`, `sfx_cast_dark`, `sfx_countdown`, `sfx_win`, `sfx_lose` | Spells and match events |

Still to be wired up (stage them if found; the game will use them later): `balloon` and
`minibase`.

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

## Inventory (for the extract step on Max's PC)
Write `extracted/inventory.md`: every sprite, texture, font and sound in the original, grouped by
what it is for (menus and title, HUD, wizards and their animations, bricks and spell effects,
backgrounds per mode, match moments such as countdown, win and lose, particles, audio). For each,
give its size and the logical name it is staged under, or `-` if it is not staged yet, and
suggest a logical name for the unstaged ones following the patterns above. Animations: list the
frames in order and the frame rate. This is the checklist the game's art work follows; plain
descriptions only, no code.
