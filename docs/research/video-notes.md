# Notes from gameplay videos

Plain-language notes from watching public gameplay videos of the original frame by frame on the
PC (yt-dlp + ffmpeg, frames kept outside the repo). Videos are compressed 1080p, so these are
good for layout, colours, motion and timing, not for exact pixels. Pixel figures are at 1280×720
unless said otherwise; "cell" is one brick square.

Sources (2026-10-04):
- A 2-player local Super Cup longplay, PC, 1080p60, no commentary (Race, Survival and Puzzle
  matches with the cup standings screen between them).
- A 3-player PS4 run through all modes, 1080p30, no commentary (Select Players, Select Mode,
  Race, Survival Normal, Puzzle, each to the popup).

## In-match HUD: there are no bottom player panels
The original draws nothing along the bottom of the screen. Everything a player needs sits in
their own lane:
- **Next brick card.** A white card with a dark blue top bar hangs from the top edge of the
  screen in each lane, about 75 px wide and 65 px tall (about 9% of the screen height), with
  the next brick drawn small inside. It stays at a fixed spot in the lane, about 78 px left of
  the lane's drop column (it doesn't follow the brick).
- **Survival brick count.** A yellow number (gradient, dark outline, the match text style) just
  left of the card, overlapping its lower left corner: bricks left to place (61, 52, 31, …).
- **Place label.** "1ST", "2ND", "3RD" under each wizard's cloud in Race, small, light text
  with a dark outline; 1ST is yellow-orange, the others pale lilac.
- **Hearts.** In Survival, the 3 hearts sit in a row just under the wizard's cloud (about
  18 px each), not in a panel.
- **Spell bubbles.** The 1 or 2 spells a player holds float as round bubbles (about 45 px)
  just above and to the right of the wizard, the second one lower right of the first and
  overlapping it.

## Drop guide, spell lines, bricks
- **Drop guide.** A pale, see-through column the width of the falling brick, from the top edge
  of the screen all the way down to the top of the tower (it runs behind the brick and the
  card). Race: light cyan. Survival: lilac. It never stops at the brick.
- **Spell lines.** A row of small diamonds (about 14 px), one colour per line (cyan, green,
  pink, yellow-green…), with no round caps at the ends. The line runs across the lane at the
  wizard's height and the wizard floats on it.
- **Bricks.** Solid, fully opaque colours with a dark outline and the inner shading of the art.
  They never look washed out or see-through. A stone-cursed tower is dark grey-purple bricks.

## Race win, measured at 4 frames a second (3-player match)
Time from the moment the winner's hold countdown ends:
- **0 s:** the finish line, spell lines, next cards and place labels disappear at once. A red
  pointed roof (spire) appears high above the winner's tower and drops onto its top over about
  1.25 s.
- **0 s:** the losers' towers turn pale blue-grey at once (desaturated, like a frozen picture),
  and the losers' wizards vanish in a small poof within 0.5–0.75 s.
- **2.0 s:** WINNER! appears small over the winner, pops large and tilted at 2.25 s, settles
  smaller and level at 2.5 s, just above the roof.
- **2.0–2.75 s:** the camera zooms out until all towers and their bases fit on screen. A pale
  spotlight column then stands over the winner.
- **3.5 s:** black round bombs appear on several bricks of each loser tower at once, flash
  white, and the towers blow apart over the next 2 s; pieces fall into the water with
  splashes from about 5.75 s.
- **About 6 s:** the popup slides in at the bottom right (Rematch, Select Mode, Main Menu).

During the hold, every tower above the finish line shows its own countdown over it: the leader's
in yellow-orange, the others in pink.

## Other match ends
- **Survival:** when the last other wizard is out, WINNER! shows over the survivor and its brick
  count stays in yellow at the top; the losers' towers turn pale grey and crumble the same way.
- **Puzzle:** each player's score shows big at the top of the lane: the winner's in yellow with
  WINNER! under it, the others in pink (15 / 10 / 9 in one match). The laser stays visible.

## Menus seen in the videos
- **Select Mode** has a third column after mode and difficulty: "Single match" or a cup.
- **A new match** starts with a ribbon banner across the screen naming mode and difficulty
  ("SURVIVAL NORMAL").
- **Cup standings** (between cup matches): a dark screen with a white panel, "SUPER CUP" title
  between two laurels with small cups, one row per player (wizard face, then a row of trophy
  slots filling up), and "CONTINUE" under it.

## Still to measure
Cast beams and the light cast's lightning, the spell bubbles' cast and "used" animations, the
curse-wave start shake (no Survival Pro wave in these videos), and the explosion's frames.
