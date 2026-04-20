use bevy::prelude::*;
use physics::debug::DebugGizmoHitbox;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct AbilityKey(pub String);

impl AbilityKey {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Reflect, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AbilitySlot {
    pub key: AbilityKey,
    pub cooldown_remaining: f32,
}

impl AbilitySlot {
    pub fn new(key: &str) -> Self {
        Self {
            key: AbilityKey::new(key),
            cooldown_remaining: 0.0,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown_remaining <= 0.0
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct AbilityLoadout {
    pub slots: Vec<AbilitySlot>,
}

#[derive(Asset, Reflect, Serialize, Deserialize, Clone, Debug)]
pub struct AbilityDef {
    pub key: String,
    pub cooldown_secs: f32,
    pub effect: AbilityEffect,
}

#[derive(Reflect, Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum AbilityEffect {
    MeleeHit {
        range: f32,
        angle_deg: f32,
        damage: f32,
        lifetime_frames: u32,
    },
    Projectile {
        speed: f32,
        size: f32,
        damage: f32,
        max_range: f32,
    },
}

impl AbilityEffect {
    pub fn to_debug_gizmo(&self) -> DebugGizmoHitbox {
        match self {
            AbilityEffect::MeleeHit { range, angle_deg, .. } => {
                DebugGizmoHitbox::PieSlice { range: *range, angle_deg: *angle_deg }
            }
            AbilityEffect::Projectile { size, .. } => {
                DebugGizmoHitbox::Circle { radius: *size }
            }
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct HitboxCaster(pub Entity);

#[derive(Component, Debug)]
pub struct MeleeHitbox {
    pub caster: Entity,
    pub damage: f32,
    pub range: f32,
    pub angle_deg: f32,
    pub lifetime_frames: u32,
    /// World-space origin of the hitbox (caster position at spawn).
    pub origin: Vec2,
    /// Facing angle in radians at spawn time.
    pub facing_rad: f32,
    pub already_hit: Vec<Entity>,
}

#[derive(Component, Debug)]
pub struct ProjectileHitbox {
    pub caster: Entity,
    pub damage: f32,
    pub speed: f32,
    pub size: f32,
    pub max_range: f32,
    pub distance_traveled: f32,
    pub direction: Vec2,
    pub already_hit: Vec<Entity>,
}
