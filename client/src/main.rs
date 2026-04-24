use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use shared::dummy::DummyPlugin;
use core::time::Duration;
use lightyear::prelude::client::*;
use shared::FIXED_TIMESTEP_HZ;

mod menu;
mod setup;
mod systems;

use abilities::{client::AbilityClientPlugin, AbilityPlugin};
use menu::MenuPlugin;
use systems::BattleArenaClientPlugin;

use {
    bevy::window::PresentMode, bevy::winit::WinitSettings,
    renderer::client::BattleArenaClientRendererPlugin, renderer::BattleArenaRendererPlugin,
};

fn main() {
    let tick_duration = Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ);

    let mut app = build_app(tick_duration);

    app.add_plugins(shared::SharedPlugin);
    app.add_plugins(BattleArenaClientPlugin);
    app.add_plugins(AbilityPlugin);
    app.add_plugins(AbilityClientPlugin);
    app.add_plugins(DummyPlugin);
    app.add_plugins(MenuPlugin);

    app.add_plugins((
        BattleArenaRendererPlugin,
        BattleArenaClientRendererPlugin::new("BattleArena".to_string()),
    ));

    app.run();
}

fn build_app(tick_duration: Duration) -> App {
    let mut app = App::new();
    app.add_plugins(assets::AssetPlugin);
    app.add_plugins(
            bevy::DefaultPlugins
                .build()
                .set(bevy::asset::AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::INFO,
                    filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_enhanced_input::action::fns=error".to_string(),
                    ..default()
                })
                .set(bevy::window::WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        title: "BattleArena".to_string(),
                        resolution: (1024, 768).into(),
                        present_mode: PresentMode::AutoVsync,
                        prevent_default_event_handling: true,
                        ..Default::default()
                    }),
                    ..default()
                }),
        );
    app.insert_resource(WinitSettings::continuous());
    app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    app.add_plugins(ClientPlugins { tick_duration });
    app
}
