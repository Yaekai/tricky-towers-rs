# Menus

Read from Max's install at 1920×1080. Positions are centres measured from the middle
of the screen, y up, in pixels.

## Title screen
There is no logo on this screen; the logo (with its debris and plants) is on the
"Press [continue] to start" screen shown before the tiles.

| Tile | Size | Centre | Art over the tile: offset, size |
|---|---|---|---|
| Local Battle | 437×335 | (−249, 225.5) | (−5, −25), 538×272 |
| Single Player | 435×335 | (−246, −155.5) | (17, −42), 488×235 |
| Online Battle | 441×335 | (278, 232.5) | (−21, −19), 467×283 |
| Shop | 450×165 | (277, −62.5) | (−20, −9), 483×128 |
| Help | 197×150 | (146, −250) | – |
| Settings | 197×150 | (396, −251) | cogs (0, 0), 181×134 |
| Quit | 436×68 | (0, −406) | – |

Some tiles hang from their top edge and others stand on their bottom edge, which is
why the raw anchored positions overlap; the centres above are worked out from them.
The art over a tile is bigger than the tile and pokes out of it. Tiles that aren't
picked show a grey version of it.

A picked tile has a coloured background, its art in colour, a frame over it, a header
bar, four corners (69×68 on big tiles, 38×34 on small ones) and five sun-ray layers
(556×556). There is no scale change. Ray layer 1 turns from 0 to 180° and layer 2
from 0 to −90° every 2 s, looping. The corners pulse linearly: 5 px out at 0 s and 1 s, at rest at 0.5 s (on small
tiles too).
Text is white on the picked tile.

## Player select ("Select Players")
Four seat columns between 5% and 95% of the width, each 22.5% wide, under an 800×90
header bar hanging from the top centre (9-sliced; borders 84 left, 92 right, 75 top, 15
bottom). Per seat, from the screen centre (y up): a 332×283 character panel at +115 with
the wizard window centred on the same point, small arrows at x ±165 on that line, a
332×100 brick-pack panel at −110 and a 367×78 name plate at −230. Panels and the outline
are 9-sliced with 60-pixel borders, the name plate with about 13. Small corners frame
both panels of a taken seat.

The wizard sleeps until someone joins. A taken seat shows the coloured panel, five
sun-ray layers cut to 327×267, the outline and the awake wizard. Seats 2 and 4 have no
awake picture of their own: the original reuses the title screen's wizards. Wizard
offsets from the panel centre and sizes: asleep 1 (0, 0) 213×250, 2 (0, 38) 225×323,
3 (−9, 14) 237×275, 4 (0, 11.7) 212×267; awake 1 (0, 0) 235×314, 2 (0, 21) 259×344,
3 (−3, 4) 279×300, 4 (7.5, 7) 253×322. Texts: "Select Players", "[continue] To join"
(two lines), "Player {n}".

## Mode select ("Select Mode")
Two columns, each a 480×800 panel, 540 px apart (x −270 and +270, y 0): first the mode
(Race, Survival, Puzzle, Random), then the difficulty (Easy, Normal, Special, Random).
Each shows one tile at a time, with 94×55 arrows 130 px beyond the top and bottom of the
513×445 frame around the active column's tile (frame at x −5.5 from the column, with the
large corners). Mode pictures: race (−3, 9) 487×411, survival (6, 0) 482×393, puzzle
(−7, −1) 505×392, random (0, 0) 495×393. Difficulty tiles: a 461×393 background at
(−5, 0), its own version for Special, with pictures easy (−3.9, 35.5) 209×157, normal
(−2, 38) 328×175, Special (−2.5, 42) 370×188, random (0, 50) 365×208, and the light and
dark spell icons 56×56 at (85, −190) and (145, −190). The third difficulty is PRO inside
the game, but the screen says "Special". The small mode-title banners with wings and
stars are shown at the start of a match, not here.

## Single Player
- One seat (Player 1's panel and wizard) 200 px above the centre.
- A list of 550×78 buttons from 175 px below the centre, 100 px apart: "Trials", then "Endless".
- "Endless" opens a screen with three wide buttons (Race, Survival, Puzzle, 690×173 pictures
  `menu_mode_endless`, `_survival`, `_puzzle`) with wizard faces (`menu_challenge_wizard2`-`4`),
  beside an 800×800 leaderboard.
- In this rewrite only Endless Race plays; Trials, Endless Survival and Endless Puzzle show
  dimmed with "(soon)". The Endless buttons' and leaderboard's spots, the faces' place and size,
  and the selected look are guesses (src/single.rs) until read.

## Pause
"Game Paused" with Resume, Retry and Quit.

## End of a match
A "Game Completed" popup: the place ("1st Place" to "4th Place", or "Forfeit!"), the XP
gained ("{xp} XP") counting up, and "Level Up!" or "Level Down!". Buttons: Next match,
Rematch, Main menu. Wizard levels: Rookie Magician, Apprentice Magician, Magician,
Rookie Sorcerer, Apprentice Sorcerer, Sorcerer, Rookie Wizard, Apprentice Wizard,
Wizard, Expert Wizard, then Master Wizard Level N. Cups add a results table ("First to
{N} points", each player's score) and then a "{cup} Winner!" popup with the cup.
