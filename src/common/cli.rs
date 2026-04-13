#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use core::str::FromStr;
use core::time::Duration;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::state::app::StatesPlugin;
use clap::{Parser, Subcommand};

#[cfg(feature = "client")]
use crate::common::client::{ExampleClient, connect};
#[cfg(all(feature = "gui", feature = "client"))]
use crate::common::client_renderer::ExampleClientRendererPlugin;
#[cfg(feature = "server")]
use crate::common::server::{ExampleServer, ServerTransports, start};
#[cfg(all(feature = "gui", feature = "server"))]
use crate::common::server_renderer::ExampleServerRendererPlugin;
use crate::common::shared::{CLIENT_PORT, SERVER_ADDR, SERVER_PORT, SHARED_SETTINGS};
use lightyear::link::RecvLinkConditioner;
#[cfg(feature = "client")]
use lightyear::prelude::client::*;
use lightyear::prelude::*;
#[cfg(feature = "gui")]
use {
    bevy::window::PresentMode,
    bevy::winit::{UpdateMode, WinitSettings},
};

/// CLI options to create an [`App`]
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Option<Mode>,
}

impl Cli {
    pub fn client_id(&self) -> Option<u64> {
        match &self.mode {
            #[cfg(feature = "client")]
            Some(Mode::Client { client_id }) => *client_id,
            #[cfg(all(feature = "client", feature = "server"))]
            Some(Mode::HostClient { client_id }) => *client_id,
            _ => None,
        }
    }

    pub fn create_app(add_inspector: bool) -> App {
        #[cfg(feature = "gui")]
        let app = new_gui_app(add_inspector);
        #[cfg(not(feature = "gui"))]
        let app = new_headless_app();
        app
    }

    pub fn build_app(&self, tick_duration: Duration, add_inspector: bool) -> App {
        let mut app = Cli::create_app(add_inspector);
        match self.mode {
            #[cfg(feature = "client")]
            Some(Mode::Client { client_id }) => {
                app.add_plugins((
                    lightyear::prelude::client::ClientPlugins { tick_duration },
                    #[cfg(feature = "gui")]
                    ExampleClientRendererPlugin::new(format!("Client {client_id:?}")),
                ));
                app
            }
            #[cfg(feature = "server")]
            Some(Mode::Server) => {
                app.add_plugins((
                    lightyear::prelude::server::ServerPlugins { tick_duration },
                    #[cfg(feature = "gui")]
                    ExampleServerRendererPlugin::new("Server".to_string()),
                ));
                app
            }
            #[cfg(all(feature = "client", feature = "server"))]
            Some(Mode::HostClient { client_id }) => {
                app.add_plugins((
                    lightyear::prelude::client::ClientPlugins { tick_duration },
                    lightyear::prelude::server::ServerPlugins { tick_duration },
                    #[cfg(feature = "gui")]
                    ExampleClientRendererPlugin::new(format!("Host-Client {client_id:?}")),
                    #[cfg(feature = "gui")]
                    ExampleServerRendererPlugin::new("Host-Server".to_string()),
                ));
                app
            }
            None => {
                panic!("Mode is required");
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn spawn_connections(&self, app: &mut App) {
        match self.mode {
            #[cfg(feature = "client")]
            Some(Mode::Client { client_id }) => {
                app.world_mut().spawn(ExampleClient {
                    client_id: client_id.expect("You need to specify a client_id via `-c ID`"),
                    client_port: CLIENT_PORT,
                    server_addr: SERVER_ADDR,
                    conditioner: None,
                    shared: SHARED_SETTINGS,
                });
                app.add_systems(Startup, connect);
            }
            #[cfg(feature = "server")]
            Some(Mode::Server) => {
                app.world_mut().spawn(ExampleServer {
                    conditioner: None,
                    transport: ServerTransports {
                        local_port: SERVER_PORT,
                    },
                    shared: SHARED_SETTINGS,
                });
                app.add_systems(Startup, start);
            }
            #[cfg(all(feature = "client", feature = "server"))]
            Some(Mode::HostClient { client_id }) => {
                let server = app.world_mut().spawn(ExampleServer {
                    conditioner: None,
                    transport: ServerTransports {
                        local_port: SERVER_PORT,
                    },
                    shared: SHARED_SETTINGS,
                }).id();

                app.world_mut().spawn((
                    Client::default(),
                    Name::new("HostClient"),
                    LinkOf { server },
                ));
                app.add_systems(Startup, (start, connect).chain());
            }
            _ => {}
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    #[cfg(feature = "client")]
    Client {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
    #[cfg(feature = "server")]
    Server,
    #[cfg(all(feature = "client", feature = "server"))]
    HostClient {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
}

impl Default for Mode {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "client", feature = "server"))] {
                Mode::HostClient { client_id: None }
            } else if #[cfg(feature = "server")] {
                Mode::Server
            } else {
                Mode::Client { client_id: None }
            }
        }
    }
}

impl Default for Cli {
    fn default() -> Self {
        cli()
    }
}

pub fn cli() -> Cli {
    Cli::parse()
}

#[cfg(feature = "gui")]
pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("BattleArena"),
            resolution: (1024, 768).into(),
            present_mode: PresentMode::AutoVsync,
            prevent_default_event_handling: true,
            ..Default::default()
        }),
        ..default()
    }
}

pub fn log_plugin() -> LogPlugin {
    LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn".to_string(),
        ..default()
    }
}

#[cfg(feature = "gui")]
pub fn new_gui_app(add_inspector: bool) -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(bevy::asset::AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .set(window_plugin()),
    );
    app.insert_resource(WinitSettings::continuous());
    app
}

pub fn new_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        log_plugin(),
        StatesPlugin,
        DiagnosticsPlugin,
    ));
    app
}
