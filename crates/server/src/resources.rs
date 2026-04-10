use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use shared::{protocol::MoveInput, types::{NetworkId, Pos2}};

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
