use std::time::Duration;

use avian2d::prelude::*;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::Vec2;

use crate::protocol::InputBits;
use crate::tick::TICK_DELTA;

pub const PLAYER_SPEED: f32 = 200.0; // world units per second
pub const PLAYER_RADIUS: f32 = 16.0; // matches circle collider radius in scene.rs

/// Component written by client/server each tick to drive the shared movement system.
#[derive(Component)]
pub struct PhysicsInput(pub InputBits);

/// Intermediate position computed by `apply_physics_movement`, flushed to avian's
/// `Position` by `apply_pending_positions`. Split to avoid conflicting `&mut Position`
/// access with MoveAndSlide's internal queries.
#[derive(Component)]
pub struct PendingPosition(pub Vec2);

/// System set ordering for the shared movement pipeline. Configured to run
/// before `PhysicsSet::Prepare` so avian sees the final positions each tick.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsMovementSet {
    /// Client writes `PhysicsInput` (tick_prediction) and applies corrections here.
    /// Server drains `InputQueue` and writes `PhysicsInput` here.
    PrepareInput,
    /// Shared: reads `PhysicsInput`, runs MoveAndSlide, writes `PendingPosition`.
    Move,
    /// Shared: flushes `PendingPosition` → `Position`.
    /// Client syncs `PredictedPosition` here. Server sends acks/corrections here.
    Flush,
}

fn input_velocity(input: InputBits) -> Vec2 {
    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;

    if input.is_set(InputBits::RIGHT) { dx += 1.0; }
    if input.is_set(InputBits::LEFT)  { dx -= 1.0; }
    if input.is_set(InputBits::UP)    { dy += 1.0; }
    if input.is_set(InputBits::DOWN)  { dy -= 1.0; }

    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        dx /= len;
        dy /= len;
    }
    Vec2::new(dx * PLAYER_SPEED, dy * PLAYER_SPEED)
}

/// Advances `position` by one physics tick given `input`, sliding along obstacles.
/// Returns the resulting position. Used by both the shared movement system and
/// the client's `apply_correction` re-simulation loop.
pub fn apply_movement_input(
    move_and_slide: &MoveAndSlide,
    collider: &Collider,
    position: Vec2,
    input: InputBits,
    delta: Duration,
    filter: &SpatialQueryFilter,
) -> Vec2 {
    let velocity = input_velocity(input);

    move_and_slide
        .move_and_slide(
            collider,
            position,
            0.0,
            velocity,
            delta,
            &MoveAndSlideConfig::default(),
            filter,
            |_| MoveAndSlideHitResponse::Accept,
        )
        .position
}

fn apply_physics_movement(
    mut commands: Commands,
    move_and_slide: MoveAndSlide,
    players: Query<(Entity, &Collider, &Position, &PhysicsInput)>,
) {
    let delta = Duration::from_secs_f32(TICK_DELTA);
    for (entity, collider, position, physics_input) in &players {
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let new_pos = apply_movement_input(&move_and_slide, collider, position.0, physics_input.0, delta, &filter);
        commands.entity(entity).insert(PendingPosition(new_pos)).remove::<PhysicsInput>();
    }
}

fn apply_pending_positions(
    mut commands: Commands,
    mut players: Query<(Entity, &PendingPosition, &mut Position)>,
) {
    for (entity, pending, mut position) in &mut players {
        position.0 = pending.0;
        commands.entity(entity).remove::<PendingPosition>();
    }
}

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            (
                PhysicsMovementSet::PrepareInput,
                PhysicsMovementSet::Move,
                PhysicsMovementSet::Flush,
            )
            .chain()
            .before(PhysicsSystems::Prepare),
        )
        .add_systems(FixedUpdate, apply_physics_movement.in_set(PhysicsMovementSet::Move))
        .add_systems(FixedUpdate, apply_pending_positions.in_set(PhysicsMovementSet::Flush));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_input_zero_velocity() {
        let vel = input_velocity(InputBits::default());
        assert_eq!(vel.x, 0.0);
        assert_eq!(vel.y, 0.0);
    }

    #[test]
    fn right_input_positive_x_velocity() {
        let mut input = InputBits::default();
        input.set(InputBits::RIGHT);
        let vel = input_velocity(input);
        assert!(vel.x > 0.0);
        assert_eq!(vel.y, 0.0);
    }

    #[test]
    fn diagonal_speed_equals_cardinal_speed() {
        let mut cardinal = InputBits::default();
        cardinal.set(InputBits::RIGHT);
        let cardinal_vel = input_velocity(cardinal);

        let mut diagonal = InputBits::default();
        diagonal.set(InputBits::RIGHT);
        diagonal.set(InputBits::UP);
        let diagonal_vel = input_velocity(diagonal);

        assert!((cardinal_vel.length() - diagonal_vel.length()).abs() < 1e-5);
    }
}
