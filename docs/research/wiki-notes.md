# Notes from the Tricky Towers fan wiki

Plain-language notes from search results quoting https://tricky-towers.fandom.com (the site itself
is blocked from the cloud session, so only what search engines quote is here). Fan-written, so
anything that matters should still be confirmed on Max's install; where it disagrees with a
PC readout, the PC readout wins.

## Default keyboard controls (wiki "Controls" page)
In a match:
- Left / Right arrow: move the brick.
- Down arrow: drop faster.
- Up arrow: rotate 90° to the right.
- Space: rotate 90° to the left.
- U, I: nudge (left, right).
- J: light magic. K: dark magic.
- Tab: list of players (online only).

In menus: Space or Enter selects, Esc goes back or exits, K turns all explanations off.
All of them can be changed under Settings → Keybinding.

## Settings (wiki "Settings" page)
Page 1: Sound effects (0 to 100%, default 100%), Music (0 to 100%, default 100%), Explanations
(on/off, default on: tips and objectives), Statistics (online stats, bricks placed, medals
earned...), Credits, Keybinding.
Page 2: Full screen (default on), Resolution (512×384 to 1920×1080, default 1920×1080), Quality
(Performance, Normal, Magical; default Magical), V-sync (default off), Language (18, default
English).

## Modes (wiki "Modes", "Local battle", "Online battle", "Single player")
- Three top-level modes: Local battle (up to 4 players on one machine), Online battle (up to 4
  online), Single player (Trials and Endless).
- Cups belong to online play: the "Cup Winner" achievement is for winning an online cup. Nothing
  found mentions cups in local battle.

## Trials (wiki "Trials" / "Single player")
- 50 trials in 5 tiers of 10: Rookie (1-10, open from the start), Apprentice (11-20, 8 medals),
  Pro (21-30, 17 medals), Expert (31-40, 26 medals), Master (41-50, 36 medals).
- Medals are earned by completing trials; how many a trial can give isn't in the quotes.
- Trial kinds: Race (reach the finish before time runs out), Survival (3 lives, a set number of
  bricks), Puzzle (a set number of bricks under a laser; a brick above it or a dropped brick fails).

## Endless (wiki "Single player")
- Race challenge: race up and keep the bricks stacked before a grey filter catches up (the
  pacer line and its tint).
- Puzzle challenge: build while conserving bricks, staying under a laser that is raised once
  all the bricks of the current group of given bricks are placed.
- Survival challenge: stack as many bricks as possible; each lost brick costs a life.

## Achievements (wiki "Achievements", 29 Steam achievements)
Ones that tell us about rules: Close Call (win an online survival match with 1 heart left),
Cup Winner (win an online cup), Baby Steps / Doing Good (unlock the Apprentice / Pro trials at
8 / 17 medals), Master Wizard (complete all 50 trials), Brick Stacker (66 bricks in endless
survival), 99 Bricks! (a 99-brick tower in endless survival), Shadow Stacker (a 99-brick tower
in endless puzzle), Green Fingers (connect 7 bricks with one ivy spell), All-round Magician (use
every spell once), Speed. Precision. (win a normal race within 80 s without dropping a brick).

## Official site (trickytowers.com FAQ and tournament kit, via search quotes; the site is blocked here)
- Spell list, in the FAQ's words summarised: binds touching bricks (Ivy), a small piece to build
  on (Minibase), petrifies a brick (Petrify), removes your last placed brick (Undo), places a
  couple of petrified bricks (Auto-build), bricks rotate on their own (Auto-rotate), soapy
  bubbles float bricks away (Bubble), **makes a brick large and heavy** (Large), bricks drop much
  faster (Fast), grass makes it hard to see (Grassy), very slippery bricks (Ice), mist (Mist),
  harder shapes (Mystery), no rotating (No-rotate), a balloon slows bricks (Slow), tiny ghosts
  steal your brick (Ghost), "a combination of dark magic" and "an even stronger combination".
  Another quote of the same FAQ also mentions a spell giving extra lives (Extra life). Matches
  our 17 spells and the combos.
  Open question: "large and heavy" — the PC readout says every brick has mass 1; check whether
  Large changes a brick's mass.
- Wizard levels: Rookie Magician ... Expert Wizard, then "Master Wizard Level 1", "Level 2",
  "Level 3" and so on. Confirms `XpRules::name` (level 11 = Master Wizard Level 1).
- The tournament kit ("tournament roster with official rules", papercraft crowns, a printable
  Super Cup) is for players running their own events; its rules weren't quoted.

## WeirdBeard press kit (weirdbeard.nl/presskit, via search quotes; the site is blocked here)
- Official feature list: physics puzzle gameplay; local and online multiplayer for 2 to 4
  players; three multiplayer modes (Race, Survival, Puzzle); challenging single-player trials
  (50); a leaderboard mode ("endless", how long you survive); 17 light and dark spells; full
  controller support; music by Jonathan vd Wijngaarden.
