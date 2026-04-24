use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Asset, Reflect, Serialize, Deserialize, Clone, Debug)]
pub struct CharacterDef {
    pub key: String,
    pub max_health: f32,
    pub move_speed: f32,
    /// Ordered ability keys assigned to slots, e.g. ["melee"] for slot1 only.
    pub ability_slots: Vec<String>,
    pub playable: bool,
}
