mod gltf;

use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::prelude::*;

/// Place this component on an entity to import a GLTF/GLB file as children.
#[derive(Component)]
pub struct ImportGltf(pub String);

pub struct ImportPlugin;

impl Plugin for ImportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (initiate_import, process_pending_gltf));
    }
}

#[derive(Component)]
struct PendingGltf(Handle<Gltf>);

fn initiate_import(
    mut commands: Commands,
    query: Query<(Entity, &ImportGltf), Without<PendingGltf>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, import) in &query {
        let handle: Handle<Gltf> = asset_server.load(import.0.clone());
        commands.entity(entity).insert(PendingGltf(handle));
    }
}

fn process_pending_gltf(
    mut commands: Commands,
    query: Query<(Entity, &ImportGltf, &PendingGltf)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
) {
    for (entity, import, pending) in &query {
        let Some(gltf) = gltf_assets.get(&pending.0) else {
            continue;
        };
        gltf::spawn_nodes(gltf, &import.0, entity, &gltf_nodes, &gltf_meshes, &mut commands);
        commands.entity(entity).remove::<ImportGltf>().remove::<PendingGltf>();
    }
}
