use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use shared::{protocol::InputBits, tick::TickNumber, types::{NetworkId, Pos2}};

#[derive(Component)]
pub struct Position(pub Pos2);

/// One entry in the server-side input queue for a player.
pub struct InputEntry {
    pub tick: TickNumber,
    pub input: InputBits,
    /// Position the client reported after simulating this tick.
    pub client_pos: Pos2,
}

/// Ordered queue of inputs received from the client, sorted by tick ascending.
/// The server drains this each fixed step, simulating one entry per tick.
#[derive(Component, Default)]
pub struct InputQueue(pub VecDeque<InputEntry>);

#[derive(Resource, Default)]
pub struct CurrentTick(pub TickNumber);

/// O(1) reverse lookup from NetworkId to the corresponding player Entity.
/// Kept in sync by handle_server_events.
#[derive(Resource, Default)]
pub struct PlayerRegistry(pub HashMap<NetworkId, Entity>);
