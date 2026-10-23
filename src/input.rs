//! Keyboard and gamepad input. In a local battle each device drives the player whose
//! seat it took on Select Players; otherwise player 1 uses the keyboard and gamepads
//! drive players 2 to 4 in the order they were connected (with one player, the first
//! gamepad drives player 1 too). Input is written into each tower's
//! [`PlayerInput`] once per frame; the fixed-step gameplay systems consume it.

use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;

use crate::collapse::{CollapseEvent, CollapseKind};
use crate::gameplay::{Match, MatchRequest, Phase, PlayerInput, Tower};
use crate::keybinding::{Action, KeyBindings};

pub struct KeyboardInputPlugin;

impl Plugin for KeyboardInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindings>()
            .add_systems(
                PreUpdate,
                (
                    read_keyboard.run_if(resource_exists::<ButtonInput<KeyCode>>),
                    read_gamepads,
                )
                    .chain()
                    .after(bevy::input::InputSystems),
            )
            .add_systems(
                Update,
                (rumble_losers, rumble_large_bricks).run_if(resource_exists::<Messages<GamepadRumbleRequest>>),
            );
    }
}

/// Whether a pad is a PlayStation one (Sony's USB vendor id), which shows its own
/// button pictures in the original (its PlayStation input profile).
pub(crate) fn is_playstation(pad: &Gamepad) -> bool {
    pad.vendor_id() == Some(0x054C)
}

/// Whether a PlayStation pad swaps Continue and Back: in Japanese, Circle confirms and Cross
/// goes back (JapanesePS4InputMappingFilter, and the PlayStation profile's swapped pictures).
pub(crate) fn swaps_continue(pad: &Gamepad) -> bool {
    is_playstation(pad) && crate::letters::language() == "jp"
}

/// A pad's Continue and Back buttons (South and East, swapped by `swaps_continue`).
pub(crate) fn continue_back(pad: &Gamepad) -> (GamepadButton, GamepadButton) {
    if swaps_continue(pad) {
        (GamepadButton::East, GamepadButton::South)
    } else {
        (GamepadButton::South, GamepadButton::East)
    }
}

/// Which player a gamepad drives: pads drive players 2, 3, 4 in the order they
/// connected; alone, the first pad drives player 1.
fn player_for_slot(players: usize, slot: usize) -> usize {
    if players <= 1 { slot } else { slot + 1 }
}

/// The people seated on Select Players, in player order, when they are the match's
/// players (a local battle); None otherwise (single player, scripted matches).
fn seated(lobby: Option<&crate::select::Lobby>, players: usize) -> Option<Vec<crate::select::Seat>> {
    let humans: Vec<_> = lobby?.seats.iter().copied().filter(|s| s.human()).collect();
    (players > 1 && humans.len() == players).then_some(humans)
}

/// The player a pad drives: its seat's player in a local battle (none if it didn't
/// join), else by connection order.
fn pad_player(seats: Option<&[crate::select::Seat]>, players: usize, slot: usize, pad: Entity) -> Option<usize> {
    match seats {
        Some(seats) => seats.iter().position(|s| *s == crate::select::Seat::Pad(pad)),
        None => Some(player_for_slot(players, slot)),
    }
}

