mod components;
mod gltf;

pub use components::{MaterialPath, MeshPath};

use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::prelude::*;
use bevy::scene::DynamicSceneRoot;

/// Place this component on an entity to import a scene from the given asset
/// path as children. Supports `.scn` (dynamic scene) and `.glb`/`.gltf`.
#[derive(Component)]
pub struct ImportScene(pub String);

pub struct ImportPlugin;

impl Plugin for ImportPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MeshPath>();
        app.register_type::<MaterialPath>();
        app.add_systems(Update, resolve_mesh_paths);
        app.add_systems(Update, resolve_material_paths);
        app.add_systems(Update, initiate_import);
        app.add_systems(Update, process_pending_gltf);
    }
}

#[derive(Component)]
struct PendingGltf(Handle<Gltf>);

fn initiate_import(
    mut commands: Commands,
    query: Query<(Entity, &ImportScene), Without<PendingGltf>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, import) in &query {
        let path = &import.0;
        if path.ends_with(".scn") {
            let handle: Handle<DynamicScene> = asset_server.load(path.clone());
            commands
                .entity(entity)
                .insert(DynamicSceneRoot(handle))
                .remove::<ImportScene>();
        } else if path.ends_with(".glb") || path.ends_with(".gltf") {
            let handle: Handle<Gltf> = asset_server.load(path.clone());
            commands.entity(entity).insert(PendingGltf(handle));
        } else {
            warn!("ImportScene: unrecognized extension for '{}'", path);
            commands.entity(entity).remove::<ImportScene>();
        }
    }
}

fn process_pending_gltf(
    mut commands: Commands,
    query: Query<(Entity, &ImportScene, &PendingGltf)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
) {
    for (entity, import, pending) in &query {
        let Some(gltf) = gltf_assets.get(&pending.0) else {
            continue;
        };
        gltf::spawn_nodes(gltf, &import.0, entity, &gltf_nodes, &gltf_meshes, &mut commands);
        commands.entity(entity).remove::<ImportScene>().remove::<PendingGltf>();
    }
}

fn resolve_mesh_paths(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &MeshPath), Without<Mesh3d>>,
) {
    for (entity, mesh_path) in &query {
        commands
            .entity(entity)
            .insert(Mesh3d(asset_server.load(mesh_path.0.clone())));
    }
}

fn resolve_material_paths(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &MaterialPath), Without<MeshMaterial3d<StandardMaterial>>>,
) {
    for (entity, material_path) in &query {
        commands
            .entity(entity)
            .insert(MeshMaterial3d::<StandardMaterial>(
                asset_server.load(material_path.0.clone()),
            ));
    }
}
