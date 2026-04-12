use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use shared::{components::NetworkId, protocol::MoveInput, types::Pos2};

#[derive(Component)]
pub struct PlayerPosition(pub Pos2);

/// Ordered queue of inputs received from the client, sorted by tick ascending.
/// The bool indicates whether to verify the client pos and send ack/correction.
#[derive(Component, Default)]
pub struct InputQueue(pub VecDeque<(MoveInput, bool)>);

/// O(1) reverse lookup from NetworkId to the corresponding Bevy Entity.
/// Covers all networked entities, not just players.
#[derive(Resource, Default)]
pub struct EntityRegistry(pub HashMap<NetworkId, Entity>);

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

/// How far the client-reported position may differ from the server's before
/// a correction is issued (squared, in world units).
pub const CORRECTION_EPSILON_SQ: f32 = 0.1;

pub const PLAYER_PREFAB: shared::components::PrefabId = shared::components::PrefabId(0);
