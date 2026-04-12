mod game;
mod plugin;
mod resources;

use std::{net::UdpSocket, time::SystemTime};

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_renet::{
    netcode::{NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::ConnectionConfig,
    RenetServer, RenetServerPlugin,
};

pub const PROTOCOL_ID: u64 = 1;
pub const SERVER_ADDR: &str = "0.0.0.0:5000";

fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let public_addr = SERVER_ADDR.parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let server = RenetServer::new(ConnectionConfig::default());

    (server, transport)
}

fn main() {
    let (server, transport) = new_renet_server();

    App::new()
        .add_plugins(bevy::log::LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(bevy::scene::ScenePlugin)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(RenetServerPlugin)
        .add_plugins(NetcodeServerPlugin)
        .add_plugins(plugin::ServerGamePlugin)
        .add_plugins(bevy::window::WindowPlugin::default())
        .insert_resource(server)
        .insert_resource(transport)
        .insert_resource(Time::<Fixed>::from_hz(shared::tick::TICK_RATE as f64))
        .run();
}

