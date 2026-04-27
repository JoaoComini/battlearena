use avian2d::prelude::{Collider, Position, Rotation, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use protocol::Health;
use crate::types::{AbilityCast, AbilityDef, AbilityEffect, AbilityLoadout, HitboxCaster, MeleeHitbox, ProjectileHitbox};

pub fn tick_cooldowns(mut query: Query<&mut AbilityLoadout>, time: Res<Time>) {
    for mut loadout in &mut query {
        for slot in &mut loadout.slots {
            slot.cooldown_remaining =
                (slot.cooldown_remaining - time.delta_secs()).max(0.0);
        }
    }
}

pub fn pie_slice_collider(range: f32, angle_deg: f32, facing_rad: f32) -> Option<Collider> {
    let half = (angle_deg / 2.0).to_radians();
    let steps = 8usize;
    let mut points = vec![Vec2::ZERO];
    let adjusted = facing_rad + std::f32::consts::FRAC_PI_2;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = adjusted - half + t * 2.0 * half;
        points.push(Vec2::from_angle(a) * range);
    }
    Collider::convex_hull(points)
}

/// Spawns a hitbox entity for the given cast. Used by both server (authoritative) and client (local simulation).
pub fn spawn_hitbox(caster: Entity, cast: &AbilityCast, def: &AbilityDef, commands: &mut Commands) {
    let origin = cast.origin;
    let facing_rad = cast.facing_rad;

    match def.effect {
        AbilityEffect::MeleeHit { range, angle_deg, damage, lifetime_frames } => {
            let Some(collider) = pie_slice_collider(range, angle_deg, facing_rad) else {
                return;
            };
            commands.spawn((
                HitboxCaster(caster),
                MeleeHitbox {
                    caster,
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
                HitboxCaster(caster),
                ProjectileHitbox {
                    caster,
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

pub fn move_projectiles(
    mut query: Query<(Entity, &mut ProjectileHitbox, &mut Position)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut proj, mut position) in &mut query {
        let delta = proj.direction * proj.speed * time.delta_secs();
        position.0 += delta;
        proj.distance_traveled += delta.length();
        if proj.distance_traveled >= proj.max_range {
            commands.entity(entity).despawn();
        }
    }
}

pub fn apply_hitbox_damage(
    mut hitboxes: Query<(Entity, &mut MeleeHitbox, &Collider)>,
    spatial_query: SpatialQuery,
    mut health_query: Query<&mut Health>,
    mut commands: Commands,
) {
    for (entity, mut hitbox, collider) in &mut hitboxes {
        let filter = SpatialQueryFilter::from_excluded_entities([hitbox.caster]);

        let hits = spatial_query.shape_intersections(
            &collider,
            hitbox.origin,
            0.0,
            &filter,
        );

        for hit in hits {
            if !hitbox.already_hit.contains(&hit) {
                if let Ok(mut health) = health_query.get_mut(hit) {
                    health.current -= hitbox.damage;
                }
                hitbox.already_hit.push(hit);
            }
        }

        hitbox.lifetime_frames = hitbox.lifetime_frames.saturating_sub(1);
        if hitbox.lifetime_frames == 0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn apply_projectile_damage(
    mut projectiles: Query<(Entity, &mut ProjectileHitbox, &Collider, &Position)>,
    spatial_query: SpatialQuery,
    mut health_query: Query<&mut Health>,
    mut commands: Commands,
) {
    for (entity, mut proj, collider, position) in &mut projectiles {
        let filter = SpatialQueryFilter::from_excluded_entities([proj.caster, entity]);

        let hits = spatial_query.shape_intersections(
            collider,
            position.0,
            0.0,
            &filter,
        );

        for hit in hits {
            if !proj.already_hit.contains(&hit) {
                if let Ok(mut health) = health_query.get_mut(hit) {
                    health.current -= proj.damage;
                }
                proj.already_hit.push(hit);
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}