/// Rumbles a losing player's controller as their tower falls apart: once while it
/// waits to give way, and again with the rumble after the blasts.
fn rumble_losers(
    config: Res<crate::config::GameConfig>,
    mut events: MessageReader<CollapseEvent>,
    mut roofs: MessageReader<crate::gameplay::RoofPlaced>,
    gamepads: Query<Entity, With<Gamepad>>,
    towers: Query<&Tower>,
    lobby: Option<Res<crate::select::Lobby>>,
    mut rumble: MessageWriter<GamepadRumbleRequest>,
) {
    let mut pads: Vec<Entity> = gamepads.iter().collect();
    pads.sort();
    let players = towers.iter().filter(|t| t.human).count();
    let seats = seated(lobby.as_deref(), players);
    let look = &config.moments;
    // (tower, seconds, both motors): a loser's 2 s on the big motor (GameBaskController),
    // the winner's roof set down 1.5 s on both (ROOF_PLACED).
    let mut wants: Vec<(usize, f32, bool)> = events
        .read()
        .filter(|e| matches!(e.kind, CollapseKind::Begin))
        .map(|e| (e.tower, look.pad_rumble, false))
        .collect();
    wants.extend(
        roofs
            .read()
            .filter_map(|r| towers.get(r.tower).ok())
            .map(|t| (t.index, look.roof_rumble, true)),
    );
    for (index, seconds, both) in wants {
        let Some(player) = towers.iter().find(|t| t.index == index).and_then(|t| t.player) else {
            continue;
        };
        let pad = (0..pads.len())
            .find(|&slot| pad_player(seats.as_deref(), players, slot, pads[slot]) == Some(player))
            .map(|slot| pads[slot]);
        if let Some(gamepad) = pad {
            let strength = look.pad_rumble_strength;
            rumble.write(GamepadRumbleRequest::Add {
                duration: std::time::Duration::from_secs_f32(seconds),
                intensity: if both {
                    GamepadRumbleIntensity { strong_motor: strength, weak_motor: strength }
                } else {
                    GamepadRumbleIntensity::strong_motor(strength)
                },
                gamepad,
            });
        }
    }
}

/// LargeBrickControllerRumbleEffect: once a Large brick has grown, its player's pad rumbles
/// on the big motor while it is steered, and 0.2 s more after it first touches down.
fn rumble_large_bricks(
    config: Res<crate::config::GameConfig>,
    gamepads: Query<Entity, With<Gamepad>>,
    towers: Query<(&Tower, &crate::gameplay::TowerState)>,
    bricks: Query<&crate::gameplay::Brick>,
    placed: Query<&crate::gameplay::Brick, Added<crate::gameplay::Placed>>,
    lobby: Option<Res<crate::select::Lobby>>,
    mut rumble: MessageWriter<GamepadRumbleRequest>,
) {
    let mut pads: Vec<Entity> = gamepads.iter().collect();
    if pads.is_empty() {
        return;
    }
    pads.sort();
    let players = towers.iter().filter(|(t, _)| t.human).count();
    let seats = seated(lobby.as_deref(), players);
    let strength = config.moments.pad_rumble_strength;
    let mut buzz = |tower: &Tower, seconds: f32| {
        let Some(player) = tower.player else { return };
        if let Some(slot) = (0..pads.len())
            .find(|&slot| pad_player(seats.as_deref(), players, slot, pads[slot]) == Some(player))
        {
            rumble.write(GamepadRumbleRequest::Add {
                duration: std::time::Duration::from_secs_f32(seconds),
                intensity: GamepadRumbleIntensity::strong_motor(strength),
                gamepad: pads[slot],
            });
        }
    };
    for (tower, state) in &towers {
        // Steered and grown: a short buzz each frame keeps it going.
        if state.active.and_then(|e| bricks.get(e).ok()).is_some_and(|b| b.scale > 1.0) {
            buzz(tower, 0.05);
        }
    }
    for brick in &placed {
        if brick.scale > 1.0
            && let Ok((tower, _)) = towers.get(brick.tower)
        {
            buzz(tower, 0.2);
        }
    }
}

// The keys are player 1's `KeyBindings` (src/keybinding.rs), the original's defaults
// unless changed in Settings → Keybinding.
// Our earlier keys, not the original's; kept here to put back if wanted:
// const LEFT: [KeyCode; 2] = [KeyCode::ArrowLeft, KeyCode::KeyA];
// const RIGHT: [KeyCode; 2] = [KeyCode::ArrowRight, KeyCode::KeyD];
// const DOWN: [KeyCode; 2] = [KeyCode::ArrowDown, KeyCode::KeyS];
// const ROTATE: [KeyCode; 3] = [KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Space];
// const ROTATE_CCW: [KeyCode; 1] = [KeyCode::KeyZ];
// const CAST_LIGHT: [KeyCode; 1] = [KeyCode::KeyQ];
// const CAST_DARK: [KeyCode; 1] = [KeyCode::KeyE];
// /// Held with a move key, makes it a nudge instead.
// const NUDGE: [KeyCode; 2] = [KeyCode::ShiftLeft, KeyCode::ShiftRight];
// DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
// R restarts a match; the original restarts through Pause → Retry or Rematch.
const RESTART: [KeyCode; 1] = [KeyCode::KeyR];

