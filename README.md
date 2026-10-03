# Tricky Towers, rebuilt in Rust + Bevy

A clean-room reimplementation of the physics brick-stacking game *Tricky Towers*,
written so it is easy to mod. It contains no code or assets from the original game.

## Run
```
cargo run                          # play with assets/config/game.ron
cargo run -- my_mod.ron            # play with another config
cargo test                         # config + headless gameplay tests
```
Controls: arrows or A/D to move, Shift + Left/Right to nudge, Up/W/Space to rotate clockwise and
Z anticlockwise, Down/S to drop faster, Q to cast your light spell and E your dark one, R to
restart, Esc for the menu. In the menu: Up/Down picks the mode, 1 to 4 how many people play,
Left/Right the number of AI wizards, Enter starts.

Players 2 to 4 use gamepads: stick or d-pad to move, down to drop, LT/RT to nudge, A/B to rotate,
X for the light spell and Y for the dark one. With one player, a gamepad works for player 1 too.

![A race against three AI wizards](docs/screenshot.png)

## What works
Values below come from reading Max's own copy of the original; the notes are in `docs/research/`.
- Controls like the original: half-cell moves (first repeat after 0.2 s, then every 0.05 s), a
  separate one-cell nudge that shoves what it hits, 90° turns both ways with no wall kicks (a
  blocked turn places the brick), and a drop that accelerates while held.
- The steered brick is moved directly and shrunk slightly; its first contact places it, at rest,
  as a normal physics body (avian2d). Gravity, friction, mass and damping match the original.
- The drop speed rises with the tallest tower and with each player who is out.
- Race (finish 45 cells up, coming down after 120 s), Survival (66 bricks, 3 lives) and Puzzle.
- Spells as the original awards them: each tower has a spell line, and reaching it gives a pair,
  one light spell and one dark curse; casting either throws the other away. Rubber-banding rules
  give players who are behind stronger spells.
  - Light (on yourself): Ivy, Petrify, Undo, Minibase, Auto Build, Extra Life (Survival).
  - Dark (on everyone else): Large, Fast, Ice, No Rotate, Auto Rotate, Mystery, Bubble, Balloon
    (Slow), Mist, Grassy, and their combinations.
- Local multiplayer for up to 4 players, the original's main way to play: keyboard for player 1,
  gamepads for the rest.
- Optional AI wizards on their own towers. The original has none in battles; they're an extra.
- Mode menu, HUD, drop guide, next-brick preview, seeded 7-bag randomizer.
- Everything tunable (controls, physics, mode rules, spell weights, AI skill) is in
  `assets/config/game.ron`.

## What's missing
See `PLAN.md`. In short: original art and sound loaded from your own install (the loader is in, see
`docs/art-layout.md`), the Ghost spell, Survival curse waves and Puzzle floors, Trials and Endless, and
difficulty levels (the defaults are Normal).

## Modding
Copy `assets/config/game.ron`, change what you want (any field can be left out),
and pass it as the first argument. Bricks can be any polyomino. Code-level mods
can replace any plugin in `src/lib.rs`, or drive `PlayerInput` from a bot.

## Credits
*Tricky Towers* is by WeirdBeard Games. This project is an unaffiliated fan reimplementation;
to play the real thing, buy it.
