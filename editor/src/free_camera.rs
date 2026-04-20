use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

/// Marker for the free camera entity.
#[derive(Component)]
pub struct FreeCamera;

pub struct FreeCameraPlugin;

impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (pan_orbit, zoom));
    }
}

/// Hold right mouse button to look around, WASD/QE to move.
fn pan_orbit(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<FreeCamera>>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };

    // Look around while right mouse button is held.
    if mouse_button.pressed(MouseButton::Right) {
        let delta = motion.delta;
        if delta != Vec2::ZERO {
            let yaw = Quat::from_rotation_y(-delta.x * 0.003);
            let pitch = Quat::from_rotation_x(-delta.y * 0.003);
            transform.rotation = yaw * transform.rotation * pitch;
        }
    }

    // WASD + QE movement in local camera space.
    let speed = 10.0 * time.delta_secs();
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) { direction -= Vec3::Z; }
    if keyboard.pressed(KeyCode::KeyS) { direction += Vec3::Z; }
    if keyboard.pressed(KeyCode::KeyA) { direction -= Vec3::X; }
    if keyboard.pressed(KeyCode::KeyD) { direction += Vec3::X; }
    if keyboard.pressed(KeyCode::KeyE) { direction += Vec3::Y; }
    if keyboard.pressed(KeyCode::KeyQ) { direction -= Vec3::Y; }

    if direction != Vec3::ZERO {
        let movement = transform.rotation * direction.normalize() * speed;
        transform.translation += movement;
    }
}

/// Scroll wheel zooms by moving the camera forward/backward.
fn zoom(
    scroll: Res<AccumulatedMouseScroll>,
    mut camera: Query<&mut Transform, With<FreeCamera>>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };

    if scroll.delta.y != 0.0 {
        let forward = *transform.forward();
        transform.translation += forward * scroll.delta.y;
    }
}
