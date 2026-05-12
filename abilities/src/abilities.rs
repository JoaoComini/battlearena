use crate::attributes::{Health, Modifier};
use crate::types::{Ability, AbilityInstance, CommandsAbilityExt};
use bevy::prelude::{Commands, Entity};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum AbilityKind {
    Melee(Melee),
    Projectile(Projectile),
}

impl AbilityKind {
    pub fn cast(&self, instance: Entity, inst: &AbilityInstance, commands: &mut Commands) {
        match self {
            AbilityKind::Melee(a) => a.cast(instance, inst, commands),
            AbilityKind::Projectile(a) => a.cast(instance, inst, commands),
        }
    }
}

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Melee {
    pub cast_secs: f32,
    pub range: f32,
    pub angle_deg: f32,
    pub damage: f32,
}

impl Ability for Melee {
    fn cast(&self, instance: Entity, inst: &AbilityInstance, commands: &mut Commands) {
        let inst = *inst;
        let ability = *self;
        commands.cast_ability(instance, self.cast_secs, move |commands| {
            commands.activate_ability(instance, inst.caster, inst.slot, inst.cooldown_secs);
            commands.melee_hit(
                instance,
                ability.range,
                ability.angle_deg,
                move |hit, commands| {
                    commands.apply_effect::<Health>(hit, -ability.damage, Modifier::Add);
                },
                move |commands| {
                    commands.end_ability(instance);
                },
            );
        });
    }
}

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Projectile {
    pub speed: f32,
    pub size: f32,
    pub max_range: f32,
    pub damage: f32,
}

impl Ability for Projectile {
    fn cast(&self, instance: Entity, inst: &AbilityInstance, commands: &mut Commands) {
        let inst = *inst;
        let ability = *self;
        commands.activate_ability(instance, inst.caster, inst.slot, inst.cooldown_secs);
        commands.projectile(
            instance,
            inst.caster,
            inst.origin,
            inst.facing_rad,
            ability.speed,
            ability.size,
            ability.max_range,
            move |hit, commands| {
                commands.apply_effect::<Health>(hit, -ability.damage, Modifier::Add);
                commands.end_ability(instance);
            },
        );
    }
}
