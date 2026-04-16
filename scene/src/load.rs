use bevy::prelude::*;
use std::path::Path;

/// Returns a handle to a `DynamicScene` loaded from a `.scn.ron` file.
///
/// The caller is responsible for spawning the scene via `SceneSpawner` or
/// inserting it as a `DynamicSceneRoot` component. The `ScenePlugin` system
/// `resolve_gltf_mesh_refs` will then attach real `Mesh3d` handles to any
/// spawned entity that carries a `GltfMeshRef`.
///
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
///     let handle = scene::load_scene(&asset_server, "scenes/arena.scn.ron");
///     commands.spawn(DynamicSceneRoot(handle));
/// }
/// ```
pub fn load_scene(asset_server: &AssetServer, path: impl AsRef<Path>) -> Handle<DynamicScene> {
    asset_server.load(path.as_ref().to_string_lossy().into_owned())
}
