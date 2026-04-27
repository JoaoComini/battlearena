use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Asset path to a mesh within a GLTF file (e.g. `"assets/models/arena.glb#Mesh0/Primitive0"`).
/// Stored as plain data so it round-trips through Bevy's DynamicScene pipeline.
/// `resolve_mesh_refs` converts this into a `Mesh3d` handle at runtime.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MeshRef(pub String);

/// Asset path to a material within a GLTF file (e.g. `"assets/models/arena.glb#Material0"`).
/// `resolve_material_refs` converts this into a `MeshMaterial3d` handle at runtime.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MaterialRef(pub String);
