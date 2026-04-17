use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use editor::EditorPlugin;
use import::ImportPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                file_path: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                ..default()
            }),
        )
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(ImportPlugin)
        .add_plugins(EditorPlugin)
        .run();
}
