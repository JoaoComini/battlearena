mod collider_gizmos;
mod file_dialog;
mod free_camera;
mod hierarchy;
mod selection;
mod spawn;
mod transform_gizmo;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use import::ImportPlugin;
use scene::ScenePlugin;

use file_dialog::FileDialogPlugin;
use free_camera::FreeCameraPlugin;
use selection::SelectionPlugin;
use spawn::SpawnPlugin;
use transform_gizmo::TransformGizmoPlugin;
use ui::UiPlugin;

use collider_gizmos::ColliderGizmosPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                file_path: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                watch_for_changes_override: Some(true),
                ..default()
            }),
        )
        .add_plugins((
            ImportPlugin,
            EguiPlugin::default(),
            ScenePlugin,
            MeshPickingPlugin,
            FreeCameraPlugin,
            SpawnPlugin,
            SelectionPlugin,
            UiPlugin,
            TransformGizmoPlugin,
            ColliderGizmosPlugin,
            FileDialogPlugin,
        ))
        .run();
}
