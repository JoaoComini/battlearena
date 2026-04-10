use bevy::prelude::*;
use bevy_renet::{renet::DefaultChannel, RenetClient};
use shared::{
    protocol::{C2S, MoveInput, S2C},
    tick::TickNumber,
};

use crate::resources::{
    CurrentInput, EntityRegistry, InputHistory, LocalPlayer, PendingCorrection, PredictedPosition,
    ServerTime, SnapshotBuffer, SpawnRegistry,
};

/// Drains both server channels and dispatches all inbound messages:
/// - ReliableOrdered: entity lifecycle (spawn/despawn), corrections, acks.
/// - Unreliable: world snapshots.
pub fn recv_server(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut registry: ResMut<EntityRegistry>,
    spawn_registry: Res<SpawnRegistry>,
    time: Res<Time<bevy::prelude::Real>>,
    mut server_time: ResMut<ServerTime>,
    mut local: Query<(Entity, &mut InputHistory), With<LocalPlayer>>,
    mut buffers: Query<&mut SnapshotBuffer>,
) {
    // --- Reliable channel ---
    let mut latest_ack: Option<TickNumber> = None;
    let mut latest_correction: Option<(TickNumber, shared::types::Pos2)> = None;

    let mut messages = Vec::new();
    while let Some(msg) = client.receive_message(DefaultChannel::ReliableOrdered) {
        messages.push(msg);
    }

    for msg in messages {
        match postcard::from_bytes(&msg) {
            Ok(S2C::EntitySpawned { id, prefab, pos, owner }) => {
                if registry.0.contains_key(&id) {
                    continue;
                }
                let Some(spawn_fn) = spawn_registry.0.get(&prefab) else {
                    warn!("No spawn function registered for prefab {:?}", prefab);
                    continue;
                };
                let entity = commands.spawn(id).id();
                spawn_fn(&mut commands, entity, pos, owner);
                registry.0.insert(id, entity);
            }
            Ok(S2C::EntityDespawned { id }) => {
                if let Some(entity) = registry.0.remove(&id) {
                    commands.entity(entity).despawn();
                }
            }
            Ok(S2C::Correction { tick, pos }) => {
                info!("Correction received: tick={tick}, pos=({}, {})", pos.x, pos.y);
                if latest_correction.map_or(true, |(t, _)| tick > t) {
                    latest_correction = Some((tick, pos));
                }
            }
            Ok(S2C::Ack { tick }) => {
                if latest_ack.map_or(true, |t| tick > t) {
                    latest_ack = Some(tick);
                }
            }
            _ => {}
        }
    }

    // The effective ack is the highest tick confirmed by either Ack or Correction.
    let ack_tick = match (latest_ack, latest_correction.map(|(t, _)| t)) {
        (Some(a), Some(c)) => Some(a.max(c)),
        (Some(a), None) => Some(a),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };

    if let Ok((entity, mut history)) = local.single_mut() {
        if let Some(ack) = ack_tick {
            while history.0.front().map_or(false, |(t, _, _)| *t <= ack) {
                history.0.pop_front();
            }
        }
        if let Some((tick, pos)) = latest_correction {
            commands.entity(entity).insert(PendingCorrection { tick, pos });
        }
    }

    // --- Unreliable channel ---
    let client_now = time.elapsed_secs_f64();

    while let Some(msg) = client.receive_message(DefaultChannel::Unreliable) {
        if let Ok(S2C::WorldSnapshot { server_time: st, players }) = postcard::from_bytes(&msg) {
            server_time.update(st, client_now);
            for state in &players {
                if let Some(&entity) = registry.0.get(&state.id) {
                    if let Ok(mut buf) = buffers.get_mut(entity) {
                        buf.push(st, state.pos);
                    }
                }
            }
        }
    }
}

/// Each fixed tick: send the current input and predicted position to the server,
/// piggybacking the oldest unacknowledged direction change for loss recovery.
pub fn send_input_tick(
    mut client: ResMut<RenetClient>,
    input: Res<CurrentInput>,
    local: Query<(&PredictedPosition, &InputHistory), With<LocalPlayer>>,
) {
    let Ok((predicted, history)) = local.single() else {
        return;
    };

    // The last entry in history is what tick_prediction just recorded for this tick.
    let Some(&(tick, _, _)) = history.0.back() else {
        return;
    };

    let current = MoveInput { tick, input: input.0, pos: predicted.0 };

    // Find the oldest unacknowledged direction change for loss recovery.
    // We want the first tick of the new direction, not the last tick of the old one.
    let old = history.0.iter().zip(history.0.iter().skip(1))
        .find(|((_, prev_input, _), (_, cur_input, _))| prev_input.0 != cur_input.0)
        .map(|(_, (old_tick, old_input, old_pos))| MoveInput {
            tick: *old_tick,
            input: *old_input,
            pos: *old_pos,
        });

    let msg = C2S::InputTick { current, old };
    if let Ok(bytes) = postcard::to_allocvec(&msg) {
        client.send_message(DefaultChannel::Unreliable, bytes);
    }
}
