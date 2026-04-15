mod setup;
pub mod systems;

pub use setup::{BattleArenaClient, ClientTransports, connect};
pub use systems::BattleArenaClientPlugin;
