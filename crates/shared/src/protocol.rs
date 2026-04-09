use serde::{Deserialize, Serialize};

use crate::{tick::TickNumber, types::{NetworkId, PlayerState, Pos2, PrefabId}};

/// Client-to-Server messages.
#[derive(Debug, Serialize, Deserialize)]
pub enum C2S {
    /// Sent every tick as an unreliable datagram.
    /// `pos` is the position the client computed *after* applying `input` at `tick`.
    /// The server uses it to detect mispredictions.
    InputTick { tick: TickNumber, input: InputBits, pos: Pos2 },
}

/// Server-to-Client messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum S2C {
    /// Sent on the reliable stream when a new networked entity is created.
    /// `owner` is the renet ClientId of the client that owns this entity, if any.
    EntitySpawned { id: NetworkId, prefab: PrefabId, pos: Pos2, owner: Option<u64> },

    /// Sent on the reliable stream when a networked entity is destroyed.
    EntityDespawned { id: NetworkId },

    /// Sent every server tick as an unreliable datagram to all clients.
    WorldSnapshot { tick: TickNumber, players: Vec<PlayerState> },

    /// Sent on the reliable channel when the server detects a misprediction.
    /// The client must snap to `pos` at `tick` and re-simulate its input buffer.
    Correction { tick: TickNumber, pos: Pos2 },
}

/// Packed bitfield for directional movement input. One byte on the wire.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct InputBits(pub u8);

impl InputBits {
    pub const UP: u8    = 1 << 0;
    pub const DOWN: u8  = 1 << 1;
    pub const LEFT: u8  = 1 << 2;
    pub const RIGHT: u8 = 1 << 3;

    pub fn set(&mut self, bit: u8) {
        self.0 |= bit;
    }

    pub fn is_set(self, bit: u8) -> bool {
        self.0 & bit != 0
    }
}
