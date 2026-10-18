# What's next

The open work, most useful first. "Estimated" means the value is ours, not yet read from the
original; those are marked `(?)` in `src/`. The status table in the [README](../README.md) gives
the overview; `docs/pc-tasks.md` and `docs/research/compare-2026-10-06.md` keep the detailed
working notes.

## For Max (when you have time)
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

## Not planned
The Shop. XP and wizard levels exist as data only, since the original
gives XP only online.
