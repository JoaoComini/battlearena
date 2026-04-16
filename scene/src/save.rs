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
    use avian2d::prelude::*;
    use bevy::scene::serde::SceneDeserializer;
    use serde::de::DeserializeSeed;

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ScenePlugin));
        app
    }

    fn deserialize_scene(ron: &str, world: &World) -> bevy::scene::DynamicScene {
        let type_registry = world.resource::<AppTypeRegistry>();
        let mut deserializer = ron::de::Deserializer::from_str(ron).unwrap();
        SceneDeserializer {
            type_registry: &type_registry.read(),
        }
        .deserialize(&mut deserializer)
        .unwrap()
    }

    fn find_component<T: 'static>(scene: &bevy::scene::DynamicScene) -> Option<&T> {
        scene.entities[0]
            .components
            .iter()
            .find_map(|c| c.try_downcast_ref::<T>())
    }

    #[test]
    fn roundtrip_gltf_node_ref() {
        let mut app = make_app();

        let path = "models/arena.glb";
        let index = 3;

        app.world_mut().spawn(GltfNodeRef {
            path: path.to_string(),
            index,
        });

        let tmp = tempfile::NamedTempFile::new().unwrap();
        save_scene(app.world_mut(), tmp.path()).unwrap();

        let ron = std::fs::read_to_string(tmp.path()).unwrap();
        let scene = deserialize_scene(&ron, app.world());

        assert_eq!(scene.entities.len(), 1);
        let node_ref =
            find_component::<GltfNodeRef>(&scene).expect("GltfNodeRef not found in saved scene");

        assert_eq!(node_ref.path, path);
        assert_eq!(node_ref.index, index);
    }

    #[test]
    fn roundtrip_with_transform() {
        let mut app = make_app();

        let translation = Vec3::new(1.0, 2.0, 3.0);

        app.world_mut().spawn((
            GltfNodeRef {
                path: "models/arena.glb".to_string(),
                index: 0,
            },
            Transform::from_translation(translation),
        ));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        save_scene(app.world_mut(), tmp.path()).unwrap();

        let ron = std::fs::read_to_string(tmp.path()).unwrap();
        let scene = deserialize_scene(&ron, app.world());

        let transform =
            find_component::<Transform>(&scene).expect("Transform not found in saved scene");

        assert_eq!(transform.translation, translation);
    }

    #[test]
    fn roundtrip_with_collider_constructor() {
        let mut app = make_app();

        let radius = 30.0_f32;

        app.world_mut().spawn((
            GltfNodeRef {
                path: "models/arena.glb".to_string(),
                index: 1,
            },
            ColliderConstructor::Circle { radius },
        ));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        save_scene(app.world_mut(), tmp.path()).unwrap();

        let ron = std::fs::read_to_string(tmp.path()).unwrap();
        let scene = deserialize_scene(&ron, app.world());

        let constructor = find_component::<ColliderConstructor>(&scene)
            .expect("ColliderConstructor not found in saved scene");

        assert_eq!(*constructor, ColliderConstructor::Circle { radius });
    }
}
