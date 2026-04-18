use avian2d::prelude::{ColliderConstructor, Position, RigidBody, Rotation};
use bevy::prelude::*;
use bevy::scene::serde::SceneSerializer;

use import::{MaterialPath, MeshPath};
use std::path::Path;

/// Extracts all descendants of `root` from `world` and saves them as a RON
/// scene file. The root entity itself is not saved.
pub fn save(root: Entity, world: &World, path: impl AsRef<Path>) -> Result<(), SaveError> {
    let descendants: Vec<Entity> = iter_descendants(root, world);

    let top_level: std::collections::HashSet<Entity> = world
        .get::<Children>(root)
        .map(|c| c.iter().collect())
        .unwrap_or_default();

    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let registry = type_registry.read();

    let mut dynamic_scene = DynamicSceneBuilder::from_world(world)
        .deny_all_components()
        .allow_component::<Name>()
        .allow_component::<Transform>()
        .allow_component::<Visibility>()
        .allow_component::<ChildOf>()
        .allow_component::<Children>()
        .allow_component::<MeshPath>()
        .allow_component::<MaterialPath>()
        .allow_component::<ColliderConstructor>()
        .allow_component::<RigidBody>()
        .allow_component::<Position>()
        .allow_component::<Rotation>()
        .extract_entities(descendants.into_iter())
        .build();

    let child_of_type_path = std::any::type_name::<ChildOf>();
    for dynamic_entity in &mut dynamic_scene.entities {
        if top_level.contains(&dynamic_entity.entity) {
            dynamic_entity.components.retain(|c| {
                c.get_represented_type_info().map(|t| t.type_path()) != Some(child_of_type_path)
            });
        }
    }

    let serializer = SceneSerializer::new(&dynamic_scene, &registry);
    let serialized = ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default())
        .map_err(SaveError::Serialize)?;

    std::fs::write(path, serialized).map_err(SaveError::Io)?;
    Ok(())
}

fn iter_descendants(root: Entity, world: &World) -> Vec<Entity> {
    let mut result = Vec::new();
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if let Some(children) = world.get::<Children>(entity) {
            for child in children.iter() {
                result.push(child);
                stack.push(child);
            }
        }
    }
    result
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
