use bevy::prelude::*;
use bevy_renet::{
    renet::{ClientId, DefaultChannel, ServerEvent},
    RenetServer, RenetServerEvent,
};
use shared::{
    logic::apply_input,
    protocol::{C2S, S2C},
    types::{NetworkId, PlayerState, Pos2, PrefabId},
};

use crate::resources::{CurrentTick, InputEntry, InputQueue, EntityRegistry, Position};

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

/// The tick number the server last simulated for this player.
/// Sent back in WorldSnapshot so clients know what has been authoritative.
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

/// Reads InputTick messages from all clients and enqueues them in tick order.
pub fn receive_inputs(
    mut server: ResMut<RenetServer>,
    registry: Res<EntityRegistry>,
    mut queues: Query<&mut InputQueue>,
) {
    for client_id in server.clients_id() {
        let id = NetworkId(client_id);
        let Some(&entity) = registry.0.get(&id) else { continue };
        let Ok(mut queue) = queues.get_mut(entity) else { continue };

        while let Some(msg) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            if let Ok(C2S::InputTick { tick, input, pos: client_pos }) = postcard::from_bytes(&msg) {
                let entry = InputEntry { tick, input, client_pos };
                let pos = queue.0.partition_point(|e| e.tick < tick);
                if queue.0.get(pos).map_or(true, |e| e.tick != tick) {
                    queue.0.insert(pos, entry);
                }
            }
        }
    }
}

/// Drains each player's input queue, simulates movement, and issues corrections
/// when the client-reported position diverges from the server's simulation.
pub fn tick_game(
    mut server: ResMut<RenetServer>,
    mut players: Query<(
        &NetworkId,
        &mut Position,
        &mut InputQueue,
        &mut LastSimulatedTick,
    )>,
) {
    for (net_id, mut pos, mut queue, mut last_tick) in &mut players {
        let client_id: ClientId = net_id.0;

        while let Some(entry) = queue.0.pop_front() {
            let server_pos = apply_input(pos.0, entry.input);
            pos.0 = server_pos;
            last_tick.0 = entry.tick;

            let dx = server_pos.x - entry.client_pos.x;
            let dy = server_pos.y - entry.client_pos.y;
            if dx * dx + dy * dy > CORRECTION_EPSILON_SQ {
                let msg = S2C::Correction { tick: entry.tick, pos: server_pos };
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
    tick: Res<CurrentTick>,
    players: Query<(&NetworkId, &Position, &LastSimulatedTick)>,
) {
    let snapshot: Vec<PlayerState> = players
        .iter()
        .map(|(net_id, pos, last_tick)| PlayerState {
            id: *net_id,
            pos: pos.0,
            ack_tick: last_tick.0,
        })
        .collect();

    let msg = S2C::WorldSnapshot { tick: tick.0, players: snapshot };
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

pub fn advance_tick(mut tick: ResMut<CurrentTick>) {
    tick.0 = tick.0.wrapping_add(1);
}
