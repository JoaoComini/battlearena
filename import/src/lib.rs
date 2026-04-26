use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

/// Place this component on an entity to import a GLTF/GLB file as a child scene.
#[derive(Component)]
pub struct ImportGltf(pub String);

pub struct ImportPlugin;

impl Plugin for ImportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, initiate_import);
    }
}

fn initiate_import(
    mut commands: Commands,
    query: Query<(Entity, &ImportGltf)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, import) in &query {
        let handle = asset_server.load(
            GltfAssetLabel::Scene(0).from_asset(import.0.clone()),
        );
        commands
            .entity(entity)
            .insert(SceneRoot(handle))
            .remove::<ImportGltf>();
    }
}
