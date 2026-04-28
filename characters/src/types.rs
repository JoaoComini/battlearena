use abilities::types::{AbilityCast, AbilityLoadout, AbilitySlot};
use bevy::prelude::*;
use physics::MovementSpeed;
use protocol::{CharacterType, Health};
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

#[derive(Bundle)]
pub struct CharacterBundle {
    pub health: Health,
    pub move_speed: MovementSpeed,
    pub character_type: CharacterType,
    pub loadout: AbilityLoadout,
    pub ability_cast: AbilityCast,
}

impl CharacterDef {
    pub fn to_bundle(&self) -> CharacterBundle {
        CharacterBundle {
            health: Health { current: self.max_health, max: self.max_health },
            move_speed: MovementSpeed(self.move_speed),
            character_type: CharacterType(self.key.clone()),
            loadout: AbilityLoadout {
                slots: self.ability_slots.iter().map(|k| AbilitySlot::new(k)).collect(),
            },
            ability_cast: AbilityCast::default(),
        }
    }
}
