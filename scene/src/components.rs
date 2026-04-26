use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

/// Scene format: source GLB path + per-node component overrides.
/// Outer key: node name. Inner key: full type path. Value: component data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideScene {
    pub source: String,
    pub overrides: HashMap<String, HashMap<String, ron::Value>>,
}


/// Place this component on an entity to load an override-based `.scn` file.
#[derive(Component)]
pub struct LoadScene(pub String);

/// Holds pending overrides to be applied once `SceneInstanceReady` fires.
#[derive(Component)]
pub struct PendingOverrides(pub HashMap<String, HashMap<String, ron::Value>>);

/// Tracks the source GLB asset path for a loaded scene root.
#[derive(Component)]
pub struct SceneSourcePath(pub String);
