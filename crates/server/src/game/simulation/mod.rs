mod systems;

use bevy::prelude::*;
use bevy_renet::RenetServer;
use shared::{physics::PhysicsMovementSet, states::AppState};

use super::net::systems as net_systems;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            systems::prepare_physics_inputs
                .in_set(PhysicsMovementSet::PrepareInput)
                .after(net_systems::receive_inputs)
                .run_if(resource_exists::<RenetServer>)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            FixedUpdate,
            systems::verify_and_respond
                .in_set(PhysicsMovementSet::Flush)
                .run_if(resource_exists::<RenetServer>)
                .run_if(in_state(AppState::InGame)),
        );
    }
}
