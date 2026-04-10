use std::collections::HashMap;
use std::collections::VecDeque;

use bevy::prelude::*;
use shared::{
    protocol::InputBits,
    tick::TickNumber,
    types::{NetworkId, Pos2, PrefabId},
};

/// Marker component for the local player entity.
#[derive(Component)]
pub struct LocalPlayer;

/// The client's own connection id, received from the server in Welcome.
/// Used to recognize which EntitySpawned belongs to the local player.
#[derive(Resource)]
pub struct LocalClientId(pub u64);

/// Frame-level input sampled from the keyboard. Global because input is captured
/// once per frame and consumed by the fixed-update simulation.
#[derive(Resource, Default)]
pub struct CurrentInput(pub InputBits);

/// Client-side predicted position for the local player, updated each fixed tick.
#[derive(Component, Default)]
pub struct PredictedPosition(pub Pos2);

/// Predicted position from the previous fixed tick, used for visual interpolation.
#[derive(Component, Default)]
pub struct PreviousPredictedPosition(pub Pos2);

/// Ring buffer of unacknowledged inputs: (tick, input, post-step position).
/// Entries are removed once the server acknowledges a tick >= their tick.
#[derive(Component, Default)]
pub struct InputHistory(pub VecDeque<(TickNumber, InputBits, Pos2)>);

/// Ring buffer of (server_elapsed, pos) snapshots for a remote entity.
/// Used for snapshot interpolation: render at (now - INTERP_DELAY), lerping
/// between the two bracketing entries.
#[derive(Component, Default)]
pub struct SnapshotBuffer(pub VecDeque<(f64, Pos2)>);

impl SnapshotBuffer {
    const CAP: usize = 32;

    pub fn push(&mut self, elapsed: f64, pos: Pos2) {
        if self.0.back().map_or(false, |(t, _)| *t >= elapsed) {
            return;
        }
        self.0.push_back((elapsed, pos));
        if self.0.len() > Self::CAP {
            self.0.pop_front();
        }
    }

    /// Interpolate position at the given target time.
    /// Returns the oldest known position if target is before all snapshots,
    /// or the newest if target is ahead (no extrapolation).
    pub fn sample(&self, target: f64) -> Option<Pos2> {
        let buf = &self.0;
        if buf.is_empty() {
            return None;
        }
        if target <= buf.front().unwrap().0 {
            return Some(buf.front().unwrap().1);
        }
        if target >= buf.back().unwrap().0 {
            return Some(buf.back().unwrap().1);
        }
        // Find the two snapshots bracketing target.
        let idx = buf.partition_point(|(t, _)| *t <= target);
        let (t0, p0) = buf[idx - 1];
        let (t1, p1) = buf[idx];
        let frac = ((target - t0) / (t1 - t0)) as f32;
        Some(Pos2 {
            x: p0.x + (p1.x - p0.x) * frac,
            y: p0.y + (p1.y - p0.y) * frac,
        })
    }
}

/// Smoothed estimate of the offset between server time and client time.
/// `estimated_server_time = client_now + offset`
///
/// Updated every time a WorldSnapshot arrives. The smoothing factor prevents
/// jitter from individual late or early packets from causing visible jumps.
#[derive(Resource, Default)]
pub struct ServerTime {
    pub offset: f64,
    pub initialized: bool,
}

impl ServerTime {
    const SMOOTH: f64 = 0.05;

    pub fn update(&mut self, server_time: f64, client_now: f64) {
        let sample = server_time - client_now;
        if !self.initialized {
            self.offset = sample;
            self.initialized = true;
        } else {
            self.offset += (sample - self.offset) * Self::SMOOTH;
        }
    }

    pub fn estimate(&self, client_now: f64) -> f64 {
        client_now + self.offset
    }
}

/// Pending position correction from the server. Written by recv_reliable,
/// consumed and removed by apply_correction.
#[derive(Component)]
pub struct PendingCorrection {
    pub tick: shared::tick::TickNumber,
    pub pos: Pos2,
}

/// O(1) reverse lookup from NetworkId to the corresponding Bevy Entity.
/// Covers all networked entities, not just players.
#[derive(Resource, Default)]
pub struct EntityRegistry(pub HashMap<NetworkId, Entity>);

/// Monotonically increasing local tick counter, incremented each fixed step.
#[derive(Resource, Default)]
pub struct LocalTick(pub shared::tick::TickNumber);


/// Spawn function signature: given Commands, the entity, its initial position,
/// and the owner's client id (if any), attach whatever components are needed.
pub type SpawnFn = Box<dyn Fn(&mut Commands, Entity, Pos2, Option<u64>) + Send + Sync>;

/// Maps PrefabId to a spawn function registered at startup.
#[derive(Resource, Default)]
pub struct SpawnRegistry(pub HashMap<PrefabId, SpawnFn>);
