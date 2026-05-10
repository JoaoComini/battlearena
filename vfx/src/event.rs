use bevy::prelude::*;

pub struct VfxContext {
    pub position: Vec2,
    pub entity: Entity,
    pub cast_duration: Option<f32>,
}
