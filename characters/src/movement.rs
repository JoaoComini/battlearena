use abilities::MovementSpeed;
use avian2d::prelude::{LinearVelocity, Rotation};
use bevy::prelude::*;
use inputs::Inputs;
use lightyear::prelude::input::native::ActionState;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            bevy::app::FixedUpdate,
            (set_lin_velocity, set_rotation).chain(),
        );
    }
}

fn set_lin_velocity(
    mut query: Query<(&mut LinearVelocity, &ActionState<Inputs>, &MovementSpeed)>,
) {
    for (mut velocity, input, speed) in &mut query {
        let Inputs::PlayerInput(player_input) = &input.0;
        let direction = &player_input.movement;
        let mut dir = Vec2::ZERO;
        if direction.up {
            dir.y += 1.0;
        }
        if direction.down {
            dir.y -= 1.0;
        }
        if direction.left {
            dir.x -= 1.0;
        }
        if direction.right {
            dir.x += 1.0;
        }
        velocity.0 = dir.normalize_or_zero() * speed.0;
    }
}

fn set_rotation(mut query: Query<(&mut Rotation, &ActionState<Inputs>)>) {
    for (mut rotation, input) in &mut query {
        let Inputs::PlayerInput(player_input) = &input.0;
        *rotation = Rotation::radians(player_input.movement.angle);
    }
}
