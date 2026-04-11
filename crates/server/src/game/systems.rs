use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_renet::{
    renet::{ClientId, DefaultChannel, ServerEvent},
    RenetServer, RenetServerEvent,
};
use shared::{
    physics::{PhysicsInput, PLAYER_RADIUS},
    protocol::{C2S, MoveInput, S2C},
    types::{NetworkId, PlayerState, Pos2, PrefabId},
};

use crate::resources::{InputQueue, EntityRegistry, PlayerPosition};

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

/// Pending verification data: after the shared movement system runs, compare
/// the authoritative Position against the client-reported position.
#[derive(Component)]
pub struct PendingVerification {
    pub tick: shared::tick::TickNumber,
    pub reported_pos: Pos2,
    pub client_id: ClientId,
}

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
    if queue.0.get(pos).map_or(true, |(e, _)| e.tick != mv.tick) {
        queue.0.insert(pos, (mv, verify));
    }
}

/// Drains one input per player from the queue and writes PhysicsInput for the
/// shared movement system to consume. Records PendingVerification when needed.
pub fn prepare_physics_inputs(
    mut commands: Commands,
    mut players: Query<(Entity, &NetworkId, &mut InputQueue, &mut LastSimulatedTick)>,
) {
    for (entity, net_id, mut queue, mut last_simulated) in &mut players {
        let Some((mv, verify)) = queue.0.pop_front() else { continue };

        last_simulated.0 = mv.tick;
        commands.entity(entity).insert(PhysicsInput(mv.input));

        if verify {
            commands.entity(entity).insert(PendingVerification {
                tick: mv.tick,
                reported_pos: mv.pos,
                client_id: net_id.0,
            });
        }
    }
}

/// After the shared movement system has flushed Position, compare the authoritative
/// position against what the client reported and send Ack or Correction.
pub fn verify_and_respond(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut players: Query<(Entity, &Position, &PendingVerification, &mut PlayerPosition)>,
) {
    for (entity, position, verification, mut player_pos) in &mut players {
        let server_pos = Pos2 { x: position.0.x, y: position.0.y };
        player_pos.0 = server_pos;

        let dx = server_pos.x - verification.reported_pos.x;
        let dy = server_pos.y - verification.reported_pos.y;
        let msg = if dx * dx + dy * dy > CORRECTION_EPSILON_SQ {
            S2C::Correction { tick: verification.tick, pos: server_pos }
        } else {
            S2C::Ack { tick: verification.tick }
        };

        if let Ok(bytes) = postcard::to_allocvec(&msg) {
            server.send_message(verification.client_id, DefaultChannel::ReliableOrdered, bytes);
        }

        commands.entity(entity).remove::<PendingVerification>();
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
