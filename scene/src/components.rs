use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Asset path to a mesh, e.g. `"models/arena.glb#Mesh0/Primitive0"`.
/// Resolved at runtime into a `Mesh3d` component by `ImportPlugin`.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MeshPath(pub String);

/// Asset path to a standard material, e.g. `"models/arena.glb#Material0"`.
/// Resolved at runtime into a `MeshMaterial3d<StandardMaterial>` by `ImportPlugin`.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MaterialPath(pub String);

/// Place this component on an entity to load a `.scn` dynamic scene as children.
#[derive(Component)]
pub struct LoadScene(pub String);
