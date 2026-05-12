use avian2d::prelude::{Collider, CollisionLayers, LayerMask, Position, Rotation};
use physics::GameLayer;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::attributes::{Attribute, EffectEvent, Modifier};

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

#[derive(Component, Reflect, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct AbilityCooldowns {
    pub remaining: Vec<f32>,
}

impl AbilityCooldowns {
    pub fn new(count: usize) -> Self {
        Self {
            remaining: vec![0.0; count],
        }
    }

    pub fn is_ready(&self, slot: usize) -> bool {
        self.remaining.get(slot).map_or(false, |&r| r <= 0.0)
    }
}

pub trait Ability: Send + Sync + 'static {
    fn cast(&self, instance: Entity, inst: &AbilityInstance, commands: &mut Commands);
}

#[derive(Asset, Reflect, Serialize, Deserialize, Clone, Debug)]
pub struct AbilityDef {
    pub cooldown_secs: f32,
    pub ability: crate::abilities::AbilityKind,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct AbilityInstance {
    pub caster: Entity,
    pub slot: usize,
    pub origin: Vec2,
    pub facing_rad: f32,
    pub cooldown_secs: f32,
}

pub trait CommandsAbilityExt {
    fn apply_effect<A: Attribute>(&mut self, target: Entity, value: f32, modifier: Modifier);

    fn cast_ability(
        &mut self,
        instance: Entity,
        secs: f32,
        on_done: impl FnOnce(&mut Commands) + Send + Sync + 'static,
    );

    fn end_ability(&mut self, instance: Entity);

    fn activate_ability(
        &mut self,
        instance: Entity,
        caster: Entity,
        slot: usize,
        cooldown_secs: f32,
    );

    fn melee_hit(
        &mut self,
        instance: Entity,
        range: f32,
        angle_deg: f32,
        on_hit: impl Fn(Entity, &mut Commands) + Send + Sync + 'static,
        on_end: impl FnOnce(&mut Commands) + Send + Sync + 'static,
    );

    fn projectile(
        &mut self,
        instance: Entity,
        caster: Entity,
        origin: Vec2,
        facing_rad: f32,
        speed: f32,
        size: f32,
        max_range: f32,
        on_hit: impl Fn(Entity, LayerMask, &mut Commands) + Send + Sync + 'static,
    );
}

impl CommandsAbilityExt for Commands<'_, '_> {
    fn apply_effect<A: Attribute>(&mut self, target: Entity, value: f32, modifier: Modifier) {
        self.write_message(EffectEvent::<A>::new(target, value, modifier));
    }

    fn cast_ability(
        &mut self,
        instance: Entity,
        secs: f32,
        on_done: impl FnOnce(&mut Commands) + Send + Sync + 'static,
    ) {
        let task = self
            .spawn(CastingTask {
                remaining_secs: secs,
                on_done: Some(Box::new(on_done)),
            })
            .id();
        self.entity(instance).add_child(task);
    }

    fn end_ability(&mut self, instance: Entity) {
        self.entity(instance).despawn();
    }

    fn activate_ability(
        &mut self,
        instance: Entity,
        caster: Entity,
        slot: usize,
        cooldown_secs: f32,
    ) {
        self.queue(move |world: &mut World| {
            if let Some(mut cooldowns) = world.get_mut::<AbilityCooldowns>(caster) {
                if let Some(r) = cooldowns.remaining.get_mut(slot) {
                    *r = cooldown_secs;
                }
            }
        });
        self.entity(instance).insert(Active);
    }

    fn melee_hit(
        &mut self,
        instance: Entity,
        range: f32,
        angle_deg: f32,
        on_hit: impl Fn(Entity, &mut Commands) + Send + Sync + 'static,
        on_end: impl FnOnce(&mut Commands) + Send + Sync + 'static,
    ) {
        let task = self
            .spawn(MeleeHitTask {
                range,
                angle_deg,
                on_hit: Box::new(on_hit),
                on_end: Some(Box::new(on_end)),
            })
            .id();
        self.entity(instance).add_child(task);
    }

    fn projectile(
        &mut self,
        instance: Entity,
        caster: Entity,
        origin: Vec2,
        facing_rad: f32,
        speed: f32,
        size: f32,
        max_range: f32,
        on_hit: impl Fn(Entity, LayerMask, &mut Commands) + Send + Sync + 'static,
    ) {
        let direction = Vec2::from_angle(facing_rad + std::f32::consts::FRAC_PI_2);
        self.spawn((
            ProjectileTask {
                instance,
                caster,
                speed,
                size,
                max_range,
                distance_traveled: 0.0,
                direction,
                already_hit: Vec::new(),
                on_hit: Box::new(on_hit),
            },
            Position(origin),
            Rotation::radians(facing_rad),
            Collider::circle(size),
            CollisionLayers::new(GameLayer::Projectile, !LayerMask::from(GameLayer::Projectile)),
        ));
    }
}

#[derive(Component)]
pub struct CastingTask {
    pub remaining_secs: f32,
    pub on_done: Option<Box<dyn FnOnce(&mut Commands) + Send + Sync>>,
}

#[derive(Component)]
pub struct MeleeHitTask {
    pub range: f32,
    pub angle_deg: f32,
    pub on_hit: Box<dyn Fn(Entity, &mut Commands) + Send + Sync>,
    pub on_end: Option<Box<dyn FnOnce(&mut Commands) + Send + Sync>>,
}

#[derive(Component)]
pub struct ProjectileTask {
    pub instance: Entity,
    pub caster: Entity,
    pub speed: f32,
    pub size: f32,
    pub max_range: f32,
    pub distance_traveled: f32,
    pub direction: Vec2,
    pub already_hit: Vec<Entity>,
    pub on_hit: Box<dyn Fn(Entity, LayerMask, &mut Commands) + Send + Sync>,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Active;

#[derive(Component, Debug)]
pub struct HitMarker {
    pub timer: Timer,
}

impl HitMarker {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.4, TimerMode::Once),
        }
    }
}
