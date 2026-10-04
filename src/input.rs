//! Keyboard and gamepad input. Player 1 uses the keyboard; gamepads drive
//! players 2 to 4 in the order they were connected (with one player, the first
//! gamepad drives player 1 too). Input is written into each tower's
//! [`PlayerInput`] once per frame; the fixed-step gameplay systems consume it.

use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;

use crate::collapse::{CollapseEvent, CollapseKind};
use crate::gameplay::{Match, MatchRequest, Phase, PlayerInput, Tower};

pub struct KeyboardInputPlugin;

impl Plugin for KeyboardInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
            rumble_losers.run_if(resource_exists::<Messages<GamepadRumbleRequest>>),
        );
    }
}

/// Which player a gamepad drives: pads drive players 2, 3, 4 in the order they
/// connected; alone, the first pad drives player 1.
fn player_for_slot(players: usize, slot: usize) -> usize {
    if players <= 1 { slot } else { slot + 1 }
}

/// Rumbles a losing player's controller as their tower falls apart: once while it
/// waits to give way, and again with the rumble after the blasts.
fn rumble_losers(
    config: Res<crate::config::GameConfig>,
    mut events: MessageReader<CollapseEvent>,
    gamepads: Query<Entity, With<Gamepad>>,
    towers: Query<&Tower>,
    mut rumble: MessageWriter<GamepadRumbleRequest>,
) {
    let mut pads: Vec<Entity> = gamepads.iter().collect();
    pads.sort();
    let players = towers.iter().filter(|t| t.human).count();
    let look = &config.moments;
    for event in events.read() {
        let seconds = match event.kind {
            CollapseKind::Begin => look.pad_rumble.0,
            CollapseKind::Rumble(_) => look.pad_rumble.1,
            _ => continue,
        };
        let Some(player) = towers
            .iter()
            .find(|t| t.index == event.tower)
            .and_then(|t| t.player)
        else {
            continue;
        };
        let pad = (0..pads.len())
            .find(|&slot| player_for_slot(players, slot) == player)
            .map(|slot| pads[slot]);
        if let Some(gamepad) = pad {
            rumble.write(GamepadRumbleRequest::Add {
                duration: std::time::Duration::from_secs_f32(seconds),
                intensity: GamepadRumbleIntensity::strong_motor(look.pad_rumble_strength),
                gamepad,
            });
        }
    }
}

// The original's default keyboard keys (docs/research/wiki-notes.md, "Default keyboard
// controls"; to be confirmed on the install): arrows move and drop, Up turns right,
// Space turns left, U/I nudge, J/K cast the light and dark spell.
pub(crate) const LEFT: [KeyCode; 1] = [KeyCode::ArrowLeft];
pub(crate) const RIGHT: [KeyCode; 1] = [KeyCode::ArrowRight];
const DOWN: [KeyCode; 1] = [KeyCode::ArrowDown];
pub(crate) const ROTATE: [KeyCode; 1] = [KeyCode::ArrowUp];
pub(crate) const ROTATE_CCW: [KeyCode; 1] = [KeyCode::Space];
pub(crate) const NUDGE_LEFT: [KeyCode; 1] = [KeyCode::KeyU];
pub(crate) const NUDGE_RIGHT: [KeyCode; 1] = [KeyCode::KeyI];
const CAST_LIGHT: [KeyCode; 1] = [KeyCode::KeyJ];
const CAST_DARK: [KeyCode; 1] = [KeyCode::KeyK];
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
    game: Res<Match>,
    mut request: ResMut<MatchRequest>,
    mut towers: Query<(&Tower, &mut PlayerInput)>,
) {
    if game.phase == Phase::Menu {
        return;
    }
    // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
    if keys.any_just_pressed(RESTART) {
        request.pending = true;
    }
    let Some((_, mut input)) = towers.iter_mut().find(|(tower, _)| tower.is_first_player()) else {
        return;
    };
    input.left_held = keys.any_pressed(LEFT);
    input.right_held = keys.any_pressed(RIGHT);
    input.fast_fall_held = keys.any_pressed(DOWN);
    // Presses accumulate until a fixed step consumes them, so quick taps
    // between steps are never lost.
    input.left_pressed |= keys.any_just_pressed(LEFT);
    input.right_pressed |= keys.any_just_pressed(RIGHT);
    input.nudge_left_pressed |= keys.any_just_pressed(NUDGE_LEFT);
    input.nudge_right_pressed |= keys.any_just_pressed(NUDGE_RIGHT);
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
    input.rotate_pressed |= keys.any_just_pressed(ROTATE);
    input.rotate_ccw_pressed |= keys.any_just_pressed(ROTATE_CCW);
    input.cast_light_pressed |= keys.any_just_pressed(CAST_LIGHT);
    input.cast_dark_pressed |= keys.any_just_pressed(CAST_DARK);
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
) {
    let mut pads: Vec<_> = gamepads.iter().collect();
    pads.sort_by_key(|(e, _)| *e);
    let players = towers.iter().filter(|(t, _)| t.human).count();
    for (slot, (entity, pad)) in pads.into_iter().enumerate() {
        // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
        // Start restarts a finished match; in the original the popup does that.
        if game.phase == Phase::Over && pad.just_pressed(GamepadButton::Start) {
            request.pending = true;
        }
        if game.phase == Phase::Menu {
            continue;
        }
        let player = player_for_slot(players, slot);
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
        input.rotate_pressed |= pad.just_pressed(GamepadButton::South);
        input.rotate_ccw_pressed |= pad.just_pressed(GamepadButton::East);
        input.cast_light_pressed |= pad.just_pressed(GamepadButton::West);
        input.cast_dark_pressed |= pad.just_pressed(GamepadButton::North);
        *before = PadState { left, right };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;

    #[test]
    fn a_losing_players_pad_rumbles_at_the_start_and_with_the_collapse() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(GameConfig::default())
            .add_message::<CollapseEvent>()
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
        assert_eq!(
            send(&mut app, 1, CollapseKind::Rumble(2.5)),
            vec![(pad, 2.5)]
        );
        assert!(send(&mut app, 1, CollapseKind::Thud).is_empty());
    }
}
