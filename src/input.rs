//! Keyboard input for your tower. Translates keys into that tower's
//! [`PlayerInput`] once per frame; the fixed-step gameplay systems consume it.

use bevy::prelude::*;

use crate::gameplay::{Match, MatchRequest, Phase, PlayerInput, Tower};

pub struct KeyboardInputPlugin;

impl Plugin for KeyboardInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            read_keyboard.run_if(resource_exists::<ButtonInput<KeyCode>>),
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
    let Some((_, mut input)) = towers.iter_mut().find(|(tower, _)| tower.human) else {
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
