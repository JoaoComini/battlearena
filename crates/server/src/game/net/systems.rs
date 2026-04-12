use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_renet::{
    renet::{DefaultChannel, ServerEvent},
    RenetServer, RenetServerEvent,
};
use shared::{
    components::{NetworkId, PrefabId},
    physics::PLAYER_RADIUS,
    protocol::{C2S, MoveInput, S2C},
    types::{PlayerState, Pos2},
};

use crate::resources::{
    EntityRegistry, InputQueue, LastSimulatedTick, PendingDespawn, PendingWelcome, PlayerPosition,
    PLAYER_PREFAB,
};

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
                Name::new(format!("Player_{client_id}")),
                id,
                PLAYER_PREFAB,
                PlayerPosition(Pos2::ZERO),
                InputQueue::default(),
                LastSimulatedTick::default(),
                PendingWelcome,
                RigidBody::Kinematic,
                Position(Vec2::ZERO),
                Collider::circle(PLAYER_RADIUS),
                CustomPositionIntegration,
            )).id();
            registry.0.insert(id, entity);
        }
        ServerEvent::ClientDisconnected { client_id, reason } => {
            info!("Player {client_id} disconnected: {reason}");
            let id = NetworkId(client_id);
            if let Some(entity) = registry.0.remove(&id) {
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
    if queue.0.get(pos).is_none_or(|(e, _)| e.tick != mv.tick) {
        queue.0.insert(pos, (mv, verify));
    }
}

/// Broadcasts the full world snapshot to all clients.
pub fn broadcast_state(
    mut server: ResMut<RenetServer>,
    time: Res<Time<Real>>,
    players: Query<(&NetworkId, &PlayerPosition)>,
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
    existing: Query<(&NetworkId, &PrefabId, &PlayerPosition), Without<PendingWelcome>>,
) {
    for (entity, net_id) in &pending {
        for (existing_id, prefab, pos) in &existing {
            let owner = Some(existing_id.0);
            let msg = S2C::EntitySpawned { id: *existing_id, prefab: *prefab, pos: pos.0, owner };
            if let Ok(bytes) = postcard::to_allocvec(&msg) {
                server.send_message(net_id.0, DefaultChannel::ReliableOrdered, bytes);
            }
        }

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
