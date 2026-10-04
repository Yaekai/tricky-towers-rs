# What each spell does in the original

Plain-language notes from Max's installed copy. Units are brick cells.

## Light
- **Ivy:** when your brick lands, it and every brick touching it (within 0.2) become one rigid
  body; existing vine clusters merge in. If any brick in the cluster is petrified, the whole
  cluster turns static. Undo on a vined brick breaks the cluster apart.
- **Petrify:** your current brick becomes static, like the ground, when it lands.
- **Undo:** the last placed brick is zapped for 0.75 s, then removed. It does not count as dropped.
- **Minibase:** a stone ledge appears 3 to 5 cells left or right of centre (random side, rounded to
  0.5), at height max(tower, 2) + 3. It counts as part of the tower.
- **Extra life** (Survival): a heart orb at x −5, height max(tower, 10) + 3. Touch it with a brick
  for one more life.
- **Auto-build** (far behind only): the next 5 bricks drop at once to a random spot over the tower
  and petrify.

## Dark
- **Large:** the current brick grows to 2× over 0.3 s (+0.5 for each extra Large).
- **Fast:** for the next 5 bricks, the fall speed is at least the drop speed + 7.5.
- **Ice:** the next 4 bricks have friction 0.06 (normal is 0.4).
- **No-rotate:** turning is blocked; the brick shakes for 0.3 s when you try.
- **Auto-rotate:** winds back 15° over 0.2 s, turns 105° over 0.45 s (90° net), waits 1 s, repeats.
  Manual turning is off.
- **Mystery:** the current brick becomes one of 3 shapes from a bag: a 2×4 block (mass 1.5), a
  10-cell blob (mass 1.75), or a table, a 4×2 slab on two 1×1 feet (mass 1.75).
- **Bubble:** lifts the victim's last brick (not if vined or petrified) with an upward pull of
  21.5 × scale, easing to 18 over 1 s, then 17.5 (gravity is 18). It tilts 3 to 12°, and pops
  after 3 s or when a brick hits it. A bubbled brick does not count toward height.
- **Slow:** a balloon rises from the tower top at 2.5 cells/s, following the brick's x. Once the
  brick touches it, it moves sideways at most 1 cell/s and falls at 6 easing to 3; holding down
  brings it back to 6. The balloon pops on landing.
- **Mist:** fog 5 cells tall at the tower top for about 15 s.
- **Grassy:** grass grows over the last brick and every brick within 5 cells (looks only).
- **Ghost:** up to 3 skulls grab the victim's last brick (not petrified or vined), carry it up and
  away from the centre, and drop it after 4 s.

There are 16 combinations of two to four effects on the current brick, such as Large + Ice or
Fast + Mystery. Two of them repeat for 4 bricks: Fast + No-rotate, and Mystery + Ice + Auto-rotate.
