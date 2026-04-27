use avian2d::prelude::*;
use bevy::prelude::*;
use characters::registry::CharacterRegistry;
use characters::types::CharacterDef;
use lightyear::prelude::*;
use physics::PLAYER_SIZE;
use protocol::{CharacterType, Health};
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Dummy;

#[derive(Resource)]
struct DummyRespawnTimer(Timer);

#[derive(Resource)]
struct SpawnDummyFlag;

pub struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Dummy>();

        app.add_observer(on_dummy_spawn);
        app.add_systems(Update, draw_dummy_healthbar);
        app.add_systems(Startup, initial_spawn_dummy);
        app.add_systems(
            FixedUpdate,
            (check_dummy_health, tick_dummy_respawn, maybe_spawn_dummy).chain(),
        );
    }
}

fn on_dummy_spawn(trigger: On<Add, Dummy>, mut commands: Commands) {
    let entity = trigger.entity;
    commands.queue(move |world: &mut World| {
        let capsule_height = PLAYER_SIZE;
        let capsule_radius = PLAYER_SIZE * 0.35;
        let half_height = capsule_height * 0.5 + capsule_radius;
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Capsule3d::new(capsule_radius, capsule_height));
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.3, 1.0),
                ..default()
            });
        world.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, half_height, 0.0),
            ChildOf(entity),
        ));
    });
}

fn draw_dummy_healthbar(dummies: Query<(&Position, &Health), With<Dummy>>, mut gizmos: Gizmos) {
    const BAR_WIDTH: f32 = 60.0;
    const BAR_HEIGHT: f32 = 10.0;
    const BAR_Y: f32 = 90.0;

    for (position, health) in &dummies {
        let center = Vec3::new(position.x, BAR_Y, -position.y);
        let pct = (health.current / health.max).clamp(0.0, 1.0);

        gizmos.rect(
            Isometry3d::new(center, Quat::IDENTITY),
            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
            Color::srgb(0.4, 0.0, 0.0),
        );

        let filled_width = BAR_WIDTH * pct;
        let offset_x = (BAR_WIDTH - filled_width) * 0.5;
        gizmos.rect(
            Isometry3d::new(center - Vec3::X * offset_x, Quat::IDENTITY),
            Vec2::new(filled_width, BAR_HEIGHT),
            Color::srgb(0.0, 0.8, 0.1),
        );
    }
}

fn do_spawn_dummy(
    commands: &mut Commands,
    registry: &CharacterRegistry,
    char_assets: &Assets<CharacterDef>,
) {
    let max_health = registry
        .get("dummy", char_assets)
        .map(|d| d.max_health)
        .unwrap_or(100.0);

    commands.spawn((
        Dummy,
        CharacterType("dummy".to_string()),
        Health { current: max_health, max: max_health },
        Position::from_xy(10.0, 0.0),
        RigidBody::Static,
        Collider::circle(0.4),
        Replicate::to_clients(NetworkTarget::All),
    ));
}

fn initial_spawn_dummy(
    mut commands: Commands,
    registry: Res<CharacterRegistry>,
    char_assets: Res<Assets<CharacterDef>>,
) {
    do_spawn_dummy(&mut commands, &registry, &char_assets);
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
            commands
                .insert_resource(DummyRespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)));
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
        commands.insert_resource(SpawnDummyFlag);
    }
}

fn maybe_spawn_dummy(
    flag: Option<Res<SpawnDummyFlag>>,
    mut commands: Commands,
    registry: Res<CharacterRegistry>,
    char_assets: Res<Assets<CharacterDef>>,
) {
    if flag.is_none() {
        return;
    }
    commands.remove_resource::<SpawnDummyFlag>();
    do_spawn_dummy(&mut commands, &registry, &char_assets);
}
