# Match moments in the original

Plain-language notes from Max's installed copy.

## Match text
- Countdown numbers, WINNER!, WIZARD OUT!, FINISH!, FAILED and WAVE use the Enzo letters, not
  pictures. Each is drawn twice: a back layer (letters fattened a little, a dark outline
  (37,13,40), face (158,23,64), or (160,24,101) when red) and a front layer, a white face
  coloured by a vertical gradient picture: gold for winners and the countdown, red for losers,
  bronze for a draw. Both have a solid shadow half a unit below in (227,33,61).
- Our build draws the shadow and a face tinted with the middle of the staged gradient; the
  fattened back layer, its outline and the gradient across each letter are not drawn yet.

## Race countdown
- "3, 2, 1", one a second while a tower holds past the finish line, 10 units above the line
  over the leading tower and 7 over the others.

## End of a match
- WINNER!: big gold text with turning sunrays behind it; it moves in over 2.6 s, idles, and
  moves out over 0.25 s.
- WIZARD OUT!: red text; moves in over 2 s, stays 2 s, moves out over 1 s.
- Winner spotlight: 0.5 s after the win a beam fades in over 1 s over the winning lane,
  centred on the lane's bottom, 1.5 wide and as tall as the screen. Opacity 151/255 in Race,
  130/255 in Survival, full in Puzzle.
- The veil once someone wins: not one per losing lane but two mirrored gradient strips that
  start 8 units left and right of the winner's lane centre, each the 387-pixel sprite
  stretched 5 times outward (about 60 units), at opacity 0.1. Only a 16-unit gap around the
  winner stays clear. The sprite is dark along its length and fades out at its right end
  (its pivot): the left strip is drawn as it is, the right one mirrored, so both fades face
  the winner. When the computer beats a lone player, one strip stretched 10 times
  covers the screen.
- Trophy (cups only): pops from 40% to full size over 0.6 s with an elastic bounce.
- No cup after a single match: cups show only at the end of a cup, sized Tiny, Big, Super or
  Online (staged as `trophy`, `trophy_big`, `trophy_super`, `trophy_online`).

## Lanes
The screen is 73.85 units wide with 3.42 margins and 1-unit gaps between lanes, so
each of n lanes is (67 − (n − 1)) / n wide: 33 for 2 players, 21.67 for 3 and 16 for 4.

## When the Game Completed popup opens, and XP (read from the PC install, 2026-10-04)

When the popup opens
- It opens once every tower has finished its end sequence.
- The winner's sequence counts as done straight away. WINNER! appears, the wizard casts and speaks over the next 2 s, and the banner stays up under the popup.
- Each loser's sequence:
  - Wait 1.5 s, with a 2 s controller rumble.
  - The tower collapses. Every brick stops being solid. Every 3rd brick is blown up one after another, or every brick if that wizard was already out, up to 100 bricks. Each blast has a small camera shake (0.2 × 0.26 for 0.05 s) and an explosion sound. The gap between blasts is (bricks still to go, at most 10) × a random 0.002 to 0.007 s.
  - A 2.5 s rumble shake (0.1 × 0.13) with a looping rumble sound.
  - A heavy brick sound.
- So the popup comes about 4 to 5 s after the match ends. Our 4 s guess is close.
- It slides up from the bottom-right corner in 0.25 s. It is not centred and does not hide WINNER!. Layout: tools/extract notes, "Game Completed popup".

XP (only used online and in cups; local matches give none)
- The winner gets 600 on Easy, 800 on Normal, 1000 on Pro.
- Last place gets 100. Second to last gets a quarter of the winner's XP, third to last half. Anyone else gets 0. Quitting costs 250.
- XP needed for each level: 0, 500, 2,500, 10,000, 25,000, 45,000, 75,000, 112,500, 162,500, 225,000, 300,000. After that, every 75,000 is another level.
- Level names: Rookie Magician, Apprentice Magician, Magician, Rookie Sorcerer, Apprentice Sorcerer, Sorcerer, Rookie Wizard, Apprentice Wizard, Wizard, Expert Wizard, then "Master Wizard Level N".

## Losing towers falling apart (read from the PC install, 2026-10-04)
- **Order of events for a loser** (after the winner background shows): wait 1.5 s, rumble the
  pad 2 s, then the collapse. The winner meanwhile gets WINNER! text, waits 0.5 s, casts its
  "other" animation with a voice line, waits 1.5 s and stops.
- **"Not solid" means free, not ghost.** In the original, `solid` is the frozen state: a solid
  brick's body is static. The collapse turns every brick of the tower (vine links too) back
  into a normal moving body, so the tower topples and falls under gravity while still
  colliding with itself and the base. The videos match this: the losers' towers turn pale grey
  (frozen) at the win, then crumble.
- **Which bricks blow up:** walking the tower's parts in their stored order (placement order),
  every 3rd brick (index 0, 3, 6…; every brick when the wizard was already out), at most 100.
- **Pace:** each blast runs at the same time as a camera shake (strength 0.2 × 0.26, 0.05 s) and
  the explosion sound (pitch 0.95–1.05). Before the next blast it waits
  min(10, blasts still to come) × a random 0.002–0.007 s, so a whole tower goes off in well
  under a second.
- **After the blasts:** a camera shake of 0.1 × 0.13 for 2.5 s with the rumble sound looping
  as long as it lasts, then the heavy brick landing sound once. The effect counts as finished
  when that sound has played.
