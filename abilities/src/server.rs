use avian2d::prelude::{Collider, Position, Rotation};
use bevy::prelude::*;
use lightyear::prelude::*;
use protocol::UseAbility;
use crate::registry::AbilityRegistry;
use crate::systems::{
    apply_hitbox_damage, apply_projectile_damage, move_projectiles,
    pie_slice_collider, tick_cooldowns,
};
use crate::types::{
    AbilityDef, AbilityEffect, AbilityLoadout, HitboxCaster, MeleeHitbox, ProjectileHitbox,
};

pub struct AbilityServerPlugin;

impl Plugin for AbilityServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ability_hitbox_spawned);
        app.add_systems(
            FixedUpdate,
            (
                tick_cooldowns,
                receive_ability_requests,
                move_projectiles,
                apply_hitbox_damage,
                apply_projectile_damage,
            )
                .chain(),
        );
    }
}

fn on_ability_hitbox_spawned(
    trigger: On<Add, AbilityEffect>,
    hitboxes: Query<(&AbilityEffect, &HitboxCaster)>,
    controlled_by: Query<&ControlledBy>,
    mut commands: Commands,
) {
    let Ok((effect, caster)) = hitboxes.get(trigger.entity) else { return };
    let Ok(ctrl) = controlled_by.get(caster.0) else { return };

    commands
        .entity(trigger.entity)
        .insert((
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
            effect.to_debug_gizmo(),
        ));
}

fn receive_ability_requests(
    mut receivers: Query<(Entity, &mut MessageReceiver<UseAbility>)>,
    mut players: Query<(Entity, &ControlledBy, &Position, &Rotation, &mut AbilityLoadout)>,
    registry: Res<AbilityRegistry>,
    assets: Res<Assets<AbilityDef>>,
    mut commands: Commands,
) {
    let pending: Vec<(Entity, Vec<UseAbility>)> = receivers
        .iter_mut()
        .filter_map(|(conn_entity, mut receiver)| {
            let msgs: Vec<UseAbility> = receiver.receive().collect();
            if msgs.is_empty() { None } else { Some((conn_entity, msgs)) }
        })
        .collect();

    for (conn_entity, msgs) in pending {
        let Some((player_entity, _, position, rotation, mut loadout)) =
            players.iter_mut().find(|(_, ctrl, _, _, _)| ctrl.owner == conn_entity)
        else {
            continue;
        };

        let origin = position.0;
        let facing_rad = rotation.as_radians();

        for msg in msgs {
            let Some(slot) = loadout.slots.get_mut(msg.slot) else { continue };
            if !slot.is_ready() { continue; }

            let Some(def) = registry.get(&slot.key, &assets) else { continue };
            slot.cooldown_remaining = def.cooldown_secs;

            match def.effect {
                AbilityEffect::MeleeHit { range, angle_deg, damage, lifetime_frames } => {
                    let Some(collider) = pie_slice_collider(range, angle_deg, facing_rad) else {
                        continue;
                    };
                    commands.spawn((
                        HitboxCaster(player_entity),
                        MeleeHitbox {
                            caster: player_entity,
                            damage,
                            range,
                            angle_deg,
                            lifetime_frames,
                            origin,
                            facing_rad,
                            already_hit: Vec::new(),
                        },
                        def.effect.clone(),
                        Position(origin),
                        Rotation::radians(facing_rad),
                        collider,
                    ));
                }
                AbilityEffect::Projectile { speed, size, damage, max_range } => {
                    let direction = Vec2::from_angle(facing_rad + std::f32::consts::FRAC_PI_2);
                    commands.spawn((
                        HitboxCaster(player_entity),
                        ProjectileHitbox {
                            caster: player_entity,
                            damage,
                            speed,
                            size,
                            max_range,
                            distance_traveled: 0.0,
                            direction,
                            already_hit: Vec::new(),
                        },
                        def.effect.clone(),
                        Position(origin),
                        Rotation::radians(facing_rad),
                        Collider::circle(size),
                    ));
                }
            }
        }
    }
}
