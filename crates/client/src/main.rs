mod game;
mod plugin;
mod resources;

use std::{net::UdpSocket, time::SystemTime};

use avian2d::prelude::PhysicsPlugins;
use bevy::prelude::*;
use bevy_renet::{
    netcode::{ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport},
    renet::ConnectionConfig,
    RenetClient, RenetClientPlugin,
};

const PROTOCOL_ID: u64 = 1;
const SERVER_ADDR: &str = "127.0.0.1:5000";

fn new_renet_client() -> (RenetClient, NetcodeClientTransport, u64) {
    let server_addr = SERVER_ADDR.parse().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;

    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(ConnectionConfig::default());

    (client, transport, client_id)
}

fn main() {
    let (client, transport, client_id) = new_renet_client();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Arena".into(),
                resolution: bevy::window::WindowResolution::new(800, 600),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RenetClientPlugin)
        .add_plugins(NetcodeClientPlugin)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(client)
        .insert_resource(transport)
        .insert_resource(resources::LocalClientId(client_id))
        .insert_resource(Time::<Fixed>::from_hz(shared::tick::TICK_RATE as f64))
        .add_plugins(plugin::ClientGamePlugin)
        .run();
}
