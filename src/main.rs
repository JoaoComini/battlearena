//! BattleArena — migrated to lightyear.
//!
//! Run with:
//!   cargo run -- server
//!   cargo run -- client -c 1

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use battlearena::shared::{self, SharedPlugin};

#[cfg(feature = "client")]
use battlearena::client::ExampleClientPlugin;
#[cfg(feature = "server")]
use battlearena::server::ExampleServerPlugin;
#[cfg(feature = "gui")]
use battlearena::renderer::ExampleRendererPlugin;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use clap::{Parser, Subcommand};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;

#[cfg(feature = "server")]
use lightyear::prelude::server::*;
#[cfg(feature = "client")]
use lightyear::prelude::client::*;
use lightyear::prelude::*;

// ── Constants ────────────────────────────────────────────────────────────────

const SERVER_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), shared::SERVER_PORT);

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    #[cfg(feature = "server")]
    Server,

    #[cfg(feature = "client")]
    Client {
        #[arg(short, long, default_value = "1")]
        client_id: u64,
    },
}

// ── App builder ──────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let tick_duration = Duration::from_secs_f64(1.0 / shared::TICK_RATE);

    let mut app = build_app(&cli.mode, tick_duration);

    app.add_plugins(SharedPlugin);
    spawn_connections(&mut app, &cli.mode);

    match &cli.mode {
        #[cfg(feature = "client")]
        Mode::Client { .. } => {
            app.add_plugins(ExampleClientPlugin);
            #[cfg(feature = "gui")]
            app.add_plugins(ExampleRendererPlugin);
        }
        #[cfg(feature = "server")]
        Mode::Server => {
            app.add_plugins(ExampleServerPlugin);
        }
    }

    app.run();
}

// ── App construction ─────────────────────────────────────────────────────────

fn build_app(mode: &Mode, tick_duration: Duration) -> App {
    let mut app = App::new();

    match mode {
        #[cfg(feature = "server")]
        Mode::Server => {
            app.add_plugins((
                MinimalPlugins,
                log_plugin(),
                bevy::state::app::StatesPlugin,
                bevy::diagnostic::DiagnosticsPlugin,
                lightyear::prelude::server::ServerPlugins { tick_duration },
            ));
        }
        #[cfg(feature = "client")]
        Mode::Client { .. } => {
            app.add_plugins((
                DefaultPlugins
                    .build()
                    .set(log_plugin())
                    .set(window_plugin()),
                lightyear::prelude::client::ClientPlugins { tick_duration },
            ));
            app.insert_resource(bevy::winit::WinitSettings::continuous());
        }
    }

    app
}

// ── Connection spawning ───────────────────────────────────────────────────────

fn spawn_connections(app: &mut App, mode: &Mode) {
    match mode {
        #[cfg(feature = "server")]
        Mode::Server => {
            spawn_server(app);
        }
        #[cfg(feature = "client")]
        Mode::Client { client_id } => {
            spawn_client(app, *client_id);
        }
    }
}

// ── Server helpers ────────────────────────────────────────────────────────────

#[cfg(feature = "server")]
fn spawn_server(app: &mut App) {
    use lightyear::netcode::server_plugin::NetcodeConfig;

    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), shared::SERVER_PORT);
    app.world_mut().spawn((
        Name::new("Server"),
        LocalAddr(server_addr),
        ServerUdpIo::default(),
        NetcodeServer::new(NetcodeConfig {
            protocol_id: shared::PROTOCOL_ID,
            private_key: shared::PRIVATE_KEY,
            ..Default::default()
        }),
    ));

    app.add_systems(Startup, start_server);
}

#[cfg(feature = "server")]
fn start_server(mut commands: Commands, server: Single<Entity, With<Server>>) {
    commands.trigger(Start {
        entity: server.into_inner(),
    });
}

// ── Client helpers ────────────────────────────────────────────────────────────

#[cfg(feature = "client")]
fn spawn_client(app: &mut App, client_id: u64) {
    use lightyear::netcode::client_plugin::NetcodeConfig;
    use lightyear::link::RecvLinkConditioner;

    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), shared::CLIENT_PORT);
    let auth = Authentication::Manual {
        server_addr: SERVER_ADDR,
        client_id,
        private_key: shared::PRIVATE_KEY,
        protocol_id: shared::PROTOCOL_ID,
    };
    let netcode_config = NetcodeConfig {
        client_timeout_secs: 3,
        token_expire_secs: -1,
        ..default()
    };

    app.world_mut().spawn((
        Client::default(),
        Link::new(None::<RecvLinkConditioner>),
        LocalAddr(client_addr),
        PeerAddr(SERVER_ADDR),
        ReplicationReceiver::default(),
        PredictionManager::default(),
        Name::new("Client"),
        UdpIo::default(),
        NetcodeClient::new(auth, netcode_config).expect("failed to create NetcodeClient"),
    ));

    app.add_systems(Startup, connect_client);
}

#[cfg(feature = "client")]
fn connect_client(mut commands: Commands, client: Single<Entity, With<Client>>) {
    commands.trigger(Connect {
        entity: client.into_inner(),
    });
}

// ── Bevy plugin helpers ───────────────────────────────────────────────────────

fn log_plugin() -> LogPlugin {
    LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn".to_string(),
        ..default()
    }
}

#[cfg(feature = "gui")]
fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "BattleArena".to_string(),
            resolution: (1024u32, 768u32).into(),
            ..default()
        }),
        ..default()
    }
}

#[cfg(not(feature = "gui"))]
fn window_plugin() -> WindowPlugin {
    WindowPlugin::default()
}
