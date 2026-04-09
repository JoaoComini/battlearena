/// Server tick rate in Hz.
pub const TICK_RATE: u32 = 60;

/// Duration of one server tick in seconds.
pub const TICK_DELTA: f32 = 1.0 / TICK_RATE as f32;

/// How far behind real-time remote entities are rendered, in seconds.
/// Needs to be large enough that the snapshot buffer always has two entries
/// bracketing the interp target. 8 ticks gives a comfortable margin locally
/// and handles moderate network jitter.
pub const INTERP_DELAY: f64 = 2.0 * TICK_DELTA as f64;

/// Monotonic tick counter. Wrapping is intentional.
pub type TickNumber = u32;
