# What's next

Start with the [Claude handover](handoff-2026-11-05-claude.md) and the
[behavior audit](research/parity-audit-2026-11-03.md). The ordered list here
supersedes the older working notes below.

## Active backlog

1. **Give multiplayer Puzzle fixed, centered viewports.** First slice merged
   2026-11-05 (`src/lanes.rs`): a back camera draws the scene, one camera per tower
   draws its GamePositionHelper rectangle (`config::Lanes` margins and gaps), and the
   main camera draws DesertForeground's sand and the UI last. The zoom uses the
   rectangle's aspect; HUD, hearts, scores and banners add `LaneViews::shift`; grab
   shaders sample by whole-picture UV. Checked in the cloud with placeholder art for
   two, three and four seats, pause and results, plus unchanged Survival and solo
   Endless Puzzle, and a test that each tower stays centred as the zoom widens.
   PC pass with staged art (2026-11-05, offscreen, 2-4 seats, 1280x720, 1920x1080,
   3440x1440, 1280x1024): each tower stays in the middle of its lane as the view zooms
   out; lasers sit at one height in every lane; the drop guide, front sand and heat
   haze line up; next-brick cards, counters and banners sit over their own tower; the
   Puzzle trial, solo Endless Puzzle, Race and Survival look unchanged. Wide stacks
   only zoom past ~30 cells (2 seats), ~19 (3) or ~13 (4) at 16:9, so the pass
   zoomed with a local test config (Puzzle laser 30, `large_scale` 4). Still open:
   - Fixed 2026-11-05 (towers 400 cells apart in Puzzle battles, `gameplay::PUZZLE_BATTLE_SPACING`; every tower draws its copy of the laser): *Neighbours bleed in when zoomed out.* A lane camera sees `lane width x scale`
     around its tower, but towers are only one lane spacing apart, so past a scale
     of about 1 + gap/lane width a wide neighbour's bricks show at the next lane's
     edge (seen with 4 seats). The original culls by layer instead: each tower's
     world, bricks and camera sit on their own `Tower<n>` layer and its camera draws
     only that (`AbstractGameController` 257-284; spell collectors on
     `TowerBackground<n>`). Ours needs a render layer per tower on everything a tower
     owns (`lanes::follow_lanes` puts every lane camera on layer 0).
   - Fixed 2026-11-05 for Puzzle battles (`lanes::screen_box`); Race and Survival still use the full screen: *16:9 rectangle.* The original fits every tower viewport into a 16:9 rectangle
     (`GamePositionHelper._Initialize` / `ModifyGamePositionForRatio`), testing
     `Screen.width / Screen.height` with integer division: 3440x1440 (2) is
     pillarboxed to the middle 2560 px; 1280x1024 and 16:10 (1) are letterboxed to
     width / 1.78. Ours always uses the full width and height (`lanes::lane_rect`,
     `lanes::viewport`, the aspect in `visuals::frame_towers`), so at 5:4 each lane
     is narrower in world space, the wizards are cut at the lane's right edge, and
     the DRAW! banners are wider than their lanes and touch each other.
   - *HUD in the box.* Each battle HUD (the next-brick card and its count) is built
     with its tower's viewport, the one GamePositionHelper fits in the 16:9 box, and
     its root is anchored to that rectangle's corners (`AbstractHUD`, the modes'
     `_CreateHud`). So when letterboxed the cards hang from the box's top, not the
     screen's: done for split Puzzle screens (`hud::place_items`). Sizes: the HUD
     canvas (`UI HUD` in the Game scene) is Screen Space - Camera with a CanvasScaler
     set to Scale With Screen Size, reference 1920x1080, screen match mode Expand, so
     one canvas unit is min(width/1920, height/1080) pixels: the 16:9 box's height /
     1080 on any screen. Its `Container` fills the canvas (HUDSafeZoneScaler adds
     nothing at VISIBLE_SCREEN_RATIO 1). So split Puzzle HUD items now take their size
     from the box's height and the card's sideways offset from the box's width
     (`hud::fill_items`, `hud::place_items`): at 1280x1024 the card is drawn exactly as
     at 1280x720, at 3440x1440 as at 2560x1440.
   - *Race and Survival battles* (read only, not changed): they keep the default
     `GetGamePosition`, so they get the same 16:9-fitted lane viewports, one camera
     per tower (`camera.rect` = the viewport), and the same HUD anchoring and Expand
     scaling. Each tower also has a background and a foreground camera (its
     `TowerBackground<n>` / `TowerForeground<n>` layers) whose rect keeps the lane's
     centre but is as wide as the screen allows around it. Ours draw those modes
     with one full-screen camera and size their HUD from the full height.
   - 125% scaling and window resizing need a windowed check by Max (offscreen runs
     always render at 1x).
   - *Shakes in a split Puzzle battle.* Every original camera shake moves the camera
     container of the tower whose effect fired it (AbstractGameController injects its
     own `_cameraContainer`; CollapseTowerEffect and LargeBrickModifier do the same).
     So only that tower's lane shakes: each lane camera now adds its own tower's
     `CameraShake::lane_offsets` (`lanes::follow_lanes`), the collapse blasts and
     rumble and a Large brick's held shake go to their tower's lane, and the post
     shader's lane shift is off while split. Race and Survival keep the old way
     (one camera, lanes shifted in post).
   - *Forcefield on Puzzle cuts.* Puzzle battles keep AbstractGameMode's brick effects
     (BrickClipper in GAME and ROOF), and BrickClipper measures against its own tower's
     ZoomableCamera edges. So a split Puzzle screen now draws the Forcefield at each
     lane camera's edges (`LaneViews` centre_x ± half_width, `particles::clip_fields`);
     a Puzzle trial and solo Puzzle still draw none.
   Fixed 2026-11-05: a Puzzle battle's WINNER! now follows the count alone, so a
   debug `Win` no longer adds a second one.
