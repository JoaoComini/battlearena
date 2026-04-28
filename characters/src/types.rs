use abilities::types::{AbilityCooldowns, AbilityDef, AbilityLoadout, AbilitySlot};
use abilities::{Health, MovementSpeed};
use bevy::prelude::*;
use physics::MoveAndSlideBundle;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterId(pub String);

/// Serialized form — ability slots are asset path strings.
#[derive(Serialize, Deserialize)]
pub(crate) struct CharacterDefRaw {
    pub key: String,
    pub max_health: f32,
    pub move_speed: f32,
    pub ability_slots: Vec<String>,
    pub playable: bool,
    pub visual: Option<String>,
}

#[derive(Asset, Reflect, Clone, Debug)]
pub struct CharacterDef {
    pub key: String,
    pub max_health: f32,
    pub move_speed: f32,
    pub ability_slots: Vec<Handle<AbilityDef>>,
    pub playable: bool,
    pub visual: Option<Handle<Scene>>,
}

#[derive(Component, Clone)]
pub struct Character(pub Handle<CharacterDef>);

#[derive(Bundle)]
pub struct CharacterBundle {
    pub health: Health,
    pub move_speed: MovementSpeed,
    pub loadout: AbilityLoadout,
    pub cooldowns: AbilityCooldowns,
    pub physics: MoveAndSlideBundle,
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
            loadout: AbilityLoadout {
                slots: self
                    .ability_slots
                    .iter()
                    .map(|h| AbilitySlot::new(h.clone()))
                    .collect(),
            },
            cooldowns: AbilityCooldowns::new(slot_count),
            physics: MoveAndSlideBundle::default(),
        }
    }
}
