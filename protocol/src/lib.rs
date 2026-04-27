use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use physics::{MovementSpeed, PlayerPhysicsBundle};
use serde::{Deserialize, Serialize};

#[derive(Bundle)]
pub struct PlayerBundle {
    pub id: PlayerId,
    pub physics: PlayerPhysicsBundle,
}

impl PlayerBundle {
    pub fn new(id: PeerId, position: Vec2) -> Self {
        let h = (((id.to_bits().wrapping_mul(30)) % 360) as f32) / 360.0;
        let color = Color::hsl(h, 0.8, 0.5);
        Self {
            id: PlayerId(id),
            physics: PlayerPhysicsBundle::default(),
        }
    }
}

// Components
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LocalPlayer;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

fn lerp_health(start: Health, end: Health, t: f32) -> Health {
    Health {
        current: start.current + (end.current - start.current) * t,
        max: end.max,
    }
}

/// Identifies which character definition this entity uses.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterType(pub String);

// Channels
pub struct Channel1;

// Protocol
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // components
        app.register_component::<PlayerId>();
        app.register_component::<Position>()
            .add_prediction()
            .add_should_rollback(|a: &Position, b: &Position| (a.0 - b.0).length() >= 0.001)
            .add_linear_interpolation();

        app.register_component::<Rotation>()
            .add_prediction()
            .add_should_rollback(|a: &Rotation, b: &Rotation| false);

        app.register_component::<Health>();

        app.register_component::<MovementSpeed>()
            .add_prediction()
            .add_should_rollback(|a: &MovementSpeed, b: &MovementSpeed| {
                (a.0 - b.0).abs() >= 0.001
            });

        app.register_component::<CharacterType>();

        // channels
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);
    }
}