fn read_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    game: Res<Match>,
    mut request: ResMut<MatchRequest>,
    mut towers: Query<(&Tower, &mut PlayerInput)>,
    lobby: Option<Res<crate::select::Lobby>>,
) {
    if game.phase == Phase::Menu {
        return;
    }
    // In a local battle the keyboard drives the player whose seat it took.
    let players = towers.iter().filter(|(t, _)| t.human).count();
    let player = match seated(lobby.as_deref(), players) {
        Some(seats) => match seats.iter().position(|s| *s == crate::select::Seat::Keyboard) {
            Some(p) => p,
            None => return,
        },
        None => 0,
    };
    // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
    if keys.any_just_pressed(RESTART) {
        request.pending = true;
    }
    let Some((_, mut input)) = towers.iter_mut().find(|(tower, _)| tower.player == Some(player)) else {
        return;
    };
    let key = |action| bindings.key(action);
    input.left_held = keys.pressed(key(Action::MoveLeft));
    input.right_held = keys.pressed(key(Action::MoveRight));
    input.fast_fall_held = keys.pressed(key(Action::Drop));
    // Presses accumulate until a fixed step consumes them, so quick taps
    // between steps are never lost.
    input.left_pressed |= keys.just_pressed(key(Action::MoveLeft));
    input.right_pressed |= keys.just_pressed(key(Action::MoveRight));
    input.nudge_left_pressed |= keys.just_pressed(key(Action::NudgeLeft));
    input.nudge_right_pressed |= keys.just_pressed(key(Action::NudgeRight));
    // Shift + Left/Right nudged before the original's U/I keys were known:
    // let nudge = keys.any_pressed(NUDGE);
    // input.left_held = !nudge && keys.any_pressed(LEFT);
    // input.right_held = !nudge && keys.any_pressed(RIGHT);
    // if nudge {
    //     input.nudge_left_pressed |= keys.any_just_pressed(LEFT);
    //     input.nudge_right_pressed |= keys.any_just_pressed(RIGHT);
    // } else {
    //     input.left_pressed |= keys.any_just_pressed(LEFT);
    //     input.right_pressed |= keys.any_just_pressed(RIGHT);
    // }
    input.rotate_pressed |= keys.just_pressed(key(Action::TurnRight));
    input.rotate_ccw_pressed |= keys.just_pressed(key(Action::TurnLeft));
    input.cast_light_pressed |= keys.just_pressed(key(Action::LightMagic));
    input.cast_dark_pressed |= keys.just_pressed(key(Action::DarkMagic));
}

/// Stick or d-pad direction last frame, to turn holds into presses.
#[derive(Default)]
struct PadState {
    left: bool,
    right: bool,
}

