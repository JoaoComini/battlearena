use crate::protocol::*;
use bevy::prelude::*;
use core::time::Duration;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
    }
}

pub const TICK_RATE: f64 = 60.0;

/// How often the server flushes replication updates to clients.
/// Sending every ~6 ticks is a good balance for a 60 Hz simulation.
pub const SEND_INTERVAL: Duration = Duration::from_millis(100);

pub const SERVER_PORT: u16 = 5000;
pub const CLIENT_PORT: u16 = 0; // let the OS assign a port

pub const PROTOCOL_ID: u64 = 0;
pub const PRIVATE_KEY: [u8; 32] = [0u8; 32];

/// Speed in world units per second. Kept identical to the original codebase.
const PLAYER_SPEED: f32 = 200.0;

/// Shared movement logic — called by both the server (on the authoritative entity)
/// and the client (on the Predicted entity). Must be bit-identical on both sides
/// so that lightyear's prediction comparison yields no false rollbacks.
pub fn shared_movement_behaviour(mut position: Mut<PlayerPosition>, input: &Inputs) {
    let Inputs::Direction(dir) = input;

    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;

    if dir.right { dx += 1.0; }
    if dir.left  { dx -= 1.0; }
    if dir.up    { dy += 1.0; }
    if dir.down  { dy -= 1.0; }

    // Normalize diagonal so speed is consistent in all directions.
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        dx /= len;
        dy /= len;
    }

    let dt = (1.0 / TICK_RATE) as f32;
    position.x += dx * PLAYER_SPEED * dt;
    position.y += dy * PLAYER_SPEED * dt;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::change_detection::MutUntyped;

    fn make_input(up: bool, down: bool, left: bool, right: bool) -> Inputs {
        Inputs::Direction(Direction { up, down, left, right })
    }

    fn moved(pos: Vec2, input: Inputs) -> Vec2 {
        let mut position = PlayerPosition(pos);
        // SAFETY: we own `position` and it lives for the duration of the closure.
        let tick = bevy::ecs::component::Tick::new(0);
        let mut position_mut = Mut::new(
            &mut position,
            &mut bool::default(),
            tick,
            tick,
        );
        shared_movement_behaviour(position_mut, &input);
        position.0
    }

    #[test]
    fn no_input_stays_put() {
        let pos = Vec2::new(10.0, 20.0);
        let result = moved(pos, make_input(false, false, false, false));
        assert_eq!(result, pos);
    }

    #[test]
    fn right_input_increases_x() {
        let result = moved(Vec2::ZERO, make_input(false, false, false, true));
        assert!(result.x > 0.0);
        assert_eq!(result.y, 0.0);
    }

    #[test]
    fn diagonal_speed_equals_cardinal_speed() {
        let cardinal = moved(Vec2::ZERO, make_input(false, false, false, true));
        let diagonal = moved(Vec2::ZERO, make_input(true, false, false, true));

        let cardinal_dist = cardinal.length();
        let diagonal_dist = diagonal.length();

        assert!((cardinal_dist - diagonal_dist).abs() < 1e-5);
    }
}
