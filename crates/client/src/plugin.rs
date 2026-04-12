use bevy::prelude::*;
use shared::physics::PlayerMovementPlugin;

use crate::{
    game::{
        input::InputPlugin,
        net::NetPlugin,
        prediction::PredictionPlugin,
        render::RenderPlugin,
        scene::ScenePlugin,
    },
    resources::{CurrentInput, EntityRegistry, LocalTick, ServerTime},
};

pub struct ClientGamePlugin;

impl Plugin for ClientGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerMovementPlugin)
            .init_resource::<CurrentInput>()
            .init_resource::<EntityRegistry>()
            .init_resource::<LocalTick>()
            .init_resource::<ServerTime>()
            .add_plugins((ScenePlugin, InputPlugin, NetPlugin, PredictionPlugin, RenderPlugin));
    }
}
