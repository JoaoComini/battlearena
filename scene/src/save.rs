use avian2d::prelude::{ColliderConstructor, Position, RigidBody, Rotation};
use bevy::prelude::*;
use bevy::reflect::serde::ReflectSerializer;
use bevy::reflect::TypeRegistry;
use std::collections::HashMap;
use std::path::Path;

use crate::components::{OverrideScene, SceneSourcePath};

pub fn save(root: Entity, world: &World, path: impl AsRef<Path>) -> Result<(), SaveError> {
    let source = world
        .get::<SceneSourcePath>(root)
        .ok_or(SaveError::MissingSourcePath)?
        .0
        .clone();

    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let registry = type_registry.read();

    let mut overrides: HashMap<String, HashMap<String, ron::Value>> = HashMap::new();

    for entity in iter_descendants(root, world) {
        let Some(name) = world.get::<Name>(entity) else {
            continue;
        };
        let name_str = name.as_str().to_string();

        let mut components: HashMap<String, ron::Value> = HashMap::new();

        serialize_component::<ColliderConstructor>(entity, world, &registry, &mut components);
        serialize_component::<RigidBody>(entity, world, &registry, &mut components);
        serialize_component::<Position>(entity, world, &registry, &mut components);
        serialize_component::<Rotation>(entity, world, &registry, &mut components);

        if !components.is_empty() {
            overrides.insert(name_str, components);
        }
    }

    let scene = OverrideScene { source, overrides };
    let serialized = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default())
        .map_err(SaveError::Serialize)?;

    std::fs::write(path, serialized).map_err(SaveError::Io)?;
    Ok(())
}

fn serialize_component<T: Component + Reflect>(
    entity: Entity,
    world: &World,
    registry: &TypeRegistry,
    out: &mut HashMap<String, ron::Value>,
) {
    let Some(component) = world.get::<T>(entity) else {
        return;
    };
    let type_path = component.reflect_type_path().to_string();
    let serializer = ReflectSerializer::new(component, registry);
    let ron_str = match ron::to_string(&serializer) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to serialize {type_path}: {e}");
            return;
        }
    };

    let wrapped: ron::Value = match ron::from_str(&ron_str) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to parse serialized {type_path}: {e}");
            return;
        }
    };

    let inner = match wrapped {
        ron::Value::Map(mut m) => m.values_mut().next().cloned(),
        _ => None,
    };

    if let Some(value) = inner {
        out.insert(type_path, value);
    }
}

fn iter_descendants(root: Entity, world: &World) -> Vec<Entity> {
    let mut result = Vec::new();
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if let Some(children) = world.get::<Children>(entity) {
            for &child in children {
                result.push(child);
                stack.push(child);
            }
        }
    }
    result
}

#[derive(Debug)]
pub enum SaveError {
    MissingSourcePath,
    Serialize(ron::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::MissingSourcePath => {
                write!(f, "scene has no SceneSourcePath — open a GLB first")
            }
            SaveError::Serialize(e) => write!(f, "scene serialization error: {e}"),
            SaveError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for SaveError {}
