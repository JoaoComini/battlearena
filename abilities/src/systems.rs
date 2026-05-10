use crate::types::{
    AbilityCooldowns, AbilityDef, AbilityEvent, AbilityInstance, AbilityLoadout, Active, Advance,
    Casting, Ended, MeleeHitRequest, ProjectileHitbox, ProjectileRequest, TakeDamage,
};
use avian2d::prelude::{Collider, Position, Rotation, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use physics::pie_slice_collider;
use protocol::VfxTag;
use crate::attributes::Health;

pub(crate) fn advance_instance(
    trigger: On<Add, Advance>,
    mut instances: Query<&mut AbilityInstance>,
    mut casters: Query<(&AbilityLoadout, &mut AbilityCooldowns)>,
    assets: Res<Assets<AbilityDef>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    commands.entity(entity).remove::<Advance>();

    let Ok(mut instance) = instances.get_mut(entity) else { return };
    let caster = instance.caster;
    let slot_idx = instance.slot;

    let Ok((loadout, mut cooldowns)) = casters.get_mut(caster) else { return };
    let Some(handle) = loadout.slots.get(slot_idx).map(|s| s.handle.clone()) else { return };
    let Some(def) = assets.get(&handle) else { return };
    let def = def.clone();

    let Some(event) = def.events.get(instance.cursor) else {
        commands.entity(entity).insert(Ended);
        return;
    };

    match event {
        AbilityEvent::Cast { secs, vfx } => {
            if let Some(tag) = vfx {
                commands.entity(entity).insert((Casting { remaining_secs: *secs }, VfxTag(tag.clone())));
            } else {
                commands.entity(entity).insert(Casting { remaining_secs: *secs });
            }
        }
        AbilityEvent::Activate { vfx } => {
            if let Some(remaining) = cooldowns.remaining.get_mut(slot_idx) {
                *remaining = def.cooldown_secs;
            }
            if let Some(tag) = vfx {
                commands.entity(entity).insert((Active, Advance, VfxTag(tag.clone())));
            } else {
                commands.entity(entity).insert((Active, Advance));
            }
        }
        AbilityEvent::MeleeHit { range, angle_deg, damage, .. } => {
            commands.entity(entity).insert(MeleeHitRequest {
                range: *range,
                angle_deg: *angle_deg,
                damage: *damage,
            });
        }
        AbilityEvent::Projectile { speed, size, damage, max_range, trail_vfx, hit_vfx } => {
            commands.entity(entity).insert(ProjectileRequest {
                speed: *speed,
                size: *size,
                damage: *damage,
                max_range: *max_range,
                trail_vfx: trail_vfx.clone(),
                hit_vfx: hit_vfx.clone(),
            });
        }
    }
    instance.cursor += 1;
}

pub fn tick_casting(
    mut instances: Query<(Entity, &mut Casting)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut casting) in &mut instances {
        casting.remaining_secs -= time.delta_secs();
        if casting.remaining_secs <= 0.0 {
            commands.entity(entity).remove::<Casting>().insert(Advance);
        }
    }
}

pub(crate) fn process_melee_hit_request(
    instances: Query<(Entity, &MeleeHitRequest, &AbilityInstance)>,
    spatial_query: SpatialQuery,
    mut commands: Commands,
) {
    for (entity, req, instance) in &instances {
        if let Some(collider) = pie_slice_collider(req.range, req.angle_deg, instance.facing_rad) {
            let filter = SpatialQueryFilter::from_excluded_entities([instance.caster]);
            let hits =
                spatial_query.shape_intersections(&collider, instance.origin, 0.0, &filter);
            for hit in hits {
                commands.entity(hit).insert(TakeDamage(req.damage));
            }
        }
        commands.entity(entity).remove::<MeleeHitRequest>().insert(Advance);
    }
}

pub(crate) fn process_projectile_request(
    instances: Query<(Entity, &ProjectileRequest, &AbilityInstance)>,
    mut commands: Commands,
) {
    for (entity, req, instance) in &instances {
        let direction = Vec2::from_angle(instance.facing_rad + std::f32::consts::FRAC_PI_2);
        let mut projectile = commands.spawn((
            ProjectileHitbox {
                instance: entity,
                caster: instance.caster,
                damage: req.damage,
                speed: req.speed,
                size: req.size,
                max_range: req.max_range,
                distance_traveled: 0.0,
                direction,
                already_hit: Vec::new(),
                hit_vfx: req.hit_vfx.clone(),
            },
            Position(instance.origin),
            Rotation::radians(instance.facing_rad),
            Collider::circle(req.size),
        ));
        if let Some(tag) = &req.trail_vfx {
            projectile.insert(VfxTag(tag.clone()));
        }
        commands.entity(entity).remove::<ProjectileRequest>();
    }
}

pub fn despawn_ended(instances: Query<Entity, Added<Ended>>, mut commands: Commands) {
    for entity in &instances {
        commands.entity(entity).despawn();
    }
}

pub fn tick_cooldowns(mut query: Query<&mut AbilityCooldowns>, time: Res<Time>) {
    for mut cooldowns in &mut query {
        for remaining in &mut cooldowns.remaining {
            *remaining = (*remaining - time.delta_secs()).max(0.0);
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
            commands.entity(proj.instance).insert(Advance);
        }
    }
}

pub fn tick_projectile_collision(
    mut projectiles: Query<(Entity, &mut ProjectileHitbox, &Collider, &Position)>,
    spatial_query: SpatialQuery,
    mut commands: Commands,
) {
    for (entity, mut proj, collider, position) in &mut projectiles {
        let filter = SpatialQueryFilter::from_excluded_entities([proj.caster, entity]);
        let hits = spatial_query.shape_intersections(collider, position.0, 0.0, &filter);

        for hit in hits {
            if !proj.already_hit.contains(&hit) {
                proj.already_hit.push(hit);
                commands.entity(hit).insert(TakeDamage(proj.damage));
                commands.entity(entity).despawn();
                commands.entity(proj.instance).insert(Advance);
                break;
            }
        }
    }
}

pub fn apply_damage(
    targets: Query<(Entity, &TakeDamage)>,
    mut health_query: Query<&mut Health>,
    mut commands: Commands,
) {
    for (entity, TakeDamage(damage)) in &targets {
        if let Ok(mut health) = health_query.get_mut(entity) {
            health.apply_damage(*damage);
        }
        commands.entity(entity).remove::<TakeDamage>();
    }
}
