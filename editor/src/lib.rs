mod collider_gizmos;
mod free_camera;
mod hierarchy;
mod selection;
mod spawn;
mod ui;

use bevy::prelude::*;
use collider_gizmos::ColliderGizmosPlugin;
use free_camera::FreeCameraPlugin;
use selection::SelectionPlugin;
use spawn::SpawnPlugin;
use ui::UiPlugin;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FreeCameraPlugin,
            SpawnPlugin,
            SelectionPlugin,
            UiPlugin,
            ColliderGizmosPlugin,
        ));
    }
}
