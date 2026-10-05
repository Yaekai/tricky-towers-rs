# Wizards

Plain-language notes on the original's wizards, from Max's install (read on the PC; the full
frame lists are in the local `extracted/inventory.md`).

## Characters and parts
- 14 characters: Wizard01–04, WizardAnimal01–04, WizardIndie01–06.
- Each is drawn from parts: back arm, body, face, front arm, hat (some also a hat back), a
  cloud they sit on and a cloud front, and on some a cape or tail.
- Parts have pose variants (`_action`, `_cheer`, `_preAction`, `_preCast`, faces `idle01`,
  `idle02`, `laugh`, `startle01`, `startle02`, ...). Animations swap these at set times,
  sampled at 30 frames a second.

## Animations
Idle, Action, Cheer, CastStart, Casting, CastEnd, CastSelf, Laugh, LaughHard, Startle, Dying,
and for capes CapeIdle, CapeUp, CapeDown, CapeWave.
Example, Action: body plain at 0 s, `_preAction` at 0.2 s, `_action` at 0.433 s; the face goes
idle02, preAction, action at the same times; the back arm switches at 0.433 s.

## Where they float
- Beside their own tower: x = lane centre + 5.5 cells (−11 and mirrored in single player).
- Height: the tower's height + 4, eased towards it each frame (lerp by the frame time).
- They bob: x by 0.5 × sin(t), y by 0.55 × sin(2t).

## What makes them move
| Event | Animation |
|---|---|
| Brick placed | Action, but only every 3rd to 8th brick (random countdown) |
| Life lost | Startle (always) and a small shake, 0.1 × 0.13 for 0.2 s |
| Roof placed | Action, then Cheer |
| Spell picked up | Cheer |
| Light spell cast | CastSelf |
| Dark spell cast | Casting, then an evil laugh |
| Hit by a dark spell | Startle |
| Out of the game | Dying |

Laugh and Startle each have a 7 second cooldown. Each also plays that character's voice line
(STARTLE, LAUGH, LAUGH_EVIL, LAUGH_HARD, CHEER, ACTION, CAST_SELF).

## Voice lines (read 2026-10-05, PC)
Every voice line is a sound that plays when a wizard enters an animation state of the same
kind, so it follows the animation rules above (cooldowns included). Staged by
`tools/extract/wizards.ps1` as `wizard<n>_say_<kind>_<i>` (i from 1) for all 14 characters;
the older single `wizard<n>_say_<clip>` entries are still staged until the code switches.

**Kinds**: `startle`, `laugh`, `laugh_evil`, `laugh_hard`, `cheer`, `action`, `cast_self`,
`cast_other`. Dying has no kind of its own: a wizard leaving the game says a STARTLE line.

**Who says what**
- Wizards 1-4 share one pool. Its laughs (`laugh`, `laugh_evil`, `laugh_hard`) play at volume
  0.8; everything else at 1.0. Counts: startle 3, laugh 8, laugh_evil 7, laugh_hard 3, cheer 4,
  action 7, cast_self 5, cast_other 5 (some files are shared between action and the casts).
- Wizards 5-14 each have their own startle (the "sad" files), laugh, laugh_evil and cheer
  lines, and one set of cast lines used for action, cast_self and cast_other alike. None has
  its own laugh_hard: they all use wizards 1-4's three long laughs (at 0.8, pitch 1.13).
- Per-character volume: 5-8 (Spirit Animal) 1.0, except wizard 5 (platypus) 0.65; 9 (Lethal
  League) 0.3; 10-14 (Awesomenauts, TowerFall, Armello, Nuclear Throne, Octodad) 1.1.
- Pitch: 1.13 for wizards 1-8 (the original's default); 1.0 for wizards 9-14 (their games'
  own voices, played as recorded).

**Choosing a line**: plain random pick among that kind's lines each time (repeats allowed),
no pitch or volume variation beyond the fixed values above.

**When each kind plays in a match**
- startle: hit by a dark spell, a life lost, leaving the game.
- laugh_evil: after casting a dark spell (shares the 7 s laugh cooldown).
- laugh_hard: the trial's enemy wizard when it casts, and single-player mode intros.
- laugh: not used in a local match; the menus' wandering and player-driven wizards use it.
- cheer: spell picked up, after a roof is placed, the winner.
- action: the Action animation (bricks, roof).
- cast_self: casting a light spell.
- cast_other: the winner's cast at the end of a match (0.5 s after WINNER) and the beam a
  wizard casts on another tower.
- Each wizard's line is its own sound: two wizards can talk at once; nothing queues or cuts.

**Menus**: picking a character on Select Players (and poking an already picked one, at most
every 0.25 s) says one of that character's cheer lines. The Single Player pick does the same.

**Volume slider**: voices are sound effects; they follow the Sound effects slider.
