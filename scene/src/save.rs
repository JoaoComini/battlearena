use avian2d::prelude::{ColliderConstructor, RigidBody};
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use std::path::Path;

use crate::{
    format::{SceneEntity, SceneFile},
    GltfPrimitiveRef,
};

/// Extracts entities matching filter `F` from `world` and saves them to disk as RON.
pub fn save<F: QueryFilter>(world: &mut World, path: impl AsRef<Path>) -> Result<(), SaveError> {
    let scene_file = build_scene_file::<F>(world);
    let serialized = ron::ser::to_string_pretty(&scene_file, ron::ser::PrettyConfig::default())
        .map_err(SaveError::Serialize)?;
    std::fs::write(path, serialized).map_err(SaveError::Io)?;
    Ok(())
}

fn build_scene_file<F: QueryFilter>(world: &mut World) -> SceneFile {
    // Collect (entity, parent) for all matching entities.
    let matched: Vec<(Entity, Option<Entity>)> = world
        .query_filtered::<(Entity, Option<&ChildOf>), F>()
        .iter(world)
        .map(|(e, p)| (e, p.map(|p| p.parent())))
        .collect();

    let matched_ids: std::collections::HashSet<Entity> =
        matched.iter().map(|(e, _)| *e).collect();

    // Roots are matched entities whose parent is not also matched.
    let roots: Vec<Entity> = matched
        .iter()
        .filter(|(_, parent)| parent.map(|p| !matched_ids.contains(&p)).unwrap_or(true))
        .map(|(e, _)| *e)
        .collect();

    let entities = roots
        .iter()
        .map(|&e| build_scene_entity(e, &matched_ids, world))
        .collect();

    SceneFile::new(entities)
}

fn build_scene_entity(
    entity: Entity,
    matched_ids: &std::collections::HashSet<Entity>,
    world: &World,
) -> SceneEntity {
    let transform = world
        .get::<Transform>(entity)
        .copied()
        .unwrap_or_default();
    let name = world.get::<Name>(entity).map(|n| n.to_string());
    let mesh = world.get::<GltfPrimitiveRef>(entity).cloned();
    let collider = world.get::<ColliderConstructor>(entity).cloned();
    let rigid_body = world.get::<RigidBody>(entity).copied();

    let children = world
        .get::<Children>(entity)
        .map(|c| c.iter().collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|&child| matched_ids.contains(&child))
        .map(|child| build_scene_entity(child, matched_ids, world))
        .collect();

    SceneEntity {
        name,
        transform: transform.into(),
        mesh,
        collider,
        rigid_body,
        children,
    }
}

#[derive(Debug)]
pub enum SaveError {
    Serialize(ron::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Serialize(e) => write!(f, "scene serialization error: {e}"),
            SaveError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for SaveError {}

#[cfg(test)]
mod tests {
    use avian2d::prelude::{ColliderConstructor, RigidBody};
    use bevy::prelude::*;

    use crate::{load::load, GltfPrimitiveRef, ScenePlugin};

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ScenePlugin));
        app
    }

    fn make_primitive_ref() -> GltfPrimitiveRef {
        GltfPrimitiveRef {
            path: "models/arena.glb".to_string(),
            mesh_index: 0,
            primitive_index: 0,
        }
    }

    #[test]
    fn roundtrip_minimal() {
        let mut app = make_app();
        app.world_mut().spawn((
            Name::new("Node"),
            Transform::from_xyz(1.0, 2.0, 3.0),
            make_primitive_ref(),
        ));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        super::save::<With<GltfPrimitiveRef>>(app.world_mut(), tmp.path()).unwrap();

        let loaded = load(tmp.path()).unwrap();
        assert_eq!(loaded.entities.len(), 1);
        let e = &loaded.entities[0];
        assert_eq!(e.name.as_deref(), Some("Node"));
        assert_eq!(e.transform.translation, [1.0, 2.0, 3.0]);
        let mesh = e.mesh.as_ref().unwrap();
        assert_eq!(mesh.mesh_index, 0);
        assert_eq!(mesh.primitive_index, 0);
    }

    #[test]
    fn roundtrip_with_collider_and_rigid_body() {
        let mut app = make_app();
        app.world_mut().spawn((
            make_primitive_ref(),
            ColliderConstructor::Circle { radius: 30.0 },
            RigidBody::Static,
        ));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        super::save::<With<GltfPrimitiveRef>>(app.world_mut(), tmp.path()).unwrap();

        let loaded = load(tmp.path()).unwrap();
        let e = &loaded.entities[0];
        assert_eq!(e.collider, Some(ColliderConstructor::Circle { radius: 30.0 }));
        assert_eq!(e.rigid_body, Some(RigidBody::Static));
    }

    #[test]
    fn roundtrip_with_children() {
        let mut app = make_app();

        let child = app
            .world_mut()
            .spawn((Name::new("Child"), make_primitive_ref(), Transform::default()))
            .id();
        app.world_mut()
            .spawn((
                Name::new("Parent"),
                Transform::from_xyz(5.0, 0.0, 0.0),
            ))
            .add_child(child);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        super::save::<Or<(With<GltfPrimitiveRef>, With<Name>)>>(app.world_mut(), tmp.path())
            .unwrap();

        let loaded = load(tmp.path()).unwrap();
        assert_eq!(loaded.entities.len(), 1);
        assert_eq!(loaded.entities[0].children.len(), 1);
        assert_eq!(loaded.entities[0].children[0].name.as_deref(), Some("Child"));
    }
}
