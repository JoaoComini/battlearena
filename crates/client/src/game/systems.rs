use bevy::prelude::*;
use bevy_renet::{renet::DefaultChannel, RenetClient};
use shared::{
    logic::apply_input,
    protocol::{C2S, InputBits, S2C},
    tick::TickNumber,
    types::{NetworkId, PrefabId},
};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::resources::{
    CurrentInput, InputHistory, LocalClientId, LocalPlayer, PlayerRegistry, PredictedPosition,
    PreviousPredictedPosition, ServerPosition, SpawnRegistry,
};

static TICK: AtomicU32 = AtomicU32::new(0);

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
    spawn_registry.0.insert(PLAYER_PREFAB, Box::new(move |commands, entity, pos, owner| {
        let is_local = owner == Some(my_id);
        if is_local {
            commands.entity(entity).insert((
                LocalPlayer,
                PredictedPosition::default(),
                PreviousPredictedPosition::default(),
                InputHistory::default(),
                ServerPosition(pos),
                Mesh3d(capsule.clone()),
                MeshMaterial3d(local_mat.clone()),
                Transform::from_translation(Vec3::new(pos.x, 20.0, -pos.y)),
            ));
        } else {
            commands.entity(entity).insert((
                ServerPosition(pos),
                Mesh3d(capsule.clone()),
                MeshMaterial3d(remote_mat.clone()),
                Transform::from_translation(Vec3::new(pos.x, 20.0, -pos.y)),
            ));
        }
    }));
}

/// Drains the reliable channel and handles all lifecycle and correction messages:
/// - Welcome: spawns the local player entity.
/// - EntitySpawned: spawns remote entities via SpawnRegistry.
/// - EntityDespawned: despawns entities.
/// - Correction: snaps + re-simulates the local player's prediction.
///
/// All reliable messages are consumed here so no other system needs to touch
/// the ReliableOrdered channel.
pub fn recv_reliable(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut registry: ResMut<PlayerRegistry>,
    spawn_registry: Res<SpawnRegistry>,
    mut local: Query<
        (&mut PredictedPosition, &mut PreviousPredictedPosition, &InputHistory),
        With<LocalPlayer>,
    >,
) {
    let mut messages = Vec::new();
    while let Some(msg) = client.receive_message(DefaultChannel::ReliableOrdered) {
        messages.push(msg);
    }

    let mut latest_correction: Option<(TickNumber, shared::types::Pos2)> = None;

    for msg in messages {
        match postcard::from_bytes(&msg) {
            Ok(S2C::EntitySpawned { id, prefab, pos, owner }) => {
                if registry.0.contains_key(&id) {
                    continue;
                }
                let Some(spawn_fn) = spawn_registry.0.get(&prefab) else {
                    warn!("No spawn function registered for prefab {:?}", prefab);
                    continue;
                };
                let entity = commands.spawn(id).id();
                spawn_fn(&mut commands, entity, pos, owner);
                registry.0.insert(id, entity);
            }
            Ok(S2C::EntityDespawned { id }) => {
                if let Some(entity) = registry.0.remove(&id) {
                    commands.entity(entity).despawn();
                }
            }
            Ok(S2C::Correction { tick, pos }) => {
                if latest_correction.map_or(true, |(t, _)| tick > t) {
                    latest_correction = Some((tick, pos));
                }
            }
            _ => {}
        }
    }

    // Apply the latest correction (if any) in one pass.
    if let Some((corrected_tick, server_pos)) = latest_correction {
        if let Ok((mut predicted, mut prev_predicted, history)) = local.single_mut() {
            let mut pos = server_pos;
            for (tick, input, _) in &history.0 {
                if *tick > corrected_tick {
                    pos = apply_input(pos, *input);
                }
            }
            prev_predicted.0 = pos;
            predicted.0 = pos;
        }
    }
}

/// Reads keyboard input each frame.
pub fn capture_input(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<CurrentInput>) {
    let mut bits = InputBits::default();
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        bits.set(InputBits::UP);
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        bits.set(InputBits::DOWN);
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        bits.set(InputBits::LEFT);
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        bits.set(InputBits::RIGHT);
    }
    input.0 = bits;
}

/// Each fixed tick: advance the local simulation, save to history, send to server.
pub fn send_input_tick(
    mut client: ResMut<RenetClient>,
    input: Res<CurrentInput>,
    mut local: Query<
        (&mut PredictedPosition, &mut PreviousPredictedPosition, &mut InputHistory),
        With<LocalPlayer>,
    >,
) {
    let Ok((mut predicted, mut prev_predicted, mut history)) = local.single_mut() else {
        return;
    };

    let tick = TICK.fetch_add(1, Ordering::Relaxed);

    prev_predicted.0 = predicted.0;
    predicted.0 = apply_input(predicted.0, input.0);

    history.0.push_back((tick, input.0, predicted.0));
    if history.0.len() > 256 {
        history.0.pop_front();
    }

    let msg = C2S::InputTick { tick, input: input.0, pos: predicted.0 };
    if let Ok(bytes) = postcard::to_allocvec(&msg) {
        client.send_message(DefaultChannel::Unreliable, bytes);
    }
}

/// Receives WorldSnapshots and updates ServerPosition on each known entity.
pub fn recv_world_state(
    mut client: ResMut<RenetClient>,
    registry: Res<PlayerRegistry>,
    mut server_positions: Query<&mut ServerPosition>,
    mut local: Query<(&NetworkId, &mut InputHistory), With<LocalPlayer>>,
) {
    let mut latest: Option<(u32, Vec<shared::types::PlayerState>)> = None;
    while let Some(msg) = client.receive_message(DefaultChannel::Unreliable) {
        if let Ok(S2C::WorldSnapshot { tick, players }) = postcard::from_bytes(&msg) {
            if latest.as_ref().map_or(true, |(t, _)| tick > *t) {
                latest = Some((tick, players));
            }
        }
    }

    let Some((_server_tick, players)) = latest else {
        return;
    };

    for state in &players {
        if let Some(&entity) = registry.0.get(&state.id) {
            if let Ok(mut sp) = server_positions.get_mut(entity) {
                sp.0 = state.pos;
            }
        }
    }

    if let Ok((local_id, mut history)) = local.single_mut() {
        if let Some(local_state) = players.iter().find(|s| s.id == *local_id) {
            let ack = local_state.ack_tick;
            while history.0.front().map_or(false, |(t, _, _)| *t <= ack) {
                history.0.pop_front();
            }
        }
    }
}

/// Updates each player's Transform from predicted position (local) or ServerPosition (remote).
pub fn render_players(
    time: Res<Time<Fixed>>,
    mut remote: Query<(&ServerPosition, &mut Transform), Without<LocalPlayer>>,
    mut local: Query<
        (&PredictedPosition, &PreviousPredictedPosition, &mut Transform),
        With<LocalPlayer>,
    >,
) {
    let t = time.overstep_fraction();

    for (server_pos, mut transform) in &mut remote {
        transform.translation = Vec3::new(server_pos.0.x, 20.0, -server_pos.0.y);
    }

    if let Ok((predicted, prev_predicted, mut transform)) = local.single_mut() {
        let pos = lerp_pos(prev_predicted.0, predicted.0, t);
        transform.translation = Vec3::new(pos.x, 20.0, -pos.y);
    }
}

fn lerp_pos(a: shared::types::Pos2, b: shared::types::Pos2, t: f32) -> shared::types::Pos2 {
    shared::types::Pos2 {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}
