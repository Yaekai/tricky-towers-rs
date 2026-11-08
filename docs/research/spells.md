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
- **Extra Life** (Survival): a heart orb at x −5, height max(tower, 10) + 3. It drifts right and
  bobs, checking the current vertical speed before applying each step's force. It poofs on arrival
  and when it leaves. Touch it with a brick for one more life and the pickup sound.
- **Auto-build** (far behind only): the next 5 bricks drop at once to a random spot over the tower
  and petrify.

## Dark
- **Large:** the current brick grows to 2× over 0.3 s (+0.5 for each extra Large).
  Its collision shape grows with the picture throughout this interval using out-quad easing.
  A new Large cast during the growth starts at the current size.
  Its mass is the shape's initial mass multiplied by its current scale. Mystery shapes use
  their own initial mass. This also applies when an enlarged brick joins an Ivy cluster.
- **Fast:** for the next 5 bricks, the fall speed is at least the drop speed + 7.5.
- **Ice:** the next 4 bricks have friction 0.06 (normal is 0.4).
- **No-rotate:** turning is blocked. The first refused turn shakes the brick for 0.3 s. More
  attempts during the shake leave its timer running. A chain sound starts 0.08 s after the lock
  attaches or a new shake starts.
- **Auto-rotate:** winds back 15° over 0.2 s, turns 105° over 0.45 s (90° net), waits 1 s, repeats.
  Manual turning is off.
- **Mystery:** the current brick becomes one of 3 shapes from a bag: a 2×4 block (mass 1.5), a
  10-cell blob (mass 1.75), or a table, a 4×2 slab on two 1×1 feet (mass 1.75).
- **Bubble:** targets the newest placed brick without a bubble. A vined or petrified brick wears
  a bubble for 0.7 s without lifting and is skipped by subsequent casts while it wears it. A
  loose brick lifts with an upward pull of 21.5 × scale, easing to 18 over 1 s, then 17.5
  (gravity is 18). It tilts 3 to 12°, and pops after 3 s or when a brick hits it. The controlled
  brick can also pop the shell before it reaches the brick inside. The shell has two shapes per
  size and starts at 1.5× scale around a grassy brick. A lifted brick does not count toward height.
- **Slow:** a balloon starts two cells above the tower height and rises at 2.5 cells/s, following
  the brick's x. Its trigger grows during the 0.5 s pop-in, wobbles with the orb and stays active
  during its 0.5 s shrink. A catch plays the collision sound and poof, freezes the brick and shakes it for
  0.3 s. The tethered balloon then appears and grows over 0.4 s. Steering resumes 0.3 s after it
  appears. Idle fall speed is capped at 1 cell/s, and sideways steps work as usual. Once the
  balloon finishes growing, the held-Down cap falls from 6 to 3 cells/s at 8 cells/s². A new Down
  press after inflation resets the cap and plays one of two sounds. The brick still accelerates
  from its current speed. The tether rests at the wider of the brick's width and height. Down
  extends it by the difference between the current Down cap and idle fall speed until it reaches
  two extra cells. Releasing Down reels it back at 10 cells/s. The balloon disappears on landing.
  When Large lands, the balloon grows from its current size to one plus the number of Large
  curses in 0.3 s with out-quad easing, the same timing as the brick's growth. Down movement of
  the tether pauses during this tween. When it finishes, the wider dimension of the enlarged
  brick sets the new resting distance and limit, while the tether keeps its current length.
- **Mist:** fog 5 cells tall at the tower top for about 15 s.
- **Grassy:** grass grows over the last brick and every brick within 5 cells (looks only).
- **Ghost:** up to 3 skulls grab the victim's last brick (not petrified or vined), carry it up and
  away from the centre, and drop it after 4 s.

There are 16 combinations of two to four effects on the current brick, such as Large + Ice or
Fast + Mystery. Two of them repeat for 4 bricks: Fast + No-rotate, and Mystery + Ice + Auto-rotate.

Unity's circular 2D trigger uses the larger of the two animated axes for its world radius.
[Unity physics developer's explanation](https://discussions.unity.com/t/2d-physics-in-unity-2021-2-beta-850340/850340/20)
