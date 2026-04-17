use avian2d::prelude::ColliderConstructor;
use bevy::prelude::*;

use crate::spawn::Editable;

pub struct ColliderGizmosPlugin;

impl Plugin for ColliderGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, configure_gizmos);
        app.add_systems(Update, draw_colliders);
    }
}

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.0;
}

/// Draws colliders as gizmos in the X/Z plane (Y = entity's world Y position).
fn draw_colliders(
    query: Query<(&GlobalTransform, &ColliderConstructor), With<Editable>>,
    mut gizmos: Gizmos,
) {
    for (transform, collider) in &query {
        let translation = transform.translation();
        let y = translation.y;
        let center = Vec3::new(translation.x, y, translation.z);

        match collider {
            ColliderConstructor::Circle { radius } => {
                gizmos.circle(
                    Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    *radius,
                    Color::srgb(0.0, 1.0, 0.0),
                );
            }
            ColliderConstructor::Rectangle { x_length, y_length } => {
                // In the X/Z plane, x_length maps to X, y_length maps to Z.
                gizmos.rect(
                    Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    Vec2::new(*x_length, *y_length),
                    Color::srgb(0.0, 1.0, 0.0),
                );
            }
            _ => {}
        }
    }
}
