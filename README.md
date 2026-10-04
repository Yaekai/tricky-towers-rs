# Tricky Towers, rebuilt in Rust + Bevy

A clean-room reimplementation of the physics brick-stacking game *Tricky Towers*,
written so it is easy to mod. It contains no code or assets from the original game.

## Run
```
cargo run                          # play with assets/config/game.ron
cargo run -- my_mod.ron            # play with another config
cargo run -- --play survival --difficulty pro   # skip the menu
cargo run -- --play race --offscreen shots --shots 5,20   # no window or sound; saves pictures
cargo test                         # config + headless gameplay tests
```
Controls: arrows or A/D to move, Shift + Left/Right to nudge, Up/W/Space to rotate clockwise and
Z anticlockwise, Down/S to drop faster, Q to cast your light spell and E your dark one, R to
restart, Esc for the menu. In the menu: Up/Down picks the mode, Tab the difficulty (Easy, Normal,
Pro), 1 to 4 how many people play, Left/Right the number of AI wizards, Enter starts.

Players 2 to 4 use gamepads: stick or d-pad to move, down to drop, LT/RT to nudge, A/B to rotate,
X for the light spell and Y for the dark one. With one player, a gamepad works for player 1 too.
In the menu, X changes the difficulty.

![A race against three AI wizards](docs/screenshot.png)

## What works
Values below come from reading Max's own copy of the original; the notes are in `docs/research/`.
- Controls like the original: half-cell moves (first repeat after 0.2 s, then every 0.05 s), a
  separate one-cell nudge that shoves what it hits, 90° turns both ways with no wall kicks (a
  blocked turn places the brick), and a drop that accelerates while held.
- The steered brick is moved directly and shrunk slightly; its first contact places it, at rest,
  as a normal physics body (avian2d). Gravity, friction, mass and damping match the original.
- The drop speed rises with the tallest tower and with each player who is out.
- Race, Survival and Puzzle at the original's three difficulties, with their finish heights,
  drop speeds, brick counts, lasers and start spells. Pro gives everyone the same bricks; Easy
  has no dark spells.
- Survival: lives with a moment of grace after each loss, a 3 s finish countdown, and on Pro the
  "Magic Waves" that curse everyone every 12 to 18 s.
- Puzzle: a shared random laser each round, a floor per difficulty (flat-ish, gappy, or a pillar
  with ledges), dropped bricks lowering the laser, most bricks wins.
- Endless Race (`--play endless`): a line rises from below and chases your tower top. Every spell
  line you reach (at 10, 20, 30...) is a checkpoint where it stops, eases back and then rises a
  little faster, up to 1.5 cells/s. Score is 10 × height; light spells only.
- Race Pro wind pushes loose bricks one way each match (strength 3 to 4, from the original;
  how strongly that pushes is a guess for now). Puzzle lanes are 1.2× wider, as in the original.
- Cursed and Mystery bricks that are lost come back where they fell, up to 3 times.
- Spells as the original awards them: each tower has a spell line, and reaching it gives a pair,
  one light spell and one dark curse; casting either throws the other away. Rubber-banding rules
  give players who are behind stronger spells.
  - Light (on yourself): Ivy, Petrify, Undo, Minibase, Auto Build, Extra Life (Survival).
  - Dark (on everyone else): Large, Fast, Ice, No Rotate, Auto Rotate, Mystery, Bubble, Balloon
    (Slow), Mist, Grassy, Ghost, and their combinations.
- Local multiplayer for up to 4 players, the original's main way to play: keyboard for player 1,
  gamepads for the rest.
- Optional AI wizards on their own towers. The original has none in battles; they're an extra.
- Mode menu, HUD, drop guide, next-brick preview, seeded 7-bag randomizer.
- The original's art, sound and letters, loaded at runtime from your own install once extracted
  (see `docs/art-layout.md`): bricks with stone and ice versions, bases, the full-screen
  backgrounds with sky, clouds, islands and water per mode, lines, spell icons, player panels,
  animated wizards beside each tower that react to the game, music and sounds. Nothing from the
  original is in this repo; without the extracted folder the game draws plain shapes.
  To build that folder on Windows, export your install with AssetRipper and run the scripts in
  `tools/extract/` (steps in `tools/extract/README.md`).
- Everything tunable (controls, physics, mode rules, spell weights, AI skill) is in
  `assets/config/game.ron`.

## What's missing
See `PLAN.md`. In short: Trials, Race Pro wind, spell effect animations (Large
grow, Auto Rotate wind-up, rising balloon, vines growing), countdown and results screens, and the
title menu in the original's layout.

## Modding
Copy `assets/config/game.ron`, change what you want (any field can be left out),
and pass it as the first argument. Bricks can be any polyomino. Code-level mods
can replace any plugin in `src/lib.rs`, or drive `PlayerInput` from a bot.

## Credits
*Tricky Towers* is by WeirdBeard Games. This project is an unaffiliated fan reimplementation;
to play the real thing, buy it.
