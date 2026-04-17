use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use clap::Parser;
use core::time::Duration;
use lightyear::link::RecvLinkConditioner;
use lightyear::prelude::client::*;
use shared::{CLIENT_PORT, FIXED_TIMESTEP_HZ, SERVER_ADDR, SHARED_SETTINGS};

mod setup;
mod systems;

use abilities::AbilityPlugin;
use setup::{connect, BattleArenaClient, ClientTransports};
use systems::BattleArenaClientPlugin;

use {
    bevy::window::PresentMode, bevy::winit::WinitSettings,
    renderer::client::BattleArenaClientRendererPlugin, renderer::BattleArenaRendererPlugin,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short = 'c', long, default_value = None)]
    client_id: Option<u64>,
}

fn main() {
    let cli = Cli::parse();
    let client_id = cli
        .client_id
        .expect("You need to specify a client_id via `-c ID`");
    let tick_duration = Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ);

    let mut app = build_app(tick_duration, client_id);

    app.add_plugins(shared::SharedPlugin);
    app.add_plugins(BattleArenaClientPlugin);
    app.add_plugins(AbilityPlugin);

    app.world_mut().spawn(BattleArenaClient {
        client_id,
        client_port: CLIENT_PORT,
        server_addr: SERVER_ADDR,
        conditioner: Some(RecvLinkConditioner::new(
            lightyear::prelude::LinkConditionerConfig::average_condition(),
        )),
        transport: ClientTransports::Udp,
        shared: SHARED_SETTINGS,
    });
    app.add_systems(Startup, connect);

    app.add_plugins((
        BattleArenaRendererPlugin,
        BattleArenaClientRendererPlugin::new(format!("Client {client_id}")),
    ));

    app.run();
}

fn build_app(tick_duration: Duration, client_id: u64) -> App {
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
                        title: format!("BattleArena: Client {client_id}"),
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