- **One blast** hides the brick's picture at once, spawns the explosion particles at its
  centre, and removes the brick 0.7 s later.
- **Explosion particles** (three bursts, all at once, none loop):
  1. Flash: 1 particle of `gph_whiteCircleFaded_Large`, size 4, lives 0.15 s, nearly white
     (0.99, 1, 0.96), with a colour change over its life.
  2. Puffs: 8 particles of `gph_whiteCircle_Large`, size 1, speed 100, lives 0.5 s, shrinking
     to 0 over their life (still 76% at two thirds), with a colour/alpha change (fades out from
     86% of their life). The videos show these as dark puffs with white flashes.
  3. Debris: 6 particles of `gph_debris`, size 0.5, speed 20, gravity 1, lives 3 s, full size
     until 89% of their life, then shrinking to 0.
- **Sounds:** explosion `11_explosion_01`, rumble loop `026_Rumble`, closing thud
  `007_Brick_land_heavy` (staged as `sfx_explosion`, `sfx_rumble`, `sfx_brick_heavy`).
- **Pad rumble strengths** (0–255 per motor): a losing player's collapse 255 big / 0 small for
  2 s; losing a life 255 / 0 for 0.5 s; another rumble of 128 / 0 for 1.5 s in the general game
  controller; a spell effect trigger 255 / 255 for 1.5 s; a held spell effect 128 / 128 while it
  lasts; the start screen press 255 / 0 for 0.3 s. The 2.5 s blast shake has no pad rumble.

## Cups (read from the PC install, 2026-10-05)
- **Picking:** on Select Mode, the third column picks Single match or a cup (see video-notes.md).
  Local cups: **Tiny**, **Big**, **Super** (Online has its own).
- **Length:** a cup isn't a fixed number of matches. It's first to a points target: Tiny 6,
  Big 12, Super 18 (Online 9).
- **Points per match:** 4 − place: 1st gets 3, 2nd 2, 3rd 1, 4th 0. These are the "medals" the
  standings show.
- **Winning:** after each match, if exactly one player has the most points and has reached the
  target, they win the cup. If two or more are tied on the most points at or above the target,
  it's **overtime**: matches go on until one of them leads alone.
- **Modes:** each match's mode is drawn at random (from the cup's seed) from the modes allowed
  by the picks. When mode and difficulty are both "Random" (ANY), every mode is in the pool; the
  mode just played is taken out before the draw where the pool allows.
- **Between matches:** the results popup shows each player's row of medals against the target,
  with the cup's decoration (`gph_cupTinyMedium` etc. in the header), and a confirm button that
  pops in (0.15 s). The current score can also be shown as a popup during a match (fades
  0.1 s).
- **Cup winner:** a winner popup with the winner's name, their counts of 1st / 2nd / 3rd / 4th
  places, the cup's trophy flying in with the trophy sound, two turning sunray layers behind it
  (`gph_castEffectSunrays`, pale blue-white at 24%), and fireworks every 0.3–0.6 s. It closes
  by itself after 12 s (3 s fade).
  - Tiny: `gph_cupTinyLarge` 111×132 (17 px low) on 200×200 sunrays.
  - Big: `gph_cupBigLarge` 203×203 on 300×300 sunrays.
  - Super: `gph_cupSuperLarge` 307×247 on 350×350 sunrays.
  (Already staged as `trophy`, `trophy_big`, `trophy_super`.)
- **XP:** only online, non-friends cups give XP (the level-up popup after the winner popup).
  Local cups give none.

## Match intro (battles)
Read from the install (intro and countdown states, ModeTitleMessage and CountdownMessage
prefabs and their clips). Values are `GameConfig::intro`.
- **Mode title.** 0.3 s, then 0.3 s more, then a ribbon over the middle of the screen: the mode
  and difficulty in gold display letters ("RACE NORMAL", "SURVIVAL SPECIAL"), with "OVERTIME"
  under it in a cup tie. One ribbon per difficulty, each with its own picture: Easy 920 px wide
  with one star at each end, Normal 1220 px with three, Special 1420 px with five; Easy and
  Special sit 40 px lower than Normal (centre 120 / 160 px above the screen centre). Move-in
  0.6 s: it opens to full height in 0.08 s while its width overshoots and settles (Normal 642 →
  1289 → 1209 → 1229 → 1220). It stays 0.7 s, then moves out in 0.37 s: up 15 px, down to 95 px
  below its place, fading from 0.2 s. A countdown sound plays as it comes up.
- **Which matches show it.** Every single match; in a one-mode cup only the first match, and any
  overtime match.
- **The mode's own intro** (skipped by Rematch and Retry): Race waits 0.5 s, raises the camera's
  top and bottom together by the finish height less 17 cells over 0.8 s (in-out quad), holds
  0.5 s, comes back down in 0.386 s, and the countdown starts 0.2 s after it starts down.
  Survival counts each brick count up from 0, linear over 2 s (1 s with 33 bricks), with a
  count-up sound, then waits 0.3 s; the next-brick card flips through random bricks meanwhile
  (not done yet). Puzzle waits 0.5 s, brings the laser in, waits 0.5 s (the laser's entrance isn't
  done yet).
- **3-2-1.** Gold numbers 100 px above the centre: 3 at 0 s, 2 at 0.75 s, 1 at 1.5 s, each with
  the count sound pitched up (1, 1.17, 1.33) and a small pop (scale 1.15 back to 1). Play starts
  at 2.25 s. In Race the word "RACE" then shows, scaling 1.2 back to 1, until 3.0 s.