2. **Reproduce the Trial 9 stack-height mismatch.** Replay the same placements
   and rotations for `SP_10` and record each brick's settled position and top.
   Use the existing `trial brick over laser` diagnostic and compare the first
   divergence with the original. The observed stack reached 2.47 cells against
   the 2.20 laser. Floor geometry and the laser rule have already been checked.
   Keep `puzzle.trial_laser_slack` at 0.3 until the same stack clears the original
   threshold with a demonstrated fix. Rerun stacking, Ivy and cup regressions.
3. **Compare spell and results timing in play.** Check Slow, Bubble on a stuck
   brick, Extra Life, No Rotate, and the cup winner's cast against the original.
   Record the trigger, delay and outcome for each difference before editing.
   Then resolve the remaining `(?)` values in `casts`, `moments` and cup timing
   from clips or repeatable measurements. Use [PC tasks](pc-tasks.md) as evidence
   and a queue for measurements.
4. **Run a focused display, text and controller pass.** Check Pause, Settings,
   Select Players, Trials and results at 125% Windows scaling and 21:9. Cover long
   Russian strings, Arabic layout and Japanese confirm/back glyphs. Verify Start
   and Large rumble and the pause audio mix on hardware when a hands-on session
   is available. The PlayStation touchpad Credits-close mapping remains open.
   Measure frame times in a four-player decorated match to check the clip pass.
5. **Close the offline release checklist after the parity checks.** Audit
   `DEBUG-TOOL` entries and the release build with [the checklist](release-checklist.md).
   Carry forward the existing decision about AI seats and mod tooling, or record
   it as a release decision if still open. List any remaining measured gaps.

Online Battle's connected lobby, invites, matchmaking and network play are a
separate workstream. The initial menus exist. The Shop is outside the current
plan. Treat older suggestions for those areas as historical notes.

## Historical working notes

The open work, most useful first. "Estimated" means the value is ours, not yet read from the
original; those are marked `(?)` in `src/`. The status table in the [README](../README.md) gives
the overview; `docs/pc-tasks.md` and `docs/research/compare-2026-10-06.md` keep the detailed
working notes.

## For Max (when you have time)
- **AI seats:** the original's local battle has no computer players. Keep them in the release (they
  are handy alone) or take them out?
