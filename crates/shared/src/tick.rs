/// Server tick rate in Hz.
pub const TICK_RATE: u32 = 60;

/// Duration of one server tick in seconds.
pub const TICK_DELTA: f32 = 1.0 / TICK_RATE as f32;

/// How far behind real-time remote entities are rendered, in seconds.
/// Two ticks gives enough buffer to always have two snapshots to interpolate between.
pub const INTERP_DELAY: f64 = 3.0 * TICK_DELTA as f64;

/// Monotonic tick counter. Wrapping is intentional.
pub type TickNumber = u32;
