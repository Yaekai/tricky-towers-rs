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
