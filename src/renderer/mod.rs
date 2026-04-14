#[cfg(all(any(feature = "gui2d", feature = "gui3d"), feature = "client"))]
pub mod client;
#[cfg(all(any(feature = "gui2d", feature = "gui3d"), feature = "server"))]
pub mod server;

use crate::protocol::*;
use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Clone)]
pub struct BattleArenaRendererPlugin;

impl Plugin for BattleArenaRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);
        app.add_systems(Update, draw_boxes);
    }
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub(crate) fn draw_boxes(mut gizmos: Gizmos, players: Query<(&Position, &PlayerColor)>) {
    for (position, color) in &players {
        gizmos.rect_2d(
            Isometry2d::from_translation(position.0),
            Vec2::ONE * PLAYER_SIZE,
            color.0,
        );
    }
}
