pub mod debug;

use avian2d::physics_transform::{
    ApplyPosToTransform, PhysicsTransformConfig, PhysicsTransformSystems,
};
use avian2d::prelude::*;
use bevy::prelude::*;

pub const PLAYER_SIZE: f32 = 0.8;

pub fn pie_slice_collider(range: f32, angle_deg: f32, facing_rad: f32) -> Option<Collider> {
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

#[derive(Component, Default)]
pub struct MoveAndSlideResult(pub Vec2, pub Vec2, pub f32);

#[derive(Bundle)]
pub struct MoveAndSlideBundle {
    pub rigid_body: RigidBody,
    pub custom_position_integration: CustomPositionIntegration,
    pub collider: Collider,
    pub results: MoveAndSlideResult,
}

impl Default for MoveAndSlideBundle {
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

        app.add_systems(FixedUpdate, (move_and_slide, apply_move_and_slide).chain());

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
