use serde::{Deserialize, Serialize};

pub use crate::components::NetworkId;

/// 2D position in world-space units.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
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
