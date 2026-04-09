use bevy::prelude::*;
use bevy_renet::{
    renet::{ClientId, DefaultChannel, ServerEvent},
    RenetServer, RenetServerEvent,
};
use shared::{
    logic::apply_input,
    protocol::{C2S, MoveInput, S2C},
    types::{NetworkId, PlayerState, Pos2, PrefabId},
};

use crate::resources::{InputQueue, EntityRegistry, Position};

/// How far the client-reported position may differ from the server's before
/// a correction is issued (squared, in world units).
const CORRECTION_EPSILON_SQ: f32 = 0.1;

pub const PLAYER_PREFAB: PrefabId = PrefabId(0);

/// Marker: entity is pending broadcast of EntityDespawned, then despawn.
#[derive(Component)]
pub struct PendingDespawn;

/// Marker: this player entity has not yet been sent a Welcome message.
#[derive(Component)]
pub struct PendingWelcome;

/// The last tick simulated for this player. Used to reject stale inputs.
#[derive(Component, Default)]
pub struct LastSimulatedTick(pub shared::tick::TickNumber);

/// Triggered by bevy_renet when a client connects or disconnects.
pub fn handle_server_events(
    event: On<RenetServerEvent>,
    mut commands: Commands,
    mut registry: ResMut<EntityRegistry>,
    _players: Query<(Entity, &NetworkId)>,
) {
    match **event {
        ServerEvent::ClientConnected { client_id } => {
            info!("Player {client_id} connected");
            let id = NetworkId(client_id);
            let entity = commands.spawn((
                id,
                PLAYER_PREFAB,
                Position(Pos2::ZERO),
                InputQueue::default(),
                LastSimulatedTick::default(),
                PendingWelcome,
            )).id();
            registry.0.insert(id, entity);
        }
        ServerEvent::ClientDisconnected { client_id, reason } => {
            info!("Player {client_id} disconnected: {reason}");
            let id = NetworkId(client_id);
            if let Some(entity) = registry.0.remove(&id) {
                // Defer the actual despawn so broadcast_despawn can send the message first.
                commands.entity(entity).insert(PendingDespawn);
            }
        }
    }
}

/// Reads InputTick messages from all clients and enqueues them in tick order,
/// rejecting ticks already simulated. Also enqueues piggybacked old moves.
pub fn receive_inputs(
    mut server: ResMut<RenetServer>,
    registry: Res<EntityRegistry>,
    mut queues: Query<(&mut InputQueue, &LastSimulatedTick)>,
) {
    for client_id in server.clients_id() {
        let id = NetworkId(client_id);
        let Some(&entity) = registry.0.get(&id) else { continue };
        let Ok((mut queue, last_simulated)) = queues.get_mut(entity) else { continue };

        while let Some(msg) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            match postcard::from_bytes(&msg) {
                Ok(C2S::InputTick { current, old }) => {
                    if let Some(old_move) = old {
                        info!("Old move received from {client_id}: tick={}", old_move.tick);
                        enqueue_if_new(&mut queue, last_simulated, old_move, false);
                    }
                    enqueue_if_new(&mut queue, last_simulated, current, true);
                }
                Err(e) => warn!("Failed to deserialize InputTick from {client_id}: {e}"),
            }
        }
    }
}

fn enqueue_if_new(queue: &mut InputQueue, last_simulated: &LastSimulatedTick, mv: MoveInput, verify: bool) {
    if mv.tick <= last_simulated.0 {
        return;
    }
    let pos = queue.0.partition_point(|(e, _)| e.tick < mv.tick);
    if queue.0.get(pos).map_or(true, |(e, _)| e.tick != mv.tick) {
        queue.0.insert(pos, (mv, verify));
    }
}

/// Drains each player's input queue, simulates movement, and issues corrections
/// or acks depending on whether the client-reported position was correct.
pub fn tick_game(
    mut server: ResMut<RenetServer>,
    mut players: Query<(&NetworkId, &mut Position, &mut InputQueue, &mut LastSimulatedTick)>,
) {
    for (net_id, mut pos, mut queue, mut last_simulated) in &mut players {
        let client_id: ClientId = net_id.0;

        while let Some((mv, verify)) = queue.0.pop_front() {
            // Simulate all ticks from last_simulated+1 up to mv.tick.
            let steps = mv.tick.saturating_sub(last_simulated.0);
            for _ in 0..steps {
                pos.0 = apply_input(pos.0, mv.input);
            }
            last_simulated.0 = mv.tick;

            if verify {
                let dx = pos.0.x - mv.pos.x;
                let dy = pos.0.y - mv.pos.y;
                let msg = if dx * dx + dy * dy > CORRECTION_EPSILON_SQ {
                    S2C::Correction { tick: mv.tick, pos: pos.0 }
                } else {
                    S2C::Ack { tick: mv.tick }
                };
                if let Ok(bytes) = postcard::to_allocvec(&msg) {
                    server.send_message(client_id, DefaultChannel::ReliableOrdered, bytes);
                }
            }
        }
    }
}

/// Broadcasts the full world snapshot to all clients.
pub fn broadcast_state(
    mut server: ResMut<RenetServer>,
    time: Res<Time<Real>>,
    players: Query<(&NetworkId, &Position)>,
) {
    let snapshot: Vec<PlayerState> = players
        .iter()
        .map(|(net_id, pos)| PlayerState { id: *net_id, pos: pos.0 })
        .collect();

    let msg = S2C::WorldSnapshot { server_time: time.elapsed_secs_f64(), players: snapshot };
    if let Ok(bytes) = postcard::to_allocvec(&msg) {
        server.broadcast_message(DefaultChannel::Unreliable, bytes);
    }
}

/// For every newly connected player:
/// - Sends EntitySpawned for all existing entities to the new client.
/// - Broadcasts EntitySpawned for the new player to all other clients.
pub fn send_initial_state(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    pending: Query<(Entity, &NetworkId), With<PendingWelcome>>,
    existing: Query<(&NetworkId, &PrefabId, &Position), Without<PendingWelcome>>,
) {
    for (entity, net_id) in &pending {
        // Send EntitySpawned for every already-existing entity to the new client.
        for (existing_id, prefab, pos) in &existing {
            let owner = Some(existing_id.0);
            let msg = S2C::EntitySpawned { id: *existing_id, prefab: *prefab, pos: pos.0, owner };
            if let Ok(bytes) = postcard::to_allocvec(&msg) {
                server.send_message(net_id.0, DefaultChannel::ReliableOrdered, bytes);
            }
        }

        // Send this player's own EntitySpawned to them and broadcast to all others.
        let owner = Some(net_id.0);
        let spawned = S2C::EntitySpawned { id: *net_id, prefab: PLAYER_PREFAB, pos: Pos2::ZERO, owner };
        if let Ok(bytes) = postcard::to_allocvec(&spawned) {
            server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);
        }

        commands.entity(entity).remove::<PendingWelcome>();
    }
}

/// Broadcasts EntityDespawned for disconnected players then despawns their entities.
pub fn broadcast_despawn(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    pending: Query<(Entity, &NetworkId), With<PendingDespawn>>,
) {
    for (entity, net_id) in &pending {
        let msg = S2C::EntityDespawned { id: *net_id };
        if let Ok(bytes) = postcard::to_allocvec(&msg) {
            server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);
        }
        commands.entity(entity).despawn();
    }
}

