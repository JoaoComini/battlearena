pub mod dummy;

use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use inputs::InputPlugin;
use physics::PhysicsPlugin;
use protocol::*;
use scene::ScenePlugin;

use crate::dummy::DummyPlugin;

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;
pub const SERVER_PORT: u16 = 5888;
pub const CLIENT_PORT: u16 = 0;
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);
pub const SEND_INTERVAL: Duration = Duration::from_nanos(1e9 as u64 / 60);
pub const STEAM_APP_ID: u32 = 480;

#[derive(Copy, Clone, Debug)]
pub struct SharedSettings {
    pub protocol_id: u64,
    pub private_key: [u8; 32],
}

pub const SHARED_SETTINGS: SharedSettings = SharedSettings {
    protocol_id: 0,
    private_key: [0; 32],
};

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        app.add_plugins(PhysicsPlugin);
        app.add_plugins(InputPlugin);
        app.add_plugins(ScenePlugin);
        // app.add_plugins(DummyPlugin);
        app.add_systems(Startup, spawn_scene);
    }
}

pub fn spawn_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DynamicSceneRoot(asset_server.load("assets/models/arena.scn.ron")));
}
