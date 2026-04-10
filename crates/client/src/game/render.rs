use avian2d::prelude::Position;
use bevy::prelude::*;
use shared::types::Pos2;

use crate::resources::{LocalPlayer, PredictedPosition, PreviousPredictedPosition, ServerTime, SnapshotBuffer};

/// Updates each player's Transform.
/// Remote players: snapshot interpolation at (estimated_server_now - INTERP_DELAY).
/// Local player: predicted position interpolated over the fixed-timestep overshoot.
pub fn render_players(
    time: Res<Time<bevy::prelude::Real>>,
    fixed_time: Res<Time<Fixed>>,
    server_time: Res<ServerTime>,
    mut remote: Query<(&SnapshotBuffer, &mut Transform, &mut Position), Without<LocalPlayer>>,
    mut local: Query<
        (&PredictedPosition, &PreviousPredictedPosition, &mut Transform),
        With<LocalPlayer>,
    >,
) {
    let interp_target = server_time.estimate(time.elapsed_secs_f64()) - shared::tick::INTERP_DELAY;
    for (buf, mut transform, mut position) in &mut remote {
        if let Some(pos) = buf.sample(interp_target) {
            transform.translation = Vec3::new(pos.x, 20.0, -pos.y);
            position.0 = Vec2::new(pos.x, pos.y);
        }
    }

    let t = fixed_time.overstep_fraction();
    if let Ok((predicted, prev_predicted, mut transform)) = local.single_mut() {
        let pos = lerp_pos(prev_predicted.0, predicted.0, t);
        transform.translation = Vec3::new(pos.x, 20.0, -pos.y);
    }
}

fn lerp_pos(a: Pos2, b: Pos2, t: f32) -> Pos2 {
    Pos2 {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}
