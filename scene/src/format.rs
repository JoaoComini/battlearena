use avian2d::prelude::{ColliderConstructor, RigidBody};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::GltfPrimitiveRef;

pub const SCENE_FORMAT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneFile {
    pub version: u32,
    pub entities: Vec<SceneEntity>,
}

impl SceneFile {
    pub fn new(entities: Vec<SceneEntity>) -> Self {
        Self {
            version: SCENE_FORMAT_VERSION,
            entities,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneEntity {
    pub name: Option<String>,
    pub transform: SceneTransform,
    pub mesh: Option<GltfPrimitiveRef>,
    pub collider: Option<ColliderConstructor>,
    pub rigid_body: Option<RigidBody>,
    pub children: Vec<SceneEntity>,
}

impl SceneEntity {
    pub fn new(transform: Transform) -> Self {
        Self {
            name: None,
            transform: transform.into(),
            mesh: None,
            collider: None,
            rigid_body: None,
            children: vec![],
        }
    }
}

/// A serde-friendly representation of `bevy::prelude::Transform`.
///
/// `Transform` itself derives `Serialize`/`Deserialize` via Bevy's `Reflect`
/// pipeline, but its serde impls are not publicly available without pulling in
/// the full reflection machinery. This thin wrapper keeps the scene format
/// independent of Bevy internals.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SceneTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl From<Transform> for SceneTransform {
    fn from(t: Transform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl From<SceneTransform> for Transform {
    fn from(t: SceneTransform) -> Self {
        Transform {
            translation: t.translation.into(),
            rotation: Quat::from_array(t.rotation),
            scale: t.scale.into(),
        }
    }
}
