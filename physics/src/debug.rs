use avian2d::prelude::*;
use bevy::prelude::*;

pub struct PhysicsDebugRenderPlugin;

impl Plugin for PhysicsDebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, configure_gizmos);
        app.add_systems(Update, debug_render_colliders);
    }
}

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.0;
}

fn debug_render_colliders(query: Query<(&Position, &Collider)>, mut gizmos: Gizmos) {
    let color = Color::srgb(0.0, 1.0, 0.0);
    for (position, collider) in &query {
        let center = Vec3::new(position.x, 0.0, -position.y);
        let iso3 = Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
        if let Some(ball) = collider.shape().as_ball() {
            gizmos.circle(iso3, ball.radius as f32, color);
        } else if let Some(cuboid) = collider.shape().as_cuboid() {
            gizmos.rect(
                iso3,
                Vec2::new(
                    cuboid.half_extents.x as f32 * 2.0,
                    cuboid.half_extents.y as f32 * 2.0,
                ),
                color,
            );
        } else if let Some(hull) = collider.shape().as_convex_polygon() {
            let pts = hull.points();
            for i in 0..pts.len() {
                let a = Vec2::new(pts[i].x as f32, pts[i].y as f32);
                let b = Vec2::new(
                    pts[(i + 1) % pts.len()].x as f32,
                    pts[(i + 1) % pts.len()].y as f32,
                );
                let wa = Vec3::new(position.x + a.x, 0.0, -(position.y + a.y));
                let wb = Vec3::new(position.x + b.x, 0.0, -(position.y + b.y));
                gizmos.line(wa, wb, color);
            }
        }
    }
}
