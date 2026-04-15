use crate::protocol::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use lightyear::prelude::input::native::ActionState;

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
        // Run physics in FixedUpdate, after our movement systems.
        // This ensures transform_to_position syncs our Transform writes into Position
        // before lightyear's UpdateHistory snapshots Position in FixedPostUpdate.
        app.add_plugins(PhysicsPlugins::new(FixedUpdate));
        app.configure_sets(
            FixedUpdate,
            PhysicsSystems::StepSimulation.after(move_and_slide),
        );
        app.add_systems(FixedUpdate, (movement, move_and_slide).chain());
        app.add_systems(Startup, spawn_scene);
    }
}

pub fn spawn_scene(mut commands: Commands) {
    commands.spawn((
        Floor,
        Transform::from_xyz(0.0, 0.0, -FLOOR_THICKNESS * 0.5),
    ));

    for (x, y) in [
        ( PILLAR_OFFSET,  PILLAR_OFFSET),
        (-PILLAR_OFFSET,  PILLAR_OFFSET),
        ( PILLAR_OFFSET, -PILLAR_OFFSET),
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

pub const SHARED_SETTINGS: SharedSettings = SharedSettings {
    protocol_id: 0,
    private_key: [0; 32],
};

pub(crate) fn move_and_slide(
    mut query: Query<(Entity, &mut Transform, &mut LinearVelocity, &Collider)>,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
) {
    for (entity, mut transform, mut lin_vel, collider) in &mut query {
        let MoveAndSlideOutput {
            position,
            projected_velocity,
        } = move_and_slide.move_and_slide(
            collider,
            transform.translation.xy(),
            transform.rotation.to_euler(EulerRot::XYZ).2,
            lin_vel.0,
            time.delta(),
            &MoveAndSlideConfig::default(),
            &SpatialQueryFilter::from_excluded_entities([entity]),
            |_| MoveAndSlideHitResponse::Accept,
        );
        transform.translation = position.extend(transform.translation.z);
        lin_vel.0 = projected_velocity;
    }
}

pub(crate) fn movement(mut query: Query<(&mut LinearVelocity, &ActionState<Inputs>)>) {
    const MOVE_SPEED: f32 = 200.0;
    for (mut velocity, input) in &mut query {
        let Inputs::Direction(direction) = &input.0;
        let mut dir = Vec2::ZERO;
        if direction.up {
            dir.y += 1.0;
        }
        if direction.down {
            dir.y -= 1.0;
        }
        if direction.left {
            dir.x -= 1.0;
        }
        if direction.right {
            dir.x += 1.0;
        }
        velocity.0 = dir.normalize_or_zero() * MOVE_SPEED;
    }
}
