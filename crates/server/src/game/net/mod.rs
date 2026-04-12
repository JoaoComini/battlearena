pub mod systems;

use bevy::prelude::*;
use bevy_renet::RenetServer;
use shared::{physics::PhysicsMovementSet, states::AppState};

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(systems::handle_server_events)
            .add_systems(
                FixedUpdate,
                (systems::send_initial_state, systems::broadcast_despawn, systems::receive_inputs)
                    .chain()
                    .in_set(PhysicsMovementSet::PrepareInput)
                    .run_if(resource_exists::<RenetServer>)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                systems::broadcast_state
                    .in_set(PhysicsMovementSet::Flush)
                    .run_if(resource_exists::<RenetServer>)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
