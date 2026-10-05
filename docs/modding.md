# Modding

Everything the game plays by is data: rules, timings, bricks, spells and art names. There
are three ways to change it, and they stack.

## The mods folder
Make a folder called `mods` next to the game (in the folder you run it from, or next to the
program). Each mod is either:

- a single file, `mods/<name>.ron`, with config changes, or
- a folder, `mods/<name>/`, holding `config.ron` (config changes), `manifest.ron` (an art and
  sound pack), or both.

Mods apply in name order, so `mods/b.ron` wins over `mods/a.ron`. The log says which mods
loaded, and a mod that doesn't read is skipped with the reason.

### Config changes
Write only the fields you want to change. Sections merge field by field, so this changes the
race finish line and gravity and keeps everything else:

```ron
// mods/tall_races.ron
(
    gravity: 24.0,
    race: (finish_height: 40.0),
)
```

Lists and tuples are replaced whole: a mod that sets `pieces: [...]` gives the full brick list.
Every field and what it means is in `src/config.rs`; `assets/config/game.ron` lists them all
with the shipped values.

### Art and sound packs
`mods/<name>/manifest.ron` uses the same format as `extracted/manifest.ron`
(`docs/art-layout.md`), with file paths inside the mod's folder:

```ron
// mods/neon_bricks/manifest.ron
{
    "brick_I": (file: "bricks/I.png", pixels_per_unit: 32.0),
    "music_race": (file: "race.ogg"),
}
```

An entry replaces the extracted one with the same name. Anything a pack leaves out still comes
from your own install, and anything missing from both is drawn as plain shapes.

## A whole other config
`cargo run -- path/to/my.ron` starts from that file instead of `assets/config/game.ron`; the
mods folder still applies on top.

## Code
Every system lives in a plugin listed in `src/lib.rs`, and every tower is steered through its
`PlayerInput`, so a plugin can replace a system or drive a tower from a bot.
