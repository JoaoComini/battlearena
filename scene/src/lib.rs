mod components;
mod save;
mod load;

pub use components::GltfNodeRef;
pub use save::{save_scene, SaveSceneError};
pub use load::load_scene;

use bevy::gltf::{GltfAssetLabel, GltfMesh, GltfNode};
use bevy::prelude::*;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GltfNodeRef>();
        app.add_systems(Update, resolve_gltf_node_refs);
    }
}

/// Resolves `GltfNodeRef` components into mesh/material child entities.
///
/// For each entity with a `GltfNodeRef` that hasn't been resolved yet:
/// - Loads the `GltfNode` asset directly via `GltfAssetLabel::Node(index)`.
/// - Spawns one child entity per primitive in the node's mesh, each with
///   `Mesh3d` and `MeshMaterial3d`.
///
/// Waits until all required assets are loaded before acting, so it is safe
/// to run every frame — it is a no-op until assets are ready.
fn resolve_gltf_node_refs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    query: Query<(Entity, &GltfNodeRef), Without<Children>>,
) {
    for (entity, node_ref) in &query {
        let node_handle: Handle<GltfNode> = asset_server
            .load(GltfAssetLabel::Node(node_ref.index).from_asset(node_ref.path.clone()));

        let Some(node) = gltf_nodes.get(&node_handle) else {
            continue;
        };

        let Some(mesh_handle) = &node.mesh else {
            continue;
        };
        let Some(gltf_mesh) = gltf_meshes.get(mesh_handle) else {
            continue;
        };

        let children: Vec<Entity> = gltf_mesh
            .primitives
            .iter()
            .map(|primitive| {
                let mut child = commands.spawn((
                    Mesh3d(primitive.mesh.clone()),
                    Transform::default(),
                ));
                if let Some(material) = primitive.material.clone() {
                    child.insert(MeshMaterial3d(material));
                }
                child.id()
            })
            .collect();

        commands.entity(entity).add_children(&children);
    }
}
