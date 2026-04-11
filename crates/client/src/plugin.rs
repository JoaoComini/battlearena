use bevy::prelude::*;
use shared::physics::{PhysicsMovementSet, PlayerMovementPlugin};

use crate::{
    game::{
        input::capture_input,
        net::{recv_server, send_input_tick},
        prediction::{apply_correction, tick_prediction, update_predicted_position},
        render::render_players,
        scene::setup_scene,
    },
    resources::{CurrentInput, EntityRegistry, LocalTick, ServerTime, SpawnRegistry},
};

pub struct ClientGamePlugin;

impl Plugin for ClientGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerMovementPlugin)
            .init_resource::<CurrentInput>()
            .init_resource::<EntityRegistry>()
            .init_resource::<LocalTick>()
            .init_resource::<ServerTime>()
            .init_resource::<SpawnRegistry>()
            .add_systems(Startup, setup_scene)
            .add_systems(
                PreUpdate,
                (
                    capture_input,
                    recv_server.run_if(bevy_renet::client_connected),
                ),
            )
            .add_systems(
                FixedUpdate,
                (tick_prediction, apply_correction)
                    .chain()
                    .in_set(PhysicsMovementSet::PrepareInput)
                    .run_if(bevy_renet::client_connected),
            )
            .add_systems(
                FixedUpdate,
                send_input_tick
                    .after(PhysicsMovementSet::PrepareInput)
                    .run_if(bevy_renet::client_connected),
            )
            .add_systems(
                FixedUpdate,
                update_predicted_position
                    .in_set(PhysicsMovementSet::Flush)
                    .run_if(bevy_renet::client_connected),
            )
            .add_systems(Update, render_players);
    }
}
