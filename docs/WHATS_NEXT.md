# What's next

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

## Open from Max's 2026-10-18 playtest
- Trial 9 (SP_10): a correct stack measured 2.47 against the 2.20 laser; the original's floor
  (HousePuzzleLongI) and rule (part top > maxY - CAMERA_OFFSET) match ours, so the 0.27 is
  unexplained. `puzzle.trial_laser_slack` (0.3) is temporary; remove it once the cause is found.
- First-time digits can still blink once (letters::SdfCache only helps repeats): pre-make 0-9
  for a look the first time a number is drawn.
