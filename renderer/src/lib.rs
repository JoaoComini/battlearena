#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

pub const PLAYER_SIZE: f32 = 50.0;

use avian2d::debug_render::PhysicsDebugPlugin;
use avian2d::prelude::*;
use bevy::prelude::*;
use inputs::Inputs;
use lightyear::prelude::input::native::InputMarker;
use protocol::*;
use shared::*;

pub struct BattleArenaRendererPlugin;

impl Plugin for BattleArenaRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsDebugPlugin::default());
        app.add_systems(Startup, (init, add_scene_meshes).chain());
        app.add_observer(on_player_spawn);
        app.add_observer(on_mesh_spawn);
        app.add_systems(Update, follow_local_player);
    }
}

fn init(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, -300.0, 500.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: 60_f32.to_radians(),
            ..default()
        }),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(100.0, 600.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn add_scene_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    floor: Query<Entity, With<Floor>>,
    pillars: Query<Entity, With<Pillar>>,
) {
    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.35),
        perceptual_roughness: 0.9,
        ..default()
    });
    let pillar_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.55, 0.5),
        perceptual_roughness: 0.7,
        ..default()
    });

    if let Ok(entity) = floor.single() {
        let visual = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(ARENA_SIZE, FLOOR_THICKNESS, ARENA_SIZE))),
                MeshMaterial3d(floor_material),
                Transform::default(),
            ))
            .id();
        commands.entity(entity).add_child(visual);
    }

    for entity in &pillars {
        let visual = commands
            .spawn((
                Mesh3d(meshes.add(Cylinder::new(PILLAR_RADIUS, PILLAR_HEIGHT))),
                MeshMaterial3d(pillar_material.clone()),
                Transform::default(),
            ))
            .id();
        commands.entity(entity).add_child(visual);
    }
}

#[derive(Component)]
struct PlayerVisual;

fn on_player_spawn(
    trigger: On<Add, (Position, PlayerColor)>,
    query: Query<&PlayerColor>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.entity;
    let Ok(color) = query.get(entity) else {
        return;
    };

    let capsule_height = PLAYER_SIZE;
    let capsule_radius = PLAYER_SIZE * 0.35;
    let half_height = capsule_height * 0.5 + capsule_radius;

    let mesh = meshes.add(Capsule3d::new(capsule_radius, capsule_height));
    let material = materials.add(StandardMaterial {
        base_color: color.0,
        ..default()
    });

    let visual = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, half_height),
            PlayerVisual,
        ))
        .id();

    commands.entity(entity).add_child(visual);
}

fn on_mesh_spawn(trigger: On<Add, Mesh3d>, mut transforms: Query<&mut Transform>) {
    if let Ok(mut transform) = transforms.get_mut(trigger.entity) {
        transform.rotation *= Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    }
}

fn follow_local_player(
    local_player: Query<&Position, With<InputMarker<Inputs>>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(position) = local_player.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    camera_transform.translation = Vec3::new(position.x, position.y - 300.0, 500.0);
    camera_transform.look_at(Vec3::new(position.x, position.y, 0.0), Vec3::Y);
}
