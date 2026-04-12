use crate::protocol::*;
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct ExampleRendererPlugin;

impl Plugin for ExampleRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene);
        app.add_systems(Update, sync_transforms);
        app.add_observer(handle_predicted_spawn);
        app.add_observer(handle_interpolated_spawn);
    }
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 600.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.4, 0.0)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(2000.0, 2000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.15, 0.15),
            ..default()
        })),
    ));
}

/// Attach a capsule mesh to the local player's Predicted entity when it spawns.
fn handle_predicted_spawn(
    trigger: On<Add, (PlayerId, Predicted)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    colors: Query<&PlayerColor, With<Predicted>>,
) {
    let entity = trigger.entity;
    let color = colors
        .get(entity)
        .map(|c| c.0)
        .unwrap_or(Color::WHITE);

    commands.entity(entity).insert((
        Mesh3d(meshes.add(Capsule3d::new(16.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            ..default()
        })),
        Transform::default(),
    ));
}

/// Attach a capsule mesh to Interpolated (remote) player entities.
fn handle_interpolated_spawn(
    trigger: On<Add, Interpolated>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    colors: Query<&PlayerColor>,
) {
    let entity = trigger.entity;
    let color = colors
        .get(entity)
        .map(|c| c.0)
        .unwrap_or(Color::srgb(1.0, 0.2, 0.2));

    commands.entity(entity).insert((
        Mesh3d(meshes.add(Capsule3d::new(16.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            ..default()
        })),
        Transform::default(),
    ));
}

/// Keep the Bevy Transform in sync with the replicated/predicted PlayerPosition.
/// PlayerPosition is a flat 2D coordinate; we map it to the XZ plane so the
/// camera (which looks down from Y) sees correct movement.
fn sync_transforms(mut query: Query<(&PlayerPosition, &mut Transform)>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(pos.x, 20.0, -pos.y);
    }
}
