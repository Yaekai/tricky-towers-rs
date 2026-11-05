# Offline presentation audit, 2026-11-03

Behavior read locally from the player's install, compared with the current Rust
implementation. These are plain-language observations; original source and assets
remain in ignored directories.

## Final trial and Credits

The game-state controller's Next action opens Credits after the last trial. The
ended match remains underneath. Closing Credits returns to the Start screen with
the logo and prompt, rather than the trial list. Newly unlocked trial tiers still
take priority and return to their list first.

The Credits popup's mapped keyboard close inputs are Return, Space and Escape.
Up/Down and W/S scroll; ordinary letters and horizontal arrows do not close it.
The pad's vertical d-pad/stick scrolls, and mapped face, Start, shoulder and trigger
buttons close. Pointer clicks close the popup too. The original also maps a
PlayStation touchpad button; Bevy's current mapping does not expose it reliably.

Sources: GameStateController, CreditsPopup, the input action mappings.

## Wizards

- Every character uses the shared loud-laugh recordings at volume 0.8 and pitch
  1.13, including Animal and Indie characters. Other speech keeps its character's
  own mix.
- On becoming invincible, a wizard starts visible and switches its descendant
  sprite renderers off after 0.05 seconds, then alternates every 0.05 seconds.
  Its lower HUD and spell clouds belong to those descendants. Invincibility
  ending or becoming the winner restores its active visibility. Enemy wizards
  have no such blink.
- The placement count before Action is picked from three through seven inclusive;
  the original integer random call excludes its upper bound of eight.
- The wizard's second update emits the normal brick-appearance poof two cells
  above it. This applies to ordinary and enemy rigs. The player-controlled cup
  winner is separate: its popup emits a poof immediately at its spawn position
  on the popup's render layer, followed by the wizard-lose sound.

Sources: InitSpeechAudioCommand, SpeechAudioManager, SpeechAudioStruct, Wizard,
WizardAnimationController and AbstractWizard.

## Cup winner's light cast

Pressing light magic starts CastSelf. The bolt starts at the clip's cast-self
event (0.4333 seconds, already recorded in `magic.cast_self`). The bolt follows
the wizard's wand as it flies and faces either way, while the endpoint stays
where the cast put it. A different animation interrupting the cast prevents
that event. One of the two light-cast sounds follows the event by 0.1 seconds.
The popup only takes input from the controller assigned to the winning wizard.
The Rust rig still uses its configured local wand offset for each skin.

Sources: ControllableWizard and WizardAnimationController; the staged CastSelf
clip event timing.

## Pause and cup appearance follow-up, 2026-11-04

Pause now preserves the match phase and freezes virtual time before the engine's
time update. Pause UI, audio restoration, the screen-clear fade and automation use
real time. This retains the HUD/clouds and freezes shader globals as well as CPU
animation and physics. Active pause lasts through fade-out; its open state ends
when closing begins, restoring SFX/music and hiding the cursor at that point.
Brick-control sounds and spell-button reactions are blocked during pause.

The cup winner's immediate appearance poof is implemented on the popup camera's
layer at its spawn position and scale, without applying it to nudge ghost clones.

## In-match cup standings, 2026-11-05

The Game Info standings fade the popup's full UI in and out over 0.1 seconds of
unscaled time with the project's default out-quad easing. The panel stays in
place, with medals already filled and no Continue button. It is hidden when the
fade-out completes. The between-match results keep their own slide and medal fill.

Sources: TournamentCurrentScorePopup and DOTweenSettings from the local install.

## Screen transitions and start animation, 2026-11-05

The original's state controller leaves the old screen in place until the
overlay reaches black. Each fade takes 0.076831 seconds of real time with the
default out-quad easing. Rust commits queued navigation at black and keeps the
cover up while the new match waits for its fixed-step initialization. Popups
such as Help and match explanations stay on their own animation paths.

Pressing Continue before the logo settles skips the entrance. Once settled,
Continue plays Rumble, a one-second clip with a shaking logo, staggered falling
rubble, and 0.075-second prompt blinking. The controller that started it gets
a 0.3-second full-strength pulse on its strong motor. Another Continue skips
the wait, and a pointer click goes directly to the title transition.

Sources: AbstractBricksStateController, TransitionEffect, StartStateController,
the Rumble clip and the Start scene from the local install.

## Full pause menu fade, 2026-11-05

The original's PauseMenu applies its 0.3-second real-time fade to the entire
popup. The dark overlay, text, selection bar and controls now fade together.
The same UI opacity path handles the in-match cup standings. Fading glyphs
reuse a private material, leaving the shared font cache unchanged.

## Confirmed work still outstanding

- Race and Survival now cut attached decorations at the brick's lane bounds.
  Puzzle's cameras use the same
  widest tower across the match. The required view width is the larger of 10
  cells and the widest tower plus 2. Dividing by the lane aspect ratio gives
  the height, with a 40-cell minimum and the bottom on the floor. The Rust
  camera already follows that sizing rule. Each original camera is centred in
  its own viewport, so its scene stays centred and clips at its own screen edge
  as the width grows (PuzzleGameMode, WidthTowerCamera, AbstractGameController).

## Trial 9 bounds measurement, 2026-11-05

The original's Brick updates its bounds by taking the minimum and maximum of
each collider's world-space bounds. The default Avian/Parry bounds for a compound
shape rotate its enclosing local rectangle. That rectangle includes empty
corners and can report a false top for a tilted T, J or L.

A headless Trial 9 fixture places a tilted T with a conservative top of 2.47
cells and an actual top of 2.20. The trial stays open, then fails when the same
brick is moved high enough for its actual shape to reach the laser. Tower
measurements and Puzzle zaps now use the bounds of the individual shapes.

A second headless test uses the published solution's placements and the stock
trial config with slack set to zero. From the centre of the floor, the six
placements use half-cell taps of 3, -5, 5, 0, -1 and -1. The turns are left,
left, right, left, right and none. This route clears on both the old and new
height calculation. The tilted-brick fixture fails on the old calculation at
2.47 cells. The input sequence from the earlier 2.47-cell run remains unverified,
and the shipped trial slack stays at 0.3 cells.
[All puzzle solutions in one picture.](https://www.speedrun.com/tricky_towers/guides/ciy29)
