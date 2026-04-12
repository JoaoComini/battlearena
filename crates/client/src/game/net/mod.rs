pub mod systems;

use bevy::prelude::*;
use bevy_renet::client_connected;
use shared::{
    physics::PhysicsMovementSet,
    states::AppState,
};

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            systems::recv_server.run_if(client_connected),
        )
        .add_systems(
            FixedUpdate,
            systems::send_input_tick
                .after(PhysicsMovementSet::PrepareInput)
                .run_if(client_connected)
                .run_if(in_state(AppState::InGame)),
        );
    }
}
