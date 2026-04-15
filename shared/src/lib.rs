use avian2d::prelude::*;
use bevy::prelude::*;
use inputs::InputPlugin;
use physics::{PhysicsPlugin};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use protocol::*;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_PORT: u16 = 5888;
pub const CLIENT_PORT: u16 = 0;
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);
pub const SEND_INTERVAL: Duration = Duration::from_millis(16);
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

pub const PILLAR_OFFSET: f32 = 300.0;
pub const PILLAR_RADIUS: f32 = 30.0;
pub const PILLAR_HEIGHT: f32 = 200.0;
pub const ARENA_SIZE: f32 = 800.0;
pub const FLOOR_THICKNESS: f32 = 20.0;

#[derive(Component)]
pub struct Pillar;

#[derive(Component)]
pub struct Floor;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        app.add_plugins(PhysicsPlugin);
        app.add_plugins(InputPlugin);
        app.add_systems(Startup, spawn_scene);
    }
}

pub fn spawn_scene(mut commands: Commands) {
    commands.spawn((Floor, Transform::from_xyz(0.0, 0.0, -FLOOR_THICKNESS * 0.5)));

    for (x, y) in [
        (PILLAR_OFFSET, PILLAR_OFFSET),
        (-PILLAR_OFFSET, PILLAR_OFFSET),
        (PILLAR_OFFSET, -PILLAR_OFFSET),
        (-PILLAR_OFFSET, -PILLAR_OFFSET),
    ] {
        commands.spawn((
            Pillar,
            RigidBody::Static,
            Collider::circle(PILLAR_RADIUS),
            Transform::from_xyz(x, y, PILLAR_HEIGHT * 0.5),
        ));
    }
}
