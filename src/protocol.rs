use avian2d::prelude::*;
use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub const PLAYER_SIZE: f32 = 50.0;

// ── Character stats (loaded from .ron files) ──────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub enum AttackType {
    Melee,
    Ranged,
}

/// Loaded from `assets/characters/<name>.ron`. Not a Component — used to seed
/// per-entity stat components at spawn time.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CharacterStats {
    pub color: CharacterColor,
    pub max_health: f32,
    pub move_speed: f32,
    pub attack_type: AttackType,
}

/// HSL color representation for RON files.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CharacterColor {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
    pub alpha: f32,
}

impl From<CharacterColor> for Color {
    fn from(c: CharacterColor) -> Self {
        Color::hsla(c.hue, c.saturation, c.lightness, c.alpha)
    }
}

impl CharacterStats {
    /// Load stats from `assets/characters/<filename>.ron` using std::fs.
    pub fn load(filename: &str) -> Result<Self, String> {
        let path = format!("assets/characters/{filename}");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {path}: {e}"))?;
        ron::from_str(&content).map_err(|e| format!("Failed to parse {path}: {e}"))
    }
}

// ── Character kind ────────────────────────────────────────────────────────────

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Reflect, Default, Hash)]
pub enum CharacterKind {
    #[default]
    Peta,
    Comini,
}

impl CharacterKind {
    pub fn ron_filename(self) -> &'static str {
        match self {
            CharacterKind::Peta => "peta.ron",
            CharacterKind::Comini => "comini.ron",
        }
    }

    pub fn load_stats(self) -> Result<CharacterStats, String> {
        CharacterStats::load(self.ron_filename())
    }

    pub fn display_name(self) -> &'static str {
        match self {
            CharacterKind::Peta => "Peta",
            CharacterKind::Comini => "Comini",
        }
    }
}

// ── Per-entity stat components ────────────────────────────────────────────────

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MoveSpeed(pub f32);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self { current: max, max }
    }
}

// ── C2S message ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SelectCharacter(pub CharacterKind);

// ── Player bundles ────────────────────────────────────────────────────────────

#[derive(Bundle)]
pub(crate) struct PlayerPhysicsBundle {
    pub(crate) rigid_body: RigidBody,
    pub(crate) custom_position_integration: CustomPositionIntegration,
    pub(crate) collider: Collider,
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
pub(crate) struct PlayerBundle {
    id: PlayerId,
    color: PlayerColor,
    kind: CharacterKind,
    speed: MoveSpeed,
    health: Health,
    physics: PlayerPhysicsBundle,
}

impl PlayerBundle {
    pub(crate) fn new(id: PeerId, kind: CharacterKind, stats: &CharacterStats) -> Self {
        Self {
            id: PlayerId(id),
            color: PlayerColor(Color::from(stats.color.clone())),
            kind,
            speed: MoveSpeed(stats.move_speed),
            health: Health::full(stats.max_health),
            physics: PlayerPhysicsBundle::default(),
        }
    }
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub(crate) PeerId);

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub(crate) Color);

// ── Channels ──────────────────────────────────────────────────────────────────

pub struct Channel1;
pub struct Channel2;

// ── Inputs ────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Direction {
    pub(crate) up: bool,
    pub(crate) down: bool,
    pub(crate) left: bool,
    pub(crate) right: bool,
}

impl Direction {
    pub(crate) fn is_none(&self) -> bool {
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

// ── Protocol ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // inputs
        app.add_plugins(input::native::InputPlugin::<Inputs>::default());

        // components
        app.register_component::<PlayerId>();
        app.register_component::<PlayerColor>();
        app.register_component::<CharacterKind>();
        app.register_component::<MoveSpeed>();
        app.register_component::<Health>();

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

        app.add_channel::<Channel2>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);

        // messages
        app.register_message::<SelectCharacter>()
            .add_direction(NetworkDirection::ClientToServer);
    }
}
