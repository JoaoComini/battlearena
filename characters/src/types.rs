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

/// Raw status values derived from a `CharacterDef`, used to build player components.
pub struct CharacterStatus {
    pub key: String,
    pub max_health: f32,
    pub move_speed: f32,
    pub ability_slots: Vec<String>,
}

impl CharacterDef {
    pub fn to_status(&self) -> CharacterStatus {
        CharacterStatus {
            key: self.key.clone(),
            max_health: self.max_health,
            move_speed: self.move_speed,
            ability_slots: self.ability_slots.clone(),
        }
    }
}

impl Default for CharacterStatus {
    fn default() -> Self {
        CharacterStatus {
            key: "unknown".to_string(),
            max_health: 100.0,
            move_speed: 200.0,
            ability_slots: vec!["melee".to_string()],
        }
    }
}
