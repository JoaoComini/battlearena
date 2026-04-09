use bevy::prelude::*;
use bevy_renet::{renet::DefaultChannel, RenetClient};
use shared::{
    logic::apply_input,
    protocol::{C2S, InputBits, MoveInput, S2C},
    tick::TickNumber,
    types::PrefabId,
};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::resources::{
    CurrentInput, EntityRegistry, InputHistory, LocalClientId, LocalPlayer, PendingCorrection,
    PredictedPosition, PreviousPredictedPosition, ServerTime, SnapshotBuffer, SpawnRegistry,
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

/// Drains the reliable channel and handles all lifecycle and correction messages:
/// - EntitySpawned: spawns remote entities via SpawnRegistry.
/// - EntityDespawned: despawns entities.
/// - Correction: snaps + re-simulates the local player's prediction.
///
/// All reliable messages are consumed here so no other system needs to touch
/// the ReliableOrdered channel.
pub fn recv_reliable(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut registry: ResMut<EntityRegistry>,
    spawn_registry: Res<SpawnRegistry>,
    mut local: Query<(Entity, &mut InputHistory), With<LocalPlayer>>,
) {
    let mut messages = Vec::new();
    while let Some(msg) = client.receive_message(DefaultChannel::ReliableOrdered) {
        messages.push(msg);
    }

    let mut latest_ack: Option<TickNumber> = None;
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
                info!("Correction received: tick={tick}, pos=({}, {})", pos.x, pos.y);
                if latest_correction.map_or(true, |(t, _)| tick > t) {
                    latest_correction = Some((tick, pos));
                }
            }
            Ok(S2C::Ack { tick }) => {
                if latest_ack.map_or(true, |t| tick > t) {
                    latest_ack = Some(tick);
                }
            }
            _ => {}
        }
    }

    // The effective ack is the highest tick confirmed by either Ack or Correction.
    let ack_tick = match (latest_ack, latest_correction.map(|(t, _)| t)) {
        (Some(a), Some(c)) => Some(a.max(c)),
        (Some(a), None) => Some(a),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };

    if let Ok((entity, mut history)) = local.single_mut() {
        if let Some(ack) = ack_tick {
            while history.0.front().map_or(false, |(t, _, _)| *t <= ack) {
                history.0.pop_front();
            }
        }
        if let Some((tick, pos)) = latest_correction {
            commands.entity(entity).insert(PendingCorrection { tick, pos });
        }
    }
}

/// Consumes a PendingCorrection on the local player, re-simulates from the
/// input history, and snaps the predicted position.
pub fn apply_correction(
    mut commands: Commands,
    mut local: Query<
        (Entity, &mut PredictedPosition, &mut PreviousPredictedPosition, &InputHistory, &PendingCorrection),
        With<LocalPlayer>,
    >,
) {
    let Ok((entity, mut predicted, mut prev_predicted, history, correction)) = local.single_mut() else {
        return;
    };

    let mut pos = correction.pos;
    for (tick, input, _) in &history.0 {
        if *tick > correction.tick {
            pos = apply_input(pos, *input);
        }
    }
    prev_predicted.0 = pos;
    predicted.0 = pos;

    commands.entity(entity).remove::<PendingCorrection>();
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
    if history.0.len() > 64 {
        history.0.pop_front();
    }

    let current = MoveInput { tick, input: input.0, pos: predicted.0 };

    // Find the oldest unacknowledged direction change for loss recovery.
    // We want the first tick of the new direction, not the last tick of the old one.
    let old = history.0.iter().zip(history.0.iter().skip(1))
        .find(|((_, prev_input, _), (_, cur_input, _))| prev_input.0 != cur_input.0)
        .map(|(_, (old_tick, old_input, old_pos))| MoveInput {
            tick: *old_tick,
            input: *old_input,
            pos: *old_pos,
        });

    let msg = C2S::InputTick { current, old };
    if let Ok(bytes) = postcard::to_allocvec(&msg) {
        client.send_message(DefaultChannel::Unreliable, bytes);
    }
}

/// Receives WorldSnapshots, updates the ServerTime offset estimate, and pushes
/// positions into each entity's SnapshotBuffer keyed by server timestamp.
pub fn recv_world_state(
    mut client: ResMut<RenetClient>,
    registry: Res<EntityRegistry>,
    time: Res<Time<bevy::prelude::Real>>,
    mut server_time: ResMut<ServerTime>,
    mut buffers: Query<&mut SnapshotBuffer>,
) {
    let client_now = time.elapsed_secs_f64();

    while let Some(msg) = client.receive_message(DefaultChannel::Unreliable) {
        if let Ok(S2C::WorldSnapshot { server_time: st, players }) = postcard::from_bytes(&msg) {
            server_time.update(st, client_now);
            for state in &players {
                if let Some(&entity) = registry.0.get(&state.id) {
                    if let Ok(mut buf) = buffers.get_mut(entity) {
                        buf.push(st, state.pos);
                    }
                }
            }
        }
    }
}

/// Updates each player's Transform.
/// Remote players: snapshot interpolation at (estimated_server_now - INTERP_DELAY).
/// Local player: predicted position interpolated over the fixed-timestep overshoot.
pub fn render_players(
    time: Res<Time<bevy::prelude::Real>>,
    fixed_time: Res<Time<Fixed>>,
    server_time: Res<ServerTime>,
    mut remote: Query<(&SnapshotBuffer, &mut Transform), Without<LocalPlayer>>,
    mut local: Query<
        (&PredictedPosition, &PreviousPredictedPosition, &mut Transform),
        With<LocalPlayer>,
    >,
) {
    let interp_target = server_time.estimate(time.elapsed_secs_f64()) - shared::tick::INTERP_DELAY;
    for (buf, mut transform) in &mut remote {
        if let Some(pos) = buf.sample(interp_target) {
            transform.translation = Vec3::new(pos.x, 20.0, -pos.y);
        }
    }

    let t = fixed_time.overstep_fraction();
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
