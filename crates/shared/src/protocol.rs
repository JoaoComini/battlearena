use serde::{Deserialize, Serialize};

use crate::{
    components::{NetworkId, PrefabId},
    tick::TickNumber,
    types::{PlayerState, Pos2},
};

/// A single client move: the input held from the previous move up to and
/// including `tick`, and the predicted position after simulating `tick`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MoveInput {
    pub tick: TickNumber,
    pub input: InputBits,
    pub pos: Pos2,
}

/// Client-to-Server messages.
#[derive(Debug, Serialize, Deserialize)]
pub enum C2S {
    /// Sent every fixed tick as an unreliable datagram.
    /// `current` is the latest move (may span multiple ticks with the same input).
    /// `old_move` is the oldest unacknowledged direction change, piggybacked for
    /// loss recovery.
    InputTick { current: MoveInput, old: Option<MoveInput> },
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
    /// `server_time` is the server's elapsed time in seconds at the moment of broadcast.
    WorldSnapshot { server_time: f64, players: Vec<PlayerState> },

    /// Sent on the reliable channel when the server detects a misprediction.
    /// The client must snap to `pos` at `tick` and re-simulate its input buffer.
    Correction { tick: TickNumber, pos: Pos2 },

    /// Sent on the reliable channel when the server has processed a client's input
    /// and the prediction was correct. The client can trim history up to this tick.
    Ack { tick: TickNumber },
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
