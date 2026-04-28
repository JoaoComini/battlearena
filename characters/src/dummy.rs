use abilities::Health;
use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use physics::PLAYER_SIZE;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Dummy;

#[derive(Component)]
struct DummyRespawn(Timer);

pub struct DummyServerPlugin;

impl Plugin for DummyServerPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Dummy>();
        app.add_observer(on_dummy_spawn);
        app.add_systems(Startup, do_spawn_dummy);
        app.add_systems(
            FixedUpdate,
            (check_dummy_health, tick_dummy_respawn).chain(),
        );
    }
}

pub struct DummyClientPlugin;

impl Plugin for DummyClientPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Dummy>();
        app.add_observer(on_dummy_spawn);
    }
}

fn on_dummy_spawn(
    trigger: On<Add, Dummy>,
    query: Query<&Position>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.entity;
    let capsule_height = PLAYER_SIZE;
    let capsule_radius = PLAYER_SIZE * 0.35;
    let half_height = capsule_height * 0.5 + capsule_radius;
    let pos = query.get(entity).map(|p| p.0).unwrap_or_default();
    commands
        .entity(entity)
        .insert(Transform::from_xyz(pos.x, 0.0, -pos.y));
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(capsule_radius, capsule_height))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.3, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, half_height, 0.0),
        ChildOf(entity),
    ));
}

fn do_spawn_dummy(mut commands: Commands) {
    commands.spawn((
        Dummy,
        Position::from_xy(10.0, 0.0),
        RigidBody::Static,
        Collider::circle(PLAYER_SIZE * 0.5),
        Health {
            current: 100.0,
            max: 100.0,
        },
        Replicate::to_clients(NetworkTarget::All),
    ));
}

fn check_dummy_health(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<Dummy>>,
    pending: Query<(), With<DummyRespawn>>,
) {
    if !pending.is_empty() {
        return;
    }
    for (entity, health) in &query {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
            commands.spawn(DummyRespawn(Timer::from_seconds(2.0, TimerMode::Once)));
        }
    }
}

fn tick_dummy_respawn(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DummyRespawn)>,
    time: Res<Time>,
) {
    for (entity, mut respawn) in &mut query {
        respawn.0.tick(time.delta());
        if respawn.0.just_finished() {
            commands.entity(entity).despawn();
            do_spawn_dummy(commands.reborrow());
        }
    }
}
