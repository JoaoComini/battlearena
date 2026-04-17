use avian2d::prelude::ColliderConstructor;
use bevy::prelude::*;
use crate::spawn::ActiveSceneRoot;


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
    root_query: Query<Entity, With<ActiveSceneRoot>>,
    children_query: Query<&Children>,
    collider_query: Query<(&GlobalTransform, &ColliderConstructor)>,
    mut gizmos: Gizmos,
) {
    let Ok(root) = root_query.single() else {
        return;
    };

    for entity in iter_descendants(root, &children_query) {
        let Ok((transform, collider)) = collider_query.get(entity) else {
            continue;
        };

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

fn iter_descendants(root: Entity, children_query: &Query<&Children>) -> Vec<Entity> {
    let mut result = Vec::new();
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                result.push(child);
                stack.push(child);
            }
        }
    }
    result
}
