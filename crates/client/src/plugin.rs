use bevy::prelude::*;

use crate::{
    game::systems::{
        apply_correction, capture_input, recv_reliable, recv_world_state, render_players,
        send_input_tick, setup_scene,
    },
    resources::{CurrentInput, EntityRegistry, ServerTime, SpawnRegistry},
};

pub struct ClientGamePlugin;

impl Plugin for ClientGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentInput>()
            .init_resource::<EntityRegistry>()
            .init_resource::<ServerTime>()
            .init_resource::<SpawnRegistry>()
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (
                    capture_input,
                    recv_reliable.run_if(bevy_renet::client_connected),
                    recv_world_state.run_if(bevy_renet::client_connected),
                    apply_correction.after(recv_reliable),
                    render_players,
                ),
            )
            .add_systems(
                FixedUpdate,
                send_input_tick.run_if(bevy_renet::client_connected),
            );
    }
}
