# Tricky Towers, rebuilt in Rust + Bevy

A clean-room reimplementation of the physics brick-stacking game *Tricky Towers*,
written so it is easy to mod. It contains no code or assets from the original game.

## Run
```
cargo run                          # play with assets/config/game.ron
cargo run -- my_mod.ron            # play with another config
cargo test                         # config + headless gameplay tests
```
Controls: arrows or WASD to move, Up/W/Space to rotate, Down/S to drop faster, R to restart.

## What works
- Bricks fall under player control, move in half-cell steps and rotate 90 degrees with wall kicks.
- On touching the tower a brick is handed to the physics engine (avian2d), so towers wobble and topple.
- Race mode: build past the finish line and hold it for 3 seconds.
- Survival mode: place N bricks; each dropped brick costs a life.
- Seeded 7-bag randomizer, so runs are reproducible.
- Everything tunable (physics feel, brick shapes, mode rules) is in `assets/config/game.ron`.

## What's missing
See `PLAN.md`. In short: spells, puzzle mode, wizard and AI opponents, multiplayer,
art, sound, menus, and tuning against the real game.

## Modding
Copy `assets/config/game.ron`, change what you want (any field can be left out),
and pass it as the first argument. Bricks can be any polyomino. Code-level mods
can replace any plugin in `src/lib.rs`, or drive `PlayerInput` from a bot.

## Credits
*Tricky Towers* is by WeirdBeard Games. This project is an unaffiliated fan reimplementation;
to play the real thing, buy it.
