use avian2d::prelude::{Collider, Position, Rotation, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use lightyear::prelude::input::native::ActionState;
use protocol::Health;
use inputs::Inputs;
use crate::types::{AbilityDef, AbilityEffect, AbilityLoadout, AbilitySlot, MeleeHitbox};
use crate::registry::AbilityRegistry;

pub fn tick_cooldowns(mut query: Query<&mut AbilityLoadout>, time: Res<Time>) {
    for mut loadout in &mut query {
        for slot in &mut loadout.slots {
            slot.cooldown_remaining =
                (slot.cooldown_remaining - time.delta_secs()).max(0.0);
        }
    }
}

fn pie_slice_collider(range: f32, angle_deg: f32, facing_rad: f32) -> Option<Collider> {
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

pub fn activate_abilities(
    mut query: Query<(Entity, &ActionState<Inputs>, &mut AbilityLoadout, &Position, &Rotation)>,
    registry: Res<AbilityRegistry>,
    assets: Res<Assets<AbilityDef>>,
    mut commands: Commands,
) {
    for (entity, action_state, mut loadout, position, rotation) in &mut query {
        let Inputs::PlayerInput(input) = &action_state.0;
        let pressed = [input.abilities.slot1, input.abilities.slot2];

        for (i, slot) in loadout.slots.iter_mut().enumerate() {
            let slot: &mut AbilitySlot = slot;
            let Some(&slot_pressed) = pressed.get(i) else { continue };
            if !slot_pressed || !slot.is_ready() {
                continue;
            }

            let Some(def) = registry.get(&slot.key, &assets) else { continue };

            slot.cooldown_remaining = def.cooldown_secs;

            let AbilityEffect::MeleeHit { range, angle_deg, damage, lifetime_frames } = def.effect;

            let facing_rad = rotation.as_radians();
            let origin = position.0;

            let Some(collider) = pie_slice_collider(range, angle_deg, facing_rad) else { continue };

            commands.spawn((
                MeleeHitbox {
                    caster: entity,
                    damage,
                    range,
                    angle_deg,
                    lifetime_frames,
                    origin,
                    facing_rad,
                    already_hit: Vec::new(),
                },
                Position(origin),
                Rotation::radians(facing_rad),
                collider,
            ));
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

