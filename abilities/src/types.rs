use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct AbilitySlot {
    pub handle: Handle<AbilityDef>,
}

impl AbilitySlot {
    pub fn new(handle: Handle<AbilityDef>) -> Self {
        Self { handle }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct AbilityLoadout {
    pub slots: Vec<AbilitySlot>,
}

/// Replicated cooldown state — one entry per slot, in order.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct AbilityCooldowns {
    pub remaining: Vec<f32>,
}

impl AbilityCooldowns {
    pub fn new(count: usize) -> Self {
        Self { remaining: vec![0.0; count] }
    }

    pub fn is_ready(&self, slot: usize) -> bool {
        self.remaining.get(slot).map_or(false, |&r| r <= 0.0)
    }
}

#[derive(Reflect, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum AbilityEvent {
    Cast { secs: f32 },
    Activate,
    MeleeHit { range: f32, angle_deg: f32, damage: f32 },
    Projectile { speed: f32, size: f32, damage: f32, max_range: f32 },
}

#[derive(Asset, Reflect, Serialize, Deserialize, Clone, Debug)]
pub struct AbilityDef {
    pub key: String,
    pub cooldown_secs: f32,
    pub events: Vec<AbilityEvent>,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AbilityInstance {
    pub caster: Entity,
    pub slot: usize,
    pub origin: Vec2,
    pub facing_rad: f32,
    pub cursor: usize,
}

/// Server-internal: inserted to advance the instance cursor and insert the next request.
#[derive(Component)]
pub(crate) struct Advance;

#[derive(Component)]
pub(crate) struct MeleeHitRequest {
    pub range: f32,
    pub angle_deg: f32,
    pub damage: f32,
}

#[derive(Component)]
pub(crate) struct ProjectileRequest {
    pub speed: f32,
    pub size: f32,
    pub damage: f32,
    pub max_range: f32,
}

/// Replicated: instance is in its cast-time phase.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Casting {
    pub remaining_secs: f32,
}

/// Replicated: the instance has fired at least once (Activate processed). Never removed.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Active;

/// The instance has fully resolved and should be despawned.
#[derive(Component)]
pub struct Ended;

#[derive(Component)]
pub struct TakeDamage(pub f32);

#[derive(Component, Debug)]
pub struct HitMarker {
    pub timer: Timer,
}

impl HitMarker {
    pub fn new() -> Self {
        Self { timer: Timer::from_seconds(0.4, TimerMode::Once) }
    }
}

#[derive(Component, Debug)]
pub struct ProjectileHitbox {
    pub instance: Entity,
    pub caster: Entity,
    pub damage: f32,
    pub speed: f32,
    pub size: f32,
    pub max_range: f32,
    pub distance_traveled: f32,
    pub direction: Vec2,
    pub already_hit: Vec<Entity>,
}
