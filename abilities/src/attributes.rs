use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MovementSpeed(pub f32);

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn apply_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).clamp(0.0, self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

pub fn lerp_health(start: Health, end: Health, t: f32) -> Health {
    Health {
        current: start.current + (end.current - start.current) * t,
        max: end.max,
    }
}
