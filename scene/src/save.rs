use bevy::prelude::*;
use bevy::scene::DynamicSceneBuilder;
use std::path::Path;

use crate::GltfNodeRef;

/// Saves all entities with a `GltfMeshRef` component to a `.scn.ron` file.
///
/// Uses Bevy's `DynamicScene` serialization — all components on matching
/// entities that are registered in the `AppTypeRegistry` will be included.
pub fn save_scene(world: &mut World, path: impl AsRef<Path>) -> Result<(), SaveSceneError> {
    let mut query = world.query_filtered::<Entity, With<GltfNodeRef>>();
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(query.iter(world))
        .build();

    let type_registry = world.resource::<AppTypeRegistry>();
    let serialized = scene
        .serialize(&type_registry.read())
        .map_err(SaveSceneError::Serialize)?;

    std::fs::write(path, serialized).map_err(SaveSceneError::Io)?;

    Ok(())
}

#[derive(Debug)]
pub enum SaveSceneError {
    Serialize(ron::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for SaveSceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveSceneError::Serialize(e) => write!(f, "scene serialization error: {e}"),
            SaveSceneError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for SaveSceneError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScenePlugin;
    use bevy::scene::serde::SceneDeserializer;
    use serde::de::DeserializeSeed;

    #[test]
    fn roundtrip() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ScenePlugin));

        let path = "models/arena.glb";
        let index = 3;

        app.world_mut().spawn(GltfNodeRef {
            path: path.to_string(),
            index,
        });

        let tmp = tempfile::NamedTempFile::new().unwrap();
        save_scene(app.world_mut(), tmp.path()).unwrap();

        let ron = std::fs::read_to_string(tmp.path()).unwrap();
        let type_registry = app.world().resource::<AppTypeRegistry>();
        let mut deserializer = ron::de::Deserializer::from_str(&ron).unwrap();
        let scene = SceneDeserializer {
            type_registry: &type_registry.read(),
        }
        .deserialize(&mut deserializer)
        .unwrap();

        assert_eq!(scene.entities.len(), 1);
        let entity = &scene.entities[0];
        let node_ref = entity
            .components
            .iter()
            .find_map(|c| c.try_downcast_ref::<GltfNodeRef>())
            .expect("GltfNodeRef component not found in saved scene");

        assert_eq!(node_ref.path, path);
        assert_eq!(node_ref.index, index);
    }
}
