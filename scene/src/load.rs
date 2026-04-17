use bevy::prelude::*;
use std::path::Path;

/// Spawns a `.scn.ron` scene directly into the world via `SceneSpawner`.
///
/// Unlike `DynamicSceneRoot`, this spawns entities at the top level with no
/// extra root entity. The `ScenePlugin` system `resolve_gltf_mesh_refs` will
/// attach real `Mesh3d` handles to any spawned entity that carries a
/// `GltfMeshRef`.
///
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// fn setup(asset_server: Res<AssetServer>, mut scene_spawner: ResMut<SceneSpawner>) {
///     scene::load_scene(&asset_server, &mut scene_spawner, "scenes/arena.scn.ron");
/// }
/// ```
pub fn load_scene(
    asset_server: &AssetServer,
    scene_spawner: &mut SceneSpawner,
    path: impl AsRef<Path>,
) {
    let handle: Handle<DynamicScene> =
        asset_server.load(path.as_ref().to_string_lossy().into_owned());
    scene_spawner.spawn_dynamic(handle);
}
