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
            (movement, set_rotation, move_and_slide, apply_move_and_slide).chain(),
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

pub fn movement(mut query: Query<(&mut LinearVelocity, &ActionState<Inputs>)>) {
    const MOVE_SPEED: f32 = 200.0;
    for (mut velocity, input) in &mut query {
        let Inputs::Direction(direction) = &input.0;
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
        let Inputs::Direction(direction) = &input.0;
        *rotation = Rotation::radians(direction.angle);
    }
}
