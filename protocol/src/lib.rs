use avian2d::prelude::*;
use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub const PLAYER_SIZE: f32 = 50.0;

// Player
#[derive(Bundle)]
pub struct PlayerPhysicsBundle {
    pub rigid_body: RigidBody,
    pub custom_position_integration: CustomPositionIntegration,
    pub collider: Collider,
}

impl Default for PlayerPhysicsBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Kinematic,
            custom_position_integration: CustomPositionIntegration,
            collider: Collider::circle(PLAYER_SIZE * 0.5),
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub id: PlayerId,
    pub color: PlayerColor,
    pub physics: PlayerPhysicsBundle,
}

impl PlayerBundle {
    pub fn new(id: PeerId, position: Vec2) -> Self {
        let h = (((id.to_bits().wrapping_mul(30)) % 360) as f32) / 360.0;
        let color = Color::hsl(h, 0.8, 0.5);
        Self {
            id: PlayerId(id),
            color: PlayerColor(color),
            physics: PlayerPhysicsBundle::default(),
        }
    }
}

// Components

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

// Channels
pub struct Channel1;

// Inputs

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Direction {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Direction {
    pub fn is_none(&self) -> bool {
        !self.up && !self.down && !self.left && !self.right
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Reflect)]
pub enum Inputs {
    Direction(Direction),
}

impl Default for Inputs {
    fn default() -> Self {
        Self::Direction(Direction::default())
    }
}

impl MapEntities for Inputs {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

// Protocol
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // inputs
        app.add_plugins(input::native::InputPlugin::<Inputs>::default());

        // components
        app.register_component::<PlayerId>();
        app.register_component::<PlayerColor>();
        app.register_component::<Position>()
            .add_prediction()
            .add_should_rollback(|a: &Position, b: &Position| (a.0 - b.0).length() >= 0.001)
            .add_linear_interpolation();

        // channels
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);
    }
}
