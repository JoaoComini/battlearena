use std::time::Duration;

use avian2d::prelude::*;
use crate::protocol::InputBits;

pub const PLAYER_SPEED: f32 = 200.0; // world units per second
pub const PLAYER_RADIUS: f32 = 16.0; // matches Capsule3d::new(16.0, 24.0) in scene.rs

fn input_velocity(input: InputBits) -> bevy_math::Vec2 {
    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;

    if input.is_set(InputBits::RIGHT) { dx += 1.0; }
    if input.is_set(InputBits::LEFT)  { dx -= 1.0; }
    if input.is_set(InputBits::UP)    { dy += 1.0; }
    if input.is_set(InputBits::DOWN)  { dy -= 1.0; }

    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        dx /= len;
        dy /= len;
    }
    bevy_math::Vec2::new(dx * PLAYER_SPEED, dy * PLAYER_SPEED)
}

/// Advances `position` by one physics tick given `input`, sliding along obstacles.
/// Returns the resulting position.
/// Called by both the server (authoritatively) and the client (for prediction).
pub fn apply_input(
    move_and_slide: &MoveAndSlide,
    collider: &Collider,
    position: bevy_math::Vec2,
    input: InputBits,
    delta: Duration,
    filter: &SpatialQueryFilter,
) -> bevy_math::Vec2 {
    let velocity = input_velocity(input);

    move_and_slide
        .move_and_slide(
            collider,
            position,
            0.0,
            velocity,
            delta,
            &MoveAndSlideConfig::default(),
            filter,
            |_| MoveAndSlideHitResponse::Accept,
        )
        .position
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_input_zero_velocity() {
        let vel = input_velocity(InputBits::default());
        assert_eq!(vel.x, 0.0);
        assert_eq!(vel.y, 0.0);
    }

    #[test]
    fn right_input_positive_x_velocity() {
        let mut input = InputBits::default();
        input.set(InputBits::RIGHT);
        let vel = input_velocity(input);
        assert!(vel.x > 0.0);
        assert_eq!(vel.y, 0.0);
    }

    #[test]
    fn diagonal_speed_equals_cardinal_speed() {
        let mut cardinal = InputBits::default();
        cardinal.set(InputBits::RIGHT);
        let cardinal_vel = input_velocity(cardinal);

        let mut diagonal = InputBits::default();
        diagonal.set(InputBits::RIGHT);
        diagonal.set(InputBits::UP);
        let diagonal_vel = input_velocity(diagonal);

        assert!((cardinal_vel.length() - diagonal_vel.length()).abs() < 1e-5);
    }
}
