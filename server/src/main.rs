use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use core::time::Duration;
use lightyear::prelude::server::*;
use shared::{FIXED_TIMESTEP_HZ, SERVER_PORT, SHARED_SETTINGS};

mod setup;
mod systems;

use abilities::{AbilityPlugin};
use setup::{BattleArenaServer, ServerTransports, start};
use systems::BattleArenaServerPlugin;

#[cfg(feature = "gui")]
use {
    bevy::window::PresentMode,
    bevy::winit::WinitSettings,
    renderer::BattleArenaRendererPlugin,
    renderer::server::BattleArenaServerRendererPlugin,
};

fn main() {
    let tick_duration = Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ);

    let mut app = build_app(tick_duration);

    app.add_plugins(shared::SharedPlugin);
    app.add_plugins(BattleArenaServerPlugin);
    app.add_plugins(AbilityPlugin);

    app.world_mut().spawn(BattleArenaServer {
        conditioner: None,
        transport: ServerTransports::Udp {
            local_port: SERVER_PORT,
        },
        shared: SHARED_SETTINGS,
    });
    app.add_systems(Startup, start);

    #[cfg(feature = "gui")]
    app.add_plugins((
        BattleArenaRendererPlugin,
        BattleArenaServerRendererPlugin::new("Server".to_string()),
    ));

    app.run();
}

fn build_app(tick_duration: Duration) -> App {
    #[cfg(feature = "gui")]
    {
        let mut app = App::new();
        app.add_plugins(assets::AssetPlugin);
        app.add_plugins(
            bevy::DefaultPlugins
                .build()
                .set(LogPlugin {
                    level: Level::INFO,
                    filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_enhanced_input::action::fns=error".to_string(),
                    ..default()
                })
                .set(bevy::window::WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        title: "BattleArena: Server".to_string(),
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
        app.add_plugins(ServerPlugins { tick_duration });
        app
    }
    #[cfg(not(feature = "gui"))]
    {
        let mut app = App::new();
        app.add_plugins(assets::AssetPlugin);
        app.add_plugins((
            bevy::app::ScheduleRunnerPlugin::default(),
            LogPlugin {
                level: Level::INFO,
                filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_enhanced_input::action::fns=error".to_string(),
                ..default()
            },
            StatesPlugin,
            DiagnosticsPlugin,
        ));
        app.add_plugins(ServerPlugins { tick_duration });
        app
    }
}
