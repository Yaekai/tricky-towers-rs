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
- Losers: a dark veil 10 wide fades to opacity 0.1 over 1 s.
- Trophy (cups only): pops from 40% to full size over 0.6 s with an elastic bounce.
- No cup after a single match: cups show only at the end of a cup, sized Tiny, Big, Super or
  Online (staged as `trophy`, `trophy_big`, `trophy_super`, `trophy_online`).
