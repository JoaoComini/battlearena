use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DebugGizmoHitbox {
    PieSlice { range: f32, angle_deg: f32 },
    Circle { radius: f32 },
}

pub struct PhysicsDebugRenderPlugin;

impl Plugin for PhysicsDebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<DebugGizmoHitbox>();
        app.add_systems(Startup, configure_gizmos);
        app.add_systems(Update, (debug_render_colliders, draw_ability_hitboxes));
    }
}

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.0;
}

fn debug_render_colliders(
    query: Query<(&Position, &Collider)>,
    mut gizmos: Gizmos,
) {
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
                let b = Vec2::new(pts[(i + 1) % pts.len()].x as f32, pts[(i + 1) % pts.len()].y as f32);
                let wa = Vec3::new(position.x + a.x, 0.0, -(position.y + a.y));
                let wb = Vec3::new(position.x + b.x, 0.0, -(position.y + b.y));
                gizmos.line(wa, wb, color);
            }
        }
    }
}

fn draw_ability_hitboxes(
    hitboxes: Query<(&DebugGizmoHitbox, &Position, &Rotation)>,
    mut gizmos: Gizmos,
) {
    for (hitbox, position, rotation) in &hitboxes {
        match hitbox {
            DebugGizmoHitbox::PieSlice { range, angle_deg } => {
                let origin = position.0;
                let facing_rad = rotation.as_radians();
                let half = (angle_deg / 2.0).to_radians();
                let adjusted = facing_rad + std::f32::consts::FRAC_PI_2;
                let steps = 8usize;
                let center = Vec3::new(origin.x, 0.0, -origin.y);
                let mut prev = center;
                let first = {
                    let p = Vec2::from_angle(adjusted - half) * range + origin;
                    Vec3::new(p.x, 0.0, -p.y)
                };
                gizmos.line(center, first, bevy::color::palettes::css::ORANGE);
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let a = adjusted - half + t * 2.0 * half;
                    let p2d = Vec2::from_angle(a) * range + origin;
                    let p3d = Vec3::new(p2d.x, 0.0, -p2d.y);
                    if i > 0 {
                        gizmos.line(prev, p3d, bevy::color::palettes::css::ORANGE);
                    }
                    prev = p3d;
                }
                gizmos.line(prev, center, bevy::color::palettes::css::ORANGE);
            }
            DebugGizmoHitbox::Circle { radius } => {
                let origin = position.0;
                let center = Vec3::new(origin.x, 0.0, -origin.y);
                let iso3 = Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
                gizmos.circle(iso3, *radius, bevy::color::palettes::css::ORANGE);
            }
        }
    }
}

fn draw_gizmo(
    collider: Collider,
    position: Position,
    mut gizmos: Gizmos,
    color: Color,
) {
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
                let b = Vec2::new(pts[(i + 1) % pts.len()].x as f32, pts[(i + 1) % pts.len()].y as f32);
                let wa = Vec3::new(position.x + a.x, 0.0, -(position.y + a.y));
                let wb = Vec3::new(position.x + b.x, 0.0, -(position.y + b.y));
                gizmos.line(wa, wb, color);
            }
        }
}
