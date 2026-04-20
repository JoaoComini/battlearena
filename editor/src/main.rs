use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use editor::EditorPlugin;
use import::ImportPlugin;
use scene::ScenePlugin;

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
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(ScenePlugin)
        .add_plugins(ImportPlugin)
        .add_plugins(EditorPlugin)
        .run();
}
