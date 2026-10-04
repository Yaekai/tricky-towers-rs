# Gap scan of the install (2026-10-05)

A sweep of every resource and script in the original, against what we had staged, noted and
coded. This file lists what had been missed and what has been read so far. Skins are in
skins.md.

## 1. Puzzle Pro look, and the sandstorm in every Puzzle
- **Every Puzzle (Easy, Normal, Pro):** a sand strip along the bottom of the background
  (`puzzle_sand`, `gph_pinkSand`) and a two-layer **sandstorm**: scrolling see-through sand
  textures, back layer `sandstorm_back_1/2` tinted #9B8644 at 38% (behind the ruins' front),
  front layer `sandstorm_front_1/2` tinted #E9AA5D at 37% over the play area. They scroll slowly
  (time factor 0.035, offset per layer 0.025). We have neither yet.
- **Ruins colours, normal:** back #F4D9B0 to #F7E0BE, middle #E0AC92 to #D69883, front #C47464
  (parallax 0.3 / 0.5 / 0.5).
- **Puzzle Pro** swaps the sun and colours:
  - Sky gradient #AB054C (top) to #B84442 (bottom), plus 3 soft reflected gradients at heights
    9.1, 20.5 and −8.95.
  - Sun: an **eclipsed sun** (`puzzle_pro_sun`) 9 units above the centre, with 7 corona rays
    (`puzzle_pro_corona`) stretched 0.6–0.8 wide and 0.3–2 tall around it.
  - Ruins: back #C82446 to #D72C53, middle #7E163C to #670E2B, front #2E0405 (near black).
  - Same sandstorm as normal Puzzle.
- Endless Puzzle opens with an "eclipse" moment (the eclipse sound, the sky turning) during
  its intro.

## 2. Survival extras
- **Curse mode sky:** when a curse wave starts, the whole Survival background shifts over 1 s:
  gradient bottom (147, 64, 157), top (26, 13, 58), overlay (144, 52, 110); background clouds
  (56, 34, 105); clouds and islands greyed to (166, 166, 166); the mist moves 5× faster and the
  clouds 138× faster. When the wave ends it shifts back over 1 s to the normal night (bottom
  (91, 83, 173), top (51, 30, 103), overlay (91, 60, 175), clouds (89, 80, 169), white).
- **WAVE N message:** each wave shows a banner of 4 cloud pieces (`wave_cloud_1..4`) with
  "WAVE" and the wave number in the match text style.
- **Starry sky:** stars (`gph_star`) twinkle, one at a time every 2–5 s; the moon hangs on a
  hinge joint, so it swings.
- **Layers:** background islands (parallax 0.25), foreground islands (0.35), background clouds
  (0.2), two cloud layers (0.6, 0.8) and three mist layers.

## 3. Brick particles (all modes)
- **Lost brick, Race and Survival (life lost):** the mist effect, not a splash: one white flash
  (`gph_whiteCircleFaded_Large`, size 4, 0.15 s), 8 white puffs bursting out (speed 100, size 1,
  0.5 s), 8 small puffs drifting up (speed 50, size 0.8, gravity −2, 2 s) and 6 big soft lilac
  mist puffs rising (`mist_puff`, speed 15, size 4, 2 s, colour (0.94, 0.91, 1)). Lasts 3 s.
  Plus the **lost cross**.
- **Lost brick, Puzzle:** desert dust: 4 big orange soft puffs (`mist_puff`, colour
  (1, 0.70, 0.44), speed 80, size 5, 3 s) and 5 red-orange star sparks (`spark_star`, speed 150,
  size 0.6, 0.7 s), 4 cells below the brick when the camera has moved. Plus the lost cross.
- **Lost cross:** a big red cross (`lost_cross`) appears where the brick left the screen
  (2 cells in from the edge; 0.3 up from the bottom), spins a full turn (ease in, after 0.2 s)
  and shrinks to 30% over 0.8 s.
- **Impact dust:** every time a brick hits something hard enough, 3 small white dust puffs
  (`brick_impact_dust`, speed 4, size 0.5, 0.8 s) at the contact point.
- **Spawn poof:** a new brick appears with a white flash (size 5, 0.07 s), 8 white puffs
  (speed 60, size 1.8, 0.5 s) and 5 star sparks (speed 60, size 0.6, 0.5 s). The small version
  (trial enemy towers): flash size 3, 5 puffs (speed 40, size 1.4).
- **Gravel:** small dark-red bits (`gravel`, colour (0.22, 0.11, 0.11), 3 at once then 3 a
  second, speed 5, size 0.5, 1 s).

## 4. In-match HUD pieces
- **Spell button hints:** the spell bubbles beside the wizard carry the button to press,
  animated: on keyboard a key cap (`key_background`) with the bound key (J light, K dark), on a
  pad the X (light) and B (dark) buttons flicking between up and down. Each input type has its
  own animator (PC, Xbox, PlayStation).
- **Hearts:** filled heart, and a cross when a life is gone (`gph_heart_filled` / `gph_cross`,
  1.4 left of centre).
- **Place label:** a rank number with "st/nd/rd/th".
- **End messages:** WINNER / LOSE / DRAW, and for trials CHALLENGE COMPLETE / CHALLENGE FAILED,
  plus a "continue" prompt after a trial or endless run. A danger marker prefab exists but the
  game never uses it.
- **Race trial finish line:** shows the time left beside an hourglass (`hourglass`), fading in
  over 1 s, sliding to the target in 0.5 s.
- **Endless Race:** a score line follows your tower top, and the pacer line has its own texture
  (`gph_pacerRaceEndless`) and effect.
- **Finish line ends** use a flag tile (`finish_line_end`).
- **Confetti** (2 sheets of 32×96 frames) rains in the cup winner popup.

## Still to read (next)
Explanation and tutorial popups, the player overview popup, the Stats, Credits, Keybinding and
Graphics popups, the bottom navigation bar and page dots, the attract demo. Achievements: not
read; ask Max.