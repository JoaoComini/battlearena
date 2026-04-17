mod components;
mod load;
mod save;

pub use components::GltfPrimitiveRef;
pub use load::load_scene;
pub use save::{save_scene, SaveSceneError};

use bevy::gltf::{GltfAssetLabel, GltfMesh};
use bevy::prelude::*;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GltfPrimitiveRef>();
        app.add_systems(Update, resolve_gltf_primitive_refs);
    }
}

/// Resolves `GltfPrimitiveRef` components into `Mesh3d` and `MeshMaterial3d`
/// on the same entity.
fn resolve_gltf_primitive_refs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    query: Query<(Entity, &GltfPrimitiveRef), (Without<Mesh3d>, Without<MeshMaterial3d<StandardMaterial>>)>,
) {
    for (entity, prim_ref) in &query {
        let mesh_handle: Handle<GltfMesh> = asset_server.load(
            GltfAssetLabel::Mesh(prim_ref.mesh_index).from_asset(prim_ref.path.clone()),
        );

        let Some(gltf_mesh) = gltf_meshes.get(&mesh_handle) else {
            continue;
        };

        let Some(primitive) = gltf_mesh.primitives.get(prim_ref.primitive_index) else {
            continue;
        };

        let mut entity_cmd = commands.entity(entity);
        entity_cmd.insert(Mesh3d(primitive.mesh.clone()));
        if let Some(material) = primitive.material.clone() {
            entity_cmd.insert(MeshMaterial3d(material));
        }
    }
}
