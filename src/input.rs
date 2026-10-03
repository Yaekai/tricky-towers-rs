//! Keyboard input. Translates keys into [`PlayerInput`] once per frame; the
//! fixed-step gameplay systems consume it.

use bevy::prelude::*;

use crate::gameplay::PlayerInput;

pub struct KeyboardInputPlugin;

impl Plugin for KeyboardInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, read_keyboard.run_if(resource_exists::<ButtonInput<KeyCode>>));
    }
}

const LEFT: [KeyCode; 2] = [KeyCode::ArrowLeft, KeyCode::KeyA];
const RIGHT: [KeyCode; 2] = [KeyCode::ArrowRight, KeyCode::KeyD];
const DOWN: [KeyCode; 2] = [KeyCode::ArrowDown, KeyCode::KeyS];
const ROTATE: [KeyCode; 3] = [KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Space];
const RESTART: [KeyCode; 1] = [KeyCode::KeyR];

fn read_keyboard(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<PlayerInput>) {
    input.left_held = keys.any_pressed(LEFT);
    input.right_held = keys.any_pressed(RIGHT);
    input.fast_fall_held = keys.any_pressed(DOWN);
    // Presses accumulate until a fixed step consumes them, so quick taps
    // between steps are never lost.
    input.left_pressed |= keys.any_just_pressed(LEFT);
    input.right_pressed |= keys.any_just_pressed(RIGHT);
    input.rotate_pressed |= keys.any_just_pressed(ROTATE);
    input.restart_pressed |= keys.any_just_pressed(RESTART);
}