fn read_gamepads(
    game: Res<Match>,
    mut request: ResMut<MatchRequest>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut towers: Query<(&Tower, &mut PlayerInput)>,
    mut last: Local<bevy::platform::collections::HashMap<Entity, PadState>>,
    lobby: Option<Res<crate::select::Lobby>>,
) {
    let mut pads: Vec<_> = gamepads.iter().collect();
    pads.sort_by_key(|(e, _)| *e);
    let players = towers.iter().filter(|(t, _)| t.human).count();
    let seats = seated(lobby.as_deref(), players);
    for (slot, (entity, pad)) in pads.into_iter().enumerate() {
        // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
        // Start restarts a finished match; in the original the popup does that.
        if game.phase == Phase::Over && pad.just_pressed(GamepadButton::Start) {
            request.pending = true;
        }
        if game.phase == Phase::Menu {
            continue;
        }
        let Some(player) = pad_player(seats.as_deref(), players, slot, entity) else {
            continue;
        };
        let Some((_, mut input)) = towers.iter_mut().find(|(t, _)| t.player == Some(player)) else {
            continue;
        };
        let stick = pad.left_stick();
        let left = stick.x < -0.5 || pad.pressed(GamepadButton::DPadLeft);
        let right = stick.x > 0.5 || pad.pressed(GamepadButton::DPadRight);
        let down = stick.y < -0.5 || pad.pressed(GamepadButton::DPadDown);
        let before = last.entry(entity).or_default();
        // Player 1 shares with the keyboard, which already set the holds this frame.
        let shared = player == 0;
        input.left_held = left || (shared && input.left_held);
        input.right_held = right || (shared && input.right_held);
        input.fast_fall_held = down || (shared && input.fast_fall_held);
        input.left_pressed |= left && !before.left;
        input.right_pressed |= right && !before.right;
        input.nudge_left_pressed |= pad.just_pressed(GamepadButton::LeftTrigger);
        input.nudge_right_pressed |= pad.just_pressed(GamepadButton::RightTrigger);
        // The original's pad defaults (docs/research/wiki-notes.md, checked on the PC):
        // A, D-pad up or the right trigger turn right, the left trigger turns left,
        // the bumpers nudge, X casts the light spell and B the dark one.
        input.rotate_pressed |= pad.any_just_pressed([
            GamepadButton::South,
            GamepadButton::DPadUp,
            GamepadButton::RightTrigger2,
        ]);
        input.rotate_ccw_pressed |= pad.just_pressed(GamepadButton::LeftTrigger2);
        input.cast_light_pressed |= pad.just_pressed(GamepadButton::West);
        input.cast_dark_pressed |= pad.just_pressed(GamepadButton::East);
        *before = PadState { left, right };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;

    #[test]
    fn seated_pads_drive_their_own_seats_player() {
        use crate::select::Seat;
        let pad_a = Entity::from_raw_u32(7).unwrap();
        let pad_b = Entity::from_raw_u32(9).unwrap();
        // Two pads joined seats 1 and 2, nobody on the keyboard.
        let seats = [Seat::Pad(pad_a), Seat::Pad(pad_b)];
        assert_eq!(pad_player(Some(&seats), 2, 0, pad_a), Some(0));
        assert_eq!(pad_player(Some(&seats), 2, 1, pad_b), Some(1));
        // A pad that didn't join drives nobody.
        let other = Entity::from_raw_u32(11).unwrap();
        assert_eq!(pad_player(Some(&seats), 2, 2, other), None);
        // Without seats, pads follow connection order after the keyboard.
        assert_eq!(pad_player(None, 2, 0, pad_a), Some(1));
    }

    #[test]
    fn a_losing_players_pad_rumbles_once_at_the_start() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(GameConfig::default())
            .add_message::<CollapseEvent>()
            .add_message::<crate::gameplay::RoofPlaced>()
            .add_message::<GamepadRumbleRequest>()
            .add_systems(Update, rumble_losers);
        // Two human players: the first pad drives player 2.
        for index in 0..2 {
            app.world_mut().spawn(Tower {
                index,
                origin_x: 0.0,
                human: true,
                player: Some(index),
            });
        }
        let pad = app.world_mut().spawn(Gamepad::default()).id();
        let mut cursor = app
            .world()
            .resource::<Messages<GamepadRumbleRequest>>()
            .get_cursor();
        let mut send = |app: &mut App, tower: usize, kind: CollapseKind| {
            app.world_mut().write_message(CollapseEvent { tower, kind });
            app.update();
            let messages = app.world().resource::<Messages<GamepadRumbleRequest>>();
            cursor
                .read(messages)
                .filter_map(|r| match r {
                    GamepadRumbleRequest::Add {
                        gamepad, duration, ..
                    } => Some((*gamepad, duration.as_secs_f32())),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        // Player 1 is on the keyboard: nothing to rumble.
        assert!(send(&mut app, 0, CollapseKind::Begin).is_empty());
        assert_eq!(send(&mut app, 1, CollapseKind::Begin), vec![(pad, 2.0)]);
        assert!(send(&mut app, 1, CollapseKind::Rumble(2.5)).is_empty());
        assert!(send(&mut app, 1, CollapseKind::Thud).is_empty());
    }
}
