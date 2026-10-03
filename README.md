# Tricky Towers, rebuilt in Rust + Bevy

A clean-room reimplementation of the physics brick-stacking game *Tricky Towers*,
written so it is easy to mod. It contains no code or assets from the original game.

## Run
```
cargo run                          # play with assets/config/game.ron
cargo run -- my_mod.ron            # play with another config
cargo test                         # config + headless gameplay tests
```
Controls: arrows or WASD to move, Up/W/Space to rotate, Down/S to drop faster, E to cast your
spell, R to restart, Esc for the menu. In the menu: Up/Down picks the mode, Left/Right the
number of AI wizards (0 to 3), Enter starts.

![A race against three AI wizards](docs/screenshot.png)

## What works
- Bricks fall under player control, move in half-cell steps and rotate 90 degrees with wall kicks.
- On touching the tower a brick is handed to the physics engine (avian2d), so towers wobble and topple.
- A drop guide under your brick and a preview of the next one.
- Up to 3 AI wizards, each on its own tower, getting the same bricks as you in the same order.
- Race: first tower to stay past the finish line for 3 seconds wins.
- Survival: place N bricks; each dropped brick costs a life; last one standing wins.
- Puzzle: place a set number of bricks without anything resting above the laser line.
- Spells, earned every 5 bricks, one held at a time:
  - light (on yourself): Vines bind touching bricks together, Stone freezes your resting bricks, Shrink halves your next brick;
  - dark (on the tallest opponent): Enlarge makes their next brick bigger, Slippery makes it icy.
- Mode menu, HUD with every tower's height and the last spell cast.
- Seeded 7-bag randomizer, so runs are reproducible.
- Everything tunable (physics feel, brick shapes, mode rules, spells, AI skill) is in `assets/config/game.ron`.

## What's missing
See `PLAN.md`. In short: the original's full spell list and how spells are earned,
trials/campaign levels, local multiplayer, original art and sound loaded from your own
install, and tuning against the real game.

## Modding
Copy `assets/config/game.ron`, change what you want (any field can be left out),
and pass it as the first argument. Bricks can be any polyomino. Code-level mods
can replace any plugin in `src/lib.rs`, or drive `PlayerInput` from a bot.

## Credits
*Tricky Towers* is by WeirdBeard Games. This project is an unaffiliated fan reimplementation;
to play the real thing, buy it.
