/// Server tick rate in Hz.
pub const TICK_RATE: u32 = 60;

/// Duration of one server tick in seconds.
pub const TICK_DELTA: f32 = 1.0 / TICK_RATE as f32;

/// Monotonic tick counter. Wrapping is intentional.
pub type TickNumber = u32;
