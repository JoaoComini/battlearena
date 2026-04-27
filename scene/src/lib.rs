mod components;
mod save;

pub use components::{MaterialRef, MeshRef};
pub use save::{save, SaveError};

use avian2d::prelude::RigidBody;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use physics::Wall;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MeshRef>();
        app.register_type::<MaterialRef>();
        app.add_observer(resolve_refs);
        app.add_observer(tag_walls);
    }
}

fn tag_walls(
    trigger: On<SceneInstanceReady>,
    scene_spawner: Res<SceneSpawner>,
    rigid_body_query: Query<&RigidBody>,
    mut commands: Commands,
) {
    for entity in scene_spawner.iter_instance_entities(trigger.instance_id) {
        if let Ok(RigidBody::Static) = rigid_body_query.get(entity) {
            commands.entity(entity).insert(Wall);
        }
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
