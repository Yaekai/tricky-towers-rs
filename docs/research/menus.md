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
from 0 to −90° every 2 s, looping. The corners pulse outward and back every second.
Text is white on the picked tile.

## Player select ("Select Players")
Four slots in equal columns under a 100-high header bar. Each slot, top to bottom:
a 318×500 character panel with the wizard sleeping until the player joins and ready
once they have; a brick-pack panel; small left/right arrows 165 px from the centre to
cycle choices; four small corners when picked. A deselected background 332×283 at
y +115, a lower strip 332×100 at y −110 and a name plate 367×78 at y −230. An empty
slot says "[button] To join", or "Connect controller" when a pad is missing.

## Mode select ("Select Mode")
Race, Survival, Puzzle and Random tiles, each with a difficulty badge: a centre badge
81×80, side wings 191×98 and stars 37×35, in versions for easy, normal and special.
Difficulty names shown: Easy, Normal, Special, and Random for any.

## Pause
"Game Paused" with Resume, Retry and Quit.

## End of a match
A "Game Completed" popup: the place ("1st Place" to "4th Place", or "Forfeit!"), the XP
gained ("{xp} XP") counting up, and "Level Up!" or "Level Down!". Buttons: Next match,
Rematch, Main menu. Wizard levels: Rookie Magician, Apprentice Magician, Magician,
Rookie Sorcerer, Apprentice Sorcerer, Sorcerer, Rookie Wizard, Apprentice Wizard,
Wizard, Expert Wizard, then Master Wizard Level N. Cups add a results table ("First to
{N} points", each player's score) and then a "{cup} Winner!" popup with the cup.
