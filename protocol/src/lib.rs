use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use physics::MoveAndSlideBundle;
use serde::{Deserialize, Serialize};

#[derive(Bundle)]
pub struct PlayerBundle {
    pub id: PlayerId,
    pub physics: MoveAndSlideBundle,
}

impl PlayerBundle {
    pub fn new(id: PeerId, position: Vec2) -> Self {
        let h = (((id.to_bits().wrapping_mul(30)) % 360) as f32) / 360.0;
        let color = Color::hsl(h, 0.8, 0.5);
        Self {
            id: PlayerId(id),
            physics: MoveAndSlideBundle::default(),
        }
    }
}

// Components
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LocalPlayer;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Event, Clone, Debug)]
pub struct PlayerDied {
    pub client_id: PeerId,
    pub entity: Entity,
}

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

        app.register_component::<LinearVelocity>()
            .add_prediction()
            .add_should_rollback(|a: &LinearVelocity, b: &LinearVelocity| {
                (a.0 - b.0).length() >= 0.01
            });

    }
}
