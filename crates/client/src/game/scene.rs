use avian2d::prelude::*;
use bevy::prelude::*;
use shared::{
    logic::PLAYER_RADIUS,
    map::MAP_OBSTACLES,
    types::{Pos2, PrefabId},
};

use crate::resources::{
    InputHistory, LocalClientId, LocalPlayer, PredictedPosition, PreviousPredictedPosition,
    SnapshotBuffer, SpawnRegistry,
};

pub const PLAYER_PREFAB: PrefabId = PrefabId(0);

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    local_client_id: Res<LocalClientId>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 600.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.4, 0.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(2000.0, 2000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.15, 0.15),
            ..default()
        })),
    ));

    let obstacle_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.3, 0.1),
        ..default()
    });

    for obs in MAP_OBSTACLES {
        // Visual entity — 3D rendering only, no avian components
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(obs.half_x * 2.0, 60.0, obs.half_y * 2.0))),
            MeshMaterial3d(obstacle_mat.clone()),
            Transform::from_xyz(obs.center_x, 20.0, -obs.center_y),
        ));
        // Physics entity — 2D collision only, no mesh
        commands.spawn((
            RigidBody::Static,
            Position(Vec2::new(obs.center_x, obs.center_y)),
            Collider::rectangle(obs.half_x * 2.0, obs.half_y * 2.0),
        ));
    }

    // Register the player prefab. The closure captures my_id to distinguish
    // local from remote: if owner == Some(my_id), attach prediction components.
    let my_id = local_client_id.0;
    let capsule = meshes.add(Capsule3d::new(PLAYER_RADIUS, 24.0));
    let local_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    let remote_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.2, 0.2),
        ..default()
    });
    spawn_registry.0.insert(
        PLAYER_PREFAB,
        Box::new(move |commands, entity, pos: Pos2, owner| {
            let is_local = owner == Some(my_id);
            if is_local {
                commands.entity(entity).insert((
                    LocalPlayer,
                    PredictedPosition::default(),
                    PreviousPredictedPosition::default(),
                    InputHistory::default(),
                    MeshMaterial3d(local_mat.clone()),
                ));

            } else {
                commands.entity(entity).insert((
                    SnapshotBuffer::default(),
                    MeshMaterial3d(remote_mat.clone()),
                ));
            }

            commands.entity(entity).insert((
                    Transform::from_translation(Vec3::new(pos.x, 20.0, -pos.y)),
                    Mesh3d(capsule.clone()),
                    RigidBody::Kinematic,
                    Collider::circle(PLAYER_RADIUS),
                    CustomPositionIntegration,
            ));

        }),
    );
}