- **Puzzle laser:** does it keep dropping half a cell per lost brick (ours) or only once?
- **WAVE N banner:** it widens to 1.3x as it slides out (the original's animation). If it looks
  wrong in the original footage, tell me where.
- **Online Battle:** the buttons open the lobby but nothing connects. Say if you want a local
  stand-in (fake players) so the connected lobby can be pictured.
- **Font:** if any screen shows text in a font other than the original's, say which one.

## 1. Check the newest work against the original
Most recent features were checked in offscreen pictures only. Compare them in play with the
original and fix what differs:
- The brick, water, island and grab-pass shaders (lit bricks, Galaxy, Hologram, Diamond, zap
  dissolve, drop guide overlay, Puzzle heat haze, cast lanes, Endless perspective clouds).
- The cup standings panel and the cup winner popup (confetti, fireworks, the flying wizard).
- Select Players, Select Mode, Single Player, Settings and the pause screen.

## 2. Gameplay still missing or estimated
- **Puzzle**: compared with Max's clip at 15 s and 40 s; the laser, the foreground sand and the
  base rise now match.
- Done since the last list: Endless Puzzle obstacle blocks (checked, they match), Auto Rotate
  wind-up and its sounds, Petrify's stone look in hand and its grass and moss, the cup's stray
  empty medal and medal sounds, two towers touching the Puzzle laser at once (tested).
- About 40 values marked `(?)` in `src/` and the guesses in `assets/config/game.ron`
  (intro timings, cup panel timings, `moments` and `casts` values).

## 3. Look and effects
- Animation curves: `config::curve` joins clip keys with their slopes as Unity does. The WAVE N
  message, the start screen, WINNER! (with its swing in), WIZARD OUT!, the mode title ribbon and
  the 3-2-1 (with its tilt), the spell clouds and the firework, shockwave and beam droplet
  sizes use it. Particle alpha keys are gradients (straight, as the original's).

## 4. Menus
- Display scaling (125% in Windows): the Puzzle heat haze's screen copy now follows it; check
  other screens at 125% too.
- Settings → Stats follows StatsPopup (Game and Other stats rows); Resolution lists the monitor's
  own modes.
- Text weights left from the screenshot comparison. Single Player and the title match Max's
  clip.
- Languages: menu and match text follows the Language setting from the original's language
  files (`letters::Translations`), in the original's font per language (Meiryo for Russian and
  Japanese, Hans Sans for Chinese and Korean, Myriad for Arabic). Open: long Russian words run
  past fixed boxes unless they carry `letters::AutoSize` (the seat join label, Settings rows and
  title tile names do);
  Arabic is reversed letter by letter like the original (LanguageManager._ReverseText). In
  Japanese a PlayStation pad confirms with Circle and goes back with Cross (`input::continue_back`);
  not yet tried with a real PlayStation pad.

## 5. Before a release build
- Remove or switch off everything in [`release-checklist.md`](release-checklist.md)
  (`grep -rn DEBUG-TOOL src`), and decide whether AI seats in local battles stay (the original
  has none).

## 6. Online battle menus (Max, 2026-10-17)
Build the Online Battle screens (OnlinePlayerSelect, OnlineLobby, AcceptOnlineInvite scenes) so
the title's Online Battle tile opens them like the original, even though no match can be played
online yet; the networking itself may come later. OnlinePlayerSelect is done (list and rank plate) and
its buttons open the OnlineLobby screen as it stands before connecting (every slot looking for
games, or creating one when inviting). The connected lobby (players, timer, mode
indicator) and AcceptOnlineInvite need online play to show anything.

## Next ten tasks (prepared 2026-10-18)
1. Online Battle: AcceptOnlineInvite and the connected lobby (player slots filled, STARTING IN
   timer, mode indicator) as layouts, driven by a local stand-in until online play exists.
2. Stop the menu music when the lobby opens (`MusicStopEffect` in the original's join handlers).
3. Check the spell changes in Max's next two-player playtest log and notes: Slow's balloon,
   Bubble on stuck bricks, the Extra Life orb, No Rotate chains, the cup winner's lightning
   (its wand spot is a guess).
4. Compare the remaining untouched clips (Max's recordings 2 to 4, 6 to 11, 14 to 19) screen
   by screen, starting with the longest unseen ones, and fix what differs.
5. The cup winner's remaining pieces: the fade-out overlay and the `WIZARD_*_AUDIO` triggers
   checked against the original's timings.
6. Concrete (Petrify) on a loaded tower and the roof's Survival hitbox, played by hand with a
   friend, since offscreen runs only prove the code paths.
7. (Laser question answered: the original's multiplayer laser layer keeps its height fixed; only the base rises per lost brick, as we do.) Puzzle: the base rise trigger, checked against Max's Puzzle
   clip frame by frame (open since 2026-10-09).
8. The remaining `(?)` values in `src/` (about 20), read from the install where a prefab or
   clip has them: cup panel timings, `moments`, `casts` sizes.
9. Mouse in the menus left over: Single Player, Trials, Endless and Settings pages' arrows and
   buttons, checked with the real mouse.
10. Release-build cleanup: go through `release-checklist.md`, decide the AI seats question, and
    list what is still guessed for Max.

## Not planned
The Shop. XP and wizard levels exist as data only, since the original
gives XP only online.

## Lane clipping in battles (started 2026-10-24)
The original gives each tower its own camera over its share of the screen, so a brick past a
lane's edge is cut off there and BrickClipper puts the Forcefield (BRICK_CLIP_PARTICLE: stars,
streaks and dots) on the cut. Ours cuts the brick materials and the drop guide at the lane
(`clip` in `brick_material`, `Lanes` widths) and draws the Forcefield (`particles::clip_fields`).
Race and Survival now cut attached vines, grass, moss, bubbles, and Spring, Christmas
and Halloween decorations at the same edges. Puzzle's WidthTowerCamera needs separate
viewport centers and clipping as the shared widest-tower zoom grows. The Forcefield
already uses additive materials for its stars and streaks.

## Next up: single-player intro and Endless Race end (2026-10-23)
Trials and Endless runs have the original's intro (timings, the wizard's moves, the HUD
showing at its spell) and 3-2-1, and Endless Race its end sweep (`gameplay::PacerEnd`).
Check in play: the Race trial's camera sweep and the wizard's turn look like the original's.

## Next up: spell casts as a pipeline (found 2026-10-22)
Ours applies a spell the moment it is cast; the original runs each cast as an effect pipeline
(AbstractInitSpellsCommand). A dark cast on the brick in hand waits 0.5 s, pauses that brick
(PauseBrickControlEffect: it hangs, shaking, the pad rumbling 128/128), loops the zap, sends the
beam up and down (BEAM_IN / BEAM_DOWN_IN complete at 0.33 s each), applies the curse, then resumes
the brick (ResumeBrickControlEffect) and the victim's wizard startles. A light cast on the brick
pauses it and the caster's spells, shakes it during the cast-self lightning, then applies and
resumes. Tower spells (Bubble, Ghost, Grassy, Mist, Slow) and Minibase/Extra Life have their own
lists. Doing this changes timing in every match (and the AI's steering), so it wants its own
thread and a play-test. Casts' visuals in `casts.rs` already follow the clip timings.
Done since (2026-10-22/23, #140 and after): casts land late (a dark cast hangs the victim's brick
0.5 s in and lands with the curse orb, a light one hangs the caster's brick through its
lightning), the brick in hand is carried back up (MoveBrickUpEffect) for the spells that do it,
and Large's rumble. Left: a per-spell check of the remaining pre/post effects against
AbstractInitSpellsCommand.

## Next up: brick hitboxes (Max, 2026-10-22)
Max thinks the original's brick hitboxes sit closer together. The prefabs (research/ripped, `game/bricks/Brick*.prefab`) say:
each brick is a few box colliders (O one 2x2, I one 4x1, T/J/L a 3x1 plus a 1x1, S/Z two 2x0.98), no seams between
cells; the movement controller shrinks them by 0.3 while steered and grows them to 0.99 of the prefab size on landing.
Ours are separate cells at `collider_scale` 0.98. Plan: build bricks from the prefab boxes at 0.99 (`brick_collider`,
`brick_body`), then rerun `bricks_land_and_stack_on_the_platform` (a plain 0.98 -> 0.99 change makes it drop a brick) and
replay trial 9 with the brick log; remove `trial_laser_slack` if the stack then fits. Max should feel it in play before
it reaches main.

**Status 2026-10-22 (merged; Max: AI behaviour doesn't matter, the two AI tests now spare the AI tower's lives):** bricks are now the prefabs' solid boxes
(`brick_boxes`: cell rows joined, equal rows stacked) trimmed by the original's shrink (0.15 a side while
steered, 0.01 once landed; `collider_scale` 0.99). `bricks_land_and_stack_on_the_platform` still passes, but two
AI-driven headless tests fail (`the_winner_keeps_its_tower_while_the_loser_falls_apart`, `a_cup_plays_matches_until_someone_reaches_the_target`):
the AI wizard drops 4 of 15 bricks instead of 1, so stacking with solid boxes differs from cells with seams. Needs a
play-test by Max; trial_laser_slack kept until then.

## Open from Max's 2026-10-18 playtest
- Trial 9 (SP_10): a correct stack measured 2.47 against the 2.20 laser; the original's floor
  (HousePuzzleLongI) and rule (part top > maxY - CAMERA_OFFSET) match ours, so the 0.27 is
  unexplained. `puzzle.trial_laser_slack` (0.3) is temporary; remove it once the cause is found.
  Re-read 2026-10-22: CAMERA_OFFSET is 0 outside Endless Puzzle, `Brick.top` is the collider bounds' max y,
  floor height is only the camera's lowest point. Ours now also uses the collider's tight bounds, but
  that moved settled tops by just 0.005, so the cause is not found. Needs Max's next log of the failing
  stack (brick positions at the fail) to compare. Also 2026-10-22: HousePuzzleLongI's five box colliders in the
  prefab match `FLOOR_PUZZLE_LONG_I` in game.ron exactly (tops +1, -1, -1, 0, -7), so the floor isn't it. A silent
  search (4 turns x 8 cells per brick, no slack) found no stack of T J L I L O under 2.2 in ours, and Max's official
  win shows the top bricks just under the laser: the difference is in brick physics (rest depth in the gaps), not
  the rule. The `trial brick over laser` log line (DEBUG-TOOL) prints brick positions for a comparison.
- First-time digits blinking: fixed 2026-10-22 (the first digit drawn in a look makes 0-9 for it); check in play.
