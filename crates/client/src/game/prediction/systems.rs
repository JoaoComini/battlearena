use avian2d::prelude::*;
use bevy::prelude::*;
use shared::{
    physics::{apply_movement_input, PhysicsInput, PendingPosition},
    tick::TICK_DELTA,
    types::Pos2,
};

use crate::resources::{
    CurrentInput, InputHistory, LocalPlayer, LocalTick, PendingCorrection, PredictedPosition,
    PreviousPredictedPosition,
};

/// Each fixed tick: record input and hand off to the shared movement system.
pub fn tick_prediction(
    mut commands: Commands,
    input: Res<CurrentInput>,
    mut local_tick: ResMut<LocalTick>,
    mut local: Query<
        (Entity, &mut InputHistory),
        With<LocalPlayer>,
    >,
) {
    let Ok((entity, mut history)) = local.single_mut() else {
        return;
    };

    let tick = local_tick.0;
    local_tick.0 += 1;

    // Push placeholder position — filled in by update_predicted_position after flush.
    history.0.push_back((tick, input.0, Pos2::ZERO));
    if history.0.len() > 64 {
        history.0.pop_front();
    }

    commands.entity(entity).insert(PhysicsInput(input.0));
}

/// Consumes a PendingCorrection on the local player, re-simulates from the
/// input history using MoveAndSlide, and snaps the predicted position.
#[allow(clippy::type_complexity)]
pub fn apply_correction(
    mut commands: Commands,
    move_and_slide: MoveAndSlide,
    mut local: Query<
        (
            Entity,
            &Collider,
            &mut PredictedPosition,
            &mut PreviousPredictedPosition,
            &InputHistory,
            &PendingCorrection,
        ),
        With<LocalPlayer>,
    >,
) {
    let Ok((entity, collider, mut predicted, mut prev_predicted, history, correction)) =
        local.single_mut()
    else {
        return;
    };

    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let delta = std::time::Duration::from_secs_f32(TICK_DELTA);

    let mut pos = Vec2::new(correction.pos.x, correction.pos.y);
    for (tick, input, _) in &history.0 {
        if *tick > correction.tick {
            pos = apply_movement_input(&move_and_slide, collider, pos, *input, delta, &filter);
        }
    }

    let new_pos = Pos2 { x: pos.x, y: pos.y };
    prev_predicted.0 = new_pos;
    predicted.0 = new_pos;

    // Write re-simulated position directly — shared apply_pending_positions will flush it.
    commands.entity(entity).insert(PendingPosition(pos));
    commands.entity(entity).remove::<PendingCorrection>();
    // Remove any PhysicsInput written by tick_prediction this frame so the movement
    // system doesn't overwrite the correction result with a stale step.
    commands.entity(entity).remove::<PhysicsInput>();
}

/// After apply_pending_positions has flushed Position, sync PredictedPosition
/// and fix the placeholder in the last InputHistory entry.
pub fn update_predicted_position(
    mut local: Query<
        (
            &Position,
            &mut PredictedPosition,
            &mut PreviousPredictedPosition,
            &mut InputHistory,
        ),
        With<LocalPlayer>,
    >,
) {
    let Ok((position, mut predicted, mut prev_predicted, mut history)) = local.single_mut() else {
        return;
    };

    let new_pos = Pos2 { x: position.0.x, y: position.0.y };

    // Only update prev/predicted on normal prediction ticks (not after correction,
    // which already set them). We detect a correction tick by checking if predicted
    // already matches the new position (correction set it before flush).
    if predicted.0 != new_pos {
        prev_predicted.0 = predicted.0;
        predicted.0 = new_pos;
    }

    // Fix the placeholder Pos2::ZERO in the last history entry.
    if let Some(back) = history.0.back_mut() {
        back.2 = new_pos;
    }
}
