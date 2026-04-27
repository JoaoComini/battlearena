pub mod debug;

use avian2d::physics_transform::{
    ApplyPosToTransform, PhysicsTransformConfig, PhysicsTransformSystems,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use inputs::Inputs;
use lightyear::prelude::{input::native::ActionState, PredictionSystems};

pub const PLAYER_SIZE: f32 = 0.8;

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
    mut query: Query<
        (&Position, &Rotation, &mut Transform, Option<&ChildOf>),
        With<ApplyPosToTransform>,
    >,
    parents: Query<(&GlobalTransform, Option<&Position>, Option<&Rotation>)>,
) {
    for (pos, rot, mut transform, child_of) in &mut query {
        let world_rotation = Quat::from_rotation_y(rot.as_radians());

        if let Some(&ChildOf(parent)) = child_of {
            if let Ok((parent_global, parent_pos, parent_rot)) = parents.get(parent) {
                let parent_transform = parent_global.compute_transform();
                let parent_translation = parent_pos.map_or(parent_transform.translation, |p| {
                    Vec3::new(p.x, parent_transform.translation.y, -p.y)
                });
                let parent_rotation = parent_rot.map_or(parent_transform.rotation, |r| {
                    Quat::from_rotation_y(r.as_radians())
                });
                let parent_scale = parent_transform.scale;
                let effective_parent = Transform::from_translation(parent_translation)
                    .with_rotation(parent_rotation)
                    .with_scale(parent_scale);

                let new_transform = GlobalTransform::from(
                    Transform::from_translation(Vec3::new(pos.x, transform.translation.y, -pos.y))
                        .with_rotation(world_rotation),
                )
                .reparented_to(&GlobalTransform::from(effective_parent));

                transform.translation = new_transform.translation;
                transform.rotation = new_transform.rotation;
            }
        } else {
            transform.translation = Vec3::new(pos.x, transform.translation.y, -pos.y);
            transform.rotation = world_rotation;
        }
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
    const MOVE_SPEED: f32 = 4.8;
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
