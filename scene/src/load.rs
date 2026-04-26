use bevy::ecs::reflect::ReflectCommandExt;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use serde::de::DeserializeSeed;

use crate::components::{LoadScene, OverrideScene, PendingOverrides, SceneSourcePath};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, initiate_load);
        app.add_observer(apply_overrides);
    }
}

fn initiate_load(
    mut commands: Commands,
    query: Query<(Entity, &LoadScene)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, load) in &query {
        let contents = match std::fs::read_to_string(&load.0) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to read scene file '{}': {e}", load.0);
                commands.entity(entity).remove::<LoadScene>();
                continue;
            }
        };

        let scene: OverrideScene = match ron::from_str(&contents) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to parse scene file '{}': {e}", load.0);
                commands.entity(entity).remove::<LoadScene>();
                continue;
            }
        };

        let handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset(scene.source.clone()));

        commands
            .entity(entity)
            .insert((
                SceneRoot(handle),
                PendingOverrides(scene.overrides),
                SceneSourcePath(scene.source),
            ))
            .remove::<LoadScene>();
    }
}

fn apply_overrides(
    trigger: On<SceneInstanceReady>,
    pending_query: Query<&PendingOverrides>,
    names: Query<&Name>,
    scene_spawner: Res<SceneSpawner>,
    type_registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    let root = trigger.entity;

    let Ok(pending) = pending_query.get(root) else {
        return;
    };
    let pending = pending.0.clone();

    if pending.is_empty() {
        commands.entity(root).remove::<PendingOverrides>();
        return;
    }

    let registry = type_registry.read();

    let entities: Vec<(Entity, String)> = scene_spawner
        .iter_instance_entities(trigger.instance_id)
        .filter_map(|e| names.get(e).ok().map(|n| (e, n.as_str().to_string())))
        .collect();

    for (entity, name) in entities {
        let Some(overrides) = pending.get(&name) else {
            continue;
        };

        for (type_path, value) in overrides {
            let Some(registration) = registry.get_with_type_path(type_path) else {
                warn!("Type '{type_path}' not found in registry");
                continue;
            };

            let wrapped = ron::Value::Map(
                [(ron::Value::String(type_path.clone()), value.clone())]
                    .into_iter()
                    .collect(),
            );
            let ron_str = match ron::to_string(&wrapped) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to serialize '{type_path}': {e}");
                    continue;
                }
            };

            let reflect_deserializer =
                bevy::reflect::serde::TypedReflectDeserializer::new(registration, &registry);
            let mut deserializer = ron::Deserializer::from_str(&ron_str).unwrap();
            let reflected: Box<dyn bevy::reflect::PartialReflect> =
                match reflect_deserializer.deserialize(&mut deserializer) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("Failed to deserialize '{type_path}': {e}");
                        continue;
                    }
                };

            commands.entity(entity).insert_reflect(reflected);
        }
    }

    commands.entity(root).remove::<PendingOverrides>();
}
