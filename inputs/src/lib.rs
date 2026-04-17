use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Reflect)]
pub struct Direction {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub angle: f32, // radians, player-to-mouse in world space
}

impl Direction {
    pub fn is_none(&self) -> bool {
        !self.up && !self.down && !self.left && !self.right
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Reflect)]
pub struct AbilityInput {
    pub slot1: bool, // Q key
    pub slot2: bool, // E key
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Reflect)]
pub struct PlayerInput {
    pub movement: Direction,
    pub abilities: AbilityInput,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Reflect)]
pub enum Inputs {
    PlayerInput(PlayerInput),
}

impl Default for Inputs {
    fn default() -> Self {
        Self::PlayerInput(PlayerInput::default())
    }
}

impl MapEntities for Inputs {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

#[derive(Clone)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // inputs
        app.add_plugins(input::native::InputPlugin::<Inputs>::default());
    }
}
