mod systems;

use bevy::prelude::*;

use crate::resources::SpawnRegistry;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnRegistry>()
            .add_systems(Startup, systems::setup_scene);
    }
}
