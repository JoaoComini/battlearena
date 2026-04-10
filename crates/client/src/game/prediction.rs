use bevy::prelude::*;
use shared::logic::apply_input;

use crate::resources::{
    CurrentInput, InputHistory, LocalPlayer, LocalTick, PendingCorrection, PredictedPosition,
    PreviousPredictedPosition,
};

/// Each fixed tick: advance the local simulation and save to history.
pub fn tick_prediction(
    input: Res<CurrentInput>,
    mut local_tick: ResMut<LocalTick>,
    mut local: Query<
        (&mut PredictedPosition, &mut PreviousPredictedPosition, &mut InputHistory),
        With<LocalPlayer>,
    >,
) {
    let Ok((mut predicted, mut prev_predicted, mut history)) = local.single_mut() else {
        return;
    };

    let tick = local_tick.0;
    local_tick.0 += 1;

    prev_predicted.0 = predicted.0;
    predicted.0 = apply_input(predicted.0, input.0);

    history.0.push_back((tick, input.0, predicted.0));
    if history.0.len() > 64 {
        history.0.pop_front();
    }
}

/// Consumes a PendingCorrection on the local player, re-simulates from the
/// input history, and snaps the predicted position.
pub fn apply_correction(
    mut commands: Commands,
    mut local: Query<
        (Entity, &mut PredictedPosition, &mut PreviousPredictedPosition, &InputHistory, &PendingCorrection),
        With<LocalPlayer>,
    >,
) {
    let Ok((entity, mut predicted, mut prev_predicted, history, correction)) = local.single_mut() else {
        return;
    };

    let mut pos = correction.pos;
    for (tick, input, _) in &history.0 {
        if *tick > correction.tick {
            pos = apply_input(pos, *input);
        }
    }
    prev_predicted.0 = pos;
    predicted.0 = pos;

    commands.entity(entity).remove::<PendingCorrection>();
}
