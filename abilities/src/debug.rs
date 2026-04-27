use avian2d::prelude::{Collider, Position, SpatialQuery, SpatialQueryFilter};
use bevy::color::palettes::css::{GREEN, RED, YELLOW};
use bevy::prelude::*;
use physics::Wall;
use protocol::Health;

use crate::types::{HitMarker, MeleeHitbox};

pub struct AbilityDebugPlugin;

impl Plugin for AbilityDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                mark_hits_on_health_change,
                tick_hit_markers,
                draw_melee_los_rays,
                draw_hit_markers,
            ),
        );
    }
}

fn mark_hits_on_health_change(
    mut commands: Commands,
    changed: Query<Entity, Changed<Health>>,
) {
    for entity in &changed {
        commands.entity(entity).insert(HitMarker::new());
    }
}

fn tick_hit_markers(
    mut commands: Commands,
    mut markers: Query<(Entity, &mut HitMarker)>,
    time: Res<Time>,
) {
    for (entity, mut marker) in &mut markers {
        marker.timer.tick(time.delta());
        if marker.timer.just_finished() {
            commands.entity(entity).remove::<HitMarker>();
        }
    }
}

fn draw_hit_markers(
    markers: Query<&Position, With<HitMarker>>,
    mut gizmos: Gizmos,
) {
    for position in &markers {
        let center = Vec3::new(position.x, 0.0, -position.y);
        let iso3 = Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
        gizmos.circle(iso3, 1.2, YELLOW);
        gizmos.circle(iso3, 1.4, YELLOW);
    }
}

fn draw_melee_los_rays(
    hitboxes: Query<(Entity, &MeleeHitbox)>,
    target_query: Query<(&Position, &Collider)>,
    spatial_query: SpatialQuery,
    wall_query: Query<(), With<Wall>>,
    mut gizmos: Gizmos,
) {
    for (hitbox_entity, hitbox) in &hitboxes {
        // Reconstruct the same shape_intersections filter used during damage
        // to find what targets the hitbox overlaps — we only draw rays toward them.
        // Rather than re-running the full shape query here, iterate already_hit
        // plus do a quick position-based scan. Since we want to show rays for all
        // potential targets (not just confirmed hits), we draw for every entity in
        // already_hit and also for any target currently overlapping (tracked via
        // the hitbox's already_hit list is sufficient for visualization).
        for &hit in &hitbox.already_hit {
            draw_ray_pair(
                hitbox.origin,
                hitbox_entity,
                hit,
                &target_query,
                &spatial_query,
                &wall_query,
                &mut gizmos,
            );
        }
    }
}

fn draw_ray_pair(
    origin: Vec2,
    hitbox_entity: Entity,
    target: Entity,
    target_query: &Query<(&Position, &Collider)>,
    spatial_query: &SpatialQuery,
    wall_query: &Query<(), With<Wall>>,
    gizmos: &mut Gizmos,
) {
    let Ok((target_pos, target_collider)) = target_query.get(target) else { return; };

    let to_target = target_pos.0 - origin;
    let distance = to_target.length();
    if distance <= 0.001 { return; }
    let Ok(dir) = Dir2::try_from(to_target) else { return; };

    let radius = target_collider.shape().as_ball().map_or(0.4, |b| b.radius as f32);
    let perp = Vec2::new(-dir.y, dir.x) * radius;
    let los_filter = SpatialQueryFilter::from_excluded_entities([hitbox_entity, target]);

    for offset in [perp, -perp] {
        let ray_origin = origin + offset;
        let hit = spatial_query
            .cast_ray(ray_origin, dir, distance, true, &los_filter)
            .filter(|ray_hit| wall_query.contains(ray_hit.entity));

        let (end_2d, color) = match hit {
            Some(ray_hit) => (ray_origin + *dir * ray_hit.distance, RED),
            None => (ray_origin + *dir * distance, GREEN),
        };

        let start_3d = Vec3::new(ray_origin.x, 0.1, -ray_origin.y);
        let end_3d = Vec3::new(end_2d.x, 0.1, -end_2d.y);
        gizmos.line(start_3d, end_3d, color);
    }
}
