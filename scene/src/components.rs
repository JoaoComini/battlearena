use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Reference to a primitive within a GLTF mesh, identified by GLTF mesh index
/// and primitive index within that mesh.
///
/// Stored as plain data so it can be serialized via Bevy's DynamicScene /
/// reflection pipeline without involving asset handles. The `ScenePlugin`
/// system `resolve_gltf_primitive_refs` converts this into `Mesh3d` and
/// `MeshMaterial3d` components on the same entity at runtime.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Component, Serialize, Deserialize)]
pub struct GltfPrimitiveRef {
    /// Asset path to the GLTF file, e.g. `"models/arena.glb"`.
    pub path: String,
    /// Zero-based mesh index within the GLTF.
    pub mesh_index: usize,
    /// Zero-based primitive index within the mesh.
    pub primitive_index: usize,
}
