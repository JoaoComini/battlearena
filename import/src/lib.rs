use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use scene::{MaterialRef, MeshRef};

/// Place this component on an entity to import a GLTF/GLB file as a child scene.
#[derive(Component)]
pub struct ImportGltf(pub String);

pub struct ImportPlugin;

impl Plugin for ImportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, initiate_import);
        app.add_observer(tag_refs);
    }
}

fn initiate_import(
    mut commands: Commands,
    query: Query<(Entity, &ImportGltf)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, import) in &query {
        let handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset(import.0.clone()));
        commands
            .entity(entity)
            .insert(SceneRoot(handle))
            .remove::<ImportGltf>();
    }
}

fn tag_refs(
    trigger: On<SceneInstanceReady>,
    scene_spawner: Res<SceneSpawner>,
    asset_server: Res<AssetServer>,
    mesh_query: Query<(Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>)>,
    mut commands: Commands,
) {
    for entity in scene_spawner.iter_instance_entities(trigger.instance_id) {
        let Ok((mesh, material)) = mesh_query.get(entity) else {
            continue;
        };

        if let Some(mesh) = mesh {
            if let Some(path) = asset_server.get_path(mesh.id()) {
                commands.entity(entity).insert(MeshRef(path.to_string()));
            }
        }

        if let Some(material) = material {
            if let Some(path) = asset_server.get_path(material.id()) {
                commands
                    .entity(entity)
                    .insert(MaterialRef(path.to_string()));
            }
        }
    }
}
