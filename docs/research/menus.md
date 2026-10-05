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
- One seat (Player 1's panel and wizard) 200 px above the centre, and the 332×100 brick-pack
  panel 225 px below the seat's centre. No name plate on this screen.
- A list of 550×78 buttons from 175 px below the centre, 100 px apart: "Trials", then "Endless".
  The buttons are `menu_button_background`, 9-sliced 18/16/71/15 (left, right, top, bottom as
  read; check the order if it looks stretched). Endless carries the 54-high
  `menu_button_cup_background` band across its width. The picked button gets the four small
  corners (`menu_corner_small`) 10 px outside it.
- "Endless" opens a screen with three wide buttons down the right: the column's top-left is at
  (+135, +375) from the centre, 690 wide, 200 px apart. Pictures `menu_mode_endless` 690×173,
  `_survival` 689×167, `_puzzle` 690×224 drawn 28 px up. Wizard faces 110×108 at (33, 6) from a
  button's bottom-left: Race wizard 3, Survival wizard 2, Puzzle wizard 4.
- The 800×800 leaderboard sits on the left, its right edge on the centre line, 30 px below the
  centre: `gph_popupBackgroundPanel` (staged as `menu_leaderboard_panel`), 9-sliced top 51 /
  bottom 37. Rows are `gph_leaderboardEntryBackground` (`menu_leaderboard_row`), 9-sliced left 6 /
  right 7. Row size, spacing and text are guesses until read.
- In this rewrite only Endless Race plays; Trials, Endless Survival and Endless Puzzle show
  dimmed with "(soon)". The Endless buttons' selected look is a tint until read.

## Pause
A full-screen dark overlay (0.016, 0.012, 0.035 at 90%) with a panel (0.031, 0.020, 0.094 at
85%). "Game Paused" sits 480 px in from the left at +260; Resume, Retry and Quit are below
it from +120, each on a 30-high row with `menu_item_selected` (28 high, 9-sliced) behind the
picked one. The panel's size and the row pitch (60 px here) are guesses.

## End of a match
The "Game Completed" popup is not centred. It slides up from below in 0.25 s (ease out quad)
into the bottom-right corner: content 610 wide × (60 per item + 70) high, the
`popup_background` panel (9-sliced, top border 198, bottom 6) reaching 120 px past the right
edge and 100 px below the bottom. Items are right-aligned labels (465×30, 40 px in from the
right), 60 px apart, with `menu_item_selected` (24 high, 30 px in, as wide as the words)
behind the picked one. Local items: "Next match" (only when more than one mode was picked),
"Rematch", "Select Mode", "Main menu". There's no place line, and WINNER! stays visible
beside it. It opens once every loser's tower has collapsed, about 4 to 5 s after the end
(docs/research/match-moments.md).

XP only counts online and in cups. Wizard levels: Rookie Magician, Apprentice Magician,
Magician, Rookie Sorcerer, Apprentice Sorcerer, Sorcerer, Rookie Wizard, Apprentice Wizard,
Wizard, Expert Wizard, then Master Wizard Level N. Cups add a results table ("First to {N}
points", each player's score) and then a "{cup} Winner!" popup with the cup.

## Endless leaderboard rows (read from the PC install, 2026-10-04)
- The original's Endless leaderboard is **online** (Steam): Global and Friends lists, 10 rows
  each, loaded with a spinner. On disk it keeps only your own best score, your rank, and
  whether the score was sent. There's no local list of runs.
- **Row:** 42 px tall, full width of the list, `gph_leaderboardEntryBackground` tinted
  (0.82, 0.85, 0.95); your own row uses a darker tint (0.75, 0.78, 0.94).
- **Rank:** at x 15 from the left, 96.5 px wide, centred, size 26 white. Ranks 1–3 show a medal
  instead of the number: `gph_leaderboardMedalGold` (44×52), `…Silver` (40×49), `…Bronze`
  (44×52), 3 px below centre. Above 9999 the number is capped.
- **Avatar:** the player's Steam picture, 36×36, at x 130 from the left, vertically centred.
- **Name:** left aligned, size 26 white, starting at x 165 (125 when there's no avatar), with
  110 px kept free on the right.
- **Score:** right edge, 100 px wide, centred, size 26 white, the plain number.

## Settings screen layout (read from the PC install, 2026-10-05)
- Same frame as the other menus: header panel `gph_menuHeaderBackground` 800×91 at the top
  centre (title "SETTINGS"), menu arrows `btn_menuArrow` 94×55 at x ±500 on the centre line to
  switch between the two pages. The pages sit side by side 1920 px apart and slide.
- Rows sit in an 800 px wide area from 220 px below the top down to the bottom, a vertical list
  with 100 px between rows, centred (the first row is at +50 × (rows − 1)).
- **Every row:** 78 px tall, full width, `gph_settingsMenuItemBackground` (staged already as
  `menu_button_background`). Icon 100×100 box at x 79 from the left, 11 px above centre (the
  icon pictures are 43–107 px wide, see `settings_icon_*`). Title at x 150, size 36, white,
  left aligned. The picked row shows the four small corner pieces (`menu_corner_small`, 38×34)
  10 px outside each corner.
- **Slider row** (Sound effects, Music): a 360 px value area at the right edge: left arrow at
  40 px, right arrow 40 px from its right end (25×31 `settings_arrow`), and a 190×40 bar
  (`settings_slider_empty` under `settings_slider_fill`, cut to the value) starting 85 px in.
- **Choice row** (Resolution, Quality, Language): the same 360 px area with both arrows and the
  value text centred between them, size 36.
- **Toggle row** (Explanations, Full screen, V-sync): a 43×43 box (`settings_checkbox`) 60 px
  from the right edge, showing a tick (`settings_tick`, 61×49, shifted 9 px right, 3 px up) when
  on or a cross (`settings_cross`, 49×49) when off.
- **Popup row** (Stats, Credits, Keybinding): just icon and title. Choosing it opens its popup.
- **Apply row:** applies the graphics changes on page 2.

## Trials screen layout (read from the PC install, 2026-10-05)
- Header "SELECT GAME" frame like the other menus; a medal panel at the top right
  (`menu_info_background` 382×183, its centre 120 px right of and 100 px above the corner, so
  partly off screen) with a large medal (`medal_large` 70×96) and the count "12/35" (size 46),
  plus a small "MEDALS:" label.
- One page per tier, 1920 px apart, switched with the menu arrows at x ±500 (or the bumpers).
  The tier's name is the page title.
- A page holds its 10 trials in 2 columns of 5: rows 125 px apart, columns 440 px apart.
  Arrows move between them.
- **Trial button:** 423×101. The picture is per mode: `trial_button_race` (405×101),
  `trial_button_survival` (399×90) or `trial_button_puzzle` (423×101). The trial's number sits
  in the middle (size 46). A wizard face sits on the left (x −119): wizard 3 for Race, 2 for
  Survival, 4 for Puzzle (the `menu_challenge_wizard*` faces). The medal (70×96) sits at
  x +145 once the trial is beaten. The picked one has small corners on a 449×134 frame.
- **Locked tier:** a 783×215 panel (`trial_locked_background`) over the page, with "LOCKED"
  (size 60), "Earn X to unlock" (size 46), a medal, and a padlock (`trial_lock`) on a chain of
  links (`trial_chain_link`, `_short`, `_end`) running across it. When the tier unlocks, the lock
  pops to 1.3× (ease out back) with chain sounds (4 variants, 0.2 s apart), then the whole panel
  slides 1000 px up in 0.5 s after 0.2 s, with brick spawn and roof placed sounds.
- Starting a trial plays the mode's start stinger.

## Explanation (tutorial) popups (read 2026-10-05, PC)
Positions are in pixels of a 1920×1080 screen, from the centre (y up) unless said otherwise.
There is no panel: the popup fills the screen over `modal_tile` (gph_modalTile, tiled) and
everything sits straight on it. All text is white, all caps (TMP uppercase style).

| Part | Where | Size | Text size |
|---|---|---|---|
| Title | (0, 410), centred | 1600×100 | 75 |
| Subtitle (the mode's rule line) | (0, 333.5), centred | 1600×50 | 26 |
| Mode picture | (0, 61), centred | 487×414 box | - |
| Difficulty picture | (0, -235) | 209×157 box | - |
| Difficulty title (Easy / Normal / Pro) | (0, -318) | 1200×70 | 46 |
| Difficulty subtitle (the Pro line) | (0, -380) | 1200×70 | 26 |
| Light spell icon slot | (-35, -450) | 55×55, `spell_slot_light` | - |
| Dark spell icon slot | (35, -450) | 55×55, `spell_slot_dark` | - |
| CONTINUE [continue] | bottom-right, 30 px in, 50 px up | 400×50, right aligned | 46 |
| Disable all explanations | bottom-left, 30 px in, 50 px up | 850×70, left aligned | 46 |

- Text keys: title `EXPLANATION_TITLE_<MULTIPLAYER|SINGLE_PLAYER>_<RACE|SURVIVAL|PUZZLE>`,
  subtitle `EXPLANATION_<same>`, Pro line `EXPLANATION_MULTIPLAYER_<MODE>_PRO_SUBTITLE`,
  difficulty `DIFFICULTY_EASY|NORMAL|PRO`, buttons `NAVIGATION_CONTINUE`,
  `EXPLANATIONS_DISABLE`.
- Mode pictures: Race `menu_mode_race`, Survival `menu_mode_survivalsp` (battle and single
  player alike), Puzzle `menu_mode_puzzle`, single-player Race
  `menu_explanation_mode_race_sp`.
- Difficulty pictures: `menu_difficulty_easy|normal|pro`. Single-player explanations hide the
  whole difficulty block.
- The two icon slots hold what that difficulty takes away or changes (light slot, dark slot):
  Race Easy: -, magic off; Race Normal, Survival Normal: none; Race Pro: none (just the Pro
  line); Survival Easy: -, magic off; Survival Pro: -, magic off; Puzzle Easy: Undo, magic
  off; Puzzle Normal: Ivy, magic off; Puzzle Pro: magic off, magic off. "Magic off" is
  `menu_unavailble`; Ivy and Undo are `spell_icon_Ivy` / `spell_icon_Undo` (50×50 in the slot).
- In: each part slides up from 1500 px below over 0.5 s, starting at title 0.1 s, subtitle
  0.2 s, mode picture 0.3 s, the difficulty block 0.4 s, the two icons 0.5 s (ease: DOTween
  default, OutQuad). Out: the same delays, sliding 1500 px up. CONTINUE and "Disable" appear
  only once everything is in; "Disable" only while explanations are on.
- Closes on any player's Continue, a mouse click anywhere, or the Invite button ([invite],
  which also turns explanations off and saves it). The match intro starts after.
- When: before a match, only while Explanations is on in Settings. Battle: per mode and
  difficulty. Single player: `SINGLE_PLAYER_<MODE>_01` (which trials use it: not read, ?). A controls explanation popup
  (`ControlsExplanation`) comes first when the mode asks for it.

## Credits popup (read 2026-10-05, PC)
- Full screen over `modal_tile`. A first screen (1080 high) shows `logo` (818×420, centred)
  and under it, at y -260, `CREDITS_THANKS_FOR_PLAYING` (size 60, white, caps).
- Then the list from `credits_content` (Resources\content\credits.json, staged): groups of
  `titles` and `names`; a title is a language key when one exists, else shown as written;
  `nameFirst` puts the names above the titles. Titles: size 60, 78.6 high, then 10 px gap;
  names: size 46, 70 high; 50 px between groups. Centred, white.
- After the last group: `menu_weirdbeard_logo` (601×434, 100 px below its slot top), then 50 px.
- Scroll: the whole content moves up by (list height + 1080) in 60 s, linear, after 0.5 s;
  then the popup closes itself. Holding Down speeds it to 10×, holding Up runs it backwards
  at 10×; any button or a click closes it. Lines only show while on screen.
