mod components;
mod save;

pub use components::{MaterialRef, MeshRef};
pub use save::{save, SaveError};

use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MeshRef>();
        app.register_type::<MaterialRef>();
        app.add_observer(resolve_refs);
    }
}

fn resolve_refs(
    trigger: On<SceneInstanceReady>,
    scene_spawner: Res<SceneSpawner>,
    asset_server: Res<AssetServer>,
    query: Query<(Option<&MeshRef>, Option<&MaterialRef>)>,
    mut commands: Commands,
) {
    for entity in scene_spawner.iter_instance_entities(trigger.instance_id) {
        let Ok((mesh_ref, mat_ref)) = query.get(entity) else {
            continue;
        };

        if let Some(r) = mesh_ref {
            commands
                .entity(entity)
                .insert(Mesh3d(asset_server.load(r.0.clone())));
        }

        if let Some(r) = mat_ref {
            commands
                .entity(entity)
                .insert(MeshMaterial3d::<StandardMaterial>(
                    asset_server.load(r.0.clone()),
                ));
        }
    }
}
