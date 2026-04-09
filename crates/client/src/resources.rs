use std::collections::HashMap;
use std::collections::VecDeque;

use bevy::prelude::*;
use shared::{
    protocol::InputBits,
    tick::TickNumber,
    types::{NetworkId, Pos2, PrefabId},
};

/// Marker component for the local player entity.
#[derive(Component)]
pub struct LocalPlayer;

/// The client's own connection id, received from the server in Welcome.
/// Used to recognize which EntitySpawned belongs to the local player.
#[derive(Resource)]
pub struct LocalClientId(pub u64);

/// Frame-level input sampled from the keyboard. Global because input is captured
/// once per frame and consumed by the fixed-update simulation.
#[derive(Resource, Default)]
pub struct CurrentInput(pub InputBits);

/// Client-side predicted position for the local player, updated each fixed tick.
#[derive(Component, Default)]
pub struct PredictedPosition(pub Pos2);

/// Predicted position from the previous fixed tick, used for visual interpolation.
#[derive(Component, Default)]
pub struct PreviousPredictedPosition(pub Pos2);

/// Ring buffer of unacknowledged inputs: (tick, input, post-step position).
/// Entries are removed once the server acknowledges a tick >= their tick.
#[derive(Component, Default)]
pub struct InputHistory(pub VecDeque<(TickNumber, InputBits, Pos2)>);

/// Last authoritative position received from the server for this player entity.
#[derive(Component, Default)]
pub struct ServerPosition(pub Pos2);

/// O(1) reverse lookup from NetworkId to the corresponding player Entity.
/// Kept in sync by recv_handshake and recv_entity_spawned.
#[derive(Resource, Default)]
pub struct PlayerRegistry(pub HashMap<NetworkId, Entity>);

/// Spawn function signature: given Commands, the entity, its initial position,
/// and the owner's client id (if any), attach whatever components are needed.
pub type SpawnFn = Box<dyn Fn(&mut Commands, Entity, Pos2, Option<u64>) + Send + Sync>;

/// Maps PrefabId to a spawn function registered at startup.
#[derive(Resource, Default)]
pub struct SpawnRegistry(pub HashMap<PrefabId, SpawnFn>);
