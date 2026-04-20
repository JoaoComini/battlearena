use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use physics::PLAYER_SIZE;
use protocol::Health;
use serde::{Deserialize, Serialize};


#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Dummy;

#[derive(Resource)]
struct DummyRespawnTimer(Timer);


pub struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Dummy>();

        app.add_observer(on_dummy_spawn);
        app.add_systems(Update, draw_dummy_healthbar);
        app.add_systems(Startup, spawn_dummy);
        app.add_systems(FixedUpdate, (check_dummy_health, tick_dummy_respawn).chain());
    }
}

fn on_dummy_spawn(
    trigger: On<Add, Dummy>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.entity;
    let capsule_height = PLAYER_SIZE;
    let capsule_radius = PLAYER_SIZE * 0.35;
    let half_height = capsule_height * 0.5 + capsule_radius;

    let visual = commands
        .spawn((
            Mesh3d(meshes.add(Capsule3d::new(capsule_radius, capsule_height))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.3, 1.0),
                ..default()
            })),
            Transform::from_xyz(0.0, half_height, 0.0),
        ))
        .id();

    commands.entity(entity).add_child(visual);
}

fn draw_dummy_healthbar(
    dummies: Query<(&Position, &Health), With<Dummy>>,
    mut gizmos: Gizmos,
) {
    const BAR_WIDTH: f32 = 60.0;
    const BAR_HEIGHT: f32 = 10.0;
    const BAR_Y: f32 = 90.0;

    for (position, health) in &dummies {
        let center = Vec3::new(position.x, BAR_Y, -position.y);
        let pct = (health.current / health.max).clamp(0.0, 1.0);

        // background (dark red)
        gizmos.rect(
            Isometry3d::new(center, Quat::IDENTITY),
            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
            Color::srgb(0.4, 0.0, 0.0),
        );

        // foreground (green), anchored left
        let filled_width = BAR_WIDTH * pct;
        let offset_x = (BAR_WIDTH - filled_width) * 0.5;
        gizmos.rect(
            Isometry3d::new(center - Vec3::X * offset_x, Quat::IDENTITY),
            Vec2::new(filled_width, BAR_HEIGHT),
            Color::srgb(0.0, 0.8, 0.1),
        );
    }
}

fn spawn_dummy(mut commands: Commands) {
    commands.spawn((
        Dummy,
        Health { current: 100.0, max: 100.0 },
        Position::from_xy(150.0, 0.0),
        RigidBody::Static,
        Collider::circle(25.0),
        Replicate::to_clients(NetworkTarget::All),
        InterpolationTarget::to_clients(NetworkTarget::All),
    ));
}

fn check_dummy_health(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<Dummy>>,
    timer: Option<Res<DummyRespawnTimer>>,
) {
    if timer.is_some() {
        return;
    }
    for (entity, health) in &query {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
            commands.insert_resource(DummyRespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)));
        }
    }
}

fn tick_dummy_respawn(
    mut commands: Commands,
    timer: Option<ResMut<DummyRespawnTimer>>,
    time: Res<Time>,
) {
    let Some(mut timer) = timer else { return };
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        commands.remove_resource::<DummyRespawnTimer>();
        spawn_dummy(commands);
    }
}
