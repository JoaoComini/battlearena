use avian2d::physics_transform::{
    ApplyPosToTransform, PhysicsTransformConfig, PhysicsTransformSystems,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use inputs::Inputs;
use lightyear::prelude::{input::native::ActionState, PredictionSystems};

pub const PLAYER_SIZE: f32 = 50.0;

#[derive(Component, Default)]
pub struct MoveAndSlideResult(pub Vec2, pub Vec2, pub f32);

// Player
#[derive(Bundle)]
pub struct PlayerPhysicsBundle {
    pub rigid_body: RigidBody,
    pub custom_position_integration: CustomPositionIntegration,
    pub collider: Collider,
    pub results: MoveAndSlideResult,
}

impl Default for PlayerPhysicsBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Kinematic,
            custom_position_integration: CustomPositionIntegration,
            collider: Collider::circle(PLAYER_SIZE * 0.5),
            results: MoveAndSlideResult::default(),
        }
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsTransformConfig {
            position_to_transform: false,
            transform_to_position: false,
            ..default()
        });

        app.register_required_components::<Position, Transform>();
        app.register_required_components::<Rotation, Transform>();
        app.register_required_components::<Position, ApplyPosToTransform>();
        app.register_required_components::<Rotation, ApplyPosToTransform>();

        app.add_plugins(PhysicsPlugins::default());

        app.add_systems(
            FixedUpdate,
            (
                set_lin_velocity,
                set_rotation,
                move_and_slide,
                apply_move_and_slide,
            )
                .chain(),
        );

        app.configure_sets(
            FixedPostUpdate,
            (
                PhysicsSystems::StepSimulation,
                PredictionSystems::UpdateHistory,
            )
                .chain(),
        );

        app.configure_sets(
            FixedPostUpdate,
            PhysicsTransformSystems::PositionToTransform.in_set(PhysicsSystems::Writeback),
        );

        app.add_systems(
            FixedPostUpdate,
            position_to_transform.in_set(PhysicsTransformSystems::PositionToTransform),
        );
    }
}

pub fn position_to_transform(
    mut query: Query<(&Position, &Rotation, &mut Transform), With<ApplyPosToTransform>>,
) {
    for (pos, rot, mut transform) in &mut query {
        transform.translation = Vec3::new(pos.x, 0.0, -pos.y);
        transform.rotation = Quat::from_rotation_y(rot.as_radians());
    }
}

pub fn move_and_slide(
    mut query: Query<(
        Entity,
        &Position,
        &LinearVelocity,
        &Collider,
        &Rotation,
        &mut MoveAndSlideResult,
    )>,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
) {
    for (entity, position, lin_vel, collider, rotation, mut result) in &mut query {
        let MoveAndSlideOutput {
            position: new_pos,
            projected_velocity,
        } = move_and_slide.move_and_slide(
            collider,
            position.0,
            rotation.as_radians(),
            lin_vel.0,
            time.delta(),
            &MoveAndSlideConfig::default(),
            &SpatialQueryFilter::from_excluded_entities([entity]),
            |_| MoveAndSlideHitResponse::Accept,
        );
        *result = MoveAndSlideResult(new_pos, projected_velocity, position.0.y);
    }
}

pub fn apply_move_and_slide(
    mut query: Query<(&mut Position, &mut LinearVelocity, &MoveAndSlideResult)>,
) {
    for (mut position, mut lin_vel, result) in &mut query {
        position.0 = result.0;
        lin_vel.0 = result.1;
    }
}

pub fn set_lin_velocity(mut query: Query<(&mut LinearVelocity, &ActionState<Inputs>)>) {
    const MOVE_SPEED: f32 = 200.0;
    for (mut velocity, input) in &mut query {
        let Inputs::PlayerInput(player_input) = &input.0;
        let direction = &player_input.movement;
        let mut dir = Vec2::ZERO;
        if direction.up {
            dir.y += 1.0;
        }
        if direction.down {
            dir.y -= 1.0;
        }
        if direction.left {
            dir.x -= 1.0;
        }
        if direction.right {
            dir.x += 1.0;
        }
        velocity.0 = dir.normalize_or_zero() * MOVE_SPEED;
    }
}

pub fn set_rotation(mut query: Query<(&mut Rotation, &ActionState<Inputs>)>) {
    for (mut rotation, input) in &mut query {
        let Inputs::PlayerInput(player_input) = &input.0;
        *rotation = Rotation::radians(player_input.movement.angle);
    }
}

// ── Physics debug rendering (3D-aware) ───────────────────────────────────────

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
