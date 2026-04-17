use bevy::prelude::*;
use std::path::Path;

use crate::format::{SceneEntity, SceneFile};

/// Deserializes a `SceneFile` from a RON file on disk.
pub fn load(path: impl AsRef<Path>) -> Result<SceneFile, LoadError> {
    let bytes = std::fs::read(path).map_err(LoadError::Io)?;
    let scene = ron::de::from_bytes(&bytes).map_err(LoadError::Deserialize)?;
    Ok(scene)
}

/// Spawns all entities in `scene` into `world`, preserving parent/child
/// relationships. Does not use `SceneSpawner` — entities are inserted directly.
pub fn spawn(scene: &SceneFile, world: &mut World) {
    for entity in &scene.entities {
        spawn_entity(entity, None, world);
    }
}

fn spawn_entity(entity: &SceneEntity, parent: Option<Entity>, world: &mut World) {
    let transform: Transform = entity.transform.into();

    let id = world.spawn((transform, Visibility::default())).id();

    if let Some(name) = &entity.name {
        world.entity_mut(id).insert(Name::new(name.clone()));
    }
    if let Some(mesh) = &entity.mesh {
        world.entity_mut(id).insert(mesh.clone());
    }
    if let Some(collider) = &entity.collider {
        world.entity_mut(id).insert(collider.clone());
    }
    if let Some(rigid_body) = &entity.rigid_body {
        world.entity_mut(id).insert(*rigid_body);
    }
    if let Some(parent_id) = parent {
        world.entity_mut(parent_id).add_child(id);
    }

    for child in &entity.children {
        spawn_entity(child, Some(id), world);
    }
}

#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Deserialize(ron::error::SpannedError),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {e}"),
            LoadError::Deserialize(e) => write!(f, "scene deserialization error: {e}"),
        }
    }
}

impl std::error::Error for LoadError {}
