use crate::{protocol::InputBits, tick::TICK_DELTA, types::Pos2};

pub const PLAYER_SPEED: f32 = 200.0; // world units per second

/// Pure movement integration. Called by both the server (authoritatively) and
/// the client (for prediction). No side effects.
pub fn apply_input(pos: Pos2, input: InputBits) -> Pos2 {
    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;

    if input.is_set(InputBits::RIGHT) { dx += 1.0; }
    if input.is_set(InputBits::LEFT)  { dx -= 1.0; }
    if input.is_set(InputBits::UP)    { dy += 1.0; }
    if input.is_set(InputBits::DOWN)  { dy -= 1.0; }

    // Normalize diagonal movement so speed is consistent in all directions.
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        dx /= len;
        dy /= len;
    }

    Pos2 {
        x: pos.x + dx * PLAYER_SPEED * TICK_DELTA,
        y: pos.y + dy * PLAYER_SPEED * TICK_DELTA,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_input_stays_put() {
        let pos = Pos2 { x: 10.0, y: 20.0 };
        let result = apply_input(pos, InputBits::default());
        assert_eq!(result.x, pos.x);
        assert_eq!(result.y, pos.y);
    }

    #[test]
    fn right_input_increases_x() {
        let pos = Pos2::ZERO;
        let mut input = InputBits::default();
        input.set(InputBits::RIGHT);
        let result = apply_input(pos, input);
        assert!(result.x > 0.0);
        assert_eq!(result.y, 0.0);
    }

    #[test]
    fn diagonal_speed_equals_cardinal_speed() {
        let mut cardinal = InputBits::default();
        cardinal.set(InputBits::RIGHT);
        let cardinal_result = apply_input(Pos2::ZERO, cardinal);

        let mut diagonal = InputBits::default();
        diagonal.set(InputBits::RIGHT);
        diagonal.set(InputBits::UP);
        let diagonal_result = apply_input(Pos2::ZERO, diagonal);

        let cardinal_dist = (cardinal_result.x * cardinal_result.x + cardinal_result.y * cardinal_result.y).sqrt();
        let diagonal_dist = (diagonal_result.x * diagonal_result.x + diagonal_result.y * diagonal_result.y).sqrt();

        assert!((cardinal_dist - diagonal_dist).abs() < 1e-5);
    }
}
