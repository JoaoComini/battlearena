use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_renet::RenetServer;
use shared::map::MAP_OBSTACLES;

use crate::{
    game::systems::{
        apply_pending_positions, broadcast_despawn, broadcast_state, handle_server_events,
        receive_inputs, send_initial_state, tick_game,
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
        app.init_resource::<EntityRegistry>()
            .add_observer(handle_server_events)
            .add_systems(Startup, spawn_obstacles)
            .add_systems(
                FixedUpdate,
                (send_initial_state, broadcast_despawn, receive_inputs, tick_game, apply_pending_positions, broadcast_state)
                    .chain()
                    .run_if(resource_exists::<RenetServer>),
            );
    }
}
