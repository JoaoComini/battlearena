use avian2d::prelude::*;
use bevy::prelude::*;
use shared::{
    logic::apply_input,
    tick::TICK_DELTA,
    types::Pos2,
};

use crate::resources::{
    CurrentInput, InputHistory, LocalPlayer, LocalTick, PendingCorrection, PredictedPosition,
    PreviousPredictedPosition,
};

/// Computed position pending write-back to avian's Position component.
/// Split from tick_prediction/apply_correction to avoid conflicting access with MoveAndSlide.
#[derive(Component)]
pub struct ClientPendingPosition(pub Vec2);

/// Each fixed tick: advance the local simulation via MoveAndSlide and save to history.
pub fn tick_prediction(
    mut commands: Commands,
    input: Res<CurrentInput>,
    mut local_tick: ResMut<LocalTick>,
    move_and_slide: MoveAndSlide,
    mut local: Query<
        (
            Entity,
            &Collider,
            &Position,
            &mut PredictedPosition,
            &mut PreviousPredictedPosition,
            &mut InputHistory,
        ),
        With<LocalPlayer>,
    >,
) {
    let Ok((entity, collider, position, mut predicted, mut prev_predicted, mut history)) =
        local.single_mut()
    else {
        return;
    };

    let tick = local_tick.0;
    local_tick.0 += 1;

    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let new_pos_vec = apply_input(
        &move_and_slide,
        collider,
        position.0,
        input.0,
        std::time::Duration::from_secs_f32(TICK_DELTA),
        &filter,
    );

    let new_pos = Pos2 {
        x: new_pos_vec.x,
        y: new_pos_vec.y,
    };
    prev_predicted.0 = predicted.0;
    predicted.0 = new_pos;

    history.0.push_back((tick, input.0, new_pos));
    if history.0.len() > 64 {
        history.0.pop_front();
    }

    commands
        .entity(entity)
        .insert(ClientPendingPosition(new_pos_vec));
}

/// Consumes a PendingCorrection on the local player, re-simulates from the
/// input history using MoveAndSlide, and snaps the predicted position.
pub fn apply_correction(
    mut commands: Commands,
    move_and_slide: MoveAndSlide,
    mut local: Query<
        (
            Entity,
            &Collider,
            &Position,
            &mut PredictedPosition,
            &mut PreviousPredictedPosition,
            &InputHistory,
            &PendingCorrection,
        ),
        With<LocalPlayer>,
    >,
) {
    let Ok((entity, collider, _position, mut predicted, mut prev_predicted, history, correction)) =
        local.single_mut()
    else {
        return;
    };

    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let delta = std::time::Duration::from_secs_f32(TICK_DELTA);

    let mut pos = Vec2::new(correction.pos.x, correction.pos.y);
    for (tick, input, _) in &history.0 {
        if *tick > correction.tick {
            pos = apply_input(&move_and_slide, collider, pos, *input, delta, &filter);
        }
    }

    let new_pos = Pos2 { x: pos.x, y: pos.y };
    prev_predicted.0 = new_pos;
    predicted.0 = new_pos;

    commands.entity(entity).insert(ClientPendingPosition(pos));
    commands.entity(entity).remove::<PendingCorrection>();
}

/// Applies positions computed by tick_prediction/apply_correction to avian's Position component.
/// Split to avoid conflicting &mut Position access with MoveAndSlide's internal queries.
pub fn apply_client_pending_positions(
    mut commands: Commands,
    mut players: Query<(Entity, &ClientPendingPosition, &mut Position)>,
) {
    for (entity, pending, mut position) in &mut players {
        position.0 = pending.0;
        commands.entity(entity).remove::<ClientPendingPosition>();
    }
}
