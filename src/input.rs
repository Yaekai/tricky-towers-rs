//! Keyboard and gamepad input. Player 1 uses the keyboard; gamepads drive
//! players 2 to 4 in the order they were connected (with one player, the first
//! gamepad drives player 1 too). Input is written into each tower's
//! [`PlayerInput`] once per frame; the fixed-step gameplay systems consume it.

use bevy::prelude::*;

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
        );
    }
}

const LEFT: [KeyCode; 2] = [KeyCode::ArrowLeft, KeyCode::KeyA];
const RIGHT: [KeyCode; 2] = [KeyCode::ArrowRight, KeyCode::KeyD];
const DOWN: [KeyCode; 2] = [KeyCode::ArrowDown, KeyCode::KeyS];
const ROTATE: [KeyCode; 3] = [KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Space];
const ROTATE_CCW: [KeyCode; 1] = [KeyCode::KeyZ];
const CAST_LIGHT: [KeyCode; 1] = [KeyCode::KeyQ];
const CAST_DARK: [KeyCode; 1] = [KeyCode::KeyE];
/// Held with a move key, makes it a nudge instead.
const NUDGE: [KeyCode; 2] = [KeyCode::ShiftLeft, KeyCode::ShiftRight];
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
    if keys.any_just_pressed(RESTART) {
        request.pending = true;
    }
    let Some((_, mut input)) = towers.iter_mut().find(|(tower, _)| tower.is_first_player()) else {
        return;
    };
    let nudge = keys.any_pressed(NUDGE);
    input.left_held = !nudge && keys.any_pressed(LEFT);
    input.right_held = !nudge && keys.any_pressed(RIGHT);
    input.fast_fall_held = keys.any_pressed(DOWN);
    // Presses accumulate until a fixed step consumes them, so quick taps
    // between steps are never lost.
    if nudge {
        input.nudge_left_pressed |= keys.any_just_pressed(LEFT);
        input.nudge_right_pressed |= keys.any_just_pressed(RIGHT);
    } else {
        input.left_pressed |= keys.any_just_pressed(LEFT);
        input.right_pressed |= keys.any_just_pressed(RIGHT);
    }
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
        if game.phase == Phase::Over && pad.just_pressed(GamepadButton::Start) {
            request.pending = true;
        }
        if game.phase == Phase::Menu {
            continue;
        }
        // Pads drive players 2, 3, 4; alone, the first pad drives player 1.
        let player = if players <= 1 { slot } else { slot + 1 };
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
