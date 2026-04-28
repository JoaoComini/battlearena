use avian2d::prelude::Position;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use protocol::Health;

use crate::types::HitMarker;

pub struct AbilityDebugPlugin;

impl Plugin for AbilityDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                mark_hits_on_health_change,
                tick_hit_markers,
                draw_hit_markers,
            ),
        );
    }
}

fn mark_hits_on_health_change(mut commands: Commands, changed: Query<Entity, Changed<Health>>) {
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

fn draw_hit_markers(markers: Query<&Position, With<HitMarker>>, mut gizmos: Gizmos) {
    for position in &markers {
        let center = Vec3::new(position.x, 0.0, -position.y);
        let iso3 = Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
        gizmos.circle(iso3, 1.2, YELLOW);
        gizmos.circle(iso3, 1.4, YELLOW);
    }
}
