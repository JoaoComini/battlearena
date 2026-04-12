use bevy::ecs::entity::MapEntities;
use bevy::math::Curve;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

// ── Bundles ──────────────────────────────────────────────────────────────────

#[derive(Bundle)]
pub struct PlayerBundle {
    id: PlayerId,
    position: PlayerPosition,
    color: PlayerColor,
}

impl PlayerBundle {
    pub fn new(id: PeerId, position: Vec2) -> Self {
        // Deterministic pseudo-random color from client id.
        let h = (((id.to_bits().wrapping_mul(30)) % 360) as f32) / 360.0;
        Self {
            id: PlayerId(id),
            position: PlayerPosition(position),
            color: PlayerColor(Color::hsl(h, 0.8, 0.5)),
        }
    }
}

// ── Components ───────────────────────────────────────────────────────────────

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Deref, DerefMut)]
pub struct PlayerPosition(pub Vec2);

impl Ease for PlayerPosition {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        FunctionCurve::new(Interval::UNIT, move |t| {
            PlayerPosition(Vec2::lerp(start.0, end.0, t))
        })
    }
}

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

// ── Channels ─────────────────────────────────────────────────────────────────

pub struct Channel1;

// ── Inputs ───────────────────────────────────────────────────────────────────

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

// ── Protocol plugin ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // inputs
        app.add_plugins(input::native::InputPlugin::<Inputs>::default());

        // components
        app.register_component::<PlayerId>();

        app.register_component::<PlayerPosition>()
            .add_prediction()
            .add_linear_interpolation();

        app.register_component::<PlayerColor>();

        // channels
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);
    }
}
