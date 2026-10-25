# Puzzle floor generator (read from the original's code, written in our words)

The original builds a Puzzle round's floor at random from three piece types (its
`RandomPuzzleFloor`). The rules:

- The floor is `floorWidth` cells wide (the difficulty's width: 5 on Easy, 6 on Normal and Pro).
- It is built left to right. At each cell a piece is placed with probability `1 - gapChance`
  (gap chance 40% on Normal); otherwise the cell is left empty.
- A piece is chosen at random from the three piece types. Each has its own width, measured from
  its collider: piece 1 and piece 3 are one cell wide, piece 2 is two cells wide. A piece that
  would run past the floor's width is rejected and another is drawn.
- Each piece has a bottom part under it, three cells down (the bottom piece).
- At the end, one random piece is removed from the list of placed pieces, so it stays where it
  was put. If there is more than one piece and a minimum drop is set, one random piece is moved
  down by a random amount from the minimum up to 0.
- Every other piece is moved by a random amount from the minimum up to 0 (its height varies).
- The outer pieces are not forced to stand; the first and last cells always fall back to a
  piece when nothing else would fit (so the floor is never empty).

Our build (src/gameplay.rs `puzzle_floor`) uses one-cell columns only, with the gap chance and
drops of the Easy and Normal levels. It draws art piece 1 or 3 (one cell). Piece 2 (two cells
wide) is not used yet; using it needs the floor entries to carry a width, since each entry is
currently one cell (top, bottom and the art).

The random sequence can't match the original's (a different generator), so only the rules above
should match: widths of 1 and 2, the gap chance, one removed piece and the drops.

## Where the floor sits and how it "rises"

- The floor's first cell starts at `floor(-width / 2)` from the lane centre, so a 5-wide Easy
  floor runs from -3 to +2 (half a cell left of centre); a 6-wide one is centred.
- After the flat piece is taken out, one more piece (when there are two or more) is always
  dropped by 1 to the lowest drop; the rest by 0 to the lowest.
- A lost brick doesn't move anything physical: the tower's controller only adds half a cell
  to its camera offset (over 0.4 s), which shifts the camera, the laser test and the drop guide.
  The bricks keep their contacts. We draw all lanes with one camera, so we move the whole tower
  (floor, bricks and the brick in hand) up together instead; pushing only the floor up under
  the bricks threw ones resting near an edge off.
