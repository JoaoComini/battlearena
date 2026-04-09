use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// Identifies which prefab (spawn template) an entity was created from.
/// The server tags entities with this; clients use it to look up the matching
/// spawn function in their SpawnRegistry.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrefabId(pub u16);

/// Unique identifier for a networked player, shared between client and server.
/// On the server this equals the renet ClientId; the server hands it to the client
/// in the Welcome message so both sides refer to players by the same number.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

/// 2D position in world-space units.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Pos2 {
    pub x: f32,
    pub y: f32,
}

impl Pos2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

/// Authoritative server-side snapshot of one player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: NetworkId,
    pub pos: Pos2,
}
