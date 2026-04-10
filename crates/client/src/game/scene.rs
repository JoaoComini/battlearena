use bevy::prelude::*;
use shared::types::{PrefabId, Pos2};

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

    // Register the player prefab. The closure captures my_id to distinguish
    // local from remote: if owner == Some(my_id), attach prediction components.
    let my_id = local_client_id.0;
    let capsule = meshes.add(Capsule3d::new(16.0, 24.0));
    let local_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    let remote_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.2, 0.2),
        ..default()
    });
    spawn_registry.0.insert(PLAYER_PREFAB, Box::new(move |commands, entity, pos: Pos2, owner| {
        let is_local = owner == Some(my_id);
        if is_local {
            commands.entity(entity).insert((
                LocalPlayer,
                PredictedPosition::default(),
                PreviousPredictedPosition::default(),
                InputHistory::default(),
                Mesh3d(capsule.clone()),
                MeshMaterial3d(local_mat.clone()),
                Transform::from_translation(Vec3::new(pos.x, 20.0, -pos.y)),
            ));
        } else {
            commands.entity(entity).insert((
                SnapshotBuffer::default(),
                Mesh3d(capsule.clone()),
                MeshMaterial3d(remote_mat.clone()),
                Transform::from_translation(Vec3::new(pos.x, 20.0, -pos.y)),
            ));
        }
    }));
}
