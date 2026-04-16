use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Reference to a node within a GLTF file, identified by index.
///
/// Stored as plain data so it can be serialized via Bevy's DynamicScene /
/// reflection pipeline without involving asset handles. The `ScenePlugin`
/// system `resolve_gltf_node_refs` converts this into mesh/material child
/// entities at runtime using `GltfAssetLabel::Node(index)`.
///
/// The editor is responsible for mapping node names to indices when
/// constructing scene entities.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Component, Serialize, Deserialize)]
pub struct GltfNodeRef {
    /// Asset path to the GLTF file, e.g. `"models/arena.glb"`.
    pub path: String,
    /// Zero-based node index within the GLTF.
    pub index: usize,
}
