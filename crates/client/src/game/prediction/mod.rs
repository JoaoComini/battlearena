mod systems;

use bevy::prelude::*;
use bevy_renet::client_connected;
use shared::{
    physics::PhysicsMovementSet,
    states::AppState,
};

pub struct PredictionPlugin;

impl Plugin for PredictionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (systems::tick_prediction, systems::apply_correction)
                .chain()
                .in_set(PhysicsMovementSet::PrepareInput)
                .run_if(client_connected)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            FixedUpdate,
            systems::update_predicted_position
                .in_set(PhysicsMovementSet::Flush)
                .run_if(client_connected)
                .run_if(in_state(AppState::InGame)),
        );
    }
}
