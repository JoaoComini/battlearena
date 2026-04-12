mod systems;

use bevy::prelude::*;
use shared::states::AppState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            systems::capture_input.run_if(in_state(AppState::InGame)),
        );
    }
}
