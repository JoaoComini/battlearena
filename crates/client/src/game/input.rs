use bevy::prelude::*;
use shared::protocol::InputBits;

use crate::resources::CurrentInput;

/// Samples held keys each frame. Runs in PreUpdate so input is always fresh
/// before any game logic in Update or FixedUpdate.
pub fn capture_input(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<CurrentInput>) {
    let mut bits = InputBits::default();
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        bits.set(InputBits::UP);
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        bits.set(InputBits::DOWN);
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        bits.set(InputBits::LEFT);
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        bits.set(InputBits::RIGHT);
    }
    input.0 = bits;
}
