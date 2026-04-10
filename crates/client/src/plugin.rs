use bevy::prelude::*;

use crate::{
    game::{
        input::capture_input,
        net::{recv_server, send_input_tick},
        prediction::{apply_correction, tick_prediction},
        render::render_players,
        scene::setup_scene,
    },
    resources::{CurrentInput, EntityRegistry, LocalTick, ServerTime, SpawnRegistry},
};

pub struct ClientGamePlugin;

impl Plugin for ClientGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentInput>()
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
                )
            )
            .add_systems(
                Update,
                (
                    apply_correction,
                    render_players,
                ),
            )
            .add_systems(
                FixedUpdate,
                (tick_prediction, send_input_tick)
                    .chain()
                    .run_if(bevy_renet::client_connected),
            );
    }
}
