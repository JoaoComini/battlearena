use abilities::types::{AbilityCooldowns, AbilityDef, AbilityLoadout, AbilitySlot};
use bevy::prelude::*;
use physics::MovementSpeed;
use protocol::{CharacterType, Health};
use serde::{Deserialize, Serialize};

/// Serialized form — ability slots are asset path strings.
#[derive(Serialize, Deserialize)]
pub(crate) struct CharacterDefRaw {
    pub key: String,
    pub max_health: f32,
    pub move_speed: f32,
    pub ability_slots: Vec<String>,
    pub playable: bool,
}

#[derive(Asset, Reflect, Clone, Debug)]
pub struct CharacterDef {
    pub key: String,
    pub max_health: f32,
    pub move_speed: f32,
    pub ability_slots: Vec<Handle<AbilityDef>>,
    pub playable: bool,
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub health: Health,
    pub move_speed: MovementSpeed,
    pub character_type: CharacterType,
    pub loadout: AbilityLoadout,
    pub cooldowns: AbilityCooldowns,
}

impl CharacterDef {
    pub fn to_bundle(&self) -> CharacterBundle {
        let slot_count = self.ability_slots.len();
        CharacterBundle {
            health: Health {
                current: self.max_health,
                max: self.max_health,
            },
            move_speed: MovementSpeed(self.move_speed),
            character_type: CharacterType(self.key.clone()),
            loadout: AbilityLoadout {
                slots: self
                    .ability_slots
                    .iter()
                    .map(|h| AbilitySlot::new(h.clone()))
                    .collect(),
            },
            cooldowns: AbilityCooldowns::new(slot_count),
        }
    }
}
