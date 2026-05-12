use std::collections::HashMap;

use crate::attributes::{Attribute, EffectEvent, Modifier};
use crate::types::{AbilityCooldowns, AbilityInstance, CastingTask, MeleeHitTask, ProjectileTask};
use avian2d::prelude::{Collider, CollisionLayers, LayerMask, Position, SpatialQuery, SpatialQueryFilter};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use physics::{pie_slice_collider, GameLayer};

pub fn tick_casting_tasks(
    mut tasks: Query<(Entity, &mut CastingTask, &ChildOf)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (task_entity, mut task, child_of) in &mut tasks {
        task.remaining_secs -= time.delta_secs();
        if task.remaining_secs > 0.0 {
            continue;
        }
        let Some(on_done) = task.on_done.take() else {
            continue;
        };
        commands.entity(task_entity).despawn();
        on_done(&mut commands);
    }
}

pub fn process_melee_hit_tasks(
    mut tasks: Query<(Entity, &mut MeleeHitTask, &ChildOf)>,
    instances: Query<&AbilityInstance>,
    spatial_query: SpatialQuery,
    mut commands: Commands,
) {
    for (task_entity, mut task, child_of) in &mut tasks {
        let instance_entity = child_of.parent();
        let Some(inst) = instances.get(instance_entity).ok() else {
            continue;
        };
        if let Some(collider) = pie_slice_collider(task.range, task.angle_deg, inst.facing_rad) {
            let filter = SpatialQueryFilter::from_excluded_entities([inst.caster])
                .with_mask(GameLayer::Character);
            let hits = spatial_query.shape_intersections(&collider, inst.origin, 0.0, &filter);
            for hit in hits {
                (task.on_hit)(hit, &mut commands);
            }
        }
        let on_end = task.on_end.take();
        commands.entity(task_entity).despawn();
        if let Some(on_end) = on_end {
            on_end(&mut commands);
        }
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
    mut query: Query<(Entity, &mut ProjectileTask, &mut Position)>,
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

pub fn tick_projectile_collision(
    mut projectiles: Query<(Entity, &mut ProjectileTask, &Collider, &Position)>,
    collision_layers: Query<&CollisionLayers>,
    spatial_query: SpatialQuery,
    mut commands: Commands,
) {
    for (proj_entity, mut proj, collider, position) in &mut projectiles {
        let filter = SpatialQueryFilter::from_excluded_entities([proj.caster, proj_entity])
            .with_mask(!LayerMask::from(GameLayer::Projectile));
        let shape_hits = spatial_query.shape_intersections(collider, position.0, 0.0, &filter);
        let hit = shape_hits
            .into_iter()
            .find(|h| !proj.already_hit.contains(h));
        let Some(hit_entity) = hit else { continue };
        proj.already_hit.push(hit_entity);
        let memberships = collision_layers
            .get(hit_entity)
            .map(|cl| cl.memberships)
            .unwrap_or(LayerMask::from(GameLayer::Environment));
        commands.entity(proj_entity).despawn();
        (proj.on_hit)(hit_entity, memberships, &mut commands);
    }
}

pub fn apply_ability_effects<A: Attribute>(
    mut messages: MessageReader<EffectEvent<A>>,
    mut query: Query<&mut A>,
) {
    let mut grouped: HashMap<Entity, Vec<(f32, Modifier)>> = HashMap::new();
    for msg in messages.read() {
        grouped
            .entry(msg.target)
            .or_default()
            .push((msg.value, msg.modifier));
    }

    for (entity, mut effects) in grouped {
        let Ok(mut attr) = query.get_mut(entity) else {
            continue;
        };

        effects.sort_by_key(|(_, m)| *m);

        for (value, modifier) in effects {
            attr.apply_modifier(value, modifier);
        }
    }
}
