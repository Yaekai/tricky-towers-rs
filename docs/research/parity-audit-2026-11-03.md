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
event (0.4333 seconds, already recorded in `magic.cast_self`), from the wand where
the wizard is then, with its current facing. A different animation interrupting
the cast prevents that event. One of the two light-cast sounds follows the event
by 0.1 seconds.

Sources: ControllableWizard and WizardAnimationController; the staged CastSelf
clip event timing.

## Confirmed work still outstanding

- Pause stops scaled game time in the original. Rust currently pauses physics,
  but presentation timers continue and the lower HUD/spell clouds are removed.
  A complete fix must freeze the match while allowing pause-menu animation.
- Cup standings opened with Game info fade in and out over 0.1 seconds; Rust
  currently changes their visibility immediately.
- The cup winner's separate appearance poof is still missing; its sound is
  already implemented.
- The original clears through black both before and after a screen transition;
  Rust only implements the clear after the screen changes.
- Lane clipping still omits brick decorations and Puzzle's per-tower framing.

These remain implementation gaps, rather than claims that the whole game now
matches the original. Trial 9's existing physics discrepancy is separately
documented in `docs/WHATS_NEXT.md`.
