use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_renet::RenetServer;
use shared::{
    map::MAP_OBSTACLES,
    physics::{PhysicsMovementSet, PlayerMovementPlugin},
};

use crate::{
    game::systems::{
        broadcast_despawn, broadcast_state, handle_server_events,
        prepare_physics_inputs, receive_inputs, send_initial_state, verify_and_respond,
    },
    resources::EntityRegistry,
};

fn spawn_obstacles(mut commands: Commands) {
    for obs in MAP_OBSTACLES {
        commands.spawn((
            RigidBody::Static,
            Position(Vec2::new(obs.center_x, obs.center_y)),
            Collider::rectangle(obs.half_x * 2.0, obs.half_y * 2.0),
        ));
    }
}

pub struct ServerGamePlugin;

impl Plugin for ServerGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerMovementPlugin)
            .init_resource::<EntityRegistry>()
            .add_observer(handle_server_events)
            .add_systems(Startup, spawn_obstacles)
            .add_systems(
                FixedUpdate,
                (send_initial_state, broadcast_despawn, receive_inputs, prepare_physics_inputs)
                    .chain()
                    .in_set(PhysicsMovementSet::PrepareInput)
                    .run_if(resource_exists::<RenetServer>),
            )
            .add_systems(
                FixedUpdate,
                (verify_and_respond, broadcast_state)
                    .chain()
                    .in_set(PhysicsMovementSet::Flush)
                    .run_if(resource_exists::<RenetServer>),
            );
    }
}
