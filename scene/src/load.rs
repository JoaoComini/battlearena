use bevy::prelude::*;
use bevy::scene::DynamicSceneRoot;

use crate::components::{LoadScene, MaterialPath, MeshPath};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MeshPath>();
        app.register_type::<MaterialPath>();
        app.add_systems(Update, (initiate_load, resolve_mesh_paths, resolve_material_paths));
    }
}

fn initiate_load(
    mut commands: Commands,
    query: Query<(Entity, &LoadScene)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, load) in &query {
        let handle: Handle<DynamicScene> = asset_server.load(load.0.clone());
        commands
            .entity(entity)
            .insert(DynamicSceneRoot(handle))
            .remove::<LoadScene>();
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
