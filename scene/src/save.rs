use bevy::camera::visibility::VisibilityClass;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use bevy::scene::DynamicSceneBuilder;
use std::path::Path;

/// Saves entities matching `F` to a `.scn.ron` file.
///
/// Uses Bevy's `DynamicScene` serialization — all components on matching
/// entities that are registered in the `AppTypeRegistry` will be included.
/// Runtime-only components (`GlobalTransform`, `TransformTreeChanged`) are always
/// excluded from the output.
pub fn save_scene<F: QueryFilter>(
    world: &mut World,
    path: impl AsRef<Path>,
) -> Result<(), SaveSceneError> {
    let mut query = world.query_filtered::<Entity, F>();
    let saved: std::collections::HashSet<Entity> = query.iter(world).collect();

    let scene = DynamicSceneBuilder::from_world(world)
        .deny_component::<GlobalTransform>()
        .deny_component::<TransformTreeChanged>()
        .deny_component::<Visibility>()
        .deny_component::<InheritedVisibility>()
        .deny_component::<ViewVisibility>()
        .deny_component::<Mesh3d>()
        .deny_component::<MeshMaterial3d<StandardMaterial>>()
        .deny_component::<VisibilityClass>()
        .extract_entities(saved.into_iter())
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
    use crate::GltfPrimitiveRef;
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

    fn make_primitive_ref() -> GltfPrimitiveRef {
        GltfPrimitiveRef {
            path: "models/arena.glb".to_string(),
            mesh_index: 0,
            primitive_index: 0,
        }
    }

    #[test]
    fn roundtrip_gltf_primitive_ref() {
        let mut app = make_app();

        let prim_ref = GltfPrimitiveRef {
            path: "models/arena.glb".to_string(),
            mesh_index: 2,
            primitive_index: 1,
        };

        app.world_mut().spawn(prim_ref.clone());

        let tmp = tempfile::NamedTempFile::new().unwrap();
        save_scene::<With<GltfPrimitiveRef>>(app.world_mut(), tmp.path()).unwrap();

        let ron = std::fs::read_to_string(tmp.path()).unwrap();
        let scene = deserialize_scene(&ron, app.world());

        assert_eq!(scene.entities.len(), 1);
        let saved = find_component::<GltfPrimitiveRef>(&scene)
            .expect("GltfPrimitiveRef not found in saved scene");

        assert_eq!(saved.path, prim_ref.path);
        assert_eq!(saved.mesh_index, prim_ref.mesh_index);
        assert_eq!(saved.primitive_index, prim_ref.primitive_index);
    }

    #[test]
    fn roundtrip_with_transform() {
        let mut app = make_app();

        let translation = Vec3::new(1.0, 2.0, 3.0);

        app.world_mut().spawn((
            make_primitive_ref(),
            Transform::from_translation(translation),
        ));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        save_scene::<With<GltfPrimitiveRef>>(app.world_mut(), tmp.path()).unwrap();

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

        app.world_mut()
            .spawn((make_primitive_ref(), ColliderConstructor::Circle { radius }));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        save_scene::<With<GltfPrimitiveRef>>(app.world_mut(), tmp.path()).unwrap();

        let ron = std::fs::read_to_string(tmp.path()).unwrap();
        let scene = deserialize_scene(&ron, app.world());

        let constructor = find_component::<ColliderConstructor>(&scene)
            .expect("ColliderConstructor not found in saved scene");

        assert_eq!(*constructor, ColliderConstructor::Circle { radius });
    }
}
