use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use physics::PlayerPhysicsBundle;
use serde::{Deserialize, Serialize};

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

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Dummy;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LocalPlayer;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

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

// Channels
pub struct Channel1;

// Protocol
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // components
        app.register_component::<PlayerId>();
        app.register_component::<PlayerColor>();
        app.register_component::<Position>()
            .add_prediction()
            .add_should_rollback(|a: &Position, b: &Position| (a.0 - b.0).length() >= 0.001)
            .add_linear_interpolation();

        app.register_component::<Rotation>()
            .add_prediction()
            .add_should_rollback(|a: &Rotation, b: &Rotation| false);

        app.register_component::<Health>()
            .add_prediction()
            .add_should_rollback(|a: &Health, b: &Health| (a.current - b.current).abs() >= 0.001)
            .add_interpolation_with(lerp_health);

        app.register_component::<Dummy>();

        // channels
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);
    }
}