- Modes in their words, summarised: Race, beat the others to a soaring finish line; Survival,
  place a set number of bricks before everyone else without dropping too many and losing your
  health; Puzzle, place as many bricks as possible below a cutoff.
- Casting a light spell gives up the chance to cast the dark one "for a while": you choose
  between helping your own tower and hurting the others. (Matches the spell pair rule.)
- Nothing in the feature list about local cups, mods or settings beyond the above.

## Spell names as the game shows them (wiki "Spells")
Light (6): Extra Life ("Gives you an extra life."), Ivy ("Binds bricks it touches together."),
**Tower Piece** ("A small piece to build upon." — our Minibase), Petrify ("Petrifies a brick so
it can't move."), Undo ("Removes your last placed brick."), Auto Build ("Places a couple of
petrified bricks.", only for players lagging behind).
Dark (11 + 2 combos): Auto Rotate, Bubble, Large ("Makes a brick large and heavy."), **Fast
Brick**, **Grass**, **Ice Brick**, Mist, Mystery, **Rotated Brick** (the no-rotate curse; check
the exact name on screen), **Slow Down** (the balloon), **Ghosts**, and the combos **Super** ("A
combination of dark magic.") and **Super Duper** ("An even stronger combination."), both only for
players lagging behind (our "behind" and "far behind" dark sets).
Ours (`SpellKind::name`) differs: Minibase, Fast, Ice, No Rotate, Balloon, Grassy, Ghost.

## Characters and DLC (wiki "Characters", "DLCs")
- Base game: four wizards (the wiki calls them Wizard #1 to #4). Characters are cosmetic only.
- Spirit Animal Pack (2 Aug 2016): costumes for the base wizards: platypus, toucan, raccoon, fish
  (our install readout lists WizardAnimal01-04).
- Indie Friends Pack (18 Jul 2017): six guest characters (Thane, Clunk, Fish, Candyman, Last of
  the Order, Octodad; WizardIndie01-06 in the install).
- Brick packs (2017): Galaxy, Candy, Gem and Holographic bricks (texture sets).

## More achievements (wiki)
On the right track (unlock the Expert trials), Almost there (unlock the Master trials). Endless
Race: bricks may be lost freely (no lives) as long as the tower stays above the grey filter.

## Checked against the PC install (2026-10-05)
- **Keyboard defaults: confirmed.** Left/Right move, Down drops, Up turns right, Space turns left,
  U/I nudge left/right, J light spell, K dark spell, Tab player overview. All rebindable (the
  Keybinding popup in Settings).
- **Menus:** arrows and WASD both move; Enter or Space confirms and retries; Escape goes back
  and pauses; P starts Auto Build where offered; Tab opens the player overview / store; K
  invites (online). So the original does take WASD in menus. It does not use numpad arrows.
- **Gamepad defaults:** D-pad left/right move, D-pad down drops, D-pad up turns right; left
  trigger turns left, right trigger turns right; left/right bumper nudge; A (Action1) turns
  right and confirms; X (Action3) light spell; B (Action2) dark spell and back; Y (Action4)
  player overview / store / Auto Build; Start, Options or Menu pause. In menus, the bumpers
  also page left/right, and the left stick moves like the D-pad.
- **Settings: confirmed, with more options.** Two pages side by side (the second 1920 px to
  the right):
  1. Sound effects (slider), Music (slider), Explanations (on/off), Stats (popup),
     Credits (popup), Keybinding (popup).
  2. Full screen (on/off), Resolution (the screen's list), Quality (Low/Medium/High),
     V-sync (on/off), Language (19 flags: en, de, fr, es, pt, it, zh-CN, zh-TW, ja, ko, ru, pl,
     ar, tr, nl, sv, no, fl, ro), Apply (applies the graphics changes).
  Defaults: both volumes 100%, Explanations on, Quality High, V-sync off. Sliders step 10% per
  press, repeating every 0.2 s while held. Full screen starts as whatever the window is.
  Sound effects also set the ambience volume.
- **Trial tiers: confirmed** at 0 / 8 / 17 / 26 / 36 medals. **One medal per trial** beaten
  (the medal count is the number of completed trials). All 50 trials are in trials.md.
- **Endless Puzzle groups:** the first group is 6 bricks, each later one min(26, 6 + 2 × level)
  bricks. After each finished group the laser rises 4 cells (not 1) and obstacle blocks are
  added. The score is the bricks in the tower. Details in modes.md.

## Ivy (checked 2026-10-08)
- Fan wiki spell list (tricky-towers.fandom.com, Spells): Ivy is a light spell that binds the bricks
  it touches together; the wiki's achievement note says it binds up to 7 bricks.
- Steam guide: Ivy adds stability when a tower looks shaky. Trophy guide: it's best used on a brick
  touching six or more others.
- The install's IvyBrickModifier is the rule source: the cluster is static only when a member brick
  is solid (stone, zapped or petrified), joins bricks that touch (Tower.GetTowerPartsTouching) and
  takes the sum of their masses. Open: why a vined block still slides in an unsteered test; the
  wiki notes don't describe the physics.
